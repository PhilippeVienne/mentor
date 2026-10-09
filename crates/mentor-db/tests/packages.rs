//! Course packages per tenant, against a real PostgreSQL (see `common`).

mod common;

use std::path::{Path, PathBuf};

use common::database;
use mentor_content::{load_package, package_images, Image, Package};
use mentor_core::progress::{Event, LessonRules, Source};
use mentor_db::packages::{install, list, preview, remove, rollback, Outcome, Removals};
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

    assert_eq!(install(&db.owner, acme, &package, &images, "catalogue", Removals::Refuse).await.unwrap(), Outcome::Installed);

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
    install(&db.owner, acme, &package, &images, "catalogue", Removals::Refuse).await.unwrap();
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let first_stamp = tx.catalogue_stamp().await.unwrap();
    drop(tx);

    // A new version of the same package: replaced as a whole, a removed course is gone (which has to be
    // confirmed, see the next test).
    package.manifest.version = "9.9.9".into();
    let removed = package.catalogue.courses.pop().unwrap().slug;
    package.paths.clear();
    assert_eq!(install(&db.owner, acme, &package, &images, "catalogue", Removals::Confirmed).await.unwrap(), Outcome::Replaced);
    let listed = list(&db.owner, acme).await.unwrap();
    assert_eq!((listed.len(), listed[0].version.as_str(), listed[0].paths), (1, "9.9.9", 0));
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    assert!(tx.catalogue().await.unwrap().0.courses.iter().all(|course| course.slug != removed));
    assert_ne!(tx.catalogue_stamp().await.unwrap(), first_stamp);
    drop(tx);

    // Another package with the same courses: refused, and nothing of it is left behind.
    let mut twin = package.clone();
    twin.manifest.name = "twin".into();
    match install(&db.owner, acme, &twin, &[], "elsewhere", Removals::Refuse).await {
        Err(Error::PackageConflict(message)) => assert!(message.contains("already brought by the package `mentor-courses`"), "{message}"),
        other => panic!("expected a conflict, got {other:?}"),
    }
    assert_eq!(list(&db.owner, acme).await.unwrap().len(), 1);

    // Another tenant may install the very same package.
    let globex = platform::create_tenant(&db.owner, "globex", "Globex", &[]).await.unwrap();
    assert_eq!(install(&db.owner, globex, &twin, &[], "elsewhere", Removals::Refuse).await.unwrap(), Outcome::Installed);

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
    install(&db.owner, acme, &package, &[], "catalogue", Removals::Refuse).await.unwrap();

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

