//! Shared by the integration tests: one fresh database per test, on a real PostgreSQL.
//!
//! They need the URL of a superuser connection in `MENTOR_TEST_DATABASE_URL`, for instance
//! `postgres://postgres:mentor-test@127.0.0.1:55439/postgres` (see the README). Without the variable every
//! test is skipped, with a message.
//!
//! The application pool logs in as an ordinary role that is only a member of `mentor_app`: a superuser
//! would bypass row-level security and the tests would prove nothing.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use uuid::Uuid;

const APP_LOGIN: &str = "mentor_test_app";
const APP_PASSWORD: &str = "mentor-test-app";

pub struct TestDb {
    /// Owning role: migrations and platform administration.
    pub owner: PgPool,
    /// Application role: subject to row-level security.
    pub app: PgPool,
}

pub async fn database() -> Option<TestDb> {
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
