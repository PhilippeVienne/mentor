//! Web server of Mentor.
//!
//! It resolves the tenant from the request's host name and serves the home page, the catalogue, courses and
//! lessons with that tenant's brand. A signed-in learner can answer lesson quizzes and earns XP, levels and
//! badges, follows them on a dashboard, and can validate a course through its exam. Sign-in is a development one for now (see [`dev`]); labs are not run yet.
//!
//! One catalogue is loaded at start-up and shown to every tenant; per-tenant catalogues come later. Lesson
//! HTML is inserted as compiled: `mentor-content` filters what authors wrote when it compiles the catalogue.

mod api;
pub mod brand;
mod dev;
mod learning;
mod pages;
pub mod session;
mod site;

use std::path::Path;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use mentor_content::Catalogue;
use mentor_db::badges::CatalogueView;
use sqlx::PgPool;
use tower_http::services::ServeDir;

/// What every request handler needs.
#[derive(Clone)]
pub struct AppState {
    /// Pool of the application role, subject to row-level security.
    pub db: PgPool,
    pub catalogue: Arc<Catalogue>,
    /// The catalogue as the progress rules see it.
    pub view: Arc<CatalogueView>,
    /// Key that signs session cookies.
    pub secret: Arc<Vec<u8>>,
    /// Whether the password-less development sign-in is offered. Never on a reachable deployment.
    pub dev_login: bool,
}

impl AppState {
    pub fn new(db: PgPool, catalogue: Catalogue, secret: Vec<u8>, dev_login: bool) -> Self {
        let view = Arc::new(learning::view(&catalogue));
        Self { db, catalogue: Arc::new(catalogue), view, secret: Arc::new(secret), dev_login }
    }
}

/// Builds the application. `static_dir` holds the style sheets, scripts and default brand images;
/// `catalogue_dir` is served for the images that lessons refer to.
pub fn router(state: AppState, static_dir: &Path, catalogue_dir: &Path) -> Router {
    let mut app = Router::new()
        .route("/", get(pages::landing))
        .route("/catalogue/", get(pages::catalogue))
        .route("/courses/{course}/", get(pages::course))
        .route("/courses/{course}/{lesson}/", get(pages::lesson))
        .route("/dashboard/", get(pages::dashboard))
        .route("/badges/", get(pages::badges))
        .route("/healthz", get(pages::health))
        .route("/courses/{course}/exam/", get(pages::exam))
        .route("/api/progress", post(api::progress))
        .route("/api/exam/{course}/start", post(api::exam_start))
        .route("/api/exam/{course}/submit", post(api::exam_submit))
        .route("/logout", post(dev::sign_out));
    if state.dev_login {
        app = app.route("/dev/login", get(dev::form).post(dev::sign_in));
    }
    app.nest_service("/static/catalogue", ServeDir::new(catalogue_dir))
        .nest_service("/static", ServeDir::new(static_dir))
        .fallback(pages::not_found)
        .with_state(state)
}
