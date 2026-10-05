//! Conformance with v1: the catalogue compiled here is compared with the JSON produced by
//! `manage.py export_catalogue` (`conformance/v1-catalogue.json`, source commit in `conformance/V1_COMMIT`).
//!
//! The v1 export uses the French names of the v1 format; it is first rewritten with the English names of
//! `conformance/v1-names.json` (the table the migration tool uses), then compared on two levels:
//! - **structure** (identifiers, order, durations, labs, checks, solutions, correct answers): strict equality;
//!
//! v1 also had labs simulated in the browser; v2 only has real ones. What only existed for simulated labs is
//! dropped from the v1 side before comparing, and the two courses that were simulated in v1 and are rewritten
//! as real labs here are compared on everything but their labs and lesson texts (see [`REWRITTEN_SINCE_V1`]).
//!
//! - **HTML**: markup differs (another Markdown engine, no server-side highlighting), so the **text** of each
//!   fragment is compared, with tags and whitespace removed.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
static SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// Fields whose value is rendered HTML.
const HTML_FIELDS: [&str; 10] =
    ["description_html", "cheatsheet_html", "body_html", "intro_html", "intro", "text", "hint", "question", "html", "explanation"];

/// Fragments where v1 let through HTML that the author meant as text (`echo "<h1>…</h1>"` in a hint,
/// `docker logs <nom>` in an answer): the browser interpreted it and words went missing. The catalogue now
/// writes them as code, so their text differs from v1's on purpose. One exam question was true of the
/// simulated terminal only.
const CORRECTED_SINCE_V1: [&str; 5] = [
    "git-basics.lessons[3].lab.steps[1].hint",
    "git-basics.lessons[4].lab.steps[1].hint",
    "git-basics.lessons[5].lab.steps[4].hint",
    "docker-hello.lessons[3].quiz[0].options[1].html",
    // `docker compose ps` hides exited containers: the question now says `ps -a`, as a real terminal needs.
    "docker-advanced.exam.questions[30].question",
];

/// Courses whose labs were simulated in v1 and are real here: their labs, environments and the texts that
/// describe them changed on purpose. Their identifiers, order, durations, objectives, quizzes and exam are
/// still compared.
const REWRITTEN_SINCE_V1: [&str; 2] = ["docker-hello", "docker-advanced"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn text_of(html: &str) -> String {
    let text = TAG_RE.replace_all(html, "");
    let text = text
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", "\u{a0}")
        .replace("&amp;", "&")
        // Known v1 defect: inside a table, an escaped pipe in a code span was displayed with its backslash.
        .replace("\\|", "|");
    SPACE_RE.replace_all(&text, "").to_string()
}

/// Walks both trees and sorts differences into "structure" or "HTML text".
fn compare(path: &str, key: &str, v1: &Value, v2: &Value, structure: &mut Vec<String>, html: &mut Vec<String>) {
    match (v1, v2) {
        (Value::Object(a), Value::Object(b)) => {
            // `environments`: only names are compared (the devcontainer specification is not ported yet).
            if key == "environments" {
                if a.keys().collect::<Vec<_>>() != b.keys().collect::<Vec<_>>() {
                    structure.push(format!(
                        "{path}: environments {:?} != {:?}",
                        a.keys().collect::<Vec<_>>(),
                        b.keys().collect::<Vec<_>>()
                    ));
                }
                return;
            }
            let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                match (a.get(k), b.get(k)) {
                    (Some(x), Some(y)) => compare(&format!("{path}.{k}"), k, x, y, structure, html),
                    (x, _) => structure.push(format!("{path}.{k}: only present in {}", if x.is_some() { "v1" } else { "v2" })),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                structure.push(format!("{path}: {} items in v1, {} in v2", a.len(), b.len()));
                return;
            }
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                compare(&format!("{path}[{i}]"), key, x, y, structure, html);
            }
        }
        (Value::String(a), Value::String(b)) if HTML_FIELDS.contains(&key) => {
            if text_of(a) != text_of(b) {
                html.push(path.to_string());
            }
        }
        // A question id is the digest of its HTML: it can only be equal when the HTML is byte-identical.
        (Value::String(_), Value::String(_)) if key == "id" && path.contains(".exam.questions[") => {}
        (a, b) if a != b => structure.push(format!("{path}: {a} != {b}")),
        _ => {}
    }
}

/// Renames the keys of a JSON object with one section of `conformance/v1-names.json`.
fn rename(value: &mut Value, names: &Value) {
    if let Value::Object(map) = value {
        let renamed =
            std::mem::take(map).into_iter().map(|(key, v)| (names[&key].as_str().map(str::to_string).unwrap_or(key), v)).collect();
        *map = renamed;
    }
}

fn each(value: &mut Value, mut f: impl FnMut(&mut Value)) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(f),
        Value::Object(map) => map.values_mut().for_each(f),
        Value::Null => {}
        other => f(other),
    }
}

fn rename_value(value: &mut Value, names: &Value) {
    if let Some(new) = value.as_str().and_then(|old| names[old].as_str()) {
        *value = Value::from(new);
    }
}

