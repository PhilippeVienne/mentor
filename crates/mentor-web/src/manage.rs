//! Management pages, for the administrators of a tenant.
//!
//! One page for now: the course packages installed for the tenant, what they hold and who is learning in
//! them. It only shows: installing, updating and removing a package is done by the platform's operator with
//! the `mentor` command, and the page gives the commands to run. The application role could not do it anyway:
//! it can only read these tables.

use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use mentor_db::packages::{Activity, Details};
use mentor_db::TenantTx;

use crate::brand::Brand;
use crate::learning::UTC_OFFSET_MINUTES;
use crate::pages::{french_date, render, sign_in_first, unavailable};
use crate::site::{Site, Viewer};
use crate::AppState;

struct CourseRow {
    slug: String,
    title: String,
    icon: String,
    published: bool,
    lessons: usize,
    labs: usize,
    has_exam: bool,
    activity: Activity,
}

struct PackageCard {
    details: Details,
    installed: String,
    courses: Vec<CourseRow>,
    lessons: usize,
    /// Where it comes from, split for display: the repository and the commit, or a directory.
    repository: Option<String>,
    commit: Option<String>,
    folder: Option<String>,
    /// What the operator types to update it; the tenant's slug is theirs to fill in.
    update_command: String,
}

/// `https://host/repo@<commit>#folder` → its three parts; a directory has none.
fn split_source(source: &str) -> (Option<String>, Option<String>, Option<String>) {
    if !source.starts_with("https://") {
        return (None, None, None);
    }
    let (rest, folder) = match source.rsplit_once('#') {
        Some((rest, folder)) => (rest, Some(folder.to_string())),
        None => (source, None),
    };
    match rest.rsplit_once('@') {
        Some((repository, commit)) if commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()) => {
            (Some(repository.to_string()), Some(commit.to_string()), folder)
        }
        _ => (Some(rest.to_string()), None, folder),
    }
}

#[derive(Template)]
#[template(path = "manage_packages.html")]
struct PackagesPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    packages: Vec<PackageCard>,
}

#[derive(Template)]
#[template(path = "forbidden.html")]
struct ForbiddenPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
}

/// What to answer instead of a management page to someone who does not administer the tenant.
fn refusal(site: &Site) -> Option<Response> {
    match &site.viewer {
        None => Some(sign_in_first(site)),
        Some(viewer) if viewer.learner.is_admin => None,
        Some(_) => Some(render(
            StatusCode::FORBIDDEN,
            ForbiddenPage {
                brand: &site.brand,
                section: "",
                viewer: site.viewer.as_ref(),
                dev_login: site.dev_login,
                has_paths: site.has_paths,
            },
        )),
    }
}

/// `GET /manage/packages/`.
pub async fn packages(site: Site, State(state): State<AppState>) -> Response {
    if let Some(answer) = refusal(&site) {
        return answer;
    }
    let loaded = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let mut cards = Vec::new();
        for details in tx.packages().await? {
            let mut courses = Vec::with_capacity(details.courses.len());
            for course in &details.courses {
                let lessons: Vec<String> = course.lessons.iter().map(|lesson| lesson.slug.clone()).collect();
                courses.push(CourseRow {
                    slug: course.slug.clone(),
                    title: course.title.clone(),
                    icon: course.icon.clone(),
                    published: course.published,
                    lessons: lessons.len(),
                    labs: course.lessons.iter().filter(|lesson| lesson.lab.is_some()).count(),
                    has_exam: course.exam.is_some(),
                    activity: tx.course_activity(&course.slug, &lessons).await?,
                });
            }
            let (repository, commit, folder) = split_source(&details.source);
            let update_command = match &repository {
                Some(repository) => {
                    let path = folder.as_ref().map_or(String::new(), |folder| format!(" --path {folder}"));
                    format!("mentor package-install {repository} --ref <version>{path} --tenant <organisation> --dry-run")
                }
                None => "mentor package-install <dossier> --tenant <organisation> --dry-run".to_string(),
            };
            cards.push(PackageCard {
                installed: french_date(details.installed_at, UTC_OFFSET_MINUTES),
                lessons: courses.iter().map(|course| course.lessons).sum(),
                courses,
                repository,
                commit,
                folder,
                update_command,
                details,
            });
        }
        tx.commit().await?;
        Ok::<_, mentor_db::Error>(cards)
    };
    let packages = match loaded.await {
        Ok(packages) => packages,
        Err(err) => return unavailable(err),
    };
    let page = PackagesPage {
        brand: &site.brand,
        section: "manage",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        packages,
    };
    render(StatusCode::OK, page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_split_into_repository_commit_and_folder() {
        let commit = "3e841648344b678c3bc291ffa0812886915cf9ee";
        assert_eq!(
            split_source(&format!("https://github.com/acme/courses@{commit}#catalogue")),
            (Some("https://github.com/acme/courses".into()), Some(commit.into()), Some("catalogue".into()))
        );
        assert_eq!(
            split_source(&format!("https://github.com/acme/courses@{commit}")),
            (Some("https://github.com/acme/courses".into()), Some(commit.into()), None)
        );
        // A directory on the operator's machine says nothing useful to an administrator of the tenant.
        assert_eq!(split_source("/srv/packages/courses"), (None, None, None));
    }
}
