//! Loads a catalogue: index, courses, lessons, exams.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_yaml::{Mapping, Value as Yaml};

use crate::document::{render_document, Quiz, QuizOption};
use crate::environment::dockerfile_warnings;
use crate::error::{ContentError, Result};
use crate::identity::fingerprint;
use crate::lab::{Checks, Lab, LabContext};
use crate::markdown::{md_inline, Render};
use crate::yaml::{is_falsy, text};

static LESSON_FILE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(.+)\.md$").unwrap());
/// An environment is a folder directly inside its course: never a path, which could leave the course.
static ENVIRONMENT_NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]*$").unwrap());

const CHECKS_FILE: &str = "_checks.yml";
pub(crate) const COURSE_FILE: &str = "course.md";
const EXAM_FILE: &str = "exam.md";
/// `exam` is the URL segment of a course's validation exam.
const RESERVED_LESSON_IDS: [&str; 1] = ["exam"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalogue {
    pub courses: Vec<Course>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub slug: String,
    pub title: String,
    pub icon: String,
    pub summary: String,
    pub requires: Vec<String>,
    pub published: bool,
    pub accent: String,
    pub banner: String,
    pub description_html: String,
    pub cheatsheet_html: String,
    /// Real environments used by the course, by folder name, each with the `warnings` raised by reading its
    /// `Dockerfile`. The devcontainer specification is not validated here yet.
    pub environments: BTreeMap<String, Json>,
    pub exam: Option<Exam>,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub slug: String,
    /// `course--folder` identifier of the real environment, or empty.
    pub environment: String,
    pub order: u32,
    pub title: String,
    pub summary: String,
    pub minutes: u32,
    pub objectives: Vec<String>,
    pub body_html: String,
    pub lab: Option<Lab>,
    pub quiz: Vec<Quiz>,
}

/// Validation exam of a course: a pool of questions, `draw` of which are picked at random.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exam {
    pub title: String,
    /// Number of questions drawn from the pool.
    pub draw: u32,
    /// Required percentage of correct answers.
    pub pass_mark: u32,
    /// Duration in minutes.
    pub minutes: u32,
    /// Whether questions and answers are shuffled.
    pub shuffle: bool,
    pub intro_html: String,
    pub questions: Vec<ExamQuestion>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamQuestion {
    /// Digest of the question and its answers as written: stable when other questions are added or removed,
    /// when the pool is reordered and when the renderer changes.
    pub id: String,
    pub question: String,
    pub options: Vec<QuizOption>,
    pub explanation: String,
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| ContentError::new(path, format!("cannot be read: {err}")))
}

/// Separates the YAML front matter (between two `---` lines) from the Markdown body.
fn split_front_matter(source: &str, path: &Path) -> Result<(Mapping, String)> {
    let missing = || ContentError::new(path, "missing YAML front matter (a `---` block at the top of the file)");
    let rest = source.strip_prefix("---\n").ok_or_else(missing)?;
    // The front matter may be empty: the closing line then immediately follows the opening one.
    let (raw, after) = match rest.strip_prefix("---") {
        Some(after) => ("", after),
        None => rest.split_once("\n---").ok_or_else(missing)?,
    };
    let body = after.strip_prefix('\n').unwrap_or(after);
    let meta: Yaml = serde_yaml::from_str(raw).map_err(|err| ContentError::new(path, format!("invalid YAML front matter: {err}")))?;
    match meta {
        Yaml::Mapping(map) => Ok((map, body.to_string())),
        Yaml::Null => Ok((Mapping::new(), body.to_string())),
        _ => Err(ContentError::new(path, "the front matter must be a YAML mapping")),
    }
}

fn require(meta: &Mapping, keys: &[&str], path: &Path) -> Result<()> {
    let missing: Vec<&str> = keys
        .iter()
        .copied()
        .filter(|key| matches!(meta.get(*key), None | Some(Yaml::Null)) || meta.get(*key).and_then(Yaml::as_str) == Some(""))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(ContentError::new(path, format!("required field(s) missing from the front matter: {}", missing.join(", "))))
    }
}

