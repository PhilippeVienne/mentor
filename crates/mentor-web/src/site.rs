//! The site a request is addressed to: its tenant and brand, resolved from the host name, and the learner
//! signed in on it, if any.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::FromRequestParts;
use axum::http::header::{COOKIE, HOST, ORIGIN};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mentor_core::gamification::{level_info, LevelInfo, DEFAULT_LEVEL_TITLES};
use mentor_db::{Learner, TenantTx};
use uuid::Uuid;

use crate::brand::Brand;
use crate::{session, AppState};

/// A signed-in learner and where they stand.
pub struct Viewer {
    pub learner: Learner,
    pub level: LevelInfo,
}

/// Tenant and brand of the current request, and its viewer.
pub struct Site {
    pub tenant: Uuid,
    pub brand: Brand,
    pub viewer: Option<Viewer>,
    /// The development sign-in page exists on this server.
    pub dev_login: bool,
    /// The catalogue has training paths: the navigation links to them.
    pub has_paths: bool,
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

pub fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs() as i64)
}

/// `Host` without its port, in lower case.
fn host_name(host: &str) -> Option<String> {
    let name = match host.strip_prefix('[') {
        // IPv6 literal: `[::1]:8080`.
        Some(rest) => rest.split(']').next()?,
        None => host.split(':').next()?,
    };
    (!name.is_empty()).then(|| name.to_lowercase())
}

/// Whether a state-changing request comes from this very site. Browsers send `Origin` with every such
/// request; one made by another site's page carries that site's origin, and is refused. Together with the
/// `SameSite` session cookie, this is the protection against cross-site request forgery.
pub fn same_origin(parts: &Parts) -> bool {
    let header = |name| parts.headers.get(name).and_then(|value| value.to_str().ok());
    match (header(ORIGIN), header(HOST)) {
        (Some(origin), Some(host)) => origin.split_once("://").is_some_and(|(_, authority)| authority.eq_ignore_ascii_case(host)),
        _ => false,
    }
}

impl FromRequestParts<AppState> for Site {
    type Rejection = SiteError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = |name| parts.headers.get(name).and_then(|value| value.to_str().ok());
        let host = header(HOST).and_then(host_name).ok_or(SiteError::UnknownHost)?;
        let tenant = mentor_db::resolve_tenant(&state.db, &host).await?.ok_or(SiteError::UnknownHost)?;
        let mut tx = TenantTx::begin(&state.db, tenant).await?;
        let (name, settings) = tx.tenant_profile().await?.ok_or(SiteError::UnknownHost)?;

        let signed_in =
            header(COOKIE).and_then(session::from_header).and_then(|value| session::verify(&state.secret, value, tenant, now()));
        let viewer = match signed_in {
            // The learner is looked up inside this tenant: an identifier from elsewhere finds nobody.
            Some(id) => match tx.learner(id).await? {
                Some(learner) => {
                    let level = level_info(tx.total_xp(learner.id).await?, &DEFAULT_LEVEL_TITLES);
                    Some(Viewer { learner, level })
                }
                None => None,
            },
            None => None,
        };
        Ok(Site {
            tenant,
            brand: Brand::from_settings(&name, &settings),
            viewer,
            dev_login: state.dev_login,
            has_paths: !state.paths.is_empty(),
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    fn parts(headers: &[(&str, &str)]) -> Parts {
        let mut request = Request::builder().uri("/");
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        request.body(()).unwrap().into_parts().0
    }

    #[test]
    fn host_names_lose_their_port_and_case() {
        assert_eq!(host_name("Acme.Test:8300").as_deref(), Some("acme.test"));
        assert_eq!(host_name("[::1]:8300").as_deref(), Some("::1"));
        assert_eq!(host_name(""), None);
    }

    #[test]
    fn only_requests_from_the_same_origin_pass() {
        assert!(same_origin(&parts(&[("host", "acme.test:8300"), ("origin", "http://acme.test:8300")])));
        assert!(!same_origin(&parts(&[("host", "acme.test"), ("origin", "https://evil.test")])));
        // No Origin header: not a request a browser makes for a form or a fetch.
        assert!(!same_origin(&parts(&[("host", "acme.test")])));
    }
}
