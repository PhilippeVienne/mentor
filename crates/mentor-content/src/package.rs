//! Course packages: a directory with a `mentor.yml` manifest and the courses it lists.
//!
//! A package is how courses are distributed: a Git repository (or a folder of one) pinned at a commit. The
//! design is in `doc/course-packages.md`. This module reads and validates the manifest, checks the files
//! of the directory and compiles its courses with the same compiler as the built-in catalogue. Fetching a
//! repository and publishing a package for a tenant are not implemented here.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as Yaml;

use crate::catalogue::{load_courses, Catalogue, COURSE_FILE};
use crate::error::{ContentError, Result};
use crate::lab::platform_checks;
use crate::paths::{load_paths, LearningPath};
use crate::tree::{verify_tree, Limits, TreeSummary};

/// Name of the manifest, at the root of a package.
pub const MANIFEST_FILE: &str = "mentor.yml";
/// The package format this compiler reads: the manifest keys and the authoring format of courses.
pub const PACKAGE_FORMAT: u64 = 1;

const MAX_COURSES: usize = 200;
const MAX_AUTHORS: usize = 50;
const MAX_BUILD_HOSTS: usize = 50;

/// Package names and course folders: lower-case words separated by single hyphens.
static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());
static VERSION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+(\.[0-9A-Za-z]+)*)?$").unwrap());
/// The characters of an SPDX licence expression (`CC-BY-SA-4.0`, `MIT OR Apache-2.0`, `LicenseRef-Acme`).
static LICENSE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9.+()-]+( [A-Za-z0-9.+()-]+)*$").unwrap());
/// A host name, or `*.` and a host name. No address, no port, no bare `*`.
static HOST_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\*\.)?([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]([a-z0-9-]*[a-z0-9])?$").unwrap());

/// Something a package needs from the platform that not every platform offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Feature {
    /// A Docker daemon inside the environment (`customizations.mentor.dockerInDocker`).
    Docker,
}

/// Content of `mentor.yml`. Unknown keys are refused: a misspelt key must not be ignored silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Version of the package format; see [`PACKAGE_FORMAT`].
    pub format: u64,
    /// Identity of the package for whoever installs it. It does not change when the repository moves.
    pub name: String,
    /// `MAJOR.MINOR.PATCH`, for people. What is installed is a commit, never a version.
    pub version: String,
    /// Display name, plain text.
    pub title: String,
    /// One sentence, plain text.
    #[serde(default)]
    pub summary: String,
    /// SPDX expression of the licence the content is distributed under.
    pub license: String,
    /// `Name <address>` or a name, plain text.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Course folders, in display order. A folder that is not listed is not compiled.
    pub courses: Vec<String>,
    /// What the environments need from the platform.
    #[serde(default)]
    pub features: Vec<Feature>,
    /// Hosts the image builds reach, besides what the platform allows every build. Environments have no
    /// network at run time, whatever is written here.
    #[serde(default)]
    pub build_egress: Vec<String>,
}

/// A compiled package.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Package {
    pub manifest: Manifest,
    /// The courses of the manifest, compiled, in its order.
    pub catalogue: Catalogue,
    /// The training paths of the package (`paths.yml`, optional), over its own courses.
    pub paths: Vec<LearningPath>,
    /// What the directory holds, as counted before compiling.
    pub tree: TreeSummary,
}

fn plain_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

/// First value that appears twice.
fn duplicate<T: Ord>(values: &[T]) -> Option<&T> {
    let mut seen = BTreeSet::new();
    values.iter().find(|value| !seen.insert(*value))
}

impl Manifest {
    /// Parses and validates the text of a manifest; `path` only names the file in diagnostics.
    pub fn parse(source: &str, path: &Path) -> Result<Self> {
        let fail = |message: String| Err(ContentError::new(path, message));
        // The format is read first: a manifest of a later format has keys this compiler does not know, and
        // "unsupported format" is the useful message, not "unknown key".
        let raw: Yaml = match serde_yaml::from_str(source) {
            Ok(raw) => raw,
            Err(err) => return fail(format!("invalid YAML: {err}")),
        };
        let Yaml::Mapping(map) = &raw else {
            return fail("the manifest must be a YAML mapping (`format`, `name`, `version`, `title`, `license`, `courses`)".into());
        };
        match map.get("format") {
            None => return fail("`format` is required: write `format: 1`".into()),
            Some(format) if format.as_u64() == Some(PACKAGE_FORMAT) => {}
            Some(format) => {
                let written = serde_yaml::to_string(format).unwrap_or_default();
                return fail(format!(
                    "unsupported package format `{}`: this version of Mentor reads format {PACKAGE_FORMAT}",
                    written.trim()
                ));
            }
        }
        // Parsed again from the text so that messages keep their line and column.
        let manifest: Manifest = match serde_yaml::from_str(source) {
            Ok(manifest) => manifest,
            Err(err) => return fail(format!("invalid manifest: {err}")),
        };
        manifest.validate().map_err(|message| ContentError::new(path, message))?;
        Ok(manifest)
    }

