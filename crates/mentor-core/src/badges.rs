//! Badges: what a learner has achieved, and the rules that turn it into badges.
//!
//! Port of the rules of `formation/training/gamification.py` (v1). In v1 the badge list was hard-coded,
//! including badges tied to lessons of specific courses; here a badge is data ([`BadgeDef`]), so that each
//! tenant's catalogue can declare its own. [`default_badges`] keeps v1's list (French product strings and
//! slugs, which existing learners' badges refer to).

use std::collections::BTreeSet;

use serde::Serialize;

/// Prefix of the badge awarded for completing a course: `parcours-<slug>` (kept from v1).
pub const COURSE_BADGE_PREFIX: &str = "parcours-";

/// What a learner has achieved so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    pub level: u32,
    /// Lab steps validated by practising (lessons validated through an exam do not count).
    pub tasks: u32,
    /// Quizzes answered without a mistake (same exclusion).
    pub perfect_quizzes: u32,
    /// Courses validated through their exam.
    pub exams_passed: u32,
    /// Longest run of consecutive days with activity.
    pub streak: u32,
    /// `course/lesson` references of completed lessons.
    pub lessons_done: BTreeSet<String>,
    pub courses_done: BTreeSet<String>,
}

/// Condition to earn a badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    TasksAtLeast(u32),
    PerfectQuizzesAtLeast(u32),
    StreakAtLeast(u32),
    ExamsPassedAtLeast(u32),
    LevelAtLeast(u32),
    /// A given lesson is completed (`course/lesson`).
    LessonDone(String),
}

impl Rule {
    pub fn is_met(&self, stats: &Stats) -> bool {
        match self {
            Rule::TasksAtLeast(n) => stats.tasks >= *n,
            Rule::PerfectQuizzesAtLeast(n) => stats.perfect_quizzes >= *n,
            Rule::StreakAtLeast(n) => stats.streak >= *n,
            Rule::ExamsPassedAtLeast(n) => stats.exams_passed >= *n,
            Rule::LevelAtLeast(n) => stats.level >= *n,
            Rule::LessonDone(reference) => stats.lessons_done.contains(reference),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadgeDef {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub emoji: String,
    pub tier: Tier,
    pub rule: Rule,
}

/// Slugs of the badges `stats` earns and the learner does not own yet: rule badges first, in the order of
/// `definitions`, then one badge per completed course.
pub fn newly_earned(definitions: &[BadgeDef], owned: &BTreeSet<String>, stats: &Stats) -> Vec<String> {
    let from_rules = definitions.iter().filter(|badge| badge.rule.is_met(stats)).map(|badge| badge.slug.clone());
    let from_courses = stats.courses_done.iter().map(|course| format!("{COURSE_BADGE_PREFIX}{course}"));
    from_rules.chain(from_courses).filter(|slug| !owned.contains(slug)).collect()
}

/// The badges of v1. Names and descriptions are French product strings.
pub fn default_badges() -> Vec<BadgeDef> {
    let lesson = |reference: &str| Rule::LessonDone(reference.to_string());
    [
        ("premier-pas", "Premier pas", "Valide ta toute première étape de labo.", "🌱", Tier::Bronze, Rule::TasksAtLeast(1)),
        ("sans-faute", "Sans faute", "Obtiens 100 % à un quiz.", "🎯", Tier::Silver, Rule::PerfectQuizzesAtLeast(1)),
        ("regulier", "Régulier·e", "Apprends 3 jours de suite.", "🔥", Tier::Silver, Rule::StreakAtLeast(3)),
        (
            "premier-commit",
            "Premier commit",
            "Termine la leçon « Ton premier commit ».",
            "💾",
            Tier::Bronze,
            lesson("git-basics/premier-commit"),
        ),
        ("brancheur", "Brancheur·se", "Termine la leçon « Branches et fusions ».", "🌿", Tier::Bronze, lesson("git-basics/branches")),
        (
            "chasseur-conflits",
            "Chasseur·se de conflits",
            "Résous ton premier conflit de fusion.",
            "⚔️",
            Tier::Silver,
            lesson("git-basics/conflits"),
        ),
        (
            "premier-conteneur",
            "Premier conteneur",
            "Termine la leçon « Ton premier conteneur ».",
            "📦",
            Tier::Bronze,
            lesson("docker-hello/premier-conteneur"),
        ),
        (
            "dockerfile-master",
            "Dockerfile master",
            "Termine la leçon « Écrire un Dockerfile ».",
            "🧱",
            Tier::Silver,
            lesson("docker-advanced/dockerfile"),
        ),
        ("orchestre", "Chef d'orchestre", "Termine la leçon « Docker Compose ».", "🎼", Tier::Silver, lesson("docker-advanced/compose")),
        (
            "premier-test-vert",
            "Premier test vert",
            "Termine la leçon « Fonctions et modules » : tes premiers tests pytest passent.",
            "🟢",
            Tier::Bronze,
            lesson("python/fonctions-modules"),
        ),
        (
            "outil-en-ligne-de-commande",
            "Outil en ligne de commande",
            "Termine le mini-projet Python : un vrai outil, testé de bout en bout.",
            "🧰",
            Tier::Silver,
            lesson("python/mini-projet"),
        ),
        (
            "valide-par-examen",
            "Validé·e par examen",
            "Valide un cours complet en réussissant son examen.",
            "🎓",
            Tier::Silver,
            Rule::ExamsPassedAtLeast(1),
        ),
        ("niveau-3", "Contributeur·rice", "Atteins le niveau 3.", "🛠️", Tier::Silver, Rule::LevelAtLeast(3)),
        ("niveau-5", "Mainteneur·se", "Atteins le niveau 5.", "🏗️", Tier::Gold, Rule::LevelAtLeast(5)),
    ]
    .into_iter()
    .map(|(slug, name, description, emoji, tier, rule)| BadgeDef {
        slug: slug.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        emoji: emoji.to_string(),
        tier,
        rule,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_earned_from_scratch() {
        assert!(newly_earned(&default_badges(), &BTreeSet::new(), &Stats::default()).is_empty());
    }

    #[test]
    fn rules_read_the_stats() {
        let stats = Stats { tasks: 1, level: 3, lessons_done: ["git-basics/branches".to_string()].into(), ..Stats::default() };
        assert_eq!(newly_earned(&default_badges(), &BTreeSet::new(), &stats), ["premier-pas", "brancheur", "niveau-3"]);
    }

    #[test]
    fn owned_badges_are_not_earned_again() {
        let stats = Stats { tasks: 5, streak: 3, ..Stats::default() };
        let owned: BTreeSet<String> = ["premier-pas".to_string()].into();
        assert_eq!(newly_earned(&default_badges(), &owned, &stats), ["regulier"]);
    }

    #[test]
    fn each_completed_course_gives_its_badge() {
        let stats = Stats { courses_done: ["docker-hello".to_string(), "git-basics".to_string()].into(), ..Stats::default() };
        let owned: BTreeSet<String> = ["parcours-git-basics".to_string()].into();
        assert_eq!(newly_earned(&[], &owned, &stats), ["parcours-docker-hello"]);
    }

    #[test]
    fn v1_badge_list_is_complete_and_unique() {
        let badges = default_badges();
        let slugs: BTreeSet<&str> = badges.iter().map(|badge| badge.slug.as_str()).collect();
        assert_eq!((badges.len(), slugs.len()), (14, 14));
    }
}
