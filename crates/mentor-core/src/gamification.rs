//! XP and levels. Port of `formation/training/gamification.py` (v1).

use serde::Serialize;

/// XP for each validated lab step.
pub const XP_TASK: u32 = 10;
/// XP for each correct quiz answer (best score).
pub const XP_QUIZ: u32 = 8;
/// XP for a completed lesson (lab and quiz).
pub const XP_LESSON: u32 = 50;
/// XP for a completed course.
pub const XP_COURSE: u32 = 150;
/// XP for a course validated through its exam (skipping labs and quizzes).
pub const XP_EXAM: u32 = 100;

/// XP thresholds of the levels. Titles come from the tenant's branding.
pub const LEVEL_THRESHOLDS: [u32; 7] = [0, 100, 250, 450, 700, 1000, 1400];

/// Default titles (neutral brand), in threshold order. They are French product strings.
pub const DEFAULT_LEVEL_TITLES: [&str; 7] =
    ["Nouvelle recrue", "Apprenti·e", "Contributeur·rice", "Développeur·se", "Mainteneur·se", "Membre de l'équipe", "Légende de la maison"];

/// Level reached for an XP total, and progress towards the next one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LevelInfo {
    /// Level, starting at 1.
    pub level: u32,
    pub title: String,
    pub xp: u32,
    /// Threshold of the current level.
    pub floor: u32,
    /// Threshold of the next level, `None` at the last level.
    pub next: Option<u32>,
    pub next_title: Option<String>,
    /// Progress towards the next level, from 0 to 100.
    pub percent: u32,
}

/// Computes the level for `xp` points, using the brand's `titles` (one per threshold).
pub fn level_info(xp: u32, titles: &[&str; 7]) -> LevelInfo {
    let idx = LEVEL_THRESHOLDS.iter().rposition(|&threshold| xp >= threshold).unwrap_or(0);
    let floor = LEVEL_THRESHOLDS[idx];
    let next = LEVEL_THRESHOLDS.get(idx + 1).copied();
    LevelInfo {
        level: idx as u32 + 1,
        title: titles[idx].to_string(),
        xp,
        floor,
        next,
        next_title: next.map(|_| titles[idx + 1].to_string()),
        percent: match next {
            None => 100,
            // Same rounding as Python (`round`, half to even) to stay conformant with v1.
            Some(n) => round_half_even((xp - floor) * 100, n - floor),
        },
    }
}

/// `round(numerator / denominator)` with Python's banker's rounding.
fn round_half_even(numerator: u32, denominator: u32) -> u32 {
    let (quotient, remainder) = (numerator / denominator, numerator % denominator);
    match (remainder * 2).cmp(&denominator) {
        std::cmp::Ordering::Less => quotient,
        std::cmp::Ordering::Greater => quotient + 1,
        std::cmp::Ordering::Equal => quotient + (quotient & 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_level_at_zero_xp() {
        let info = level_info(0, &DEFAULT_LEVEL_TITLES);
        assert_eq!((info.level, info.floor, info.next, info.percent), (1, 0, Some(100), 0));
        assert_eq!(info.title, "Nouvelle recrue");
        assert_eq!(info.next_title.as_deref(), Some("Apprenti·e"));
    }

    #[test]
    fn progress_between_two_thresholds() {
        let info = level_info(120, &DEFAULT_LEVEL_TITLES);
        assert_eq!((info.level, info.next), (2, Some(250)));
        assert_eq!(info.percent, 13); // 20 / 150
    }

    #[test]
    fn last_level_is_capped() {
        let info = level_info(5000, &DEFAULT_LEVEL_TITLES);
        assert_eq!((info.level, info.next, info.percent), (7, None, 100));
        assert_eq!(info.title, "Légende de la maison");
        assert_eq!(info.next_title, None);
    }

    #[test]
    fn an_exact_threshold_reaches_the_next_level() {
        assert_eq!(level_info(99, &DEFAULT_LEVEL_TITLES).level, 1);
        assert_eq!(level_info(100, &DEFAULT_LEVEL_TITLES).level, 2);
    }

    #[test]
    fn rounds_half_to_even_like_python() {
        assert_eq!(round_half_even(1, 2), 0); // round(0.5) == 0
        assert_eq!(round_half_even(3, 2), 2); // round(1.5) == 2
        assert_eq!(round_half_even(5, 2), 2); // round(2.5) == 2
        assert_eq!(round_half_even(7, 4), 2); // round(1.75) == 2
    }
}
