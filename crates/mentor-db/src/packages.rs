//! Course packages installed for a tenant: what its catalogue is made of.
//!
//! Installing, replacing and removing a package is platform administration, done by the owning role
//! ([`install`], [`remove`], [`list`]). Replacing a package is an update that learners live through:
//! [`preview`] says what it changes for them, [`install`] refuses to take away a course or a lesson unless
//! told to, and [`rollback`] brings the replaced version back. The application only reads: [`TenantTx::catalogue`] gives a tenant the
//! courses and training paths of its packages, [`TenantTx::course_image`] one picture.

use mentor_content::{package_diff, Catalogue, Course, Image, LearningPath, Package, PackageDiff};
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

/// What replacing the installed version of a package would do to the tenant's learners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Impact {
    /// Version currently installed.
    pub installed_version: String,
    pub diff: PackageDiff,
    /// Learners who did something in a course or a lesson that the new version removes.
    pub learners_on_removed: i64,
    /// Learners who had completed a course that gains lessons: it will no longer count as completed.
    pub completions_lost: i64,
    /// Exam attempts in progress on a course whose pool changes.
    pub open_attempts: i64,
}

/// Whether an update may take away courses and lessons that learners have progress on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Removals {
    Refuse,
    Confirmed,
}

async fn installed_content(
    tx: &mut sqlx::PgConnection,
    tenant: Uuid,
    name: &str,
) -> Result<Option<(String, Vec<Course>, Vec<LearningPath>)>> {
    let row = sqlx::query("SELECT version, courses, paths FROM package WHERE tenant_id = $1 AND name = $2")
        .bind(tenant)
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;
    Ok(row.map(|row| (row.get("version"), row.get::<Json<Vec<Course>>, _>("courses").0, row.get::<Json<Vec<LearningPath>>, _>("paths").0)))
}

async fn impact(tx: &mut sqlx::PgConnection, tenant: Uuid, package: &Package) -> Result<Option<Impact>> {
    let Some((installed_version, courses, paths)) = installed_content(tx, tenant, &package.manifest.name).await? else {
        return Ok(None);
    };
    let diff = package_diff((&courses, &paths), (&package.catalogue.courses, &package.paths));

    // What was done, in the removed courses and the removed lessons of kept courses.
    let mut concerned: std::collections::BTreeSet<Uuid> = std::collections::BTreeSet::new();
    let touched = "(cardinality(tasks_done) > 0 OR quiz_best > 0 OR completed_at IS NOT NULL)";
    let in_removed_courses: Vec<Uuid> = sqlx::query_scalar(&format!(
        "SELECT DISTINCT learner_id FROM lesson_progress WHERE tenant_id = $1 AND course = ANY($2) AND {touched}"
    ))
    .bind(tenant)
    .bind(&diff.courses_removed)
    .fetch_all(&mut *tx)
    .await?;
    concerned.extend(in_removed_courses);
    let (mut completions_lost, mut open_attempts) = (0, 0);
    for course in &diff.courses_changed {
        if !course.lessons_removed.is_empty() {
            let learners: Vec<Uuid> = sqlx::query_scalar(&format!(
                "SELECT DISTINCT learner_id FROM lesson_progress WHERE tenant_id = $1 AND course = $2 AND lesson = ANY($3) AND {touched}"
            ))
            .bind(tenant)
            .bind(&course.slug)
            .bind(&course.lessons_removed)
            .fetch_all(&mut *tx)
            .await?;
            concerned.extend(learners);
        }
        if !course.lessons_added.is_empty() && !course.lessons_before.is_empty() {
            completions_lost += sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM (SELECT learner_id FROM lesson_progress \
                 WHERE tenant_id = $1 AND course = $2 AND lesson = ANY($3) AND completed_at IS NOT NULL \
                 GROUP BY learner_id HAVING count(*) = $4) AS finished",
            )
            .bind(tenant)
            .bind(&course.slug)
            .bind(&course.lessons_before)
            .bind(course.lessons_before.len() as i64)
            .fetch_one(&mut *tx)
            .await?;
        }
        if course.exam_changed() {
            open_attempts += sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM exam_attempt WHERE tenant_id = $1 AND course = $2 AND finished_at IS NULL AND deadline > now()",
            )
            .bind(tenant)
            .bind(&course.slug)
            .fetch_one(&mut *tx)
            .await?;
        }
    }
    Ok(Some(Impact { installed_version, diff, learners_on_removed: concerned.len() as i64, completions_lost, open_attempts }))
}

/// What installing `package` would change for the tenant's learners; `None` when no package of that name is
/// installed, so nothing would be replaced. Writes nothing.
pub async fn preview(owner: &PgPool, tenant: Uuid, package: &Package) -> Result<Option<Impact>> {
    let mut connection = owner.acquire().await?;
    impact(&mut connection, tenant, package).await
}

