//! Training paths: ordered selections of courses that lead somewhere ("to work on the infrastructure, follow
//! these courses in this order").
//!
//! They are declared in `paths.yml`, next to the course folders (format in `catalogue/README.md`, "Training paths"). A path
//! only arranges courses that exist: it holds no content of its own and changes nothing to what a course
//! requires. It is therefore validated against the compiled courses, and strictly: unlike front matter, an
//! unknown key is an error.
//!
//! A path is self-contained: every prerequisite of a course it lists is listed too, earlier. Whoever follows
//! a path from its first course to its last never meets a lock they cannot open from inside the path.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;
use serde_yaml::{Mapping, Value as Yaml};

use crate::catalogue::Course;
use crate::error::{ContentError, Result};

/// Name of the file that declares the paths, at the root of the catalogue.
pub const PATHS_FILE: &str = "paths.yml";

/// A path identifier appears in URLs: lower-case words separated by single hyphens.
static ID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());
/// The accent colour is written into style attributes: hexadecimal only.
static COLOR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$").unwrap());

const PATH_KEYS: [&str; 7] = ["id", "title", "icon", "summary", "color", "stages", "courses"];
const STAGE_KEYS: [&str; 2] = ["title", "courses"];
const ENTRY_KEYS: [&str; 2] = ["course", "optional"];

/// A training path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LearningPath {
    /// Stable identifier, used in URLs.
    pub id: String,
    pub title: String,
    pub icon: String,
    pub summary: String,
    /// Hexadecimal accent colour, or empty.
    pub accent: String,
    /// At least one. A path written as a flat list of courses has a single stage without a title.
    pub stages: Vec<PathStage>,
}

impl LearningPath {
    /// Every course of the path, in order.
    pub fn courses(&self) -> impl Iterator<Item = &PathCourse> {
        self.stages.iter().flat_map(|stage| stage.courses.iter())
    }
}

/// A group of courses followed together ("Fundamentals", "Specialisation").
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PathStage {
    /// Empty for the single stage of a flat path.
    pub title: String,
    pub courses: Vec<PathCourse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PathCourse {
    /// Slug of the course.
    pub course: String,
    /// An optional course is shown in the path but is not needed to complete it.
    pub optional: bool,
}

/// Reports errors against the paths file, naming the path they are in.
struct Reporter<'a> {
    file: &'a Path,
}

impl Reporter<'_> {
    fn fail<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(ContentError::new(self.file, message))
    }
}

fn mapping<'a>(value: &'a Yaml, what: &str, allowed: &[&str], report: &Reporter) -> Result<&'a Mapping> {
    let Yaml::Mapping(map) = value else {
        return report.fail(format!("{what} must be a mapping with the keys: {}", allowed.join(", ")));
    };
    for key in map.keys() {
        match key.as_str() {
            Some(key) if allowed.contains(&key) => {}
            Some(key) => return report.fail(format!("{what}: unknown key `{key}` (known keys: {})", allowed.join(", "))),
            None => return report.fail(format!("{what}: keys must be text (known keys: {})", allowed.join(", "))),
        }
    }
    Ok(map)
}

/// A required, non-empty text.
fn required_text(map: &Mapping, key: &str, what: &str, report: &Reporter) -> Result<String> {
    match map.get(key) {
        Some(Yaml::String(value)) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        None | Some(Yaml::Null) => report.fail(format!("{what}: `{key}` is missing")),
        Some(_) => report.fail(format!("{what}: `{key}` must be a non-empty text (quote it)")),
    }
}

fn load_entry(value: &Yaml, what: &str, report: &Reporter) -> Result<PathCourse> {
    if let Yaml::String(slug) = value {
        return Ok(PathCourse { course: slug.clone(), optional: false });
    }
    let entry = format!("{what}: a course");
    if !value.is_mapping() {
        return report.fail(format!("{entry} is either its folder name or `{{course: name, optional: true}}`"));
    }
    let map = mapping(value, &entry, &ENTRY_KEYS, report)?;
    let course = required_text(map, "course", &entry, report)?;
    let optional = match map.get("optional") {
        None => false,
        Some(Yaml::Bool(optional)) => *optional,
        Some(_) => return report.fail(format!("{what}: course `{course}`: `optional` must be true or false")),
    };
    Ok(PathCourse { course, optional })
}

fn load_courses(value: Option<&Yaml>, what: &str, report: &Reporter) -> Result<Vec<PathCourse>> {
    match value {
        Some(Yaml::Sequence(items)) if !items.is_empty() => items.iter().map(|item| load_entry(item, what, report)).collect(),
        _ => report.fail(format!("{what}: `courses` must be a list of at least one course")),
    }
}

