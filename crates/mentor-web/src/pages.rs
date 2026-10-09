//! Pages: home, catalogue, course, lesson, and the learner's own ones (dashboard, badges). Training paths
//! have their own module, [`crate::paths`].

use std::collections::{BTreeMap, BTreeSet};

use askama::Template;
use axum::extract::{Path, State};
use axum::http::header::LOCATION;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use mentor_content::{Course, LearningPath, Lesson};
use mentor_core::badges::{default_badges, Tier, COURSE_BADGE_PREFIX};
use mentor_core::gamification::{DEFAULT_LEVEL_TITLES, LEVEL_THRESHOLDS, XP_LESSON, XP_QUIZ, XP_TASK};
use mentor_core::progress::{completed_courses, LessonProgress};
use mentor_db::progress::LessonState;
use mentor_db::TenantTx;

use crate::brand::Brand;
use crate::learning::{done_positions, rules, EXAM_COOLDOWN_SECONDS, UTC_OFFSET_MINUTES};
use crate::paths::{dashboard_cards, paths_of, PathCard};
use crate::site::now;
use crate::site::{Site, Viewer};
use crate::{AppState, Content};

/// Number of courses shown on the home page.
const HOME_COURSES: usize = 8;

/// Number of badges shown on the dashboard.
const DASHBOARD_BADGES: usize = 8;

/// What the signed-in learner has done so far.
pub(crate) struct Progress {
    /// By `course/lesson` reference; a lesson never touched is absent.
    lessons: BTreeMap<String, LessonState>,
    courses_done: BTreeSet<String>,
    /// When each owned badge was awarded, in seconds since the Unix epoch.
    badges: BTreeMap<String, i64>,
}

impl Progress {
    pub(crate) fn lesson(&self, course: &Course, lesson: &Lesson) -> LessonState {
        self.lessons.get(&format!("{}/{}", course.slug, lesson.slug)).copied().unwrap_or_default()
    }
}

/// Progress of the viewer, `None` for a visitor.
pub(crate) async fn load_progress(state: &AppState, site: &Site) -> Result<Option<Progress>, mentor_db::Error> {
    let Some(viewer) = &site.viewer else { return Ok(None) };
    let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
    let lessons = tx.lesson_states(viewer.learner.id).await?;
    let badges = tx.badge_dates(viewer.learner.id).await?;
    tx.commit().await?;
    let done = lessons.iter().filter(|(_, state)| state.completed).map(|(reference, _)| reference.clone()).collect();
    Ok(Some(Progress { courses_done: completed_courses(&site.content.view.courses, &done), lessons, badges }))
}

pub(crate) fn unavailable(err: mentor_db::Error) -> Response {
    eprintln!("mentor-web: progress not loaded: {err}");
    (StatusCode::SERVICE_UNAVAILABLE, "The service is temporarily unavailable.\n").into_response()
}

/// `new`, `started`, `done`, or `exam` (completed through the course exam): the suffix of the style classes.
fn state_name(state: LessonState) -> &'static str {
    match (state.completed, state.validated_by_exam, state.started()) {
        (true, true, _) => "exam",
        (true, false, _) => "done",
        (false, _, true) => "started",
        (false, _, false) => "new",
    }
}

/// The viewer's advancement in a course.
pub(crate) struct CourseProgress<'a> {
    pub(crate) done: usize,
    pub(crate) percent: u32,
    pub(crate) completed: bool,
    pub(crate) unlocked: bool,
    /// Titles of the required courses that are not completed yet.
    pub(crate) missing: Vec<&'a str>,
}

impl<'a> CourseProgress<'a> {
    pub(crate) fn new(content: &'a Content, course: &'a Course, progress: &Progress) -> Self {
        let total = course.lessons.len();
        let done = course.lessons.iter().filter(|lesson| progress.lesson(course, lesson).completed).count();
        let missing: Vec<&str> = course
            .requires
            .iter()
            .filter(|required| !progress.courses_done.contains(*required))
            .map(|required| content.catalogue.courses.iter().find(|c| &c.slug == required).map_or(required.as_str(), |c| c.title.as_str()))
            .collect();
        Self {
            done,
            percent: (done * 100 + total / 2).checked_div(total).unwrap_or(0) as u32,
            completed: progress.courses_done.contains(&course.slug),
            unlocked: missing.is_empty(),
            missing,
        }
    }
}

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
    /// `None` for a visitor.
    progress: Option<CourseProgress<'a>>,
    /// What the catalogue filters on.
    state: &'static str,
}

