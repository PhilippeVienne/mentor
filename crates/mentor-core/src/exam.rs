//! Validation exam of a course: drawing questions, timing an attempt, grading it.
//!
//! Port of the exam rules of `formation/training/services.py` (v1). Passing the exam validates every lesson of
//! the course (see [`crate::progress::LessonProgress::validate_by_exam`]). Time is expressed in seconds since
//! the epoch, and randomness comes from the caller, so that everything here stays deterministic under test.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Network tolerance when an attempt is submitted after its deadline.
pub const GRACE_SECONDS: i64 = 30;

/// A question of the pool, as far as the rules are concerned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolQuestion {
    pub id: String,
    /// Number of options.
    pub options: usize,
    /// Index of the correct option, in the order written by the author.
    pub correct: usize,
}

/// Source of randomness. The web tier must back it with a cryptographically secure generator: a learner
/// must not be able to predict a draw.
pub trait Randomness {
    /// A uniformly distributed integer in `0..upper` (`upper` is never 0).
    fn below(&mut self, upper: usize) -> usize;
}

/// One question of an attempt, with the order in which its options are shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawnQuestion {
    pub id: String,
    /// `option_order[shown position] = index of the option as written by the author`.
    pub option_order: Vec<usize>,
}

fn shuffle<T>(items: &mut [T], rng: &mut impl Randomness) {
    for last in (1..items.len()).rev() {
        items.swap(last, rng.below(last + 1));
    }
}

/// Draws `count` questions from the pool (all of them when the pool is smaller).
///
/// With `shuffle_order`, questions and options are shown in a random order; without it, the drawn questions
/// keep the pool order and options keep the author's order.
pub fn draw(pool: &[PoolQuestion], count: usize, shuffle_order: bool, rng: &mut impl Randomness) -> Vec<DrawnQuestion> {
    let mut picked: Vec<usize> = (0..pool.len()).collect();
    shuffle(&mut picked, rng);
    picked.truncate(count);
    if !shuffle_order {
        picked.sort_unstable();
    }
    picked
        .into_iter()
        .map(|index| {
            let mut option_order: Vec<usize> = (0..pool[index].options).collect();
            if shuffle_order {
                shuffle(&mut option_order, rng);
            }
            DrawnQuestion { id: pool[index].id.clone(), option_order }
        })
        .collect()
}

/// Whether an attempt submitted at `now` is too late.
pub fn is_expired(deadline: i64, now: i64) -> bool {
    now > deadline + GRACE_SECONDS
}