/// Completes a lesson of `git-basics` for a new learner, as the web server does for a perfect quiz.
async fn learner_completing(db: &common::TestDb, tenant: uuid::Uuid, subject: &str, lessons: &[&str]) {
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let learner = tx.upsert_learner(subject, subject, false).await.unwrap().id;
    for lesson in lessons {
        let rules =
            LessonRules { course: "git-basics".into(), slug: lesson.to_string(), steps: vec![], questions: 3, lab_available: false };
        assert!(tx.record_event(learner, &rules, Event::Quiz(3), Source::Browser).await.unwrap().lesson_completed);
    }
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn an_update_says_what_it_changes_and_never_removes_without_being_told() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let (package, images) = built_in();
    // Nothing installed: nothing would be replaced.
    assert!(preview(&db.owner, acme, &package).await.unwrap().is_none());
    install(&db.owner, acme, &package, &images, "catalogue", Removals::Refuse).await.unwrap();
    // The same content again changes nothing for anybody.
    let same = preview(&db.owner, acme, &package).await.unwrap().unwrap();
    assert!(same.diff.is_empty() && same.installed_version == package.manifest.version);

    let git = package.catalogue.courses.iter().find(|course| course.slug == "git-basics").unwrap();
    let all: Vec<&str> = git.lessons.iter().map(|lesson| lesson.slug.as_str()).collect();
    // Alice finished the whole course, Bob only its first lesson, Carol the last one.
    learner_completing(&db, acme, "alice", &all).await;
    learner_completing(&db, acme, "bob", &all[..1]).await;
    learner_completing(&db, acme, "carol", &all[all.len() - 1..]).await;

    // Version 2 drops the last lesson of git-basics and a whole course nobody touched.
    let mut shorter = package.clone();
    shorter.manifest.version = "2.0.0".into();
    let dropped_course = shorter.catalogue.courses.pop().unwrap().slug;
    shorter.paths.clear();
    let dropped_lesson =
        shorter.catalogue.courses.iter_mut().find(|course| course.slug == "git-basics").unwrap().lessons.pop().unwrap().slug;
    let impact = preview(&db.owner, acme, &shorter).await.unwrap().unwrap();
    assert_eq!(impact.diff.courses_removed, std::slice::from_ref(&dropped_course));
    assert_eq!(impact.diff.courses_changed.len(), 1);
    assert_eq!(impact.diff.courses_changed[0].lessons_removed, std::slice::from_ref(&dropped_lesson));
    assert!(impact.diff.removes_progress());
    // Alice and Carol did that lesson; Bob did not.
    assert_eq!((impact.learners_on_removed, impact.completions_lost), (2, 0));

    match install(&db.owner, acme, &shorter, &images, "catalogue", Removals::Refuse).await {
        Err(Error::RemovalsNotConfirmed(diff)) => assert_eq!(diff, impact.diff),
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(list(&db.owner, acme).await.unwrap()[0].version, package.manifest.version, "nothing was written");

    assert_eq!(install(&db.owner, acme, &shorter, &images, "catalogue", Removals::Confirmed).await.unwrap(), Outcome::Replaced);
    // Nothing of what learners did is deleted: the lesson is gone from the catalogue, not from their history.
    let kept: i64 =
        sqlx::query_scalar("SELECT count(*) FROM lesson_progress WHERE tenant_id = $1 AND lesson = $2 AND completed_at IS NOT NULL")
            .bind(acme)
            .bind(&dropped_lesson)
            .fetch_one(&db.owner)
            .await
            .unwrap();
    assert_eq!(kept, 2);

    // Version 3 gives the lesson back and adds nothing else: adding needs no confirmation, and tells that
    // the one learner who had finished the shortened course (Alice) no longer has.
    let mut longer = shorter.clone();
    longer.manifest.version = "3.0.0".into();
    let lesson = git.lessons.last().unwrap().clone();
    longer.catalogue.courses.iter_mut().find(|course| course.slug == "git-basics").unwrap().lessons.push(lesson);
    let impact = preview(&db.owner, acme, &longer).await.unwrap().unwrap();
    assert_eq!(impact.diff.courses_changed[0].lessons_added, std::slice::from_ref(&dropped_lesson));
    assert!(!impact.diff.removes_progress());
    assert_eq!((impact.learners_on_removed, impact.completions_lost), (0, 1));
    assert_eq!(install(&db.owner, acme, &longer, &images, "catalogue", Removals::Refuse).await.unwrap(), Outcome::Replaced);
}

#[tokio::test]
async fn an_update_can_be_undone_and_redone() {
    let Some(db) = database().await else { return };
    let acme = platform::create_tenant(&db.owner, "acme", "Acme", &[]).await.unwrap();
    let (package, images) = built_in();
    install(&db.owner, acme, &package, &images, "catalogue", Removals::Refuse).await.unwrap();
    // A first installation has nothing to go back to.
    assert_eq!(rollback(&db.owner, acme, "mentor-courses").await.unwrap(), None);

    let mut second = package.clone();
    second.manifest.version = "2.0.0".into();
    let dropped = second.catalogue.courses.pop().unwrap().slug;
    second.paths.clear();
    let fewer_images: Vec<_> = images.iter().filter(|image| image.course != dropped).cloned().collect();
    install(&db.owner, acme, &second, &fewer_images, "catalogue", Removals::Confirmed).await.unwrap();
    let has_dropped = |catalogue: &mentor_content::Catalogue| catalogue.courses.iter().any(|course| course.slug == dropped);

    // Undone: courses, paths and pictures of the first version are back.
    assert_eq!(rollback(&db.owner, acme, "mentor-courses").await.unwrap().as_deref(), Some(package.manifest.version.as_str()));
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let (catalogue, paths) = tx.catalogue().await.unwrap();
    assert!(has_dropped(&catalogue) && paths == package.paths);
    assert!(tx.course_image(&dropped, "banniere.svg").await.unwrap().is_some());
    drop(tx);
    assert_eq!(list(&db.owner, acme).await.unwrap()[0].version, package.manifest.version);

    // Redone: undoing the undo gives the second version back.
    assert_eq!(rollback(&db.owner, acme, "mentor-courses").await.unwrap().as_deref(), Some("2.0.0"));
    let mut tx = TenantTx::begin(&db.app, acme).await.unwrap();
    let (catalogue, paths) = tx.catalogue().await.unwrap();
    assert!(!has_dropped(&catalogue) && paths.is_empty());
    assert!(tx.course_image(&dropped, "banniere.svg").await.unwrap().is_none());
    drop(tx);

    // Removing the package forgets its previous version too.
    assert!(remove(&db.owner, acme, "mentor-courses").await.unwrap());
    install(&db.owner, acme, &package, &images, "catalogue", Removals::Refuse).await.unwrap();
    assert_eq!(rollback(&db.owner, acme, "mentor-courses").await.unwrap(), None);
}