/// « 1 leçon », « 7 leçons ».
pub(crate) fn lessons_label(count: usize) -> String {
    format!("{count} leçon{}", if count > 1 { "s" } else { "" })
}

fn banner_url(course: &Course) -> Option<String> {
    (!course.banner.is_empty()).then(|| format!("/static/catalogue/{}/{}", course.slug, course.banner))
}

pub(crate) fn minutes(course: &Course) -> u32 {
    course.lessons.iter().map(|lesson| lesson.minutes).sum()
}

impl<'a> Card<'a> {
    fn new(content: &'a Content, course: &'a Course, progress: Option<&Progress>) -> Self {
        let progress = progress.map(|progress| CourseProgress::new(content, course, progress));
        let filter = match &progress {
            _ if !course.published => "bientot",
            Some(progress) if progress.completed => "termines",
            Some(progress) if progress.done > 0 => "en-cours",
            _ => "a-commencer",
        };
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
            progress,
            state: filter,
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
    has_paths: bool,
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
    has_paths: bool,
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
    has_paths: bool,
    course: &'a Course,
    banner: Option<String>,
    lessons_label: String,
    minutes: u32,
    progress: Option<CourseProgress<'a>>,
    lessons: Vec<LessonRow<'a>>,
    /// The exam of the course, for a signed-in learner.
    exam: Option<ExamInfo<'a>>,
    /// The training paths the course belongs to.
    paths: Vec<&'a LearningPath>,
}

/// The exam of a course and where the viewer stands with it.
struct ExamInfo<'a> {
    title: &'a str,
    /// Number of questions of an attempt.
    count: usize,
    pass_mark: u32,
    minutes: u32,
    /// The course is completed, whichever way.
    validated: bool,
    validated_by_exam: bool,
    locked: bool,
}

impl<'a> ExamInfo<'a> {
    fn new(content: &'a Content, course: &'a Course, progress: &Progress) -> Option<Self> {
        let exam = course.exam.as_ref()?;
        let advancement = CourseProgress::new(content, course, progress);
        Some(Self {
            title: &exam.title,
            count: (exam.draw as usize).min(exam.questions.len()),
            pass_mark: exam.pass_mark,
            minutes: exam.minutes,
            validated: advancement.completed,
            validated_by_exam: course.lessons.iter().any(|lesson| progress.lesson(course, lesson).validated_by_exam),
            locked: !advancement.unlocked,
        })
    }

    fn available(&self) -> bool {
        !self.validated && !self.locked
    }
}

#[derive(Template)]
#[template(path = "exam.html")]
struct ExamPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    course: &'a Course,
    info: ExamInfo<'a>,
    intro_html: &'a str,
    /// What the exam script needs, as JSON safe to embed in a `<script>` element.
    exam_data: String,
}

/// The introduction of an exam without the headings left at its very end: they introduced the question pool,
/// which is not shown.
fn without_trailing_headings(html: &str) -> &str {
    let mut kept = html.trim_end();
    loop {
        // Looked at as bytes: a closing heading tag is ASCII, so cutting before it stays on a character boundary.
        let closes_heading = kept.len().checked_sub(5).is_some_and(|start| {
            let tail = &kept.as_bytes()[start..];
            tail.starts_with(b"</h") && (b'1'..=b'6').contains(&tail[3]) && tail[4] == b'>'
        });
        if !closes_heading {
            return kept;
        }
        let level = &kept[kept.len() - 3..kept.len() - 1];
        let Some(start) = kept.rfind(&format!("<{level}")) else { return kept };
        kept = kept[..start].trim_end();
    }
}

/// A lesson in the programme of its course.
struct LessonRow<'a> {
    lesson: &'a Lesson,
    tasks: usize,
    questions: usize,
    /// Empty for a visitor.
    state: &'static str,
    progress: LessonState,
}

/// A lesson in the stepper at the top of a lesson page.
struct Step<'a> {
    slug: &'a str,
    title: &'a str,
    state: &'static str,
}

/// « Mes cours »: level, the lesson to resume, training paths, courses and badges.
#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    me: &'a Viewer,
    /// XP still to earn before the next level.
    to_next: Option<u32>,
    resume: Option<Resume<'a>>,
    /// Training paths, those the learner is in first.
    paths: Vec<PathCard<'a>>,
    cards: Vec<Card<'a>>,
    badges: Vec<BadgeItem>,
    badges_owned: usize,
    badges_total: usize,
}

struct Resume<'a> {
    course: &'a Course,
    lesson: &'a Lesson,
    started: bool,
}

struct BadgeItem {
    name: String,
    description: String,
    emoji: String,
    tier: &'static str,
    /// The day it was awarded, as shown; `None` while locked.
    awarded: Option<String>,
}