/// A git server description: a list of `{url, commits: [{branch, message, files, author}]}`.
fn translate_server(server: &mut Value, names: &Value) {
    each(server, |entry| each(&mut entry["commits"], |commit| rename(commit, &names["commit"])));
}

/// Rewrites the v1 export with the English names of the v2 format, so that both can be compared.
fn translate_v1(course: &mut Value, names: &Value) {
    each(&mut course["scenarios"], |scenario| {
        rename(scenario, &names["scenario"]);
        if let Some(server) = scenario.get_mut("server") {
            translate_server(server, names);
        }
    });
    let quiz_names = serde_json::json!({"explication": "explanation"});
    if course["exam"].is_object() {
        rename(&mut course["exam"], &names["exam"]);
        each(&mut course["exam"]["questions"], |question| rename(question, &quiz_names));
    }
    each(&mut course["lessons"], |lesson| {
        each(&mut lesson["quiz"], |question| rename(question, &quiz_names));
        let lab = &mut lesson["lab"];
        if !lab.is_object() {
            return;
        }
        rename(lab, &names["lab"]);
        rename_value(&mut lab["engine"], &names["engines"]);
        translate_server(&mut lab["server"], names);
        each(&mut lab["steps"], |step| {
            rename(step, &names["step"]);
            each(&mut step["checks"], |check| {
                rename(check, &serde_json::json!({"nom": "name"}));
                rename_value(&mut check["name"], &names["checks"]);
            });
            if step["effect"].is_object() {
                rename(&mut step["effect"], &serde_json::json!({"nom": "name"}));
                rename_value(&mut step["effect"]["name"], &names["effects"]);
                rename(&mut step["effect"]["args"], &names["commit"]);
            }
            each(&mut step["solution"], |action| rename(action, &names["action"]));
        });
    });
}

fn remove(value: &mut Value, keys: &[&str]) {
    if let Value::Object(map) = value {
        keys.iter().for_each(|key| {
            map.remove(*key);
        });
    }
}

/// Drops from a v1 course what only existed for simulated labs. Returns whether it had simulated labs.
fn drop_simulated(course: &mut Value) -> bool {
    remove(course, &["engine", "scenarios"]);
    let mut simulated = false;
    each(&mut course["lessons"], |lesson| {
        let lab = &mut lesson["lab"];
        if !lab.is_object() {
            return;
        }
        simulated |= lab["engine"] != "real";
        remove(lab, &["engine", "server"]);
        each(&mut lab["steps"], |step| remove(step, &["effect"]));
    });
    simulated
}

/// Leaves out of the comparison what a rewritten course changed on purpose.
fn drop_rewritten(course: &mut Value) {
    remove(course, &["environments", "description_html", "cheatsheet_html"]);
    each(&mut course["lessons"], |lesson| remove(lesson, &["lab", "environment", "body_html"]));
}

fn read_json(relative: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(relative)).unwrap()).unwrap()
}

fn report() -> (Vec<String>, Vec<String>, usize) {
    let names = read_json("conformance/v1-names.json");
    let mut v1 = read_json("conformance/v1-catalogue.json");
    let mut v2 = serde_json::to_value(mentor_content::load_catalogue(&root().join("catalogue")).expect("the catalogue compiles")).unwrap();
    let (mut structure, mut html) = (Vec::new(), Vec::new());
    let (a, b) = (v1["courses"].as_array_mut().unwrap(), v2["courses"].as_array_mut().unwrap());
    assert_eq!(a.len(), b.len(), "number of courses");
    for (x, y) in a.iter_mut().zip(b) {
        translate_v1(x, &names);
        let slug = x["slug"].as_str().unwrap().to_string();
        let simulated = drop_simulated(x);
        // Exactly the courses listed as rewritten had simulated labs: none is skipped by mistake.
        assert_eq!(simulated, REWRITTEN_SINCE_V1.contains(&slug.as_str()), "{slug}: simulated labs in v1");
        if simulated {
            drop_rewritten(x);
            drop_rewritten(y);
        }
        // Exam warnings are diagnostics, now in English: only their number is compared.
        for exam in [&mut x["exam"], &mut y["exam"]] {
            if exam.is_object() {
                exam["warnings"] = Value::from(exam["warnings"].as_array().map_or(0, Vec::len));
            }
        }
        compare(x["slug"].as_str().unwrap(), "", x, y, &mut structure, &mut html);
    }
    (structure, html, a.len())
}

#[test]
fn catalogue_structure_matches_v1() {
    let (structure, _, courses) = report();
    assert_eq!(courses, 19);
    assert!(structure.is_empty(), "{} structural difference(s):\n{}", structure.len(), structure.join("\n"));
}

#[test]
fn html_fragment_text_matches_v1() {
    let (_, mut html, _) = report();
    // Each listed fragment must still differ: a correction that is gone must leave the list.
    for corrected in CORRECTED_SINCE_V1 {
        assert!(html.iter().any(|path| path == corrected), "{corrected} no longer differs from v1");
    }
    html.retain(|path| !CORRECTED_SINCE_V1.contains(&path.as_str()));
    assert!(html.is_empty(), "{} fragment(s) whose text differs:\n{}", html.len(), html.join("\n"));
}
