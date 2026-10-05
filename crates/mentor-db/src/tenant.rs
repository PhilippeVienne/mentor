//! Tenant-bound transactions and learners.

use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::Result;

/// Finds the tenant served on `host`. This is the only lookup allowed before a tenant is known.
pub async fn resolve_tenant(app: &PgPool, host: &str) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar("SELECT resolve_tenant($1)").bind(host).fetch_one(app).await?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Learner {
    pub id: Uuid,
    pub subject: String,
    pub username: String,
    pub is_admin: bool,
}

/// A transaction bound to one tenant: every statement in it sees and writes that tenant's rows only.
///
/// The binding is a transaction-local setting, so it cannot leak to the next user of a pooled connection.
/// Nothing is persisted until [`TenantTx::commit`].
pub struct TenantTx {
    pub(crate) tx: Transaction<'static, Postgres>,
    pub(crate) tenant: Uuid,
}

impl TenantTx {
    pub async fn begin(app: &PgPool, tenant: Uuid) -> Result<Self> {
        let mut tx = app.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)").bind(tenant.to_string()).execute(&mut *tx).await?;
        Ok(Self { tx, tenant })
    }

    pub fn tenant(&self) -> Uuid {
        self.tenant
    }

    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }

    /// Display name of the tenant, read through the policies: it proves the binding is in effect.
    pub async fn tenant_name(&mut self) -> Result<Option<String>> {
        Ok(sqlx::query_scalar("SELECT name FROM tenant").fetch_optional(&mut *self.tx).await?)
    }

    /// Creates the learner on first login, or refreshes the profile sent by the identity provider.
    pub async fn upsert_learner(&mut self, subject: &str, username: &str, is_admin: bool) -> Result<Learner> {
        let row = sqlx::query(
            "INSERT INTO learner (tenant_id, subject, username, is_admin) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (tenant_id, subject) DO UPDATE SET username = excluded.username, is_admin = excluded.is_admin \
             RETURNING id, subject, username, is_admin",
        )
        .bind(self.tenant)
        .bind(subject)
        .bind(username)
        .bind(is_admin)
        .fetch_one(&mut *self.tx)
        .await?;
        Ok(Learner { id: row.get("id"), subject: row.get("subject"), username: row.get("username"), is_admin: row.get("is_admin") })
    }

    /// Number of learners visible in this transaction, that is, of this tenant.
    pub async fn count_learners(&mut self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM learner").fetch_one(&mut *self.tx).await?)
    }
}