fn field(meta: &Mapping, key: &str) -> String {
    meta.get(key).map(text).unwrap_or_default()
}

/// Real environments of a course, each validated only once.
struct Environments<'a> {
    directory: &'a Path,
    seen: BTreeMap<String, Json>,
}

impl Environments<'_> {
    fn resolve(&mut self, name: &Yaml) -> Result<()> {
        let Some(name) = name.as_str() else {
            return Err(ContentError::new(&self.directory.join(COURSE_FILE), "`environment` must be the name of a folder of the course"));
        };
        if !ENVIRONMENT_NAME_RE.is_match(name) {
            return Err(ContentError::new(
                &self.directory.join(COURSE_FILE),
                format!("`environment` must be the name of a folder of the course, not a path (got `{name}`)"),
            ));
        }
        if !self.seen.contains_key(name) {
            let folder = self.directory.join(name);
            if !folder.join("devcontainer.json").is_file() {
                return Err(ContentError::new(&folder, "real environment not found: `devcontainer.json` is missing"));
            }
            // The devcontainer specification is not validated yet; the `Dockerfile`, when there is one, is read
            // for what would keep the image from starting (see `environment`).
            let dockerfile = folder.join("Dockerfile");
            let warnings = if dockerfile.is_file() { dockerfile_warnings(&read(&dockerfile)?) } else { Vec::new() };
            self.seen.insert(name.to_string(), serde_json::json!({ "warnings": warnings }));
        }
        Ok(())
    }
}

fn load_lesson(path: &Path, course: &str, checks: &Checks, inherited: &str, environments: &mut Environments) -> Result<Lesson> {
    let (meta, body) = split_front_matter(&read(path)?, path)?;
    require(&meta, &["id", "title", "summary", "minutes"], path)?;
    let mut environment = inherited.to_string();
    if let Some(value) = meta.get("environment") {
        environment = if is_falsy(value) { String::new() } else { text(value) };
        if !environment.is_empty() {
            environments.resolve(value)?;
        }
    }
    let mut ctx = Render::new(path, course);
    let mut resolve = |name: &Yaml| environments.resolve(name);
    let mut lab_ctx = LabContext { checks, environment: environment.clone(), resolver: Some(&mut resolve) };
    let (body_html, mut collected) = render_document(&body, &mut ctx, Some(&mut lab_ctx))?;
    if collected.labs.len() > 1 {
        return ctx.fail("a lesson can contain only one `:::lab` block");
    }
    let objectives = match meta.get("objectives") {
        None | Some(Yaml::Null) => Vec::new(),
        Some(Yaml::Sequence(items)) => items.iter().map(|o| md_inline(&text(o))).collect(),
        Some(_) => return ctx.fail("`objectives` must be a list"),
    };
    let lab = collected.labs.pop();
    let effective = lab.as_ref().map(|l| l.environment.as_str()).filter(|e| !e.is_empty()).unwrap_or(&environment);
    let minutes = field(&meta, "minutes").trim().parse().or_else(|_| ctx.fail("`minutes` must be a whole number of minutes"))?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    Ok(Lesson {
        slug: field(&meta, "id"),
        environment: if effective.is_empty() { String::new() } else { format!("{course}--{effective}") },
        order: LESSON_FILE_RE.captures(name).and_then(|caps| caps[1].parse().ok()).unwrap_or(0),
        title: field(&meta, "title"),
        summary: field(&meta, "summary"),
        minutes,
        objectives,
        body_html,
        lab,
        quiz: collected.quizzes,
    })
}

