//! Validates labs (`:::lab`): steps, checks, solutions.
//!
//! Every lab runs in a real environment of its course and is verified by the server. v1 also had labs
//! simulated in the browser (`git` and `docker` engines, effects, a simulated server): they no longer exist.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_yaml::Value as Yaml;

use crate::error::Result;
use crate::markdown::{md, md_inline, Render};
use crate::yaml::{is_falsy, text, to_json};

/// The only value `engine` may still have: before simulated labs were removed, it told real labs apart.
const REAL_ENGINE: &str = "real";
/// Keys that only meant something for simulated labs.
const SIMULATED_ONLY: [&str; 2] = ["server", "effect"];

static FILE_NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9._-][A-Za-z0-9._/-]*$").unwrap());

/// Declaration of a check in `_checks.yml`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CheckSpec {
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

pub type Checks = BTreeMap<String, CheckSpec>;

/// Validates the name of a real environment (a folder of the course); fails with the message to show.
pub type Resolver<'a> = &'a mut dyn FnMut(&Yaml) -> Result<()>;

/// What a lab needs to be validated.
pub struct LabContext<'a> {
    pub checks: &'a Checks,
    /// Real environment inherited from the lesson or the course.
    pub environment: String,
    /// `None` outside a lesson, where `environment` is not allowed.
    pub resolver: Option<Resolver<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lab {
    /// Folder of the course holding the environment the lab runs in.
    pub environment: String,
    pub intro: String,
    /// Files created in the working folder when the environment starts.
    pub files: Json,
    /// Commands run when the environment starts, as the learner.
    pub commands: Json,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub text: String,
    pub hint: String,
    pub checks: Vec<Check>,
    /// Zero-based indices of the steps to validate before this one.
    pub after: Vec<usize>,
    pub solution: Vec<Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    pub name: String,
    pub args: Vec<Json>,
}

fn get<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a Yaml> {
    map.get(key)
}

/// Python's `value or default`: the value when truthy, the default otherwise.
fn or_default(value: Option<&Yaml>, default: Json) -> Json {
    match value {
        Some(v) if !is_falsy(v) => to_json(v),
        _ => default,
    }
}

fn as_list(value: &Yaml) -> Vec<Yaml> {
    match value {
        Yaml::Sequence(items) => items.clone(),
        other => vec![other.clone()],
    }
}

fn normalize_checks(raw: &Yaml, lab: &LabContext, place: &str, ctx: &Render) -> Result<Vec<Check>> {
    let mut out = Vec::new();
    for item in as_list(raw) {
        let entry = match &item {
            Yaml::Mapping(map) if map.len() == 1 => map.iter().next(),
            _ => None,
        };
        let Some((name, value)) = entry else {
            return ctx.fail(format!("{place}: a check is written `- name: argument` (a single name per item)"));
        };
        let name = text(name);
        let args: Vec<Yaml> = match value {
            Yaml::Null | Yaml::Bool(true) => Vec::new(),
            other => as_list(other),
        };
        let Some(spec) = lab.checks.get(&name) else {
            let known: Vec<&str> = lab.checks.keys().map(String::as_str).collect();
            return ctx.fail(format!("{place}: unknown check `{name}`. Available: {}", known.join(", ")));
        };
        let (required, optional) = (spec.arguments.len(), spec.optional.len());
        if args.len() < required || args.len() > required + optional {
            return ctx.fail(format!(
                "{place}: `{name}` expects {required} argument(s) ({}), got {}",
                spec.arguments.join(", "),
                args.len()
            ));
        }
        for arg in &args {
            let ok = match arg {
                Yaml::String(s) => !s.contains('\0'),
                Yaml::Number(_) => true,
                _ => false,
            };
            if !ok {
                return ctx.fail(format!("{place}: `{name}`: arguments must be text (without a NUL character)"));
            }
        }
        out.push(Check { name, args: args.iter().map(to_json).collect() });
    }
    Ok(out)
}

/// Validates the setup files and commands of a lab.
fn check_setup(data: &serde_yaml::Mapping, place: &str, ctx: &Render) -> Result<()> {
    match get(data, "files").filter(|v| !is_falsy(v)) {
        None => {}
        Some(Yaml::Mapping(files)) => {
            for (name, content) in files {
                let valid = match (name, content) {
                    (Yaml::String(name), Yaml::String(_)) => FILE_NAME_RE.is_match(name) && !name.split('/').any(|part| part == ".."),
                    _ => false,
                };
                if !valid {
                    return ctx.fail(format!("{place}: invalid file `{}` (plain relative path, text content)", text(name)));
                }
            }
        }
        Some(_) => return ctx.fail(format!("{place}: `files` must be a `name: content` mapping")),
    }
    match get(data, "commands").filter(|v| !is_falsy(v)) {
        None => Ok(()),
        Some(Yaml::Sequence(commands)) if commands.iter().all(Yaml::is_string) => Ok(()),
        Some(_) => ctx.fail(format!("{place}: `commands` must be a list of commands (text)")),
    }
}

