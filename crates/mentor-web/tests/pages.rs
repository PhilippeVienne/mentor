//! Pages served per tenant, against a real PostgreSQL and the repository's catalogue.
//!
//! Needs `MENTOR_TEST_DATABASE_URL` (a superuser URL, see the README); skipped without it. As in `mentor-db`,
//! migrations run as an ordinary owner and the server connects as an unprivileged application role.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use mentor_db::platform;
use mentor_web::{router, AppState};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const PASSWORD: &str = "mentor-test";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

async fn ensure_role(admin: &PgPool, definition: &str) {
    if let Err(err) = sqlx::query(&format!("CREATE ROLE {definition}")).execute(admin).await {
        let message = err.to_string();
        assert!(message.contains("already exists") || message.contains("duplicate"), "{message}");
    }
}

/// A server with three tenants: `acme.test` with its own brand, `plain.test` with none, and `hostile.test`
/// whose brand texts contain markup.
async fn app() -> Option<Router> {
    let Ok(url) = std::env::var("MENTOR_TEST_DATABASE_URL") else {
        eprintln!("skipped: MENTOR_TEST_DATABASE_URL is not set");
        return None;
    };
    let admin_options: PgConnectOptions = url.parse().unwrap();
    let admin = PgPoolOptions::new().max_connections(1).connect_with(admin_options.clone()).await.unwrap();
    ensure_role(&admin, &format!("mentor_test_owner LOGIN CREATEROLE PASSWORD '{PASSWORD}'")).await;
    let name = format!("mentor_web_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name} OWNER mentor_test_owner")).execute(&admin).await.unwrap();
    let connect = |login: &str| admin_options.clone().database(&name).username(login).password(PASSWORD).disable_statement_logging();
    let owner = PgPoolOptions::new().max_connections(2).connect_with(connect("mentor_test_owner")).await.unwrap();
    mentor_db::migrate(&owner).await.unwrap();
    ensure_role(&admin, &format!("mentor_test_app LOGIN PASSWORD '{PASSWORD}' IN ROLE mentor_app")).await;

    let acme = platform::create_tenant(&owner, "acme", "Acme", &["acme.test"]).await.unwrap();
    let brand =
        serde_json::json!({"name": "Acme Academy", "organisation": "Acme Corp", "contact_email": "help@acme.test", "primary": "#112233"});
    platform::set_branding(&owner, acme, &brand).await.unwrap();
    platform::create_tenant(&owner, "plain", "Plain", &["plain.test"]).await.unwrap();
    // A tenant whose administrator typed markup everywhere a brand accepts text.
    let hostile = platform::create_tenant(&owner, "hostile", "<script>alert(1)</script>", &["hostile.test"]).await.unwrap();
    let markup = serde_json::json!({"tagline": "<img src=x onerror=alert(2)>", "organisation": "\"><script>alert(3)</script>", "contact_email": "x\" onclick=\"alert(4)"});
    platform::set_branding(&owner, hostile, &markup).await.unwrap();

    let db = PgPoolOptions::new().max_connections(4).connect_with(connect("mentor_test_app")).await.unwrap();
    let catalogue_dir = root().join("catalogue");
    let catalogue = Arc::new(mentor_content::load_catalogue(&catalogue_dir).expect("the catalogue compiles"));
    Some(router(AppState { db, catalogue }, &root().join("static"), &catalogue_dir))
}

async fn get(app: &Router, host: &str, path: &str) -> (StatusCode, String) {
    let request = Request::builder().uri(path).header("host", host).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 16 * 1024 * 1024).await.unwrap();
    (status, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn each_host_gets_its_own_brand() {
    let Some(app) = app().await else { return };
    let (status, acme) = get(&app, "acme.test", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(acme.contains("Acme Academy") && acme.contains("Acme Corp") && acme.contains("help@acme.test"));
    assert!(acme.contains("--primary: #112233"));

    let (status, plain) = get(&app, "PLAIN.test:8300", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(plain.contains("<strong>Plain</strong>") && plain.contains("--primary: #4f46e5"));
    assert!(!plain.contains("Acme") && !plain.contains("mailto:"));
}

#[tokio::test]
async fn an_unknown_host_is_served_nothing() {
    let Some(app) = app().await else { return };
    for path in ["/", "/catalogue/", "/courses/git-basics/"] {
        let (status, body) = get(&app, "elsewhere.test", path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(!body.contains("Acme") && !body.contains("Plain") && !body.contains("<html"), "{path}");
    }
    // The health check is the one route that needs no tenant.
    assert_eq!(get(&app, "elsewhere.test", "/healthz").await.0, StatusCode::OK);
}

#[tokio::test]
async fn catalogue_course_and_lesson_pages_render() {
    let Some(app) = app().await else { return };
    let (status, catalogue) = get(&app, "acme.test", "/catalogue/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(catalogue.contains(r#"data-course="git-basics""#) && catalogue.contains(r#"href="/courses/git-basics/""#));
    // Unpublished courses are listed as upcoming, without a link.
    assert!(catalogue.contains("À venir") && !catalogue.contains(r#"href="/courses/django/""#));

    let (status, course) = get(&app, "acme.test", "/courses/git-basics/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(course.contains("<h1>Git basics</h1>") && course.contains(r#"href="/courses/git-basics/introduction/""#));
    assert!(course.contains("/static/catalogue/git-basics/images/banniere.svg"));

    let (status, lesson) = get(&app, "acme.test", "/courses/git-basics/introduction/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(lesson.contains("<h1>Pourquoi Git ?</h1>") && lesson.contains("Leçon 1 / 7"));
    assert!(lesson.contains("Leçon suivante") && !lesson.contains("Leçon précédente"));
    // The lesson body is compiled HTML, inserted as is.
    assert!(lesson.contains(r#"<div class="cards">"#));
    assert!(lesson.contains("Labo et quiz à venir"));
}

#[tokio::test]
async fn missing_pages_answer_404_with_the_brand() {
    let Some(app) = app().await else { return };
    for path in ["/courses/nope/", "/courses/git-basics/nope/", "/courses/django/", "/nowhere"] {
        let (status, body) = get(&app, "acme.test", path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(body.contains("Page introuvable") && body.contains("Acme Academy"), "{path}");
    }
}

#[tokio::test]
async fn static_files_and_catalogue_images_are_served() {
    let Some(app) = app().await else { return };
    assert_eq!(get(&app, "acme.test", "/static/css/portail.css").await.0, StatusCode::OK);
    assert_eq!(get(&app, "acme.test", "/static/brand/logo.svg").await.0, StatusCode::OK);
    let (status, image) = get(&app, "acme.test", "/static/catalogue/git-basics/images/banniere.svg").await;
    assert_eq!(status, StatusCode::OK);
    assert!(image.contains("<svg"));
    assert_eq!(get(&app, "acme.test", "/static/css/../../Cargo.toml").await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn text_from_the_brand_is_escaped() {
    let Some(app) = app().await else { return };
    let (status, page) = get(&app, "hostile.test", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !page.contains("<script>alert") && !page.contains("<img src=x") && !page.contains("onclick=\"alert"),
        "markup from the brand reached the page"
    );
    assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt;") || page.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
}
