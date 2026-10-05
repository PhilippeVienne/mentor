//! Pages: home, catalogue, course, lesson.

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use mentor_content::{Course, Lesson};
use mentor_core::gamification::{XP_LESSON, XP_QUIZ, XP_TASK};
use mentor_core::progress::LessonProgress;
use mentor_db::TenantTx;

use crate::brand::Brand;
use crate::learning::rules;
use crate::site::{Site, Viewer};
use crate::AppState;

/// Number of courses shown on the home page.
const HOME_COURSES: usize = 8;

/// A course as shown in a grid.
struct Card<'a> {
    slug: &'a str,
    title: &'a str,
    icon: &'a str,
    summary: &'a str,
    accent: &'a str,
    banner: Option<String>,
    published: bool,
    lessons_label: String,
    minutes: u32,
    requires: &'a [String],
}

/// « 1 leçon », « 7 leçons ».
fn lessons_label(count: usize) -> String {
    format!("{count} leçon{}", if count > 1 { "s" } else { "" })
}

fn banner_url(course: &Course) -> Option<String> {
    (!course.banner.is_empty()).then(|| format!("/static/catalogue/{}/{}", course.slug, course.banner))
}

fn minutes(course: &Course) -> u32 {
    course.lessons.iter().map(|lesson| lesson.minutes).sum()
}

impl<'a> Card<'a> {
    fn new(course: &'a Course) -> Self {
        Self {
            slug: &course.slug,
            title: &course.title,
            icon: &course.icon,
            summary: &course.summary,
            accent: &course.accent,
            banner: banner_url(course),
            published: course.published,
            lessons_label: lessons_label(course.lessons.len()),
            minutes: minutes(course),
            requires: &course.requires,
        }
    }
}

struct Stats {
    courses: usize,
    lessons: usize,
    hours: u32,
    exams: usize,
}

#[derive(Template)]
#[template(path = "landing.html")]
struct LandingPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    stats: Stats,
    cards: Vec<Card<'a>>,
}

#[derive(Template)]
#[template(path = "catalogue.html")]
struct CataloguePage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    cards: Vec<Card<'a>>,
    has_upcoming: bool,
}

#[derive(Template)]
#[template(path = "course.html")]
struct CoursePage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    course: &'a Course,
    banner: Option<String>,
    lessons_label: String,
    minutes: u32,
}

#[derive(Template)]
#[template(path = "lesson.html")]
struct LessonPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    course: &'a Course,
    lesson: &'a Lesson,
    /// Position of the lesson in its course, starting at 1.
    index: usize,
    previous: Option<&'a str>,
    next: Option<&'a str>,
    /// The lesson has a lab, which this version does not run yet.
    lab_pending: bool,
    has_quiz: bool,
    /// What the lesson script needs, as JSON safe to embed in a `<script>` element; `None` for a visitor.
    lesson_data: Option<String>,
}

/// JSON that can be written inside a `<script>` element: the characters that could end the element or start
/// markup are written as escapes.
fn script_json(value: &serde_json::Value) -> String {
    value.to_string().replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026")
}

/// What the lesson script needs to run the quiz and report progress.
fn lesson_data(course: &Course, lesson: &Lesson, progress: &LessonProgress, next_url: &str) -> serde_json::Value {
    let rules = rules(course, lesson);
    serde_json::json!({
        "course": { "slug": course.slug, "title": course.title, "engine": course.engine, "accent": course.accent },
        "lesson": {
            "slug": lesson.slug,
            "title": lesson.title,
            "tasks": rules.tasks,
            "labRequired": rules.lab_required(),
            "tasksDone": progress.tasks_done,
            "quizBest": progress.quiz_best,
            "completed": progress.completed,
            "nextUrl": next_url,
            "xp": { "task": XP_TASK, "quiz": XP_QUIZ, "lesson": XP_LESSON },
        },
        // Labs are not run by this server yet.
        "lab": null,
        // Lesson quizzes are self-assessment: answers are checked in the browser, as in v1. Exams are not.
        "quiz": lesson.quiz,
        "apiUrl": "/api/progress",
    })
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
}

