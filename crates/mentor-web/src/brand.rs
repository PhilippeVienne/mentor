//! Brand of a tenant: what a white-label deployment changes without touching code.

use serde::Deserialize;

/// Colours and texts of a tenant. Every field has a neutral default, so a tenant with no brand settings
/// still gets a complete one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Brand {
    /// Name shown in the header and in page titles.
    pub name: String,
    pub tagline: String,
    pub hero_title: String,
    /// End of the hero title, shown in the primary colour.
    pub hero_accent: String,
    pub hero_text: String,
    /// Organisation running the tenant; the brand name is used when empty.
    pub organisation: String,
    pub contact_email: String,
    pub source_url: String,
    pub primary: String,
    pub primary_strong: String,
    pub primary_text: String,
    pub secondary: String,
    pub dark_primary: String,
    pub dark_primary_strong: String,
    pub dark_primary_text: String,
}

impl Default for Brand {
    fn default() -> Self {
        Self {
            name: "Mentor".into(),
            tagline: "Apprends en pratiquant".into(),
            hero_title: "Apprends les outils du métier,".into(),
            hero_accent: "en pratiquant".into(),
            hero_text:
                "Des cours courts avec un vrai terminal dans le navigateur, des quiz, de l'XP, des niveaux et des badges à débloquer."
                    .into(),
            organisation: String::new(),
            contact_email: String::new(),
            source_url: "https://github.com/PhilippeVienne/mentor".into(),
            primary: "#4f46e5".into(),
            primary_strong: "#4338ca".into(),
            primary_text: "#4338ca".into(),
            secondary: "#06b6d4".into(),
            dark_primary: "#6366f1".into(),
            dark_primary_strong: "#818cf8".into(),
            dark_primary_text: "#a5b4fc".into(),
        }
    }
}

/// A colour that is safe to write into a style sheet: hexadecimal only.
fn is_hex_colour(value: &str) -> bool {
    matches!(value.len(), 4 | 7) && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl Brand {
    /// Builds the brand of a tenant from its stored settings. The tenant's display name is the brand name
    /// unless the settings give another. Colours that are not plain hexadecimal fall back to the defaults:
    /// they are written into a `<style>` block, where anything else could inject CSS.
    pub fn from_settings(tenant_name: &str, settings: &serde_json::Value) -> Self {
        let defaults = Self::default();
        let mut brand: Self = serde_json::from_value(settings.clone()).unwrap_or_else(|_| defaults.clone());
        if settings.get("name").and_then(|name| name.as_str()).is_none_or(str::is_empty) {
            brand.name = tenant_name.to_string();
        }
        for (colour, default) in [
            (&mut brand.primary, defaults.primary),
            (&mut brand.primary_strong, defaults.primary_strong),
            (&mut brand.primary_text, defaults.primary_text),
            (&mut brand.secondary, defaults.secondary),
            (&mut brand.dark_primary, defaults.dark_primary),
            (&mut brand.dark_primary_strong, defaults.dark_primary_strong),
            (&mut brand.dark_primary_text, defaults.dark_primary_text),
        ] {
            if !is_hex_colour(colour) {
                *colour = default;
            }
        }
        brand
    }

    /// Who the footer names.
    pub fn owner(&self) -> &str {
        if self.organisation.is_empty() {
            &self.name
        } else {
            &self.organisation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_settings_give_a_complete_brand_named_after_the_tenant() {
        let brand = Brand::from_settings("Acme", &serde_json::json!({}));
        assert_eq!((brand.name.as_str(), brand.primary.as_str(), brand.owner()), ("Acme", "#4f46e5", "Acme"));
    }

    #[test]
    fn settings_override_defaults() {
        let brand =
            Brand::from_settings("Acme", &serde_json::json!({"name": "Acme Academy", "organisation": "Acme Corp", "primary": "#112233"}));
        assert_eq!((brand.name.as_str(), brand.owner(), brand.primary.as_str()), ("Acme Academy", "Acme Corp", "#112233"));
        assert_eq!(brand.secondary, "#06b6d4");
    }

    #[test]
    fn a_colour_that_could_inject_css_is_replaced() {
        let brand = Brand::from_settings("Acme", &serde_json::json!({"primary": "red; } body { display: none"}));
        assert_eq!(brand.primary, "#4f46e5");
    }

    #[test]
    fn settings_of_the_wrong_shape_fall_back_to_defaults() {
        let brand = Brand::from_settings("Acme", &serde_json::json!({"primary": 12}));
        assert_eq!((brand.name.as_str(), brand.primary.as_str()), ("Acme", "#4f46e5"));
    }
}