/// Seconds to wait before a new attempt, after a failed one finished at `last_failure` (0 when allowed).
pub fn retry_after(last_failure: Option<i64>, cooldown_seconds: i64, now: i64) -> i64 {
    last_failure.map_or(0, |finished| (finished + cooldown_seconds - now).max(0))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuestionResult {
    pub id: String,
    /// Shown position the learner chose; `None` when missing or invalid.
    pub chosen: Option<usize>,
    /// Shown position of the correct option.
    pub correct_position: usize,
    pub correct: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub score: usize,
    pub total: usize,
    pub passed: bool,
    pub results: Vec<QuestionResult>,
}

/// Grades an attempt. `answers` maps a question id to the shown position chosen; a missing or out-of-range
/// answer is wrong. Questions that left the pool since the draw are ignored. `pass_mark` is a percentage.
pub fn grade(pool: &[PoolQuestion], drawn: &[DrawnQuestion], answers: &BTreeMap<String, usize>, pass_mark: u32) -> Grade {
    let mut results = Vec::with_capacity(drawn.len());
    for question in drawn {
        let Some(source) = pool.iter().find(|candidate| candidate.id == question.id) else { continue };
        let Some(correct_position) = question.option_order.iter().position(|&original| original == source.correct) else { continue };
        let chosen = answers.get(&question.id).copied().filter(|&position| position < question.option_order.len());
        results.push(QuestionResult { id: question.id.clone(), chosen, correct_position, correct: chosen == Some(correct_position) });
    }
    let (score, total) = (results.iter().filter(|result| result.correct).count(), results.len());
    Grade { score, total, passed: total > 0 && score * 100 >= pass_mark as usize * total, results }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic generator: replays a fixed sequence, then keeps returning 0.
    struct Fixed(Vec<usize>);

    impl Randomness for Fixed {
        fn below(&mut self, upper: usize) -> usize {
            if self.0.is_empty() {
                0
            } else {
                self.0.remove(0) % upper
            }
        }
    }

    fn pool(size: usize) -> Vec<PoolQuestion> {
        (0..size).map(|i| PoolQuestion { id: format!("q{i}"), options: 3, correct: i % 3 }).collect()
    }

    #[test]
    fn draw_without_shuffle_keeps_pool_and_option_order() {
        let drawn = draw(&pool(5), 3, false, &mut Fixed(vec![1, 3, 0, 2]));
        assert_eq!(drawn.len(), 3);
        let ids: Vec<&str> = drawn.iter().map(|q| q.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
        assert!(drawn.iter().all(|q| q.option_order == [0, 1, 2]));
    }

    #[test]
    fn draw_never_repeats_a_question_and_keeps_every_option() {
        let drawn = draw(&pool(6), 4, true, &mut Fixed(vec![5, 1, 3, 2, 0, 1, 2, 0, 1, 2, 1, 0]));
        let mut ids: Vec<&str> = drawn.iter().map(|q| q.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 4);
        for question in &drawn {
            let mut order = question.option_order.clone();
            order.sort_unstable();
            assert_eq!(order, [0, 1, 2]);
        }
    }

    #[test]
    fn draw_is_capped_by_the_pool() {
        assert_eq!(draw(&pool(2), 10, true, &mut Fixed(vec![])).len(), 2);
    }

    #[test]
    fn grading_follows_the_shown_order() {
        let pool = pool(2); // correct options: q0 → 0, q1 → 1
        let drawn = vec![
            DrawnQuestion { id: "q0".into(), option_order: vec![2, 0, 1] }, // correct option shown at position 1
            DrawnQuestion { id: "q1".into(), option_order: vec![1, 0, 2] }, // correct option shown at position 0
        ];
        let answers: BTreeMap<String, usize> = [("q0".to_string(), 1), ("q1".to_string(), 2)].into();
        let result = grade(&pool, &drawn, &answers, 50);
        assert_eq!((result.score, result.total, result.passed), (1, 2, true));
        assert_eq!(result.results[0], QuestionResult { id: "q0".into(), chosen: Some(1), correct_position: 1, correct: true });
        assert!(!result.results[1].correct);
        assert!(!grade(&pool, &drawn, &answers, 51).passed);
    }

    #[test]
    fn missing_or_invalid_answers_are_wrong() {
        let pool = pool(2);
        let drawn = draw(&pool, 2, false, &mut Fixed(vec![]));
        let answers: BTreeMap<String, usize> = [("q0".to_string(), 7)].into();
        let result = grade(&pool, &drawn, &answers, 80);
        assert_eq!((result.score, result.passed), (0, false));
        assert!(result.results.iter().all(|r| r.chosen.is_none()));
    }

    #[test]
    fn questions_removed_from_the_pool_are_ignored() {
        let drawn = vec![DrawnQuestion { id: "gone".into(), option_order: vec![0, 1] }];
        let result = grade(&pool(1), &drawn, &BTreeMap::new(), 80);
        assert_eq!((result.total, result.passed), (0, false));
    }

    #[test]
    fn timing() {
        assert!(!is_expired(1000, 1000 + GRACE_SECONDS));
        assert!(is_expired(1000, 1001 + GRACE_SECONDS));
        assert_eq!(retry_after(None, 600, 5000), 0);
        assert_eq!(retry_after(Some(5000), 600, 5100), 500);
        assert_eq!(retry_after(Some(5000), 600, 9000), 0);
    }
}
