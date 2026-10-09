//! Course packages installed for a tenant: what its catalogue is made of.
//!
//! Installing, replacing and removing a package is platform administration, done by the owning role
//! ([`install`], [`remove`], [`list`]). The application only reads: [`TenantTx::catalogue`] gives a tenant the
//! courses and training paths of its packages, [`TenantTx::course_image`] one picture.

use mentor_content::{Catalogue, Course, Image, LearningPath, Package};
use sqlx::types::Json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{Error, Result, TenantTx};

/// A package as listed for a tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    pub name: String,
    pub version: String,
    pub title: String,
    pub source: String,
    pub courses: usize,
    pub paths: usize,
}

/// What an installation changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Installed,
    /// A package of that name was already installed: its content was replaced, its place kept.
    Replaced,
}

/// Installs a compiled package for a tenant, or replaces the installed package of the same name, in one
/// transaction: the tenant's catalogue changes all at once or not at all.
///
/// Refused when another package of the tenant already brings a course or a training path of the same name:
/// progress and addresses refer to them by name.
pub async fn install(owner: &PgPool, tenant: Uuid, package: &Package, images: &[Image], source: &str) -> Result<Outcome> {
    let mut tx = owner.begin().await?;
    let name = &package.manifest.name;
    let existing: Option<i32> = sqlx::query_scalar("SELECT position FROM package WHERE tenant_id = $1 AND name = $2")
        .bind(tenant)
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;
    let position = match existing {
        Some(position) => position,
        None => {
            sqlx::query_scalar("SELECT coalesce(max(position), 0) + 1 FROM package WHERE tenant_id = $1")
                .bind(tenant)
                .fetch_one(&mut *tx)
                .await?
        }
    };
    // Items and pictures go with the row they belong to.
    sqlx::query("DELETE FROM package WHERE tenant_id = $1 AND name = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    sqlx::query(
        "INSERT INTO package (tenant_id, name, version, title, source, position, manifest, courses, paths) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(tenant)
    .bind(name)
    .bind(&package.manifest.version)
    .bind(&package.manifest.title)
    .bind(source)
    .bind(position)
    .bind(Json(&package.manifest))
    .bind(Json(&package.catalogue.courses))
    .bind(Json(&package.paths))
    .execute(&mut *tx)
    .await?;

    let courses = package.catalogue.courses.iter().map(|course| ("course", course.slug.as_str()));
    let paths = package.paths.iter().map(|path| ("path", path.id.as_str()));
    for (kind, slug) in courses.chain(paths) {
        let taken: Option<String> = sqlx::query_scalar("SELECT package FROM package_item WHERE tenant_id = $1 AND kind = $2 AND slug = $3")
            .bind(tenant)
            .bind(kind)
            .bind(slug)
            .fetch_optional(&mut *tx)
            .await?;
        if let Some(other) = taken {
            let what = if kind == "course" { "course" } else { "training path" };
            return Err(Error::PackageConflict(format!("the {what} `{slug}` is already brought by the package `{other}`")));
        }
        sqlx::query("INSERT INTO package_item (tenant_id, kind, slug, package) VALUES ($1, $2, $3, $4)")
            .bind(tenant)
            .bind(kind)
            .bind(slug)
            .bind(name)
            .execute(&mut *tx)
            .await?;
    }
    for image in images {
        sqlx::query("INSERT INTO package_image (tenant_id, package, course, path, media_type, content) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(tenant)
            .bind(name)
            .bind(&image.course)
            .bind(&image.path)
            .bind(image.media_type)
            .bind(&image.content)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(if existing.is_some() { Outcome::Replaced } else { Outcome::Installed })
}

/// Removes a package from a tenant; `false` when it was not installed. Learners' progress on its courses is
/// kept: it shows again if the package comes back.
pub async fn remove(owner: &PgPool, tenant: Uuid, name: &str) -> Result<bool> {
    let removed = sqlx::query("DELETE FROM package WHERE tenant_id = $1 AND name = $2").bind(tenant).bind(name).execute(owner).await?;
    Ok(removed.rows_affected() == 1)
}

/// The packages of a tenant, in the order of its catalogue.
pub async fn list(owner: &PgPool, tenant: Uuid) -> Result<Vec<Installed>> {
    let rows = sqlx::query(
        "SELECT name, version, title, source, jsonb_array_length(courses) AS courses, jsonb_array_length(paths) AS paths \
         FROM package WHERE tenant_id = $1 ORDER BY position, name",
    )
    .bind(tenant)
    .fetch_all(owner)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| Installed {
            name: row.get("name"),
            version: row.get("version"),
            title: row.get("title"),
            source: row.get("source"),
            courses: row.get::<i32, _>("courses") as usize,
            paths: row.get::<i32, _>("paths") as usize,
        })
        .collect())
}

impl TenantTx {
    /// A value that changes whenever the tenant's packages do: cheap to read at each request, to know
    /// whether a catalogue kept in memory is still the tenant's.
    pub async fn catalogue_stamp(&mut self) -> Result<String> {
        Ok(sqlx::query_scalar("SELECT coalesce(md5(string_agg(name || '@' || installed_at::text, ',' ORDER BY name)), '') FROM package")
            .fetch_one(&mut *self.tx)
            .await?)
    }

    /// The tenant's catalogue: the courses and training paths of its packages, in order.
    pub async fn catalogue(&mut self) -> Result<(Catalogue, Vec<LearningPath>)> {
        let rows = sqlx::query("SELECT courses, paths FROM package ORDER BY position, name").fetch_all(&mut *self.tx).await?;
        let (mut courses, mut paths) = (Vec::new(), Vec::new());
        for row in rows {
            courses.extend(row.get::<Json<Vec<Course>>, _>("courses").0);
            paths.extend(row.get::<Json<Vec<LearningPath>>, _>("paths").0);
        }
        Ok((Catalogue { courses }, paths))
    }

    /// A picture of a course of the tenant: its media type and its content.
    pub async fn course_image(&mut self, course: &str, path: &str) -> Result<Option<(String, Vec<u8>)>> {
        let row = sqlx::query("SELECT media_type, content FROM package_image WHERE course = $1 AND path = $2")
            .bind(course)
            .bind(path)
            .fetch_optional(&mut *self.tx)
            .await?;
        Ok(row.map(|row| (row.get("media_type"), row.get("content"))))
    }
}