/// Installs a compiled package for a tenant, or replaces the installed package of the same name, in one
/// transaction: the tenant's catalogue changes all at once or not at all.
///
/// Refused when another package of the tenant already brings a course or a training path of the same name
/// (progress and addresses refer to them by name), and, unless `removals` confirms it, when the new version
/// takes away a course or a lesson of the installed one. Nothing of a learner's progress is ever deleted: a
/// removed lesson that comes back in a later version comes back with it.
///
/// The replaced version is kept, one version back, for [`rollback`].
pub async fn install(
    owner: &PgPool,
    tenant: Uuid,
    package: &Package,
    images: &[Image],
    source: &str,
    removals: Removals,
) -> Result<Outcome> {
    let mut tx = owner.begin().await?;
    let name = &package.manifest.name;
    if let Some(impact) = impact(&mut tx, tenant, package).await? {
        if impact.diff.removes_progress() && removals == Removals::Refuse {
            return Err(Error::RemovalsNotConfirmed(impact.diff));
        }
        keep_previous(&mut tx, tenant, name).await?;
    }
    let outcome = write(&mut tx, tenant, package, images, source).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// Copies the installed version of a package aside, replacing the copy kept before.
async fn keep_previous(tx: &mut sqlx::PgConnection, tenant: Uuid, name: &str) -> Result<()> {
    sqlx::query("DELETE FROM package_previous WHERE tenant_id = $1 AND name = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    sqlx::query(
        "INSERT INTO package_previous (tenant_id, name, version, title, source, manifest, courses, paths) \
         SELECT tenant_id, name, version, title, source, manifest, courses, paths FROM package WHERE tenant_id = $1 AND name = $2",
    )
    .bind(tenant)
    .bind(name)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO package_previous_image (tenant_id, package, course, path, media_type, content) \
         SELECT tenant_id, package, course, path, media_type, content FROM package_image WHERE tenant_id = $1 AND package = $2",
    )
    .bind(tenant)
    .bind(name)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn write(tx: &mut sqlx::PgConnection, tenant: Uuid, package: &Package, images: &[Image], source: &str) -> Result<Outcome> {
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
    Ok(if existing.is_some() { Outcome::Replaced } else { Outcome::Installed })
}

/// Brings back the version a package had before it was last replaced, and keeps the one it replaces: rolling
/// back twice gives the update back. Returns the version restored, or `None` when no previous version is kept.
pub async fn rollback(owner: &PgPool, tenant: Uuid, name: &str) -> Result<Option<String>> {
    let mut tx = owner.begin().await?;
    let previous =
        sqlx::query("SELECT version, title, source, manifest, courses, paths FROM package_previous WHERE tenant_id = $1 AND name = $2")
            .bind(tenant)
            .bind(name)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(previous) = previous else { return Ok(None) };
    let images = sqlx::query("SELECT course, path, media_type, content FROM package_previous_image WHERE tenant_id = $1 AND package = $2")
        .bind(tenant)
        .bind(name)
        .fetch_all(&mut *tx)
        .await?;
    // The current version becomes the previous one, then the kept copy is written back in place.
    keep_previous(&mut tx, tenant, name).await?;
    let version: String = previous.get("version");
    sqlx::query("DELETE FROM package_item WHERE tenant_id = $1 AND package = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM package_image WHERE tenant_id = $1 AND package = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    sqlx::query(
        "UPDATE package SET version = $3, title = $4, source = $5, manifest = $6, courses = $7, paths = $8, installed_at = now() \
         WHERE tenant_id = $1 AND name = $2",
    )
    .bind(tenant)
    .bind(name)
    .bind(&version)
    .bind(previous.get::<String, _>("title"))
    .bind(previous.get::<String, _>("source"))
    .bind(previous.get::<Json<serde_json::Value>, _>("manifest"))
    .bind(previous.get::<Json<serde_json::Value>, _>("courses"))
    .bind(previous.get::<Json<serde_json::Value>, _>("paths"))
    .execute(&mut *tx)
    .await?;
    let courses = previous.get::<Json<Vec<Course>>, _>("courses").0;
    let paths = previous.get::<Json<Vec<LearningPath>>, _>("paths").0;
    let items = courses.iter().map(|course| ("course", course.slug.as_str())).chain(paths.iter().map(|path| ("path", path.id.as_str())));
    for (kind, slug) in items {
        // Another package may have taken a name since: the version that had it cannot come back next to it.
        let inserted =
            sqlx::query("INSERT INTO package_item (tenant_id, kind, slug, package) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
                .bind(tenant)
                .bind(kind)
                .bind(slug)
                .bind(name)
                .execute(&mut *tx)
                .await?;
        if inserted.rows_affected() == 0 {
            return Err(Error::PackageConflict(format!("`{slug}` is now brought by another package of the tenant")));
        }
    }
    for image in images {
        sqlx::query("INSERT INTO package_image (tenant_id, package, course, path, media_type, content) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(tenant)
            .bind(name)
            .bind(image.get::<String, _>("course"))
            .bind(image.get::<String, _>("path"))
            .bind(image.get::<String, _>("media_type"))
            .bind(image.get::<Vec<u8>, _>("content"))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Some(version))
}

/// Removes a package from a tenant; `false` when it was not installed. Learners' progress on its courses is
/// kept: it shows again if the package comes back.
pub async fn remove(owner: &PgPool, tenant: Uuid, name: &str) -> Result<bool> {
    let mut tx = owner.begin().await?;
    let removed = sqlx::query("DELETE FROM package WHERE tenant_id = $1 AND name = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM package_previous WHERE tenant_id = $1 AND name = $2").bind(tenant).bind(name).execute(&mut *tx).await?;
    tx.commit().await?;
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
