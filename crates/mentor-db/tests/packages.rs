//! Course packages per tenant, against a real PostgreSQL (see `common`).

mod common;

use std::path::{Path, PathBuf};

use common::database;
use mentor_content::{load_package, package_images, Image, Package};
use mentor_db::packages::{install, list, remove, Outcome};
use mentor_db::{platform, Error, TenantTx};

fn catalogue_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue")
}

/// The courses shipped with the repository, as a package with its pictures.
fn built_in() -> (Package, Vec<Image>) {
    let package = load_package(&catalogue_dir()).expect("the built-in catalogue is a valid package");
    let images = package_images(&catalogue_dir(), &package).expect("its pictures can be read");
    (package, images)
}

#[tokio::test]
async fn a_tenant_sees_only_the_packages_installed_for_it() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &[]).await.unwrap();
    let (package, images) = built_in();

    // Nothing is offered by default.
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let empty_stamp = tx.catalogue_stamp().await.unwrap();
    assert!(tx.catalogue().await.unwrap().0.courses.is_empty());
    drop(tx);

    assert_eq!(install(&db.owner, acme, &package, &images, "catalogue").await.unwrap(), Outcome::Installed);

    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let (catalogue, paths) = tx.catalogue().await.unwrap();
    // Stored compiled and read back: what the compiler produced, as it is serialised (the source text of a
    // question, kept by the compiler only to name it, is not part of the compiled course).
    assert_eq!(serde_json::to_value(&catalogue).unwrap(), serde_json::to_value(&package.catalogue).unwrap());
    assert_eq!(paths, package.paths);
    assert!(!paths.is_empty());
    assert_ne!(tx.catalogue_stamp().await.unwrap(), empty_stamp);
    let (media_type, content) = tx.course_image("git-basics", "banniere.svg").await.unwrap().expect("the banner is stored");
    assert_eq!(media_type, "image/svg+xml");
    assert!(content.starts_with(b"<svg") || content.starts_with(b"<?xml"));
    assert!(tx.course_image("git-basics", "../exam.md").await.unwrap().is_none());
    drop(tx);

    // The other tenant has no course, no path and no picture.
    let mut tx = TenantTx::begin(&db.app, globex).await.unwrap();
    assert!(tx.catalogue().await.unwrap().0.courses.is_empty());
    assert!(tx.course_image("git-basics", "banniere.svg").await.unwrap().is_none());
    assert_eq!(tx.catalogue_stamp().await.unwrap(), empty_stamp);
    drop(tx);
    assert!(list(&db.owner, globex).await.unwrap().is_empty());

    let listed = list(&db.owner, acme).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        (listed[0].name.as_str(), listed[0].courses, listed[0].paths),
        ("mentor-courses", package.catalogue.courses.len(), package.paths.len())
    );
}

#[tokio::test]
async fn installing_again_replaces_and_two_packages_cannot_bring_the_same_course() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let (mut package, images) = built_in();
    install(&db.owner, acme, &package, &images, "catalogue").await.unwrap();
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let first_stamp = tx.catalogue_stamp().await.unwrap();
    drop(tx);

    // A new version of the same package: replaced as a whole, a removed course is gone.
    package.manifest.version = "9.9.9".into();
    let removed = package.catalogue.courses.pop().unwrap().slug;
    package.paths.clear();
    assert_eq!(install(&db.owner, acme, &package, &images, "catalogue").await.unwrap(), Outcome::Replaced);
    let listed = list(&db.owner, acme).await.unwrap();
    assert_eq!((listed.len(), listed[0].version.as_str(), listed[0].paths), (1, "9.9.9", 0));
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert!(tx.catalogue().await.unwrap().0.courses.iter().all(|course| course.slug != removed));
    assert_ne!(tx.catalogue_stamp().await.unwrap(), first_stamp);
    drop(tx);

    // Another package with the same courses: refused, and nothing of it is left behind.
    let mut twin = package.clone();
    twin.manifest.name = "twin".into();
    match install(&db.owner, acme, &twin, &[], "elsewhere").await {
        Err(Error::PackageConflict(message)) => assert!(message.contains("already brought by the package `mentor-courses`"), "{message}"),
        other => panic!("expected a conflict, got {other:?}"),
    }
    assert_eq!(list(&db.owner, acme).await.unwrap().len(), 1);

    // Another tenant may install the very same package.
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &[]).await.unwrap();
    assert_eq!(install(&db.owner, globex, &twin, &[], "elsewhere").await.unwrap(), Outcome::Installed);

    assert!(remove(&db.owner, acme, "mentor-courses").await.unwrap());
    assert!(!remove(&db.owner, acme, "mentor-courses").await.unwrap());
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert!(tx.catalogue().await.unwrap().0.courses.is_empty());
    assert!(tx.course_image("git-basics", "banniere.svg").await.unwrap().is_none());
}

#[tokio::test]
async fn the_application_cannot_change_a_tenants_packages() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let (package, _) = built_in();
    install(&db.owner, acme, &package, &[], "catalogue").await.unwrap();

    // Installing is platform administration: the role the web server uses may only read.
    for statement in [
        "DELETE FROM package",
        "UPDATE package SET title = 'x'",
        "DELETE FROM package_item",
        "INSERT INTO package_image (tenant_id, package, course, path, media_type, content) SELECT tenant_id, name, 'c', 'p', 't', '' FROM package",
    ] {
        let mut connection = db.app.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)").bind(acme.to_string()).execute(&mut *connection).await.unwrap();
        let result = sqlx::query(statement).execute(&mut *connection).await;
        assert!(result.is_err(), "`{statement}` should be refused to the application role");
    }
    assert_eq!(list(&db.owner, acme).await.unwrap().len(), 1);
}
