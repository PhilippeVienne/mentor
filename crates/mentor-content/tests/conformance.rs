//! Conformance with v1: the catalogue compiled here is compared with the JSON produced by
//! `manage.py export_catalogue` (`conformance/v1-catalogue.json`, source commit in `conformance/V1_COMMIT`).
//!
//! Two levels:
//! - **structure** (identifiers, order, durations, labs, checks, solutions, correct answers): strict equality;
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
    ["description_html", "cheatsheet_html", "body_html", "intro_html", "intro", "texte", "indice", "question", "html", "explication"];

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

fn report() -> (Vec<String>, Vec<String>, usize) {
    let v1: Value = serde_json::from_str(&std::fs::read_to_string(root().join("conformance/v1-catalogue.json")).unwrap()).unwrap();
    let v2 = serde_json::to_value(mentor_content::load_catalogue(&root().join("catalogue")).expect("the catalogue compiles")).unwrap();
    let (mut structure, mut html) = (Vec::new(), Vec::new());
    let (a, b) = (v1["courses"].as_array().unwrap(), v2["courses"].as_array().unwrap());
    assert_eq!(a.len(), b.len(), "number of courses");
    for (x, y) in a.iter().zip(b) {
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
    let (_, html, _) = report();
    assert!(html.is_empty(), "{} fragment(s) whose text differs:\n{}", html.len(), html.join("\n"));
}