fn parse_step(step: &Yaml, index: usize, place: &str, lab: &LabContext, ctx: &Render) -> Result<Step> {
    let label = format!("{place}, step {index}");
    let empty = serde_yaml::Mapping::new();
    let step = step.as_mapping().unwrap_or(&empty);
    for key in ["text", "checks", "solution"] {
        if get(step, key).is_none() {
            return ctx.fail(format!("{label}: field `{key}` is required"));
        }
    }
    let solution = as_list(&step["solution"]);
    for action in &solution {
        let valid = match action {
            Yaml::String(_) => true,
            Yaml::Mapping(map) => map.len() == 1 && map.contains_key("write"),
            _ => false,
        };
        if !valid {
            return ctx.fail(format!("{label}: `solution` holds commands (text) or `{{write: {{file: content}}}}`"));
        }
    }
    let mut after = Vec::new();
    for value in get(step, "after").map(as_list).unwrap_or_default() {
        match value.as_u64() {
            Some(n) if n >= 1 && (n as usize) < index => after.push(n as usize - 1),
            _ => return ctx.fail(format!("{label}: `after` must reference earlier step numbers")),
        }
    }
    refuse_simulated_keys(step, &label, ctx)?;
    Ok(Step {
        text: md_inline(&text(&step["text"])),
        hint: get(step, "hint").filter(|v| !is_falsy(v)).map(|v| md_inline(&text(v))).unwrap_or_default(),
        checks: normalize_checks(&step["checks"], lab, &label, ctx)?,
        after,
        solution: solution.iter().map(to_json).collect(),
    })
}

/// Simulated labs had keys of their own; naming one is a mistake worth telling the author about.
fn refuse_simulated_keys(data: &serde_yaml::Mapping, place: &str, ctx: &Render) -> Result<()> {
    match SIMULATED_ONLY.iter().find(|key| get(data, key).is_some_and(|value| !is_falsy(value))) {
        Some(key) => ctx.fail(format!("{place}: `{key}` belonged to simulated labs, which no longer exist")),
        None => Ok(()),
    }
}

