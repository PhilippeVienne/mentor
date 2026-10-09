//! Training paths: the list of paths and the map of one path.
//!
//! The interface calls a path « parcours de formation »: « cours » already names a course there. Nothing is stored about
//! paths: where a learner stands in one is computed from the courses they completed.

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use mentor_content::{Course, LearningPath, Lesson};

use crate::brand::Brand;
use crate::pages::{lessons_label, load_progress, minutes, missing, render, unavailable, CourseProgress, Progress};
use crate::path_map::{layout, Layout, Slot};
use crate::site::{Site, Viewer};
use crate::AppState;

/// Where a learner stands with one course of a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Standing {
    pub published: bool,
    /// Its prerequisites are completed.
    pub unlocked: bool,
    pub completed: bool,
    /// Something was done in it.
    pub started: bool,
    /// Share of its lessons that are completed.
    pub percent: u32,
}

impl Standing {
    /// `done`, `started`, `available`, `locked` or `soon`: the suffix of the style classes.
    fn name(&self) -> &'static str {
        match self {
            Standing { published: false, .. } => "soon",
            Standing { completed: true, .. } => "done",
            Standing { unlocked: false, .. } => "locked",
            Standing { started: true, .. } => "started",
            _ => "available",
        }
    }

    /// The learner can work on it now.
    fn open(&self) -> bool {
        self.published && self.unlocked && !self.completed
    }
}

/// Where a learner stands in a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Advancement {
    /// Courses needed to complete the path, and how many of them are completed.
    pub required: usize,
    pub done: usize,
    /// Mean advancement of the required courses: each weighs the same, whatever its length.
    pub percent: u32,
    /// Every required course is completed. Optional ones do not count.
    pub completed: bool,
    /// Something was done in one of its courses, optional or not.
    pub started: bool,
    /// Position of the course to work on next: the first open required one, else the first open optional one.
    pub next: Option<usize>,
}

/// The advancement in a path, from its courses in order, each with whether it is optional.
pub(crate) fn advancement(courses: &[(bool, Standing)]) -> Advancement {
    let required: Vec<&Standing> = courses.iter().filter(|(optional, _)| !optional).map(|(_, standing)| standing).collect();
    let done = required.iter().filter(|standing| standing.completed).count();
    let total: u32 = required.iter().map(|standing| if standing.completed { 100 } else { standing.percent.min(100) }).sum();
    let completed = !required.is_empty() && done == required.len();
    let first_open = |wanted: bool| courses.iter().position(|(optional, standing)| *optional == wanted && standing.open());
    Advancement {
        required: required.len(),
        done,
        // A path is at 100 % only when completed: rounding never announces it early.
        percent: match (total as usize * 2 + required.len()).checked_div(required.len() * 2).unwrap_or(0) as u32 {
            100 if !completed => 99,
            percent => percent,
        },
        completed,
        started: courses.iter().any(|(_, standing)| standing.started || standing.completed),
        next: first_open(false).or_else(|| first_open(true)),
    }
}

/// A course of a path, resolved in the catalogue.
struct Entry<'a> {
    course: &'a Course,
    optional: bool,
    stage: usize,
    /// `None` for a visitor.
    progress: Option<CourseProgress<'a>>,
    standing: Option<Standing>,
}

/// The courses of a path, in order. A course the catalogue does not hold is left out: the compiler refuses
/// such a path, so this only happens when paths and catalogue were loaded apart.
fn entries<'a>(state: &'a AppState, path: &'a LearningPath, progress: Option<&Progress>) -> Vec<Entry<'a>> {
    let mut entries = Vec::new();
    for (stage, group) in path.stages.iter().enumerate() {
        for listed in &group.courses {
            let Some(course) = state.catalogue.courses.iter().find(|course| course.slug == listed.course) else { continue };
            let advancement = progress.map(|progress| CourseProgress::new(state, course, progress));
            let standing = progress.zip(advancement.as_ref()).map(|(progress, advancement)| Standing {
                published: course.published,
                unlocked: advancement.unlocked,
                completed: advancement.completed,
                started: course.lessons.iter().any(|lesson| {
                    let lesson = progress.lesson(course, lesson);
                    lesson.started() || lesson.completed
                }),
                percent: advancement.percent,
            });
            entries.push(Entry { course, optional: listed.optional, stage, progress: advancement, standing });
        }
    }
    entries
}

