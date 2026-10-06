//! Images of the courses: the only files of the catalogue that are served.
//!
//! The catalogue directory also holds the sources of the lessons, with the solutions of their labs, the exam
//! pools with their ticked answers, and the environments. None of that may be downloaded: a request is
//! answered only for a file of a known course, under its `images/` folder, with the extension of a picture.
//!
//! Pictures are written by course authors, who are not trusted, and an SVG can carry a script: every answer
//! forbids running anything, so that opening a picture in a tab is harmless.

use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::AppState;

/// Folder of a course whose content is public.
pub const IMAGES_FOLDER: &str = "images";

/// The media type of a picture, by its extension; `None` for anything else.
fn media_type(name: &str) -> Option<&'static str> {
    let (_, extension) = name.rsplit_once('.')?;
    Some(match extension.to_ascii_lowercase().as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        _ => return None,
    })
}

/// A plain file or folder name: no separator, no parent folder, nothing hidden.
fn plain_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// `GET /static/catalogue/{course}/images/{*file}`.
pub async fn course_image(State(state): State<AppState>, Path((course, file)): Path<(String, String)>) -> Response {
    let known = state.catalogue.courses.iter().any(|known| known.slug == course);
    let Some(media_type) = media_type(&file).filter(|_| known && file.split('/').all(plain_name)) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let images = state.catalogue_dir.join(&course).join(IMAGES_FOLDER);
    let path = images.join(&file);
    // A link inside the folder must not lead out of it.
    let inside = match (tokio::fs::canonicalize(&path).await, tokio::fs::canonicalize(&images).await) {
        (Ok(resolved), Ok(root)) => resolved.starts_with(root),
        _ => false,
    };
    let Some(bytes) = (if inside { tokio::fs::read(&path).await.ok() } else { None }) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let headers = [
        (CONTENT_TYPE, media_type),
        (X_CONTENT_TYPE_OPTIONS, "nosniff"),
        // Nothing runs and nothing is fetched from a picture opened on its own; inline styles draw an SVG.
        (CONTENT_SECURITY_POLICY, "default-src 'none'; style-src 'unsafe-inline'; sandbox"),
        (CACHE_CONTROL, "public, max-age=300"),
    ];
    (headers, bytes).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pictures_have_a_media_type() {
        assert_eq!(media_type("banniere.svg"), Some("image/svg+xml"));
        assert_eq!(media_type("Photo.JPG"), Some("image/jpeg"));
        for name in ["exam.md", "Dockerfile", "notes.svg.md", "script.js", "page.html", "svg"] {
            assert_eq!(media_type(name), None, "{name}");
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
