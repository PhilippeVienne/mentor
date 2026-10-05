//! Integration tests against a real PostgreSQL: tenant isolation and progress persistence.
//!
//! They need the URL of a superuser connection in `MENTOR_TEST_DATABASE_URL`, for instance
//! `postgres://postgres:mentor-test@127.0.0.1:55439/postgres` (see the README). Each test creates its own
//! database. Without the variable every test is skipped, with a message.
//!
//! The application pool logs in as an ordinary role that is only a member of `mentor_app`: a superuser
//! would bypass row-level security and the tests would prove nothing.

use mentor_core::gamification::{XP_LESSON, XP_QUIZ, XP_TASK};
use mentor_core::progress::{Event, LessonRules, RecordError, Source};
use mentor_db::{platform, resolve_tenant, Error, TenantTx};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use uuid::Uuid;

const APP_LOGIN: &str = "mentor_test_app";
const APP_PASSWORD: &str = "mentor-test-app";

struct TestDb {
    owner: PgPool,
    app: PgPool,
}

async fn database() -> Option<TestDb> {
    let Ok(url) = std::env::var("MENTOR_TEST_DATABASE_URL") else {
        eprintln!("skipped: MENTOR_TEST_DATABASE_URL is not set");
        return None;
    };
    let admin_options: PgConnectOptions = url.parse().expect("valid MENTOR_TEST_DATABASE_URL");
    let admin = PgPoolOptions::new().max_connections(1).connect_with(admin_options.clone()).await.expect("PostgreSQL reachable");
    let name = format!("mentor_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name}")).execute(&admin).await.unwrap();
    let owner = PgPoolOptions::new().max_connections(2).connect_with(admin_options.clone().database(&name)).await.unwrap();
    mentor_db::migrate(&owner).await.expect("migrations apply");
    // Roles are shared by the whole cluster and tests run in parallel: creating the login role may race.
    let created = sqlx::query(&format!("CREATE ROLE {APP_LOGIN} LOGIN PASSWORD '{APP_PASSWORD}' IN ROLE mentor_app")).execute(&owner).await;
    if let Err(err) = created {
        assert!(err.to_string().contains("already exists") || err.to_string().contains("duplicate"), "{err}");
    }
    let app_options = admin_options.database(&name).username(APP_LOGIN).password(APP_PASSWORD).disable_statement_logging();
    let app = PgPoolOptions::new().max_connections(4).connect_with(app_options).await.unwrap();
    Some(TestDb { owner, app })
}

fn lesson(tasks: u32, questions: u32) -> LessonRules {
    LessonRules { course: "git".into(), slug: "intro".into(), tasks, questions, server_verified: false, real_labs_available: false }
}

#[tokio::test]
async fn a_tenant_sees_only_its_own_rows() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &["acme.mentor.test"]).await.unwrap();
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &["globex.mentor.test"]).await.unwrap();

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    tx.upsert_learner("sub-1", "alice", false).await.unwrap();
    tx.commit().await.unwrap();

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert_eq!(tx.count_learners().await.unwrap(), 1);
    assert_eq!(tx.tenant_name().await.unwrap().as_deref(), Some("Acme"));

    let mut other = TenantTx::begin(&db.app, globex).await.unwrap();
    assert_eq!(other.count_learners().await.unwrap(), 0);
    // Even the list of tenants is reduced to its own row.
    assert_eq!(other.tenant_name().await.unwrap().as_deref(), Some("Globex"));
}

#[tokio::test]
async fn without_a_tenant_nothing_is_visible_or_writable() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    tx.upsert_learner("sub-1", "alice", false).await.unwrap();
    tx.commit().await.unwrap();

    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM learner").fetch_one(&db.app).await.unwrap();
    assert_eq!(visible, 0);
    let tenants: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant").fetch_one(&db.app).await.unwrap();
    assert_eq!(tenants, 0);
    let insert = sqlx::query("INSERT INTO learner (tenant_id, subject, username) VALUES ($1, 'x', 'x')").bind(acme).execute(&db.app).await;
    assert!(insert.unwrap_err().to_string().contains("row-level security"));
}