fn advancement_of(entries: &[Entry]) -> Option<Advancement> {
    let standings: Option<Vec<(bool, Standing)>> =
        entries.iter().map(|entry| entry.standing.map(|standing| (entry.optional, standing))).collect();
    standings.map(|standings| advancement(&standings))
}

/// The style class of a course for this viewer: its standing, or for a visitor only whether it exists yet.
fn state_class(entry: &Entry) -> &'static str {
    match entry.standing {
        Some(standing) => standing.name(),
        None if !entry.course.published => "soon",
        None => "plain",
    }
}

/// « ≈ 14 h », or minutes under an hour.
fn duration_label(minutes: u32) -> String {
    if minutes < 60 {
        format!("≈ {minutes} min")
    } else {
        format!("≈ {} h", (minutes + 30) / 60)
    }
}

/// A course in the strip of a path card.
pub(crate) struct Dot<'a> {
    pub(crate) icon: &'a str,
    pub(crate) state: &'static str,
}

/// A path as shown in a grid: on the list of paths and on the dashboard.
pub(crate) struct PathCard<'a> {
    pub(crate) path: &'a LearningPath,
    /// The courses of each stage.
    pub(crate) strip: Vec<Vec<Dot<'a>>>,
    /// Titles of the courses, in order: the text equivalent of the strip.
    pub(crate) titles: String,
    pub(crate) courses: usize,
    pub(crate) optional: usize,
    /// Number of stages, 0 for a flat path.
    pub(crate) stages: usize,
    pub(crate) duration: String,
    /// `None` for a visitor.
    pub(crate) advancement: Option<Advancement>,
}

impl<'a> PathCard<'a> {
    fn new(state: &'a AppState, path: &'a LearningPath, progress: Option<&Progress>) -> Self {
        let entries = entries(state, path, progress);
        let mut strip: Vec<Vec<Dot>> = path.stages.iter().map(|_| Vec::new()).collect();
        for entry in &entries {
            strip[entry.stage].push(Dot { icon: &entry.course.icon, state: state_class(entry) });
        }
        strip.retain(|stage| !stage.is_empty());
        Self {
            path,
            strip,
            titles: entries.iter().map(|entry| entry.course.title.as_str()).collect::<Vec<_>>().join(", "),
            courses: entries.len(),
            optional: entries.iter().filter(|entry| entry.optional).count(),
            stages: if path.stages.len() > 1 || path.stages.iter().any(|stage| !stage.title.is_empty()) { path.stages.len() } else { 0 },
            duration: duration_label(entries.iter().map(|entry| minutes(entry.course)).sum()),
            advancement: advancement_of(&entries),
        }
    }
}

/// Every path as a card, in catalogue order.
pub(crate) fn path_cards<'a>(state: &'a AppState, progress: Option<&Progress>) -> Vec<PathCard<'a>> {
    state.paths.iter().map(|path| PathCard::new(state, path, progress)).collect()
}

/// The cards of the dashboard: paths the learner is in first, then those not started, then completed ones.
pub(crate) fn dashboard_cards<'a>(state: &'a AppState, progress: &Progress) -> Vec<PathCard<'a>> {
    let mut cards = path_cards(state, Some(progress));
    cards.sort_by_key(|card| match card.advancement {
        Some(Advancement { completed: true, .. }) => 2,
        Some(Advancement { started: true, .. }) => 0,
        _ => 1,
    });
    cards
}

/// The paths a course belongs to, in catalogue order.
pub(crate) fn paths_of<'a>(state: &'a AppState, course: &str) -> Vec<&'a LearningPath> {
    state.paths.iter().filter(|path| path.courses().any(|listed| listed.course == course)).collect()
}

#[derive(Template)]
#[template(path = "paths.html")]
struct PathsPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    cards: Vec<PathCard<'a>>,
}

/// A course on the map.
struct Node<'a> {
    course: &'a Course,
    optional: bool,
    /// Suffix of the style classes: a [`Standing::name`], or `plain` for a visitor.
    state: &'static str,
    /// What the state says in words; empty for a visitor looking at a published course.
    label: String,
    /// A second line: what to complete first when locked, else the size of the course.
    note: String,
    /// Lessons completed, as a percentage; `None` when no bar is shown.
    percent: Option<u32>,
    /// Titles of the courses of the path it requires.
    prerequisites: Vec<&'a str>,
    /// The course to work on next.
    next: bool,
    slot: Slot,
}

struct Stage<'a> {
    title: &'a str,
    nodes: Vec<Node<'a>>,
}