fn load_path(value: &Yaml, position: usize, requires: &BTreeMap<&str, &[String]>, report: &Reporter) -> Result<LearningPath> {
    let nth = format!("path #{position}");
    let map = mapping(value, &nth, &PATH_KEYS, report)?;
    let id = required_text(map, "id", &nth, report)?;
    if !ID_RE.is_match(&id) {
        return report.fail(format!("{nth}: `id` must be lower-case words separated by hyphens, as in `web-frontend` (got `{id}`)"));
    }
    let what = format!("path `{id}`");
    let title = required_text(map, "title", &what, report)?;
    let icon = required_text(map, "icon", &what, report)?;
    let summary = required_text(map, "summary", &what, report)?;
    let accent = match map.get("color") {
        None | Some(Yaml::Null) => String::new(),
        Some(Yaml::String(colour)) if COLOR_RE.is_match(colour) => colour.clone(),
        Some(_) => return report.fail(format!("{what}: `color` must be a quoted hexadecimal colour, as in \"#2496ED\"")),
    };

    let stages = match (map.get("stages"), map.get("courses")) {
        (Some(_), Some(_)) => return report.fail(format!("{what}: give either `stages` or `courses`, not both")),
        (None, None) => return report.fail(format!("{what}: a path needs `courses` (a list) or `stages` (groups of courses)")),
        (None, courses) => vec![PathStage { title: String::new(), courses: load_courses(courses, &what, report)? }],
        (Some(Yaml::Sequence(items)), None) if !items.is_empty() => {
            let mut stages = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let nth = format!("{what}: stage #{}", index + 1);
                let stage = mapping(item, &nth, &STAGE_KEYS, report)?;
                let title = required_text(stage, "title", &nth, report)?;
                let courses = load_courses(stage.get("courses"), &format!("{what}: stage `{title}`"), report)?;
                stages.push(PathStage { title, courses });
            }
            stages
        }
        (Some(_), None) => return report.fail(format!("{what}: `stages` must be a list of at least one stage")),
    };
    let path = LearningPath { id, title, icon, summary, accent, stages };

    // Courses seen so far, and whether each is optional.
    let mut seen: BTreeMap<&str, bool> = BTreeMap::new();
    for entry in path.courses() {
        let slug = entry.course.as_str();
        let Some(required) = requires.get(slug) else {
            return report.fail(format!("{what}: unknown course `{slug}` (a course is named by its folder, listed in the catalogue)"));
        };
        if seen.contains_key(slug) {
            return report.fail(format!("{what}: the course `{slug}` is listed twice"));
        }
        for prerequisite in required.iter() {
            match seen.get(prerequisite.as_str()) {
                Some(true) if !entry.optional => {
                    return report.fail(format!(
                        "{what}: `{slug}` is needed to complete the path but requires `{prerequisite}`, which is optional in it: \
                         make `{prerequisite}` required or `{slug}` optional"
                    ));
                }
                Some(_) => {}
                None if path.courses().any(|other| &other.course == prerequisite) => {
                    return report
                        .fail(format!("{what}: `{slug}` requires `{prerequisite}`, which comes after it: list `{prerequisite}` first"));
                }
                None => {
                    return report.fail(format!(
                        "{what}: `{slug}` requires `{prerequisite}`, which is not in the path: \
                         list `{prerequisite}` before it, so that the path can be followed from start to end"
                    ));
                }
            }
        }
        seen.insert(slug, entry.optional);
    }
    if seen.values().all(|optional| *optional) {
        return report.fail(format!("{what}: every course is optional, so nothing would complete the path"));
    }
    Ok(path)
}

fn parse(source: &str, file: &Path, requires: &BTreeMap<&str, &[String]>) -> Result<Vec<LearningPath>> {
    let report = Reporter { file };
    let document: Yaml = serde_yaml::from_str(source).or_else(|err| report.fail(format!("invalid YAML: {err}")))?;
    let root = mapping(&document, "the file", &["paths"], &report)?;
    let items = match root.get("paths") {
        Some(Yaml::Sequence(items)) => items,
        _ => return report.fail("`paths` must be a list of paths"),
    };
    let mut paths: Vec<LearningPath> = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let path = load_path(item, index + 1, requires, &report)?;
        if paths.iter().any(|other| other.id == path.id) {
            return report.fail(format!("the path id `{}` is used twice", path.id));
        }
        paths.push(path);
    }
    Ok(paths)
}