#[tokio::test]
async fn a_tenant_cannot_write_into_another() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &[]).await.unwrap();

    let mut connection = db.app.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)").bind(acme.to_string()).execute(&mut *connection).await.unwrap();
    let forged = sqlx::query("INSERT INTO learner (tenant_id, subject, username) VALUES ($1, 'x', 'mallory')")
        .bind(globex)
        .execute(&mut *connection)
        .await;
    assert!(forged.unwrap_err().to_string().contains("row-level security"));
}

#[tokio::test]
async fn the_application_role_cannot_create_tenants() {
    let Some(db) = database().await else { return };
    let attempt = platform::create_tenant(&db.app, "rogue", "Rogue", &[]).await;
    assert!(matches!(attempt, Err(Error::Database(_))));
}

#[tokio::test]
async fn tenants_are_resolved_by_host_name() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &["acme.mentor.test", "learn.acme.example"]).await.unwrap();
    assert_eq!(resolve_tenant(&db.app, "ACME.mentor.test").await.unwrap(), Some(acme));
    assert_eq!(resolve_tenant(&db.app, "learn.acme.example").await.unwrap(), Some(acme));
    assert_eq!(resolve_tenant(&db.app, "unknown.example").await.unwrap(), None);
    // A host name cannot be claimed by a second tenant.
    assert!(platform::create_tenant(&db.owner, "squatter", "Squatter", &["acme.mentor.test"]).await.is_err());
}

#[tokio::test]
async fn progress_and_xp_are_persisted_once() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let lesson = lesson(1, 3);

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let alice = tx.upsert_learner("sub-1", "alice", false).await.unwrap().id;
    // The quiz is refused before the lab; nothing is credited.
    let refused = tx.record_event(alice, &lesson, Event::Quiz(3), Source::Browser).await;
    assert!(matches!(refused, Err(Error::Refused(RecordError::LabNotDone))));
    assert_eq!(tx.record_event(alice, &lesson, Event::Task(0), Source::Browser).await.unwrap().xp_gained, XP_TASK);
    assert_eq!(tx.record_event(alice, &lesson, Event::Task(0), Source::Browser).await.unwrap().xp_gained, 0);
    tx.commit().await.unwrap();

    // A new transaction reads back what was stored and continues from there.
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let done = tx.record_event(alice, &lesson, Event::Quiz(3), Source::Browser).await.unwrap();
    assert!(done.lesson_completed && done.progress.completed);
    assert_eq!(done.xp_gained, 3 * XP_QUIZ + XP_LESSON);
    assert_eq!(tx.total_xp(alice).await.unwrap(), XP_TASK + 3 * XP_QUIZ + XP_LESSON);
    tx.commit().await.unwrap();

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let again = tx.record_event(alice, &lesson, Event::Quiz(3), Source::Browser).await.unwrap();
    assert_eq!((again.xp_gained, again.lesson_completed), (0, false));
}

#[tokio::test]
async fn concurrent_requests_do_not_pay_twice() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let alice = tx.upsert_learner("sub-1", "alice", false).await.unwrap().id;
    tx.commit().await.unwrap();

    let attempt = |app: PgPool| async move {
        let mut tx = TenantTx::begin(&app, acme).await.unwrap();
        let gained = tx.record_event(alice, &lesson(4, 0), Event::Task(2), Source::Browser).await.unwrap().xp_gained;
        tx.commit().await.unwrap();
        gained
    };
    let results = tokio::join!(attempt(db.app.clone()), attempt(db.app.clone()), attempt(db.app.clone()), attempt(db.app.clone()));
    assert_eq!(results.0 + results.1 + results.2 + results.3, XP_TASK);

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert_eq!(tx.total_xp(alice).await.unwrap(), XP_TASK);
}