struct LevelRow {
    level: usize,
    title: &'static str,
    xp: u32,
}

#[derive(Template)]
#[template(path = "badges.html")]
struct BadgesPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    me: &'a Viewer,
    owned: usize,
    items: Vec<BadgeItem>,
    levels: Vec<LevelRow>,
}

const MONTHS: [&str; 12] =
    ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];

/// « 5 octobre 2026 »: the day of an instant, in the tenant's time zone.
fn french_date(epoch: i64, utc_offset_minutes: i32) -> String {
    // Civil date from a day count (Howard Hinnant's algorithm), proleptic Gregorian calendar.
    let days = (epoch + i64::from(utc_offset_minutes) * 60).div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{}{} {} {year}", day, if day == 1 { "er" } else { "" }, MONTHS[(month - 1) as usize])
}

/// Every badge of the site, rule badges first, then one per published course; owned ones carry their date.
fn badge_items(content: &Content, progress: &Progress) -> Vec<BadgeItem> {
    let awarded = |slug: &str| progress.badges.get(slug).map(|&at| french_date(at, UTC_OFFSET_MINUTES));
    let rules = default_badges().into_iter().map(|badge| BadgeItem {
        awarded: awarded(&badge.slug),
        name: badge.name,
        description: badge.description,
        emoji: badge.emoji,
        tier: match badge.tier {
            Tier::Bronze => "bronze",
            Tier::Silver => "silver",
            Tier::Gold => "gold",
        },
    });
    let courses = content.catalogue.courses.iter().filter(|course| course.published).map(|course| BadgeItem {
        awarded: awarded(&format!("{COURSE_BADGE_PREFIX}{}", course.slug)),
        name: course.title.clone(),
        description: format!("Termine le cours « {} ».", course.title),
        emoji: course.icon.clone(),
        tier: "gold",
    });
    rules.chain(courses).collect()
}

/// Where a visitor is sent when a page needs a learner.
fn sign_in_first(site: &Site) -> Response {
    let to = if site.dev_login { "/dev/login" } else { "/" };
    (StatusCode::SEE_OTHER, [(LOCATION, to)]).into_response()
}

#[derive(Template)]
#[template(path = "lesson.html")]
struct LessonPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    course: &'a Course,
    lesson: &'a Lesson,
    /// Position of the lesson in its course, starting at 1.
    index: usize,
    steps: Vec<Step<'a>>,
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
        "course": { "slug": course.slug, "title": course.title, "accent": course.accent },
        "lesson": {
            "slug": lesson.slug,
            "title": lesson.title,
            "tasks": rules.steps.len(),
            "labRequired": rules.lab_required(),
            "tasksDone": done_positions(&rules, &progress.tasks_done),
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
    has_paths: bool,
}

pub(crate) fn render(status: StatusCode, page: impl Template) -> Response {
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(err) => {
            eprintln!("mentor-web: template error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub(crate) fn missing(site: &Site) -> Response {
    render(
        StatusCode::NOT_FOUND,
        NotFoundPage {
            brand: &site.brand,
            section: "",
            viewer: site.viewer.as_ref(),
            dev_login: site.dev_login,
            has_paths: site.has_paths,
        },
    )
}

/// Published courses come first: the home page only shows the first few.
fn cards<'a>(content: &'a Content, progress: Option<&Progress>) -> Vec<Card<'a>> {
    let courses = &content.catalogue.courses;
    courses
        .iter()
        .filter(|course| course.published)
        .chain(courses.iter().filter(|course| !course.published))
        .map(|course| Card::new(content, course, progress))
        .collect()
}

pub async fn landing(site: Site, State(state): State<AppState>) -> Response {
    let content = &*site.content;
    let published: Vec<&Course> = content.catalogue.courses.iter().filter(|course| course.published).collect();
    let stats = Stats {
        courses: published.len(),
        lessons: published.iter().map(|course| course.lessons.len()).sum(),
        hours: (published.iter().map(|course| minutes(course)).sum::<u32>() + 30) / 60,
        exams: published.iter().filter(|course| course.exam.is_some()).count(),
    };
    let progress = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let mut cards = cards(content, progress.as_ref());
    cards.truncate(HOME_COURSES);
    render(
        StatusCode::OK,
        LandingPage {
            brand: &site.brand,
            section: "home",
            viewer: site.viewer.as_ref(),
            dev_login: site.dev_login,
            has_paths: site.has_paths,
            stats,
            cards,
        },
    )
}

