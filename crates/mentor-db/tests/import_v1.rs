//! Import of a v1 export, against a real PostgreSQL (see `common`).

mod common;

use std::collections::BTreeMap;

use common::database;
use mentor_core::progress::LessonRules;
use mentor_db::badges::CatalogueView;
use mentor_db::import_v1::{import, Dump, Report};
use mentor_db::{platform, Error, TenantTx};

/// A small export in the format of `manage.py dumpdata`.
const EXPORT: &str = r#"[
 {"model": "auth.user", "pk": 2, "fields": {"username": "alice", "first_name": "Alice", "last_name": "Martin", "email": "alice@example.org", "is_staff": false, "password": "ignored"}},
 {"model": "auth.user", "pk": 5, "fields": {"username": "admin", "is_staff": true}},
 {"model": "training.lesson", "pk": 10, "fields": {"course": "git-basics", "slug": "introduction", "title": "ignored"}},
 {"model": "training.lessonprogress", "pk": 1, "fields": {"user": 2, "lesson": 10, "tasks_done": [0, 1, 2], "quiz_best": 3, "completed_at": "2026-09-08T16:57:58.202Z", "validated_by_exam": false}},
 {"model": "training.lessonprogress", "pk": 2, "fields": {"user": 2, "lesson": 999, "tasks_done": [], "quiz_best": 0, "completed_at": null}},
 {"model": "training.xpevent", "pk": 1, "fields": {"user": 2, "kind": "task", "ref": "git-basics/introduction/t0", "xp": 10, "created_at": "2026-09-06T15:00:00Z"}},
 {"model": "training.xpevent", "pk": 2, "fields": {"user": 2, "kind": "task", "ref": "git-basics/introduction/t1", "xp": 10, "created_at": "2026-09-07T15:00:00Z"}},
 {"model": "training.xpevent", "pk": 3, "fields": {"user": 2, "kind": "lesson", "ref": "git-basics/introduction", "xp": 50, "created_at": "2026-09-08T16:57:58Z"}},
 {"model": "training.userbadge", "pk": 1, "fields": {"user": 2, "badge": "premier-pas", "awarded_at": "2026-09-06T15:00:01Z"}},
 {"model": "training.cohort", "pk": "promo-2026", "fields": {"name": "Promo 2026", "source": "keycloak", "report_emails": "a@example.org, b@example.org\nc@example.org", "created_at": "2026-09-01T08:00:00Z"}},
 {"model": "training.cohortmembership", "pk": 1, "fields": {"cohort": "promo-2026", "user": 2, "source": "keycloak", "joined_at": "2026-09-01T08:00:00Z"}},
 {"model": "training.examattempt", "pk": 1, "fields": {"user": 2, "course": "git-basics", "started_at": "2026-09-09T10:00:00Z", "deadline": "2026-09-09T10:20:00Z", "finished_at": "2026-09-09T10:05:00Z",
   "question_ids": ["old-a", "old-b"], "option_order": {"old-a": [1, 0, 2], "old-b": [0, 2, 1]}, "answers": {"old-a": 1}, "score": 1, "total": 2, "passed": true, "expired": false}},
 {"model": "sessions.session", "pk": "x", "fields": {}}
]"#;

#[tokio::test]
async fn imports_once_keeps_dates_and_stays_inside_the_tenant() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &[]).await.unwrap();
    let dump = Dump::parse(EXPORT).unwrap();
    let question_ids: BTreeMap<String, String> = [("old-a".to_string(), "new-a".to_string())].into();

    let report = import(&db.owner, acme, &dump, &question_ids).await.unwrap();
    assert_eq!(
        (report.learners, report.progress, report.awards, report.badges, report.cohorts, report.memberships, report.attempts),
        (2, 1, 3, 1, 1, 1, 1)
    );
    assert_eq!(report.warnings.len(), 2, "{:?}", report.warnings);
    assert!(report.warnings[0].contains("unknown user 2 or lesson 999"));
    assert!(report.warnings[1].starts_with("1 exam question identifier(s)"));

    // Replaying the import writes nothing more.
    let again = import(&db.owner, acme, &dump, &question_ids).await.unwrap();
    assert_eq!(Report { warnings: Vec::new(), ..again }, Report::default());

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert_eq!(tx.count_learners().await.unwrap(), 2);
    // Imported accounts wait for their first login under a placeholder subject.
    let alice = tx.upsert_learner("v1:alice", "alice", false).await.unwrap().id;
    assert_eq!(tx.total_xp(alice).await.unwrap(), 70);
    assert_eq!(tx.badges_of(alice).await.unwrap().into_iter().collect::<Vec<_>>(), ["premier-pas"]);
    assert_eq!(tx.cohorts_of(alice).await.unwrap(), ["promo-2026"]);
    let lesson = LessonRules {
        course: "git-basics".into(),
        slug: "introduction".into(),
        tasks: 3,
        questions: 3,
        server_verified: false,
        lab_available: false,
    };
    let progress = tx.lesson_progress(alice, &lesson).await.unwrap();
    assert!(progress.completed && progress.tasks_done.len() == 3 && progress.quiz_best == 3);
    // The three events fall on three consecutive days: the streak survives the import because dates do.
    let stats = tx.learner_stats(alice, &CatalogueView::default(), 0).await.unwrap();
    assert_eq!((stats.streak, stats.exams_passed, stats.tasks), (3, 1, 3));
    drop(tx);

    let stored: (serde_json::Value, serde_json::Value, Vec<String>) = sqlx::query_as(
        "SELECT a.questions, a.answers, c.report_emails FROM exam_attempt a JOIN cohort c ON c.tenant_id = a.tenant_id WHERE a.tenant_id = $1",
    )
    .bind(acme)
    .fetch_one(&db.owner)
    .await
    .unwrap();
    assert_eq!(stored.0, serde_json::json!([{"id": "new-a", "option_order": [1, 0, 2]}, {"id": "old-b", "option_order": [0, 2, 1]}]));
    assert_eq!(stored.1, serde_json::json!({"new-a": 1}));
    assert_eq!(stored.2, ["a@example.org", "b@example.org", "c@example.org"]);

    let mut other = TenantTx::begin(&db.app, globex).await.unwrap();
    assert_eq!(other.count_learners().await.unwrap(), 0);
}

#[tokio::test]
async fn a_file_that_is_not_an_export_is_refused() {
    assert!(matches!(Dump::parse("{\"not\": \"a list\"}"), Err(Error::InvalidExport(_))));
    assert!(matches!(Dump::parse(r#"[{"model": "auth.user", "pk": "x", "fields": {"username": "a"}}]"#), Err(Error::InvalidExport(_))));
}
