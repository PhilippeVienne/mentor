//! Cohorts: groups of learners, usually mirrored from the groups of the tenant's identity provider.

/// Identifier of the cohort for an identity-provider group path: `/promo-2027/info` gives `promo-2027-info`.
/// Empty when the group has no usable character.
pub fn cohort_slug(group: &str) -> String {
    let mut slug = String::new();
    for c in group.trim_matches('/').chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            slug.extend(c.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').chars().take(100).collect()
}

/// Display name of that cohort: the last path segment, dashes as spaces, first letter capitalised.
pub fn cohort_name(group: &str) -> String {
    let last = group.trim_matches('/').rsplit('/').next().unwrap_or_default().replace('-', " ").to_lowercase();
    let mut chars = last.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_paths_become_slugs() {
        assert_eq!(cohort_slug("/promo-2027/info"), "promo-2027-info");
        assert_eq!(cohort_slug("/Équipe Dev & Ops/"), "équipe-dev-ops");
        assert_eq!(cohort_slug("///"), "");
    }

    #[test]
    fn names_come_from_the_last_segment() {
        assert_eq!(cohort_name("/promo-2027/info"), "Info");
        assert_eq!(cohort_name("/asso-demo"), "Asso demo");
    }
}