fn load_exam(path: &Path, course: &str) -> Result<Exam> {
    let (meta, body) = split_front_matter(&read(path)?, path)?;
    require(&meta, &["title", "draw", "pass_mark", "minutes"], path)?;
    let bounded = |key: &str, low: u32, high: u32| -> Result<u32> {
        match meta.get(key).and_then(Yaml::as_u64) {
            Some(value) if (low as u64..=high as u64).contains(&value) => Ok(value as u32),
            _ => Err(ContentError::new(path, format!("`{key}` must be an integer between {low} and {high} (got: {})", field(&meta, key)))),
        }
    };
    let (draw, pass_mark, minutes) = (bounded("draw", 1, 100)?, bounded("pass_mark", 1, 100)?, bounded("minutes", 1, 240)?);
    let shuffle = match meta.get("shuffle") {
        None => true,
        Some(Yaml::Bool(value)) => *value,
        Some(_) => return Err(ContentError::new(path, "`shuffle` must be true or false")),
    };
    let mut ctx = Render::new(path, course);
    let (intro_html, collected) = render_document(&body, &mut ctx, None)?;
    let pool = collected.quizzes;
    if pool.len() < draw as usize {
        return ctx
            .fail(format!("the pool holds {} question(s) but `draw` asks for {draw}: add `:::quiz` blocks or lower `draw`", pool.len()));
    }
    let mut warnings = Vec::new();
    if pool.len() < 2 * draw as usize {
        warnings.push(format!(
            "pool of {} questions for a draw of {draw}: ideally provide 3 times as many questions as the draw (at least twice).",
            pool.len()
        ));
    }
    let mut seen = BTreeSet::new();
    let mut questions = Vec::with_capacity(pool.len());
    for entry in pool {
        // The identifier follows the source text, so that neither another renderer nor a reordering of the
        // pool changes it (see `identity`). The same question asked twice is a mistake, whatever its answers.
        let id = fingerprint(&entry.source);
        if !seen.insert(fingerprint(&entry.question)) {
            let excerpt: String = entry.question.chars().take(70).collect();
            return ctx.fail(format!("duplicate question in the pool: \"{excerpt}\""));
        }
        questions.push(ExamQuestion { id, question: entry.question, options: entry.options, explanation: entry.explanation });
    }
    Ok(Exam { title: field(&meta, "title"), draw, pass_mark, minutes, shuffle, intro_html, questions, warnings })
}

fn load_course(directory: &Path, checks: &Checks) -> Result<Course> {
    let path = directory.join(COURSE_FILE);
    if !path.exists() {
        return Err(ContentError::new(directory, "`course.md` is missing"));
    }
    let (meta, body) = split_front_matter(&read(&path)?, &path)?;
    require(&meta, &["title", "icon", "summary"], &path)?;
    let slug = directory.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    // v1 courses named the engine that simulated their labs in the browser.
    if meta.get("engine").is_some_and(|value| !is_falsy(value)) {
        return Err(ContentError::new(&path, "`engine` no longer exists: labs run in the course's real `environment`"));
    }
    if directory.join("sandbox.yml").exists() {
        return Err(ContentError::new(&directory.join("sandbox.yml"), "the simulated sandbox no longer exists: remove this file"));
    }
    let mut environments = Environments { directory, seen: BTreeMap::new() };
    let course_environment = match meta.get("environment").filter(|v| !is_falsy(v)) {
        Some(value) => {
            environments.resolve(value)?;
            text(value)
        }
        None => String::new(),
    };

    let render_page = |file: &Path, source: &str| -> Result<String> {
        let mut lab_ctx = LabContext { checks, environment: String::new(), resolver: None };
        Ok(render_document(source, &mut Render::new(file, &slug), Some(&mut lab_ctx))?.0)
    };
    let description_html = render_page(&path, &body)?;
    let cheat_path = directory.join("cheatsheet.md");
    let cheatsheet_html = if cheat_path.exists() { render_page(&cheat_path, &read(&cheat_path)?)? } else { String::new() };

    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .map_err(|err| ContentError::new(directory, format!("cannot be read: {err}")))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| LESSON_FILE_RE.is_match(n)))
        .collect();
    files.sort();
    let mut lessons = Vec::with_capacity(files.len());
    let mut seen = BTreeSet::new();
    for file in &files {
        let lesson = load_lesson(file, &slug, checks, &course_environment, &mut environments)?;
        if RESERVED_LESSON_IDS.contains(&lesson.slug.as_str()) {
            return Err(ContentError::new(directory, format!("the lesson id `{}` is reserved (URL of the validation exam)", lesson.slug)));
        }
        if !seen.insert(lesson.slug.clone()) {
            return Err(ContentError::new(directory, format!("duplicate lesson id: {}", lesson.slug)));
        }
        lessons.push(lesson);
    }

    let exam_path = directory.join(EXAM_FILE);
    let exam = if exam_path.exists() { Some(load_exam(&exam_path, &slug)?) } else { None };
    let published = meta.get("published").is_none_or(|v| !is_falsy(v));
    if published && lessons.is_empty() {
        return Err(ContentError::new(directory, "a published course must contain at least one `NN-name.md` lesson"));
    }
    Ok(Course {
        title: field(&meta, "title"),
        icon: field(&meta, "icon"),
        summary: field(&meta, "summary"),
        requires: meta.get("requires").and_then(Yaml::as_sequence).map(|items| items.iter().map(text).collect()).unwrap_or_default(),
        published,
        accent: field(&meta, "color"),
        banner: field(&meta, "banner"),
        description_html,
        cheatsheet_html,
        environments: environments.seen,
        exam,
        lessons,
        slug,
    })
}

