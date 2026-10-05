//! The site a request is addressed to: its tenant and brand, resolved from the host name.

use axum::extract::FromRequestParts;
use axum::http::header::HOST;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mentor_db::TenantTx;

use crate::brand::Brand;
use crate::AppState;

/// The site of the current request. Only its brand is needed while pages are read-only; the tenant will be
/// kept here once pages read a learner's progress.
pub struct Site {
    pub brand: Brand,
}

/// Why a request could not be served at all.
pub enum SiteError {
    /// No tenant is served on this host name. The answer carries no brand and names no tenant.
    UnknownHost,
    Database(mentor_db::Error),
}

impl IntoResponse for SiteError {
    fn into_response(self) -> Response {
        match self {
            SiteError::UnknownHost => (StatusCode::NOT_FOUND, "No site is served on this host name.\n").into_response(),
            SiteError::Database(err) => {
                eprintln!("mentor-web: database error: {err}");
                (StatusCode::SERVICE_UNAVAILABLE, "The service is temporarily unavailable.\n").into_response()
            }
        }
    }
}

impl From<mentor_db::Error> for SiteError {
    fn from(err: mentor_db::Error) -> Self {
        SiteError::Database(err)
    }
}

/// `Host` without its port, in lower case.
fn host_name(parts: &Parts) -> Option<String> {
    let host = parts.headers.get(HOST)?.to_str().ok()?;
    let name = match host.strip_prefix('[') {
        // IPv6 literal: `[::1]:8080`.
        Some(rest) => rest.split(']').next()?,
        None => host.split(':').next()?,
    };
    (!name.is_empty()).then(|| name.to_lowercase())
}

impl FromRequestParts<AppState> for Site {
    type Rejection = SiteError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let host = host_name(parts).ok_or(SiteError::UnknownHost)?;
        let tenant = mentor_db::resolve_tenant(&state.db, &host).await?.ok_or(SiteError::UnknownHost)?;
        let mut tx = TenantTx::begin(&state.db, tenant).await?;
        let (name, settings) = tx.tenant_profile().await?.ok_or(SiteError::UnknownHost)?;
        Ok(Site { brand: Brand::from_settings(&name, &settings) })
    }
}
