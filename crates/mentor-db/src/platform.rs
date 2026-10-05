//! Platform administration: operations across tenants, for the owning role only.

use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

/// Creates a tenant served on `hostnames` and returns its identifier.
pub async fn create_tenant(owner: &PgPool, slug: &str, name: &str, hostnames: &[&str]) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let hostnames: Vec<String> = hostnames.iter().map(|host| host.to_lowercase()).collect();
    sqlx::query("INSERT INTO tenant (id, slug, name, hostnames) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(slug)
        .bind(name)
        .bind(&hostnames)
        .execute(owner)
        .await?;
    Ok(id)
}