/// Compiles the whole catalogue in `directory` (`catalogue.yml` index, one sub-folder per course).
pub fn load_catalogue(directory: &Path) -> Result<Catalogue> {
    let index_path = directory.join("catalogue.yml");
    let unreadable = |err: String| ContentError::new(&index_path, format!("cannot read the catalogue index: {err}"));
    let index: Yaml = serde_yaml::from_str(&fs::read_to_string(&index_path).map_err(|e| unreadable(e.to_string()))?)
        .map_err(|e| unreadable(e.to_string()))?;
    let slugs: Vec<String> =
        index.get("courses").and_then(Yaml::as_sequence).map(|items| items.iter().map(text).collect()).unwrap_or_default();

    let checks_path = directory.join(CHECKS_FILE);
    let unreadable = |err: String| ContentError::new(&checks_path, format!("cannot read the checks: {err}"));
    #[derive(serde::Deserialize)]
    struct ChecksFile {
        checks: Checks,
    }
    let checks = serde_yaml::from_str::<ChecksFile>(&fs::read_to_string(&checks_path).map_err(|e| unreadable(e.to_string()))?)
        .map_err(|e| unreadable(e.to_string()))?
        .checks;

    let catalogue = load_courses(directory, &slugs, &checks)?;
    for course in &catalogue.courses {
        if let Some(unknown) = course.requires.iter().find(|required| !slugs.contains(required)) {
            return Err(ContentError::new(&directory.join(&course.slug).join(COURSE_FILE), format!("unknown prerequisite: {unknown}")));
        }
    }
    Ok(catalogue)
}

/// Compiles the course folders `slugs` of `directory`, in that order. Prerequisites are left to the caller.
pub(crate) fn load_courses(directory: &Path, slugs: &[String], checks: &Checks) -> Result<Catalogue> {
    let courses = slugs.iter().map(|slug| load_course(&directory.join(slug), checks)).collect::<Result<Vec<_>>>()?;
    Ok(Catalogue { courses })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_and_body() {
        let (meta, body) = split_front_matter("---\ntitle: \"A : b\"\nminutes: 5\n---\nCorps\n---\nfin", Path::new("l.md")).unwrap();
        assert_eq!(field(&meta, "title"), "A : b");
        assert_eq!(field(&meta, "minutes"), "5");
        assert_eq!(body, "Corps\n---\nfin");
    }

    #[test]
    fn missing_front_matter() {
        let err = split_front_matter("# Titre\n", Path::new("l.md")).unwrap_err();
        assert_eq!(err.to_string(), "l.md: missing YAML front matter (a `---` block at the top of the file)");
    }

    #[test]
    fn required_fields() {
        let (meta, _) = split_front_matter("---\ntitle: T\nsummary: ''\n---\n", Path::new("l.md")).unwrap();
        let err = require(&meta, &["title", "summary", "minutes"], Path::new("l.md")).unwrap_err();
        assert!(err.message.ends_with(": summary, minutes"));
    }
}
