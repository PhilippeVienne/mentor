//! Development sign-in: pick a user name, no password.
//!
//! It exists so that progress can be exercised before OIDC is there. It is mounted only when the server is
//! started with `--dev-login`, and must never be enabled on a reachable deployment: anyone could sign in as
//! anyone. Accounts created this way carry the subject `dev:<username>`.

use askama::Template;
use axum::extract::State;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Form;
use mentor_db::TenantTx;
use serde::Deserialize;

use crate::brand::Brand;
use crate::site::{now, same_origin, Site, Viewer};
use crate::{session, AppState};

#[derive(Template)]
#[template(path = "dev_login.html")]
struct LoginPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    error: &'a str,
}

fn page(site: &Site, status: StatusCode, error: &str) -> Response {
    let page = LoginPage {
        brand: &site.brand,
        section: "",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        error,
    };
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn redirect(to: &str, cookie: String) -> Response {
    (StatusCode::SEE_OTHER, [(LOCATION, to.to_string()), (SET_COOKIE, cookie)]).into_response()
}

pub async fn form(site: Site) -> Response {
    page(&site, StatusCode::OK, "")
}

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    /// Ticked to sign in as an administrator of the tenant. Like the rest of this page: development only.
    admin: Option<String>,
}

/// A user name that is safe to show and to store: short, and made of letters, digits, dots, dashes, underscores.
fn valid_username(name: &str) -> bool {
    (1..=40).contains(&name.chars().count()) && name.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

pub async fn sign_in(site: Site, State(state): State<AppState>, parts: Parts, Form(credentials): Form<Credentials>) -> Response {
    if !same_origin(&parts) {
        return page(&site, StatusCode::FORBIDDEN, "Requête refusée : origine inattendue.");
    }
    let username = credentials.username.trim();
    if !valid_username(username) {
        return page(&site, StatusCode::BAD_REQUEST, "Choisis un identifiant de 1 à 40 caractères : lettres, chiffres, point, tiret.");
    }
    let learner = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let learner = tx.upsert_learner(&format!("dev:{username}"), username, credentials.admin.is_some()).await?;
        tx.commit().await?;
        Ok::<_, mentor_db::Error>(learner)
    }
    .await;
    match learner {
        Ok(learner) => redirect("/dashboard/", session::set_cookie(&session::issue(&state.secret, site.tenant, learner.id, now()), false)),
        Err(err) => {
            eprintln!("mentor-web: sign-in failed: {err}");
            page(&site, StatusCode::SERVICE_UNAVAILABLE, "Connexion impossible pour l'instant.")
        }
    }
}

/// `POST /logout`: closes the session, whatever opened it.
pub async fn sign_out(parts: Parts) -> Response {
    if !same_origin(&parts) {
        return StatusCode::FORBIDDEN.into_response();
    }
    redirect("/", session::clear_cookie())
}
