//! Web server of Mentor.
//!
//! It resolves the tenant from the request's host name and serves the home page, the catalogue, courses and
//! lessons with that tenant's brand. A signed-in learner can answer lesson quizzes and earns XP, levels and
//! badges, follows them on a dashboard, and can validate a course through its exam. Training paths, which
//! arrange courses towards a goal, are drawn as maps (see [`path_map`]). Sign-in is a development one for now (see [`dev`]); labs are not run yet.
//!
//! Each tenant has its own catalogue: the courses of the packages installed for it, read from the database
//! (see [`Content`]); nothing is offered by default. Lesson HTML is inserted as compiled: `mentor-content`
//! filters what authors wrote when it compiles a package.

mod api;
mod assets;
pub mod brand;
mod dev;
mod learning;
mod manage;
mod pages;
mod path_map;
mod paths;
pub mod session;
mod site;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use axum::routing::{get, post};
use axum::Router;
use mentor_content::{Catalogue, LearningPath};
use mentor_db::badges::CatalogueView;
use sqlx::PgPool;
use tower_http::services::ServeDir;
use uuid::Uuid;

/// What a tenant's learners can follow: the courses and training paths of the packages installed for it.
pub struct Content {
    pub catalogue: Catalogue,
    /// The catalogue as the progress rules see it.
    pub view: CatalogueView,
    pub paths: Vec<LearningPath>,
}

impl Content {
    pub fn new(catalogue: Catalogue, paths: Vec<LearningPath>) -> Self {
        Self { view: learning::view(&catalogue), catalogue, paths }
    }
}

/// The catalogue of each tenant, kept in memory with the stamp it was read at (see `site`).
pub(crate) type Contents = Arc<RwLock<HashMap<Uuid, (String, Arc<Content>)>>>;

/// What every request handler needs.
#[derive(Clone)]
pub struct AppState {
    /// Pool of the application role, subject to row-level security.
    pub db: PgPool,
    /// Key that signs session cookies.
    pub secret: Arc<Vec<u8>>,
    /// Whether the password-less development sign-in is offered. Never on a reachable deployment.
    pub dev_login: bool,
    pub(crate) contents: Contents,
}

impl AppState {
    pub fn new(db: PgPool, secret: Vec<u8>, dev_login: bool) -> Self {
        Self { db, secret: Arc::new(secret), dev_login, contents: Contents::default() }
    }
}

/// Builds the application. `static_dir` holds the style sheets, scripts and default brand images.
pub fn router(state: AppState, static_dir: &Path) -> Router {
    let mut app = Router::new()
        .route("/", get(pages::landing))
        .route("/catalogue/", get(pages::catalogue))
        .route("/courses/{course}/", get(pages::course))
        .route("/courses/{course}/{lesson}/", get(pages::lesson))
        .route("/paths/", get(paths::index))
        .route("/paths/{path}/", get(paths::path))
        .route("/dashboard/", get(pages::dashboard))
        .route("/badges/", get(pages::badges))
        .route("/manage/packages/", get(manage::packages))
        .route("/healthz", get(pages::health))
        .route("/courses/{course}/exam/", get(pages::exam))
        .route("/api/progress", post(api::progress))
        .route("/api/exam/{course}/start", post(api::exam_start))
        .route("/api/exam/{course}/submit", post(api::exam_submit))
        .route("/logout", post(dev::sign_out));
    if state.dev_login {
        app = app.route("/dev/login", get(dev::form).post(dev::sign_in));
    }
    // Of the catalogue, only the pictures of the courses are served: its other files hold lab solutions and
    // exam answers (see `assets`).
    app.route("/static/catalogue/{course}/images/{*file}", get(assets::course_image))
        .nest_service("/static", ServeDir::new(static_dir))
        .fallback(pages::not_found)
        .with_state(state)
}
