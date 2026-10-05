//! PostgreSQL storage of Mentor.
//!
//! One database serves every tenant. Isolation is enforced **by PostgreSQL**, with row-level security keyed
//! on a transaction setting, not by remembering to filter in each query: see the first migration for the model.
//!
//! - [`migrate`] and [`platform`] are for the owning role: schema changes and tenant administration.
//! - [`TenantTx`] is the only way the application reads or writes tenant data: it opens a transaction bound
//!   to one tenant. It must be used with a connection whose role is a member of `mentor_app`; a superuser or
//!   the owning role would bypass the policies.

pub mod platform;
pub mod progress;
mod tenant;

pub use tenant::{resolve_tenant, Learner, TenantTx};

use sqlx::PgPool;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    /// The event was refused by the progress rules; nothing was written.
    #[error("event refused: {0:?}")]
    Refused(mentor_core::progress::RecordError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Applies pending migrations. Run with the owning role, never with the application role.
pub async fn migrate(owner: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(owner).await?;
    Ok(())
}