/// A prerequisite link as drawn. `state` is `done` when the prerequisite is completed, `todo` when it is
/// not, and empty for a visitor.
struct Drawn<'a> {
    wide: &'a str,
    narrow: &'a str,
    arrow: &'a str,
    state: &'static str,
}

/// What to do next in a path.
struct NextStep<'a> {
    course: &'a Course,
    lesson: &'a Lesson,
    started: bool,
}

#[derive(Template)]
#[template(path = "path.html")]
struct PathPage<'a> {
    brand: &'a Brand,
    section: &'a str,
    viewer: Option<&'a Viewer>,
    dev_login: bool,
    has_paths: bool,
    path: &'a LearningPath,
    card: PathCard<'a>,
    /// Stages have titles; a flat path has a single stage without one.
    titled: bool,
    stages: Vec<Stage<'a>>,
    map: &'a Layout,
    links: Vec<Drawn<'a>>,
    next: Option<NextStep<'a>>,
    /// For a visitor: the course the path starts with.
    first: Option<&'a Course>,
}

fn node<'a>(entry: &Entry<'a>, prerequisites: Vec<&'a str>, next: bool, slot: Slot) -> Node<'a> {
    let state = state_class(entry);
    let size = format!("{} · ≈ {} min", lessons_label(entry.course.lessons.len()), minutes(entry.course));
    let (label, note) = match (state, &entry.progress) {
        ("soon", _) => ("Bientôt disponible".to_string(), String::new()),
        ("done", _) => ("✓ Terminé".to_string(), size),
        ("locked", Some(progress)) => ("🔒 Verrouillé".to_string(), format!("Termine d'abord : {}", progress.missing.join(", "))),
        ("started", Some(progress)) => (format!("En cours · {} %", progress.percent), size),
        ("available", _) => ("À commencer".to_string(), size),
        _ => (String::new(), size),
    };
    Node {
        course: entry.course,
        optional: entry.optional,
        state,
        label,
        note,
        percent: entry.progress.as_ref().filter(|_| matches!(state, "done" | "started" | "available")).map(|progress| progress.percent),
        prerequisites,
        next,
        slot,
    }
}

pub async fn index(site: Site, State(state): State<AppState>) -> Response {
    let progress = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let page = PathsPage {
        brand: &site.brand,
        section: "paths",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        cards: path_cards(&state, progress.as_ref()),
    };
    render(StatusCode::OK, page)
}