fn render(status: StatusCode, page: impl Template) -> Response {
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(err) => {
            eprintln!("mentor-web: template error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn missing(site: &Site) -> Response {
    render(StatusCode::NOT_FOUND, NotFoundPage { brand: &site.brand, section: "", viewer: site.viewer.as_ref(), dev_login: site.dev_login })
}

/// Published courses come first: the home page only shows the first few.
fn cards(state: &AppState) -> Vec<Card<'_>> {
    let courses = &state.catalogue.courses;
    courses.iter().filter(|course| course.published).chain(courses.iter().filter(|course| !course.published)).map(Card::new).collect()
}

pub async fn landing(site: Site, State(state): State<AppState>) -> Response {
    let published: Vec<&Course> = state.catalogue.courses.iter().filter(|course| course.published).collect();
    let stats = Stats {
        courses: published.len(),
        lessons: published.iter().map(|course| course.lessons.len()).sum(),
        hours: (published.iter().map(|course| minutes(course)).sum::<u32>() + 30) / 60,
        exams: published.iter().filter(|course| course.exam.is_some()).count(),
    };
    let mut cards = cards(&state);
    cards.truncate(HOME_COURSES);
    render(
        StatusCode::OK,
        LandingPage { brand: &site.brand, section: "home", viewer: site.viewer.as_ref(), dev_login: site.dev_login, stats, cards },
    )
}

pub async fn catalogue(site: Site, State(state): State<AppState>) -> Response {
    let cards = cards(&state);
    let has_upcoming = cards.iter().any(|card| !card.published);
    render(
        StatusCode::OK,
        CataloguePage {
            brand: &site.brand,
            section: "catalogue",
            viewer: site.viewer.as_ref(),
            dev_login: site.dev_login,
            cards,
            has_upcoming,
        },
    )
}

/// A course that can be read: unpublished ones do not exist as far as visitors are concerned.
fn published_course<'a>(state: &'a AppState, slug: &str) -> Option<&'a Course> {
    state.catalogue.courses.iter().find(|course| course.slug == slug && course.published)
}

pub async fn course(site: Site, State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let Some(course) = published_course(&state, &slug) else { return missing(&site) };
    let page = CoursePage {
        brand: &site.brand,
        section: "catalogue",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        course,
        banner: banner_url(course),
        lessons_label: lessons_label(course.lessons.len()),
        minutes: minutes(course),
    };
    render(StatusCode::OK, page)
}

pub async fn lesson(site: Site, State(state): State<AppState>, Path((course_slug, lesson_slug)): Path<(String, String)>) -> Response {
    let Some(course) = published_course(&state, &course_slug) else { return missing(&site) };
    let Some(position) = course.lessons.iter().position(|lesson| lesson.slug == lesson_slug) else { return missing(&site) };
    let lesson = &course.lessons[position];
    let next = course.lessons.get(position + 1).map(|next| next.slug.as_str());
    let next_url = next.map_or(format!("/courses/{}/", course.slug), |slug| format!("/courses/{}/{slug}/", course.slug));
    let lesson_data = match &site.viewer {
        Some(viewer) => {
            let progress = async {
                let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
                let progress = tx.lesson_progress(viewer.learner.id, &rules(course, lesson)).await?;
                tx.commit().await?;
                Ok::<_, mentor_db::Error>(progress)
            };
            match progress.await {
                Ok(progress) => Some(script_json(&lesson_data(course, lesson, &progress, &next_url))),
                Err(err) => {
                    eprintln!("mentor-web: progress not loaded: {err}");
                    return StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
            }
        }
        None => None,
    };
    let page = LessonPage {
        brand: &site.brand,
        section: "catalogue",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        course,
        lesson,
        index: position + 1,
        previous: position.checked_sub(1).map(|i| course.lessons[i].slug.as_str()),
        next,
        lab_pending: lesson.lab.is_some(),
        has_quiz: !lesson.quiz.is_empty(),
        lesson_data,
    };
    render(StatusCode::OK, page)
}

pub async fn not_found(site: Site) -> Response {
    missing(&site)
}

/// Liveness and database reachability, without a tenant.
pub async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "ok\n".into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "database unreachable\n").into_response(),
    }
}
