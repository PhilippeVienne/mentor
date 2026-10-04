//! Bridges between YAML (written by authors) and JSON (compiler output).

use serde_json::Value as Json;
use serde_yaml::Value as Yaml;

/// Converts a YAML value to JSON; non-string keys are written as text.
pub fn to_json(value: &Yaml) -> Json {
    match value {
        Yaml::Null => Json::Null,
        Yaml::Bool(b) => Json::Bool(*b),
        Yaml::Number(n) => {
            if let Some(i) = n.as_i64() {
                Json::from(i)
            } else if let Some(u) = n.as_u64() {
                Json::from(u)
            } else {
                n.as_f64().map(Json::from).unwrap_or(Json::Null)
            }
        }
        Yaml::String(s) => Json::String(s.clone()),
        Yaml::Sequence(items) => Json::Array(items.iter().map(to_json).collect()),
        Yaml::Mapping(map) => Json::Object(map.iter().map(|(k, v)| (text(k), to_json(v))).collect()),
        Yaml::Tagged(tagged) => to_json(&tagged.value),
    }
}

/// Equivalent of Python's `str(value)`, for the fields v1 converted to text.
pub fn text(value: &Yaml) -> String {
    match value {
        Yaml::Null => "None".to_string(),
        Yaml::Bool(true) => "True".to_string(),
        Yaml::Bool(false) => "False".to_string(),
        Yaml::Number(n) => n.to_string(),
        Yaml::String(s) => s.clone(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim_end().to_string(),
    }
}

/// True for values Python considers falsy (`None`, `""`, `0`, `[]`, `{}`, `False`).
pub fn is_falsy(value: &Yaml) -> bool {
    match value {
        Yaml::Null => true,
        Yaml::Bool(b) => !b,
        Yaml::Number(n) => n.as_f64() == Some(0.0),
        Yaml::String(s) => s.is_empty(),
        Yaml::Sequence(items) => items.is_empty(),
        Yaml::Mapping(map) => map.is_empty(),
        Yaml::Tagged(tagged) => is_falsy(&tagged.value),
    }
}