/// Validates the text of a paths file against `courses`. `file` only names the file in diagnostics.
pub fn parse_paths(source: &str, file: &Path, courses: &[Course]) -> Result<Vec<LearningPath>> {
    let requires = courses.iter().map(|course| (course.slug.as_str(), course.requires.as_slice())).collect();
    parse(source, file, &requires)
}

/// Loads the training paths of the catalogue in `directory`, validated against its compiled `courses`.
/// A catalogue without a `paths.yml` has no path.
pub fn load_paths(directory: &Path, courses: &[Course]) -> Result<Vec<LearningPath>> {
    let file = directory.join(PATHS_FILE);
    if !file.exists() {
        return Ok(Vec::new());
    }
    let source = fs::read_to_string(&file).map_err(|err| ContentError::new(&file, format!("cannot be read: {err}")))?;
    parse_paths(&source, &file, courses)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `a`, `b` (requires `a`), `c` (requires `b`), `d`.
    fn check(source: &str) -> Result<Vec<LearningPath>> {
        let (a, b, c): (Vec<String>, Vec<String>, Vec<String>) = (vec![], vec!["a".into()], vec!["b".into()]);
        let requires = BTreeMap::from([("a", a.as_slice()), ("b", b.as_slice()), ("c", c.as_slice()), ("d", a.as_slice())]);
        parse(source, Path::new("paths.yml"), &requires)
    }

    fn error(source: &str) -> String {
        check(source).unwrap_err().to_string()
    }

    /// A path with the given body under a valid head.
    fn path(body: &str) -> String {
        format!("paths:\n  - id: ops\n    title: Ops\n    icon: \"🧭\"\n    summary: S\n{body}")
    }

    #[test]
    fn a_flat_path_is_one_untitled_stage() {
        let paths = check(&path("    color: \"#2496ED\"\n    courses: [a, b, {course: d, optional: true}]\n")).unwrap();
        assert_eq!(paths.len(), 1);
        let ops = &paths[0];
        assert_eq!((ops.id.as_str(), ops.title.as_str(), ops.icon.as_str(), ops.accent.as_str()), ("ops", "Ops", "🧭", "#2496ED"));
        assert_eq!(ops.stages.len(), 1);
        assert_eq!(ops.stages[0].title, "");
        let courses: Vec<(&str, bool)> = ops.courses().map(|entry| (entry.course.as_str(), entry.optional)).collect();
        assert_eq!(courses, [("a", false), ("b", false), ("d", true)]);
    }

    #[test]
    fn stages_keep_their_order_and_titles() {
        let body = "    stages:\n      - title: Bases\n        courses: [a, d]\n      - title: Suite\n        courses:\n          - b\n          - course: c\n            optional: true\n";
        let paths = check(&path(body)).unwrap();
        let stages: Vec<(&str, usize)> = paths[0].stages.iter().map(|stage| (stage.title.as_str(), stage.courses.len())).collect();
        assert_eq!(stages, [("Bases", 2), ("Suite", 2)]);
        assert!(paths[0].stages[1].courses[1].optional);
        assert_eq!(paths[0].accent, "");
    }

    #[test]
    fn no_paths_is_valid() {
        assert_eq!(check("paths: []\n").unwrap(), []);
    }

    #[test]
    fn the_file_and_its_keys_are_checked() {
        assert!(error("paths: [").starts_with("paths.yml: invalid YAML: "));
        assert_eq!(error("- a\n"), "paths.yml: the file must be a mapping with the keys: paths");
        assert_eq!(error("paths: []\ncourses: []\n"), "paths.yml: the file: unknown key `courses` (known keys: paths)");
        assert_eq!(error("paths: 3\n"), "paths.yml: `paths` must be a list of paths");
        assert_eq!(
            error("paths:\n  - id: ops\n    titre: Ops\n"),
            "paths.yml: path #1: unknown key `titre` (known keys: id, title, icon, summary, color, stages, courses)"
        );
        assert_eq!(
            error("paths:\n  - ops\n"),
            "paths.yml: path #1 must be a mapping with the keys: id, title, icon, summary, color, stages, courses"
        );
    }

    #[test]
    fn identity_fields_are_required_and_well_formed() {
        assert_eq!(error("paths:\n  - title: Ops\n"), "paths.yml: path #1: `id` is missing");
        assert!(error("paths:\n  - id: Dev Ops\n").contains("`id` must be lower-case words separated by hyphens"));
        assert_eq!(
            error("paths:\n  - id: ops\n    title: Ops\n    summary: S\n    courses: [a]\n"),
            "paths.yml: path `ops`: `icon` is missing"
        );
        assert_eq!(
            error("paths:\n  - id: ops\n    title: 12\n    icon: i\n    summary: S\n    courses: [a]\n"),
            "paths.yml: path `ops`: `title` must be a non-empty text (quote it)"
        );
        for colour in ["red", "\"#12345\"", "\"#fff; background: url(x)\"", "3"] {
            assert_eq!(
                error(&path(&format!("    color: {colour}\n    courses: [a]\n"))),
                "paths.yml: path `ops`: `color` must be a quoted hexadecimal colour, as in \"#2496ED\"",
                "{colour}"
            );
        }
        let twice =
            format!("{}{}", path("    courses: [a]\n"), "  - id: ops\n    title: T\n    icon: i\n    summary: S\n    courses: [d]\n");
        assert_eq!(error(&twice), "paths.yml: the path id `ops` is used twice");
    }

    #[test]
    fn a_path_cannot_be_empty() {
        assert_eq!(error(&path("")), "paths.yml: path `ops`: a path needs `courses` (a list) or `stages` (groups of courses)");
        assert_eq!(error(&path("    courses: []\n")), "paths.yml: path `ops`: `courses` must be a list of at least one course");
        assert_eq!(error(&path("    stages: []\n")), "paths.yml: path `ops`: `stages` must be a list of at least one stage");
        assert_eq!(
            error(&path("    stages:\n      - title: Bases\n        courses: []\n")),
            "paths.yml: path `ops`: stage `Bases`: `courses` must be a list of at least one course"
        );
        assert_eq!(
            error(&path("    courses: [a]\n    stages:\n      - title: Bases\n        courses: [d]\n")),
            "paths.yml: path `ops`: give either `stages` or `courses`, not both"
        );
        assert_eq!(
            error(&path("    courses: [{course: a, optional: true}, {course: d, optional: true}]\n")),
            "paths.yml: path `ops`: every course is optional, so nothing would complete the path"
        );
    }

    #[test]
    fn stages_and_course_entries_are_checked() {
        assert_eq!(error(&path("    stages:\n      - courses: [a]\n")), "paths.yml: path `ops`: stage #1: `title` is missing");
        assert_eq!(
            error(&path("    stages:\n      - title: Bases\n        cours: [a]\n")),
            "paths.yml: path `ops`: stage #1: unknown key `cours` (known keys: title, courses)"
        );
        assert_eq!(
            error(&path("    courses: [{course: a, optionnel: true}]\n")),
            "paths.yml: path `ops`: a course: unknown key `optionnel` (known keys: course, optional)"
        );
        assert_eq!(
            error(&path("    courses: [{course: a, optional: 1}]\n")),
            "paths.yml: path `ops`: course `a`: `optional` must be true or false"
        );
        assert_eq!(
            error(&path("    courses: [3]\n")),
            "paths.yml: path `ops`: a course is either its folder name or `{course: name, optional: true}`"
        );
    }

    #[test]
    fn courses_must_exist_and_appear_once() {
        assert_eq!(
            error(&path("    courses: [a, nope]\n")),
            "paths.yml: path `ops`: unknown course `nope` (a course is named by its folder, listed in the catalogue)"
        );
        assert_eq!(
            error(&path("    stages:\n      - title: Un\n        courses: [a]\n      - title: Deux\n        courses: [d, a]\n")),
            "paths.yml: path `ops`: the course `a` is listed twice"
        );
    }

    #[test]
    fn a_path_holds_the_prerequisites_of_its_courses_before_them() {
        assert_eq!(
            error(&path("    courses: [d, b]\n")),
            "paths.yml: path `ops`: `b` requires `a`, which is not in the path: list `a` before it, so that the path can be followed from start to end"
        );
        assert_eq!(error(&path("    courses: [b, a]\n")), "paths.yml: path `ops`: `b` requires `a`, which comes after it: list `a` first");
        // Across stages too: a later stage cannot hold what an earlier one needs.
        assert_eq!(
            error(&path("    stages:\n      - title: Un\n        courses: [a, c]\n      - title: Deux\n        courses: [b]\n")),
            "paths.yml: path `ops`: `c` requires `b`, which comes after it: list `b` first"
        );
        assert_eq!(
            error(&path("    courses: [{course: a, optional: true}, b]\n")),
            "paths.yml: path `ops`: `b` is needed to complete the path but requires `a`, which is optional in it: make `a` required or `b` optional"
        );
        // An optional course may build on anything before it.
        assert!(check(&path("    courses: [a, {course: b, optional: true}, {course: c, optional: true}]\n")).is_ok());
    }
}