pub async fn catalogue(site: Site, State(state): State<AppState>) -> Response {
    let content = &*site.content;
    let progress = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let cards = cards(content, progress.as_ref());
    let has_upcoming = cards.iter().any(|card| !card.published);
    render(
        StatusCode::OK,
        CataloguePage {
            brand: &site.brand,
            section: "catalogue",
            viewer: site.viewer.as_ref(),
            dev_login: site.dev_login,
            has_paths: site.has_paths,
            cards,
            has_upcoming,
        },
    )
}

/// A course that can be read: unpublished ones do not exist as far as visitors are concerned.
fn published_course<'a>(content: &'a Content, slug: &str) -> Option<&'a Course> {
    content.catalogue.courses.iter().find(|course| course.slug == slug && course.published)
}

pub async fn course(site: Site, State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let content = &*site.content;
    let Some(course) = published_course(content, &slug) else { return missing(&site) };
    let progress = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let lessons = course
        .lessons
        .iter()
        .map(|lesson| {
            let lesson_state = progress.as_ref().map(|progress| progress.lesson(course, lesson));
            LessonRow {
                lesson,
                tasks: lesson.lab.as_ref().map_or(0, |lab| lab.steps.len()),
                questions: lesson.quiz.len(),
                state: lesson_state.map_or("", state_name),
                progress: lesson_state.unwrap_or_default(),
            }
        })
        .collect();
    let page = CoursePage {
        brand: &site.brand,
        section: "catalogue",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        course,
        banner: banner_url(course),
        lessons_label: lessons_label(course.lessons.len()),
        minutes: minutes(course),
        exam: progress.as_ref().and_then(|progress| ExamInfo::new(content, course, progress)),
        progress: progress.as_ref().map(|progress| CourseProgress::new(content, course, progress)),
        lessons,
        paths: paths_of(content, &course.slug),
    };
    render(StatusCode::OK, page)
}

pub async fn lesson(site: Site, State(state): State<AppState>, Path((course_slug, lesson_slug)): Path<(String, String)>) -> Response {
    let content = &*site.content;
    let Some(course) = published_course(content, &course_slug) else { return missing(&site) };
    let Some(position) = course.lessons.iter().position(|lesson| lesson.slug == lesson_slug) else { return missing(&site) };
    let lesson = &course.lessons[position];
    let next = course.lessons.get(position + 1).map(|next| next.slug.as_str());
    let next_url = next.map_or(format!("/courses/{}/", course.slug), |slug| format!("/courses/{}/{slug}/", course.slug));
    let overall = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let steps = course
        .lessons
        .iter()
        .map(|step| Step {
            slug: &step.slug,
            title: &step.title,
            state: overall.as_ref().map_or("", |progress| state_name(progress.lesson(course, step))),
        })
        .collect();
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
        has_paths: site.has_paths,
        course,
        lesson,
        index: position + 1,
        steps,
        previous: position.checked_sub(1).map(|i| course.lessons[i].slug.as_str()),
        next,
        lab_pending: lesson.lab.is_some(),
        has_quiz: !lesson.quiz.is_empty(),
        lesson_data,
    };
    render(StatusCode::OK, page)
}

pub async fn dashboard(site: Site, State(state): State<AppState>) -> Response {
    let content = &*site.content;
    let (Some(me), Ok(progress)) = (&site.viewer, load_progress(&state, &site).await) else {
        return if site.viewer.is_none() { sign_in_first(&site) } else { StatusCode::SERVICE_UNAVAILABLE.into_response() };
    };
    let Some(progress) = progress else { return sign_in_first(&site) };
    // The first lesson not completed of the first course that is open and unfinished, in catalogue order.
    let resume = content
        .catalogue
        .courses
        .iter()
        .filter(|course| course.published)
        .filter(|course| {
            let advancement = CourseProgress::new(content, course, &progress);
            advancement.unlocked && !advancement.completed
        })
        .find_map(|course| {
            let lesson = course.lessons.iter().find(|lesson| !progress.lesson(course, lesson).completed)?;
            Some(Resume { course, lesson, started: progress.lesson(course, lesson).started() })
        });
    let mut badges = badge_items(content, &progress);
    let (badges_owned, badges_total) = (badges.iter().filter(|badge| badge.awarded.is_some()).count(), badges.len());
    badges.truncate(DASHBOARD_BADGES);
    let page = DashboardPage {
        brand: &site.brand,
        section: "dashboard",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        me,
        to_next: me.level.next.map(|next| next - me.level.xp),
        resume,
        paths: dashboard_cards(content, &progress),
        cards: cards(content, Some(&progress)).into_iter().filter(|card| card.published).collect(),
        badges,
        badges_owned,
        badges_total,
    };
    render(StatusCode::OK, page)
}