    fn validate(&self) -> std::result::Result<(), String> {
        if self.name.len() > 64 || !SLUG_RE.is_match(&self.name) {
            return Err(format!(
                "`name` must be lower-case letters, digits and single hyphens, 64 characters at most (got `{}`)",
                self.name
            ));
        }
        if self.version.len() > 64 || !VERSION_RE.is_match(&self.version) {
            return Err(format!("`version` must be written `MAJOR.MINOR.PATCH`, for instance `1.0.0` (got `{}`)", self.version));
        }
        if !plain_text(&self.title, 200) {
            return Err("`title` must be a text of 1 to 200 characters".into());
        }
        if !self.summary.is_empty() && !plain_text(&self.summary, 500) {
            return Err("`summary` must be a text of 500 characters at most".into());
        }
        if self.license.len() > 100 || !LICENSE_RE.is_match(&self.license) {
            return Err(format!(
                "`license` must be an SPDX expression such as `CC-BY-SA-4.0` or `LicenseRef-Proprietary` (got `{}`)",
                self.license
            ));
        }
        if self.authors.len() > MAX_AUTHORS || self.authors.iter().any(|author| !plain_text(author, 200)) {
            return Err(format!("`authors` must be a list of at most {MAX_AUTHORS} texts of 1 to 200 characters"));
        }
        if self.courses.is_empty() || self.courses.len() > MAX_COURSES {
            return Err(format!("`courses` must list 1 to {MAX_COURSES} course folders"));
        }
        if let Some(course) = self.courses.iter().find(|course| course.len() > 64 || !SLUG_RE.is_match(course)) {
            return Err(format!(
                "`courses`: `{course}` is not a course folder name (lower-case letters, digits and single hyphens, directly under the package root)"
            ));
        }
        if let Some(course) = duplicate(&self.courses) {
            return Err(format!("`courses`: `{course}` is listed twice"));
        }
        if duplicate(&self.features).is_some() {
            return Err("`features`: a feature is listed twice".into());
        }
        if self.build_egress.len() > MAX_BUILD_HOSTS {
            return Err(format!("`build_egress` must list at most {MAX_BUILD_HOSTS} hosts"));
        }
        if let Some(host) = self.build_egress.iter().find(|host| host.len() > 253 || !HOST_RE.is_match(host)) {
            return Err(format!(
                "`build_egress`: `{host}` is not a host name (write `deb.debian.org` or `*.debian.org`: no scheme, port, path or address)"
            ));
        }
        if let Some(host) = duplicate(&self.build_egress) {
            return Err(format!("`build_egress`: `{host}` is listed twice"));
        }
        Ok(())
    }
}

/// Reads and validates the manifest of the package in `directory`, without looking at its courses.
pub fn load_manifest(directory: &Path) -> Result<Manifest> {
    let path = directory.join(MANIFEST_FILE);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| ContentError::new(directory, format!("not a package: `{MANIFEST_FILE}` is missing at the root of the directory")))?;
    let limit = Limits::default().source_bytes;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(ContentError::new(&path, format!("the manifest must be an ordinary file of at most {limit} bytes")));
    }
    let source = fs::read_to_string(&path).map_err(|err| ContentError::new(&path, format!("cannot be read: {err}")))?;
    Manifest::parse(&source, &path)
}

/// Compiles the package in `directory` with the default [`Limits`].
///
/// In order: the manifest is validated, every file of the directory is checked (see [`verify_tree`]), then
/// the listed courses are compiled. Checks are those of the platform: a package cannot declare its own,
/// and a `_checks.yml` it would ship is not read.
pub fn load_package(directory: &Path) -> Result<Package> {
    load_package_with(directory, &Limits::default())
}

/// Same as [`load_package`], with other limits.
pub fn load_package_with(directory: &Path, limits: &Limits) -> Result<Package> {
    let manifest = load_manifest(directory)?;
    let tree = verify_tree(directory, limits)?;
    let catalogue = load_courses(directory, &manifest.courses, &platform_checks())?;
    for course in &catalogue.courses {
        if let Some(unknown) = course.requires.iter().find(|required| !manifest.courses.contains(required)) {
            return Err(ContentError::new(
                &directory.join(&course.slug).join(COURSE_FILE),
                format!("unknown prerequisite `{unknown}`: a course can only require courses of its own package"),
            ));
        }
    }
    let paths = load_paths(directory, &catalogue.courses)?;
    Ok(Package { manifest, catalogue, paths, tree })
}

/// Folder of a course whose content is public: its pictures. Everything else in a package (lesson sources
/// with lab solutions, exam pools with their answers, environments) is never given out.
pub const IMAGES_FOLDER: &str = "images";

/// The media type of a picture, by its extension; `None` for any other file.
pub fn image_media_type(name: &str) -> Option<&'static str> {
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

/// A picture of a course, as it is stored and served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub course: String,
    /// Path under the course's `images/` folder, with `/` separators.
    pub path: String,
    pub media_type: &'static str,
    pub content: Vec<u8>,
}

/// Reads the pictures of every course of a package whose tree was verified (no link, bounded sizes): the
/// files of its `images/` folders that have the extension of a picture.
pub fn package_images(directory: &Path, package: &Package) -> Result<Vec<Image>> {
    fn walk(course: &str, folder: &Path, prefix: &str, images: &mut Vec<Image>) -> Result<()> {
        let unreadable = |err: std::io::Error| ContentError::new(folder, format!("cannot be read: {err}"));
        let mut entries: Vec<_> = std::fs::read_dir(folder).map_err(unreadable)?.filter_map(|entry| entry.ok()).collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let relative = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            if path.is_dir() {
                walk(course, &path, &relative, images)?;
            } else if let Some(media_type) = image_media_type(&name) {
                let content = std::fs::read(&path).map_err(|err| ContentError::new(&path, format!("cannot be read: {err}")))?;
                images.push(Image { course: course.to_string(), path: relative, media_type, content });
            }
        }
        Ok(())
    }
    let mut images = Vec::new();
    for course in &package.catalogue.courses {
        let folder = directory.join(&course.slug).join(IMAGES_FOLDER);
        if folder.is_dir() {
            walk(&course.slug, &folder, "", &mut images)?;
        }
    }
    Ok(images)
}
