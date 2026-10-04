//! Loads a catalogue: index, courses, lessons, exams.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;
use serde_json::Value as Json;
use serde_yaml::{Mapping, Value as Yaml};
use sha1::{Digest, Sha1};

use crate::document::{render_document, Quiz, QuizOption};
use crate::error::{ContentError, Result};
use crate::lab::{Checks, Lab, LabContext, ENGINES};
use crate::markdown::{md_inline, Render};
use crate::yaml::{is_falsy, text, to_json};

static LESSON_FILE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(.+)\.md$").unwrap());

const CHECKS_FILE: &str = "_verifications.yml";
const EXAM_FILE: &str = "examen.md";
/// `/parcours/<slug>/examen/` is the URL of the validation exam.
const RESERVED_LESSON_IDS: [&str; 1] = ["examen"];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Catalogue {
    pub courses: Vec<Course>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Course {
    pub slug: String,
    pub title: String,
    pub icon: String,
    pub summary: String,
    /// Simulated engine of the course (`git`, `docker`), or empty.
    pub engine: String,
    pub requires: Vec<String>,
    pub published: bool,
    pub accent: String,
    pub banner: String,
    pub description_html: String,
    pub cheatsheet_html: String,
    pub scenarios: Json,
    /// Real environments used by the course, by folder name. The devcontainer specification is not
    /// validated here yet (to be ported with the execution plane).
    pub environments: BTreeMap<String, Json>,
    pub exam: Option<Exam>,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exam {
    #[serde(rename = "titre")]
    pub title: String,
    /// Number of questions drawn from the pool.
    #[serde(rename = "tirage")]
    pub draw: u32,
    /// Required percentage of correct answers.
    #[serde(rename = "seuil")]
    pub pass_mark: u32,
    /// Duration in minutes.
    #[serde(rename = "duree")]
    pub minutes: u32,
    /// Whether questions and answers are shuffled.
    #[serde(rename = "melange")]
    pub shuffle: bool,
    pub intro_html: String,
    pub questions: Vec<ExamQuestion>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExamQuestion {
    /// Digest of the question text: stable when other questions are added or removed.
    pub id: String,
    pub question: String,
    pub options: Vec<QuizOption>,
    #[serde(rename = "explication")]
    pub explanation: String,
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| ContentError::new(path, format!("lecture impossible : {err}")))
}

/// Separates the YAML front matter (between two `---` lines) from the Markdown body.
fn split_front_matter(source: &str, path: &Path) -> Result<(Mapping, String)> {
    let missing = || ContentError::new(path, "front matter YAML manquant (bloc `---` en tête de fichier)");
    let rest = source.strip_prefix("---\n").ok_or_else(missing)?;
    // The front matter may be empty: the closing line then immediately follows the opening one.
    let (raw, after) = match rest.strip_prefix("---") {
        Some(after) => ("", after),
        None => rest.split_once("\n---").ok_or_else(missing)?,
    };
    let body = after.strip_prefix('\n').unwrap_or(after);
    let meta: Yaml = serde_yaml::from_str(raw).map_err(|err| ContentError::new(path, format!("front matter YAML invalide : {err}")))?;
    match meta {
        Yaml::Mapping(map) => Ok((map, body.to_string())),
        Yaml::Null => Ok((Mapping::new(), body.to_string())),
        _ => Err(ContentError::new(path, "le front matter doit être un dictionnaire YAML")),
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
        Err(ContentError::new(path, format!("champ(s) obligatoire(s) manquant(s) dans le front matter : {}", missing.join(", "))))
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
            return Err(ContentError::new(
                &self.directory.join("parcours.md"),
                "`environnement` doit être le nom d'un dossier du parcours",
            ));
        };
        if !self.seen.contains_key(name) {
            let folder = self.directory.join(name);
            if !folder.join("devcontainer.json").is_file() {
                return Err(ContentError::new(&folder, "environnement réel introuvable : `devcontainer.json` manquant"));
            }
            self.seen.insert(name.to_string(), Json::Object(Default::default()));
        }
        Ok(())
    }
}

fn load_lesson(
    path: &Path,
    course: &str,
    engine: Option<&str>,
    checks: &Checks,
    inherited: &str,
    environments: &mut Environments,
) -> Result<Lesson> {
    let (meta, body) = split_front_matter(&read(path)?, path)?;
    require(&meta, &["id", "titre", "resume", "duree"], path)?;
    let mut environment = inherited.to_string();
    if let Some(value) = meta.get("environnement") {
        environment = if is_falsy(value) { String::new() } else { text(value) };
        if !environment.is_empty() {
            environments.resolve(value)?;
        }
    }
    let mut ctx = Render::new(path, course);
    let mut resolve = |name: &Yaml| environments.resolve(name);
    let mut lab_ctx = LabContext { engine, checks, environment: environment.clone(), resolver: Some(&mut resolve) };
    let (body_html, mut collected) = render_document(&body, &mut ctx, Some(&mut lab_ctx))?;
    if collected.labs.len() > 1 {
        return ctx.fail("une leçon ne peut contenir qu'un seul bloc `:::labo`");
    }
    let objectives = match meta.get("objectifs") {
        None | Some(Yaml::Null) => Vec::new(),
        Some(Yaml::Sequence(items)) => items.iter().map(|o| md_inline(&text(o))).collect(),
        Some(_) => return ctx.fail("`objectifs` doit être une liste"),
    };
    let lab = collected.labs.pop();
    let effective = lab.as_ref().map(|l| l.environment.as_str()).filter(|e| !e.is_empty()).unwrap_or(&environment);
    let minutes = field(&meta, "duree").trim().parse().or_else(|_| ctx.fail("`duree` doit être un nombre entier de minutes"))?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    Ok(Lesson {
        slug: field(&meta, "id"),
        environment: if effective.is_empty() { String::new() } else { format!("{course}--{effective}") },
        order: LESSON_FILE_RE.captures(name).and_then(|caps| caps[1].parse().ok()).unwrap_or(0),
        title: field(&meta, "titre"),
        summary: field(&meta, "resume"),
        minutes,
        objectives,
        body_html,
        lab,
        quiz: collected.quizzes,
    })
}

/// Stable identifier of a pool question: digest of its text.
fn question_id(question_html: &str) -> String {
    let digest = Sha1::digest(question_html.as_bytes());
    digest.iter().take(5).map(|byte| format!("{byte:02x}")).collect()
}

fn load_exam(path: &Path, course: &str) -> Result<Exam> {
    let (meta, body) = split_front_matter(&read(path)?, path)?;
    require(&meta, &["titre", "tirage", "seuil", "duree"], path)?;
    let bounded = |key: &str, low: u32, high: u32| -> Result<u32> {
        match meta.get(key).and_then(Yaml::as_u64) {
            Some(value) if (low as u64..=high as u64).contains(&value) => Ok(value as u32),
            _ => Err(ContentError::new(path, format!("`{key}` doit être un entier entre {low} et {high} (reçu : {})", field(&meta, key)))),
        }
    };
    let (draw, pass_mark, minutes) = (bounded("tirage", 1, 100)?, bounded("seuil", 1, 100)?, bounded("duree", 1, 240)?);
    let shuffle = match meta.get("melange") {
        None => true,
        Some(Yaml::Bool(value)) => *value,
        Some(_) => return Err(ContentError::new(path, "`melange` doit valoir true ou false")),
    };
    let mut ctx = Render::new(path, course);
    let (intro_html, collected) = render_document(&body, &mut ctx, None)?;
    let pool = collected.quizzes;
    if pool.len() < draw as usize {
        return ctx.fail(format!(
            "le pool contient {} question(s) mais `tirage` en demande {draw} : ajoute des blocs `:::quiz` ou baisse `tirage`",
            pool.len()
        ));
    }
    let mut warnings = Vec::new();
    if pool.len() < 2 * draw as usize {
        warnings.push(format!(
            "pool de {} questions pour un tirage de {draw} : prévois idéalement 3 fois plus de questions que le tirage (au moins 2 fois).",
            pool.len()
        ));
    }
    let mut seen = BTreeSet::new();
    let mut questions = Vec::with_capacity(pool.len());
    for entry in pool {
        let id = question_id(&entry.question);
        if !seen.insert(id.clone()) {
            let excerpt: String = entry.question.chars().take(70).collect();
            return ctx.fail(format!("question en double dans le pool : « {excerpt} »"));
        }
        questions.push(ExamQuestion { id, question: entry.question, options: entry.options, explanation: entry.explanation });
    }
    Ok(Exam { title: field(&meta, "titre"), draw, pass_mark, minutes, shuffle, intro_html, questions, warnings })
}

fn load_course(directory: &Path, checks: &Checks) -> Result<Course> {
    let path = directory.join("parcours.md");
    if !path.exists() {
        return Err(ContentError::new(directory, "fichier `parcours.md` manquant"));
    }
    let (meta, body) = split_front_matter(&read(&path)?, &path)?;
    require(&meta, &["titre", "icone", "resume"], &path)?;
    let slug = directory.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    let engine = match meta.get("moteur") {
        None | Some(Yaml::Null) => None,
        Some(Yaml::String(name)) if ENGINES.contains(&name.as_str()) => Some(name.clone()),
        Some(other) => return Err(ContentError::new(&path, format!("`moteur` doit valoir git ou docker (reçu : {})", text(other)))),
    };
    let engine = engine.as_deref();
    let mut environments = Environments { directory, seen: BTreeMap::new() };
    let course_environment = match meta.get("environnement").filter(|v| !is_falsy(v)) {
        Some(value) => {
            environments.resolve(value)?;
            text(value)
        }
        None => String::new(),
    };

    let render_page = |file: &Path, source: &str| -> Result<String> {
        let mut lab_ctx = LabContext { engine, checks, environment: String::new(), resolver: None };
        Ok(render_document(source, &mut Render::new(file, &slug), Some(&mut lab_ctx))?.0)
    };
    let description_html = render_page(&path, &body)?;
    let cheat_path = directory.join("antiseche.md");
    let cheatsheet_html = if cheat_path.exists() { render_page(&cheat_path, &read(&cheat_path)?)? } else { String::new() };

    let sandbox_path = directory.join("bac-a-sable.yml");
    let scenarios = if sandbox_path.exists() {
        let parsed: Yaml = serde_yaml::from_str(&read(&sandbox_path)?)
            .map_err(|err| ContentError::new(&sandbox_path, format!("YAML invalide : {err}")))?;
        parsed.get("scenarios").map(to_json).unwrap_or_else(|| Json::Object(Default::default()))
    } else {
        Json::Object(Default::default())
    };

    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .map_err(|err| ContentError::new(directory, format!("lecture impossible : {err}")))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| LESSON_FILE_RE.is_match(n)))
        .collect();
    files.sort();
    let mut lessons = Vec::with_capacity(files.len());
    let mut seen = BTreeSet::new();
    for file in &files {
        let lesson = load_lesson(file, &slug, engine, checks, &course_environment, &mut environments)?;
        if RESERVED_LESSON_IDS.contains(&lesson.slug.as_str()) {
            return Err(ContentError::new(
                directory,
                format!("l'identifiant de leçon `{}` est réservé (URL de l'examen de validation)", lesson.slug),
            ));
        }
        if !seen.insert(lesson.slug.clone()) {
            return Err(ContentError::new(directory, format!("identifiant de leçon en double : {}", lesson.slug)));
        }
        lessons.push(lesson);
    }

    let exam_path = directory.join(EXAM_FILE);
    let exam = if exam_path.exists() { Some(load_exam(&exam_path, &slug)?) } else { None };
    let published = meta.get("publie").is_none_or(|v| !is_falsy(v));
    if published && lessons.is_empty() {
        return Err(ContentError::new(directory, "un parcours publié doit contenir au moins une leçon `NN-nom.md`"));
    }
    Ok(Course {
        title: field(&meta, "titre"),
        icon: field(&meta, "icone"),
        summary: field(&meta, "resume"),
        engine: engine.unwrap_or_default().to_string(),
        requires: meta.get("prerequis").and_then(Yaml::as_sequence).map(|items| items.iter().map(text).collect()).unwrap_or_default(),
        published,
        accent: field(&meta, "couleur"),
        banner: field(&meta, "banniere"),
        description_html,
        cheatsheet_html,
        scenarios,
        environments: environments.seen,
        exam,
        lessons,
        slug,
    })
}