pub async fn path(site: Site, State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let Some(path) = state.paths.iter().find(|path| path.id == id) else { return missing(&site) };
    let progress = match load_progress(&state, &site).await {
        Ok(progress) => progress,
        Err(err) => return unavailable(err),
    };
    let entries = entries(&state, path, progress.as_ref());
    let advancement = advancement_of(&entries);
    let position = |slug: &str| entries.iter().position(|entry| entry.course.slug == slug);

    // Wide columns: one per stage; a flat path spreads its courses, one per column.
    let titled = path.stages.len() > 1 || path.stages.iter().any(|stage| !stage.title.is_empty());
    let columns: Vec<usize> = if titled {
        (0..path.stages.len()).map(|stage| entries.iter().filter(|entry| entry.stage == stage).count()).filter(|count| *count > 0).collect()
    } else {
        vec![1; entries.len()]
    };
    let edges: Vec<(usize, usize)> = entries
        .iter()
        .enumerate()
        .flat_map(|(to, entry)| entry.course.requires.iter().filter_map(|required| position(required)).map(move |from| (from, to)))
        .collect();
    let map = layout(&columns, titled, &edges);
    let links = map
        .links
        .iter()
        .map(|link| Drawn {
            wide: &link.wide,
            narrow: &link.narrow,
            arrow: &link.narrow_arrow,
            state: match entries[link.from].standing {
                Some(Standing { completed: true, .. }) => "done",
                Some(_) => "todo",
                None => "",
            },
        })
        .collect();

    let next_position = advancement.and_then(|advancement| advancement.next);
    let mut stages: Vec<Stage> = path.stages.iter().map(|stage| Stage { title: &stage.title, nodes: Vec::new() }).collect();
    for (index, entry) in entries.iter().enumerate() {
        let prerequisites = entry
            .course
            .requires
            .iter()
            .filter_map(|required| position(required))
            .map(|from| entries[from].course.title.as_str())
            .collect();
        stages[entry.stage].nodes.push(node(entry, prerequisites, next_position == Some(index), map.slots[index]));
    }
    stages.retain(|stage| !stage.nodes.is_empty());

    let next = next_position.zip(progress.as_ref()).and_then(|(index, progress)| {
        let course = entries[index].course;
        let lesson = course.lessons.iter().find(|lesson| !progress.lesson(course, lesson).completed)?;
        Some(NextStep { course, lesson, started: progress.lesson(course, lesson).started() })
    });
    let page = PathPage {
        brand: &site.brand,
        section: "paths",
        viewer: site.viewer.as_ref(),
        dev_login: site.dev_login,
        has_paths: site.has_paths,
        path,
        card: PathCard::new(&state, path, progress.as_ref()),
        titled,
        stages,
        map: &map,
        links,
        next,
        first: entries.iter().map(|entry| entry.course).find(|course| course.published),
    };
    render(StatusCode::OK, page)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW: Standing = Standing { published: true, unlocked: true, completed: false, started: false, percent: 0 };
    const DONE: Standing = Standing { completed: true, started: true, percent: 100, ..NEW };
    const LOCKED: Standing = Standing { unlocked: false, ..NEW };
    const SOON: Standing = Standing { published: false, ..NEW };

    fn half() -> Standing {
        Standing { started: true, percent: 50, ..NEW }
    }

    #[test]
    fn a_standing_has_one_name() {
        assert_eq!(
            (NEW.name(), half().name(), DONE.name(), LOCKED.name(), SOON.name()),
            ("available", "started", "done", "locked", "soon")
        );
        // A quiz answered in a lesson that is not completed yet: in progress at 0 %.
        assert_eq!(Standing { started: true, ..NEW }.name(), "started");
        // An unpublished course is announced, whatever else is true of it.
        assert_eq!(Standing { published: false, unlocked: false, ..NEW }.name(), "soon");
        // Completed through its exam while a prerequisite is missing: completed is what matters.
        assert_eq!(Standing { unlocked: false, ..DONE }.name(), "done");
    }

    #[test]
    fn a_path_advances_with_its_required_courses() {
        let fresh = advancement(&[(false, NEW), (false, LOCKED), (true, NEW)]);
        assert_eq!((fresh.required, fresh.done, fresh.percent, fresh.completed, fresh.started), (2, 0, 0, false, false));
        assert_eq!(fresh.next, Some(0));

        let midway = advancement(&[(false, DONE), (false, half()), (false, LOCKED), (true, NEW)]);
        assert_eq!((midway.required, midway.done, midway.percent, midway.completed, midway.started), (3, 1, 50, false, true));
        // The first required course that is open, even though an optional one is open too.
        assert_eq!(midway.next, Some(1));
    }

    #[test]
    fn optional_courses_do_not_count_but_can_be_next() {
        let done = advancement(&[(false, DONE), (true, NEW), (false, DONE)]);
        assert_eq!((done.required, done.done, done.percent, done.completed), (2, 2, 100, true));
        assert_eq!(done.next, Some(1));
        // An optional course alone starts the path without advancing it.
        let side = advancement(&[(false, NEW), (true, DONE)]);
        assert_eq!((side.done, side.percent, side.started, side.next), (0, 0, true, Some(0)));
        let over = advancement(&[(false, DONE), (true, DONE)]);
        assert_eq!((over.completed, over.next), (true, None));
    }

    #[test]
    fn a_path_waiting_for_a_course_is_neither_complete_nor_at_100_percent() {
        // The last required course is not published: nothing to do next, and the path is not completed.
        let waiting = advancement(&[(false, DONE), (false, SOON)]);
        assert_eq!((waiting.done, waiting.percent, waiting.completed, waiting.next), (1, 50, false, None));
        // Rounding up never shows 100 % before the last course is completed.
        let almost: Vec<(bool, Standing)> =
            (0..300).map(|i| (false, if i == 0 { Standing { started: true, percent: 99, ..NEW } } else { DONE })).collect();
        let almost = advancement(&almost);
        assert_eq!((almost.percent, almost.completed), (99, false));
        // Nothing required: nothing completes it.
        let empty = advancement(&[(true, DONE)]);
        assert_eq!((empty.required, empty.percent, empty.completed), (0, 0, false));
    }

    #[test]
    fn durations_read_in_hours_from_one_hour_on() {
        assert_eq!(
            (duration_label(45), duration_label(60), duration_label(89), duration_label(90), duration_label(815)),
            ("≈ 45 min".into(), "≈ 1 h".into(), "≈ 1 h".into(), "≈ 2 h".into(), "≈ 14 h".into())
        );
    }
}
