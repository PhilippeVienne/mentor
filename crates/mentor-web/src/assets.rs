//! Images of the courses: the only files of the catalogue that are served.
//!
//! A package also holds the sources of the lessons, with the solutions of their labs, the exam pools with
//! their ticked answers, and the environments. None of that may be downloaded, and none of it is even stored
//! for serving: when a package is installed only the pictures of its `images/` folders are kept, and a
//! request is answered only for a picture of a course of the requesting tenant.
//!
//! Pictures are written by course authors, who are not trusted, and an SVG can carry a script: every answer
//! forbids running anything, so that opening a picture in a tab is harmless.

use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use mentor_content::image_media_type;
use mentor_db::TenantTx;

use crate::site::Site;
use crate::AppState;

/// A plain file or folder name: no separator, no parent folder, nothing hidden.
fn plain_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// `GET /static/catalogue/{course}/images/{*file}`: a picture of a course of this tenant.
pub async fn course_image(site: Site, State(state): State<AppState>, Path((course, file)): Path<(String, String)>) -> Response {
    let known = site.content.catalogue.courses.iter().any(|known| known.slug == course);
    if !known || image_media_type(&file).is_none() || !file.split('/').all(plain_name) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let stored = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let image = tx.course_image(&course, &file).await?;
        tx.commit().await?;
        Ok::<_, mentor_db::Error>(image)
    };
    let (media_type, bytes) = match stored.await {
        Ok(Some(image)) => image,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            eprintln!("mentor-web: picture not read: {err}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    let headers = [
        (CONTENT_TYPE, media_type),
        (X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        // Nothing runs and nothing is fetched from a picture opened on its own; inline styles draw an SVG.
        (CONTENT_SECURITY_POLICY, "default-src 'none'; style-src 'unsafe-inline'; sandbox".to_string()),
        (CACHE_CONTROL, "public, max-age=300".to_string()),
    ];
    (headers, bytes).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pictures_have_a_media_type() {
        assert_eq!(image_media_type("banniere.svg"), Some("image/svg+xml"));
        assert_eq!(image_media_type("Photo.JPG"), Some("image/jpeg"));
        for name in ["exam.md", "Dockerfile", "notes.svg.md", "script.js", "page.html", "svg"] {
            assert_eq!(image_media_type(name), None, "{name}");
        }
    }

    #[test]
    fn names_cannot_leave_the_folder_or_be_hidden() {
        assert!(plain_name("zones-git_2.svg"));
        for name in ["", "..", ".env", "a/b", "a\\b", "é.svg", "a b.svg"] {
            assert!(!plain_name(name), "{name}");
        }
    }
}