pub async fn badges(site: Site, State(state): State<AppState>) -> Response {
    let content = &*site.content;
    let Some(me) = &site.viewer else { return sign_in_first(&site) };
    let progress = match load_progress(&state, &site).await {
        Ok(Some(progress)) => progress,
        Ok(None) => return sign_in_first(&site),
        Err(err) => return unavailable(err),
    };
    let items = badge_items(content, &progress);
    let levels = LEVEL_THRESHOLDS
        .iter()
        .zip(DEFAULT_LEVEL_TITLES)
        .enumerate()
        .map(|(i, (&xp, title))| LevelRow { level: i + 1, title, xp })
        .collect();
    let page = BadgesPage {
        brand: &site.brand,
        section: "badges",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        me,
        owned: items.iter().filter(|item| item.awarded.is_some()).count(),
        items,
        levels,
    };
    render(StatusCode::OK, page)
}

/// The validation exam of a course: its rules, then the attempt itself, run by the page script.
pub async fn exam(site: Site, State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let content = &*site.content;
    let Some(course) = published_course(content, &slug) else { return missing(&site) };
    let Some(exam) = &course.exam else { return missing(&site) };
    let Some(viewer) = &site.viewer else { return sign_in_first(&site) };
    let progress = match load_progress(&state, &site).await {
        Ok(Some(progress)) => progress,
        Ok(None) => return sign_in_first(&site),
        Err(err) => return unavailable(err),
    };
    let Some(info) = ExamInfo::new(content, course, &progress) else { return missing(&site) };
    let status = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let status = tx.exam_status(viewer.learner.id, &course.slug, EXAM_COOLDOWN_SECONDS, now()).await?;
        tx.commit().await?;
        Ok::<_, mentor_db::Error>(status)
    };
    let status = match status.await {
        Ok(status) => status,
        Err(err) => return unavailable(err),
    };
    let exam_data = serde_json::json!({
        "course": { "slug": course.slug, "title": course.title },
        "exam": { "title": info.title, "count": info.count, "passMark": info.pass_mark, "minutes": info.minutes },
        "state": { "validated": info.validated, "locked": info.locked, "open": status.open, "retryAfter": status.retry_after },
        "startUrl": format!("/api/exam/{}/start", course.slug),
        "submitUrl": format!("/api/exam/{}/submit", course.slug),
        "courseUrl": format!("/courses/{}/", course.slug),
    });
    let page = ExamPage {
        brand: &site.brand,
        section: "dashboard",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        course,
        info,
        intro_html: without_trailing_headings(&exam.intro_html),
        exam_data: script_json(&exam_data),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_written_in_french_in_the_tenants_time_zone() {
        // 2026-10-05 12:00:00 UTC.
        assert_eq!(french_date(1_791_201_600, 60), "5 octobre 2026");
        // 2024-02-29 23:30 UTC is already the first of March one hour east.
        assert_eq!(french_date(1_709_249_400, 0), "29 février 2024");
        assert_eq!(french_date(1_709_249_400, 60), "1er mars 2024");
        assert_eq!(french_date(0, 0), "1er janvier 1970");
    }

    #[test]
    fn headings_that_end_an_exam_introduction_are_dropped() {
        assert_eq!(without_trailing_headings("<p>Bonne chance.</p>\n<h2>Questions</h2>\n"), "<p>Bonne chance.</p>");
        assert_eq!(
            without_trailing_headings("<h1>Examen</h1><p>Règles.</p><h2 id=\"a\">A</h2> <h3>B</h3>"),
            "<h1>Examen</h1><p>Règles.</p>"
        );
        assert_eq!(without_trailing_headings("<h2>Règles</h2><p>Une seule réponse.</p>"), "<h2>Règles</h2><p>Une seule réponse.</p>");
        assert_eq!(without_trailing_headings("<h2>Seul</h2>"), "");
        assert_eq!(without_trailing_headings("é"), "é");
    }

    #[test]
    fn a_lesson_state_has_one_name() {
        let state =
            |tasks_done, quiz_best, completed, validated_by_exam| LessonState { tasks_done, quiz_best, completed, validated_by_exam };
        assert_eq!(state_name(state(0, 0, false, false)), "new");
        assert_eq!(state_name(state(0, 1, false, false)), "started");
        assert_eq!(state_name(state(2, 3, true, false)), "done");
        assert_eq!(state_name(state(0, 0, true, true)), "exam");
    }
}
