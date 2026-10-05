//! Web server of Mentor.
//!
//! Read-only for now: it resolves the tenant from the request's host name and serves the home page, the
//! catalogue, courses and lessons with that tenant's brand. There is no sign-in, no progress, and labs and
//! quizzes are not interactive yet.
//!
//! One catalogue is loaded at start-up and shown to every tenant; per-tenant catalogues come later. Lesson
//! HTML is trusted as compiled: it is not sanitised yet, which is acceptable only while catalogues are written
//! by the platform operator.

pub mod brand;
mod pages;
mod site;

use std::path::Path;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use mentor_content::Catalogue;
use sqlx::PgPool;
use tower_http::services::ServeDir;

/// What every request handler needs.
#[derive(Clone)]
pub struct AppState {
    /// Pool of the application role, subject to row-level security.
    pub db: PgPool,
    pub catalogue: Arc<Catalogue>,
}

/// Builds the application. `static_dir` holds the style sheets, scripts and default brand images;
/// `catalogue_dir` is served for the images that lessons refer to.
pub fn router(state: AppState, static_dir: &Path, catalogue_dir: &Path) -> Router {
    Router::new()
        .route("/", get(pages::landing))
        .route("/catalogue/", get(pages::catalogue))
        .route("/courses/{course}/", get(pages::course))
        .route("/courses/{course}/{lesson}/", get(pages::lesson))
        .route("/healthz", get(pages::health))
        .nest_service("/static/catalogue", ServeDir::new(catalogue_dir))
        .nest_service("/static", ServeDir::new(static_dir))
        .fallback(pages::not_found)
        .with_state(state)
}
