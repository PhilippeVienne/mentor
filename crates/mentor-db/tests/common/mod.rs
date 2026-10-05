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

const OWNER_LOGIN: &str = "mentor_test_owner";
const APP_LOGIN: &str = "mentor_test_app";
const PASSWORD: &str = "mentor-test";

pub struct TestDb {
    /// Owning role: migrations and platform administration.
    pub owner: PgPool,
    /// Application role: subject to row-level security.
    pub app: PgPool,
}

/// Creates a cluster-wide role; tests run in parallel, so another test may create it at the same moment.
async fn ensure_role(admin: &PgPool, definition: &str) {
    if let Err(err) = sqlx::query(&format!("CREATE ROLE {definition}")).execute(admin).await {
        let message = err.to_string();
        assert!(message.contains("already exists") || message.contains("duplicate"), "{message}");
    }
}

pub async fn database() -> Option<TestDb> {
    let Ok(url) = std::env::var("MENTOR_TEST_DATABASE_URL") else {
        eprintln!("skipped: MENTOR_TEST_DATABASE_URL is not set");
        return None;
    };
    let admin_options: PgConnectOptions = url.parse().expect("valid MENTOR_TEST_DATABASE_URL");
    let admin = PgPoolOptions::new().max_connections(1).connect_with(admin_options.clone()).await.expect("PostgreSQL reachable");
    // The superuser only sets the stage. Migrations and platform operations run as an ordinary owning role,
    // as they would in production: a superuser would hide any mistake in the policies.
    ensure_role(&admin, &format!("{OWNER_LOGIN} LOGIN CREATEROLE PASSWORD '{PASSWORD}'")).await;
    let name = format!("mentor_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name} OWNER {OWNER_LOGIN}")).execute(&admin).await.unwrap();
    let connect = |login: &str| admin_options.clone().database(&name).username(login).password(PASSWORD).disable_statement_logging();
    let owner = PgPoolOptions::new().max_connections(2).connect_with(connect(OWNER_LOGIN)).await.unwrap();
    mentor_db::migrate(&owner).await.expect("migrations apply");
    ensure_role(&admin, &format!("{APP_LOGIN} LOGIN PASSWORD '{PASSWORD}' IN ROLE mentor_app")).await;
    let app = PgPoolOptions::new().max_connections(4).connect_with(connect(APP_LOGIN)).await.unwrap();
    Some(TestDb { owner, app })
}