/// Validates the YAML body of a `:::lab` block starting at `start_line`.
pub fn parse_lab(body: &str, start_line: usize, lab: &mut LabContext, ctx: &Render) -> Result<Lab> {
    let place = format!("line {start_line}: `:::lab`");
    let parsed: Yaml = match serde_yaml::from_str(body) {
        Ok(value) => value,
        Err(err) => return ctx.fail(format!("{place}: invalid YAML: {err}")),
    };
    let empty = serde_yaml::Mapping::new();
    let data = match &parsed {
        Yaml::Mapping(map) => map,
        other if is_falsy(other) => &empty,
        _ => return ctx.fail(format!("{place}: the content must be a YAML mapping")),
    };
    // `engine: real` is accepted for the labs written when simulated ones existed; it says nothing any more.
    if let Some(engine) = get(data, "engine").filter(|value| value.as_str() != Some(REAL_ENGINE)) {
        return ctx.fail(format!(
            "{place}: `engine: {}`: simulated labs no longer exist, every lab runs in a real environment (remove `engine`)",
            text(engine)
        ));
    }
    refuse_simulated_keys(data, &place, ctx)?;
    let mut environment = lab.environment.clone();
    if let Some(value) = get(data, "environment") {
        environment = if is_falsy(value) { String::new() } else { text(value) };
        if !environment.is_empty() {
            match lab.resolver.as_mut() {
                Some(resolve) => resolve(value)?,
                None => return ctx.fail(format!("{place}: `environment` is only allowed in a lesson")),
            }
        }
    }
    if environment.is_empty() {
        return ctx.fail(format!("{place}: a lab requires an `environment` (in the lab, the lesson or the course)"));
    }
    check_setup(data, &place, ctx)?;
    let steps = match get(data, "steps") {
        Some(Yaml::Sequence(steps)) if !steps.is_empty() => steps,
        _ => return ctx.fail(format!("{place}: `steps` must be a non-empty list")),
    };
    let mut parsed_steps = Vec::with_capacity(steps.len());
    for (i, step) in steps.iter().enumerate() {
        parsed_steps.push(parse_step(step, i + 1, &place, lab, ctx)?);
    }
    Ok(Lab {
        environment,
        intro: get(data, "intro").filter(|v| !is_falsy(v)).map(|v| md(&text(v))).unwrap_or_default(),
        files: or_default(get(data, "files"), Json::Object(Default::default())),
        commands: or_default(get(data, "commands"), Json::Array(Vec::new())),
        steps: parsed_steps,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn checks() -> Checks {
        serde_yaml::from_str("command-succeeds: {arguments: [command]}\nenv-file-contains: {arguments: [path, regex]}\n").unwrap()
    }

    fn parse(body: &str, environment: &str) -> Result<Lab> {
        let checks = checks();
        let mut lab = LabContext { checks: &checks, environment: environment.to_string(), resolver: None };
        parse_lab(body, 12, &mut lab, &Render::new(Path::new("lecon.md"), "demo"))
    }

    const STEP: &str =
        "steps:\n  - text: 'Lance `git init`'\n    checks:\n      - command-succeeds: 'test -d .git'\n    solution: git init\n";

    #[test]
    fn minimal_lab() {
        let lab = parse(STEP, "environnement").unwrap();
        assert_eq!(lab.environment, "environnement");
        assert_eq!(lab.steps[0].text, "Lance <code>git init</code>");
        assert_eq!(lab.steps[0].checks, vec![Check { name: "command-succeeds".into(), args: vec![Json::from("test -d .git")] }]);
        assert_eq!(lab.steps[0].solution, vec![Json::from("git init")]);
        assert_eq!((lab.files, lab.commands), (serde_json::json!({}), serde_json::json!([])));
    }

    #[test]
    fn after_is_converted_to_indices() {
        let body = format!("{STEP}  - text: suite\n    after: 1\n    checks: [{{command-succeeds: 'true'}}]\n    solution: [git status]\n");
        assert_eq!(parse(&body, "environnement").unwrap().steps[1].after, vec![0]);
    }

    #[test]
    fn after_cannot_reference_a_later_step() {
        let body = STEP.replace("    solution", "    after: [1]\n    solution");
        let err = parse(&body, "environnement").unwrap_err();
        assert!(err.message.ends_with("step 1: `after` must reference earlier step numbers"), "{err}");
    }

    #[test]
    fn unknown_check_lists_the_available_ones() {
        let err = parse(&STEP.replace("command-succeeds", "made-up"), "environnement").unwrap_err();
        assert!(err.message.contains("unknown check `made-up`. Available: command-succeeds, env-file-contains"));
    }

    #[test]
    fn a_check_takes_the_declared_number_of_text_arguments() {
        let err = parse(&STEP.replace("command-succeeds: 'test -d .git'", "env-file-contains: README.md"), "environnement").unwrap_err();
        assert!(err.message.contains("`env-file-contains` expects 2 argument(s) (path, regex), got 1"), "{err}");
        let err = parse(&STEP.replace("'test -d .git'", "{a: b}"), "environnement").unwrap_err();
        assert!(err.message.contains("arguments must be text"), "{err}");
    }

    #[test]
    fn a_lab_requires_an_environment() {
        assert!(parse(STEP, "").unwrap_err().message.contains("a lab requires an `environment`"));
    }

    #[test]
    fn engine_real_is_tolerated_and_simulated_labs_are_refused() {
        assert!(parse(&format!("engine: real\n{STEP}"), "environnement").is_ok());
        let err = parse(&format!("engine: docker\n{STEP}"), "environnement").unwrap_err();
        assert!(err.message.contains("`engine: docker`: simulated labs no longer exist"), "{err}");
        let err = parse(&format!("server: [{{branch: main}}]\n{STEP}"), "environnement").unwrap_err();
        assert!(err.message.contains("`server` belonged to simulated labs"), "{err}");
        let err = parse(&format!("{STEP}    effect: {{server-advance: {{}}}}\n"), "environnement").unwrap_err();
        assert!(err.message.contains("step 1: `effect` belonged to simulated labs"), "{err}");
    }

    #[test]
    fn file_outside_the_folder_is_rejected() {
        let body = "files: {'../secret': x}\nsteps:\n  - {text: t, checks: [{command-succeeds: 'true'}], solution: ['true']}\n";
        assert!(parse(body, "environnement").unwrap_err().message.contains("invalid file `../secret`"));
    }
}