/// Compiles the whole catalogue in `directory` (`catalogue.yml` index, one sub-folder per course).
pub fn load_catalogue(directory: &Path) -> Result<Catalogue> {
    let index_path = directory.join("catalogue.yml");
    let unreadable = |err: String| ContentError::new(&index_path, format!("impossible de lire l'index du catalogue : {err}"));
    let index: Yaml = serde_yaml::from_str(&fs::read_to_string(&index_path).map_err(|e| unreadable(e.to_string()))?)
        .map_err(|e| unreadable(e.to_string()))?;
    let slugs: Vec<String> =
        index.get("parcours").and_then(Yaml::as_sequence).map(|items| items.iter().map(text).collect()).unwrap_or_default();

    let checks_path = directory.join(CHECKS_FILE);
    let unreadable = |err: String| ContentError::new(&checks_path, format!("impossible de lire les vérifications : {err}"));
    #[derive(serde::Deserialize)]
    struct ChecksFile {
        verifications: Checks,
    }
    let checks = serde_yaml::from_str::<ChecksFile>(&fs::read_to_string(&checks_path).map_err(|e| unreadable(e.to_string()))?)
        .map_err(|e| unreadable(e.to_string()))?
        .verifications;

    let courses = slugs.iter().map(|slug| load_course(&directory.join(slug), &checks)).collect::<Result<Vec<_>>>()?;
    for course in &courses {
        if let Some(unknown) = course.requires.iter().find(|required| !slugs.contains(required)) {
            return Err(ContentError::new(&directory.join(&course.slug).join("parcours.md"), format!("prérequis inconnu : {unknown}")));
        }
    }
    Ok(Catalogue { courses })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_and_body() {
        let (meta, body) = split_front_matter("---\ntitre: \"A : b\"\nduree: 5\n---\nCorps\n---\nfin", Path::new("l.md")).unwrap();
        assert_eq!(field(&meta, "titre"), "A : b");
        assert_eq!(field(&meta, "duree"), "5");
        assert_eq!(body, "Corps\n---\nfin");
    }

    #[test]
    fn missing_front_matter() {
        let err = split_front_matter("# Titre\n", Path::new("l.md")).unwrap_err();
        assert_eq!(err.to_string(), "l.md : front matter YAML manquant (bloc `---` en tête de fichier)");
    }

    #[test]
    fn required_fields() {
        let (meta, _) = split_front_matter("---\ntitre: T\nresume: ''\n---\n", Path::new("l.md")).unwrap();
        let err = require(&meta, &["titre", "resume", "duree"], Path::new("l.md")).unwrap_err();
        assert!(err.message.ends_with(": resume, duree"));
    }

    #[test]
    fn question_id_is_stable() {
        // Same value as `hashlib.sha1(b"Question ?").hexdigest()[:10]` in v1.
        assert_eq!(question_id("Question ?"), "782d7ec139");
    }
}
