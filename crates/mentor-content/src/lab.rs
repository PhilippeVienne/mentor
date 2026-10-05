//! Validates labs (`:::labo`): steps, checks, solutions.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_yaml::Value as Yaml;

use crate::error::Result;
use crate::markdown::{md, md_inline, Render};
use crate::yaml::{is_falsy, text, to_json};

/// Engines simulated in the browser.
pub const ENGINES: [&str; 2] = ["git", "docker"];
/// Engine of labs run in a real environment: checks are performed by the server.
pub const REAL_ENGINE: &str = "real";
const EFFECTS: [&str; 2] = ["server-advance", "server-commit"];

static FILE_NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9._-][A-Za-z0-9._/-]*$").unwrap());

/// Declaration of a check in `_verifications.yml`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CheckSpec {
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
    /// Engines that know this check; `None` means every simulated engine.
    pub engines: Option<Vec<String>>,
}

pub type Checks = BTreeMap<String, CheckSpec>;

/// Validates the name of a real environment (a folder of the course); fails with the message to show.
pub type Resolver<'a> = &'a mut dyn FnMut(&Yaml) -> Result<()>;

/// What a lab needs to be validated.
pub struct LabContext<'a> {
    /// Engine of the course, used when the lab does not name its own.
    pub engine: Option<&'a str>,
    pub checks: &'a Checks,
    /// Real environment inherited from the lesson or the course.
    pub environment: String,
    /// `None` outside a lesson, where `environnement` is not allowed.
    pub resolver: Option<Resolver<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lab {
    pub engine: String,
    pub environment: String,
    pub intro: String,
    pub files: Json,
    pub commands: Json,
    pub server: Json,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub text: String,
    pub hint: String,
    pub checks: Vec<Check>,
    /// Zero-based indices of the steps to validate before this one.
    pub after: Vec<usize>,
    pub effect: Option<Effect>,
    pub solution: Vec<Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    pub name: String,
    pub args: Vec<Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Effect {
    pub name: String,
    pub args: Json,
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

fn normalize_checks(raw: &Yaml, lab: &LabContext, engine: &str, place: &str, ctx: &Render) -> Result<Vec<Check>> {
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
        let engines = spec.engines.as_deref().unwrap_or_default();
        if !engines.is_empty() && !engines.iter().any(|e| e == engine) {
            return ctx.fail(format!("{place}: the check `{name}` does not exist for the `{engine}` engine"));
        }
        if engine == REAL_ENGINE && !engines.iter().any(|e| e == REAL_ENGINE) {
            return ctx.fail(format!(
                "{place}: the check `{name}` does not exist for the `real` engine (use the `env-…` checks, `command-succeeds`, `output-contains`…)"
            ));
        }
        let (required, optional) = (spec.arguments.len(), spec.optional.len());
        if args.len() < required || args.len() > required + optional {
            return ctx.fail(format!(
                "{place}: `{name}` expects {required} argument(s) ({}), got {}",
                spec.arguments.join(", "),
                args.len()
            ));
        }
        if engine == REAL_ENGINE {
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
        }
        out.push(Check { name, args: args.iter().map(to_json).collect() });
    }
    Ok(out)
}

/// Validates the setup files and commands of a real lab.
fn check_real_setup(data: &serde_yaml::Mapping, place: &str, ctx: &Render) -> Result<()> {
    if get(data, "server").is_some_and(|v| !is_falsy(v)) {
        return ctx.fail(format!("{place}: `server` does not exist for the `real` engine"));
    }
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

fn parse_step(step: &Yaml, index: usize, engine: &str, place: &str, lab: &LabContext, ctx: &Render) -> Result<Step> {
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
    let effect = match get(step, "effect") {
        None | Some(Yaml::Null) => None,
        Some(_) if engine == REAL_ENGINE => return ctx.fail(format!("{label}: `effect` does not exist for the `real` engine")),
        Some(Yaml::Mapping(map)) if map.len() == 1 && map.keys().all(|k| k.as_str().is_some_and(|k| EFFECTS.contains(&k))) => {
            map.iter().next().map(|(name, args)| Effect { name: text(name), args: to_json(args) })
        }
        Some(_) => return ctx.fail(format!("{label}: unknown `effect` (available: {})", EFFECTS.join(", "))),
    };
    Ok(Step {
        text: md_inline(&text(&step["text"])),
        hint: get(step, "hint").filter(|v| !is_falsy(v)).map(|v| md_inline(&text(v))).unwrap_or_default(),
        checks: normalize_checks(&step["checks"], lab, engine, &label, ctx)?,
        after,
        effect,
        solution: solution.iter().map(to_json).collect(),
    })
}

/// Validates the YAML body of a `:::labo` block starting at `start_line`.
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
    let engine = match get(data, "engine") {
        Some(value) => value.as_str().map(str::to_string),
        None => lab.engine.map(str::to_string),
    };
    let Some(engine) = engine.filter(|e| ENGINES.contains(&e.as_str()) || e == REAL_ENGINE) else {
        return ctx.fail(format!("{place}: `engine` must be git, docker or real (does the course have no engine?)"));
    };
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
    if engine == REAL_ENGINE {
        if environment.is_empty() {
            return ctx
                .fail(format!("{place}: a lab with `engine: real` requires an `environment` (in the lab, the lesson or the course)"));
        }
        check_real_setup(data, &place, ctx)?;
    }
    let steps = match get(data, "steps") {
        Some(Yaml::Sequence(steps)) if !steps.is_empty() => steps,
        _ => return ctx.fail(format!("{place}: `steps` must be a non-empty list")),
    };
    let mut parsed_steps = Vec::with_capacity(steps.len());
    for (i, step) in steps.iter().enumerate() {
        parsed_steps.push(parse_step(step, i + 1, &engine, &place, lab, ctx)?);
    }
    Ok(Lab {
        engine,
        environment,
        intro: get(data, "intro").filter(|v| !is_falsy(v)).map(|v| md(&text(v))).unwrap_or_default(),
        files: or_default(get(data, "files"), Json::Object(Default::default())),
        commands: or_default(get(data, "commands"), Json::Array(Vec::new())),
        server: or_default(get(data, "server"), Json::Array(Vec::new())),
        steps: parsed_steps,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn checks() -> Checks {
        serde_yaml::from_str(
            "command: {arguments: [regex]}\nrepo-initialized: {engines: [git]}\ncommand-succeeds: {arguments: [command], engines: [real]}\n",
        )
        .unwrap()
    }

    fn parse(body: &str, engine: Option<&str>, environment: &str) -> Result<Lab> {
        let checks = checks();
        let mut lab = LabContext { engine, checks: &checks, environment: environment.to_string(), resolver: None };
        parse_lab(body, 12, &mut lab, &Render::new(Path::new("lecon.md"), "demo"))
    }

    const STEP: &str = "steps:\n  - text: 'Lance `git init`'\n    checks:\n      - repo-initialized:\n    solution: git init\n";

    #[test]
    fn minimal_simulated_lab() {
        let lab = parse(STEP, Some("git"), "").unwrap();
        assert_eq!(lab.engine, "git");
        assert_eq!(lab.steps[0].text, "Lance <code>git init</code>");
        assert_eq!(lab.steps[0].checks, vec![Check { name: "repo-initialized".into(), args: vec![] }]);
        assert_eq!(lab.steps[0].solution, vec![Json::from("git init")]);
        assert_eq!(lab.files, serde_json::json!({}));
    }

    #[test]
    fn after_is_converted_to_indices() {
        let body = format!("{STEP}  - text: suite\n    after: 1\n    checks: [{{command: '^git'}}]\n    solution: [git status]\n");
        assert_eq!(parse(&body, Some("git"), "").unwrap().steps[1].after, vec![0]);
    }

    #[test]
    fn after_cannot_reference_a_later_step() {
        let body = STEP.replace("    solution", "    after: [1]\n    solution");
        let err = parse(&body, Some("git"), "").unwrap_err();
        assert!(err.message.ends_with("step 1: `after` must reference earlier step numbers"), "{err}");
    }

    #[test]
    fn unknown_check_lists_the_available_ones() {
        let err = parse(&STEP.replace("repo-initialized", "made-up"), Some("git"), "").unwrap_err();
        assert!(err.message.contains("unknown check `made-up`. Available: command, command-succeeds, repo-initialized"));
    }

    #[test]
    fn check_from_another_engine_is_rejected() {
        let err = parse(STEP, Some("docker"), "").unwrap_err();
        assert!(err.message.contains("does not exist for the `docker` engine"));
    }

    #[test]
    fn real_lab_requires_an_environment_and_real_checks() {
        let body = "engine: real\nsteps:\n  - text: t\n    checks: [{command-succeeds: 'true'}]\n    solution: ['true']\n";
        assert!(parse(body, None, "").unwrap_err().message.contains("requires an `environment`"));
        assert_eq!(parse(body, None, "environnement").unwrap().environment, "environnement");
        let simulated = body.replace("command-succeeds: 'true'", "command: '^x'");
        assert!(parse(&simulated, None, "environnement").unwrap_err().message.contains("does not exist for the `real` engine"));
    }

    #[test]
    fn real_lab_file_outside_the_folder_is_rejected() {
        let body =
            "engine: real\nfiles: {'../secret': x}\nsteps:\n  - {text: t, checks: [{command-succeeds: 'true'}], solution: ['true']}\n";
        assert!(parse(body, None, "environnement").unwrap_err().message.contains("invalid file `../secret`"));
    }

    #[test]
    fn lab_without_any_engine_is_rejected() {
        assert!(parse(STEP, None, "").unwrap_err().message.contains("`engine` must be git, docker or real"));
    }
}
