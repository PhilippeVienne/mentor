//! Lesson progress: applying learner events, unlocking courses, streaks.
//!
//! Port of the rules of `formation/training/services.py` (v1). Nothing here touches storage: callers load a
//! [`LessonProgress`], apply an [`Event`], persist the result and the [`Award`]s. An award is identified by its
//! `key`; storing it at most once per learner is what makes XP idempotent.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::gamification::{XP_COURSE, XP_EXAM, XP_LESSON, XP_QUIZ, XP_TASK};

/// What the rules need to know about a lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonRules {
    pub course: String,
    pub slug: String,
    /// Number of lab steps.
    pub tasks: u32,
    /// Number of quiz questions.
    pub questions: u32,
    /// The lab runs in a real environment: its steps are validated by the server only.
    pub server_verified: bool,
    /// The lab can actually be done on this platform. When it cannot (a real lab without real environments, a
    /// simulated lab on a platform that does not run them yet), it does not gate the quiz: the lesson ends
    /// with the reading and the quiz.
    pub lab_available: bool,
}

impl LessonRules {
    /// `course/lesson`, the reference used in award keys and in the set of completed lessons.
    pub fn reference(&self) -> String {
        format!("{}/{}", self.course, self.slug)
    }

    /// Whether the lab must be finished before the quiz.
    pub fn lab_required(&self) -> bool {
        self.tasks > 0 && self.lab_available
    }
}

/// A learner's state on one lesson.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LessonProgress {
    /// Zero-based indices of validated lab steps.
    pub tasks_done: BTreeSet<u32>,
    /// Best number of correct quiz answers so far.
    pub quiz_best: u32,
    pub completed: bool,
    /// The lesson was completed by passing the course exam, not by practising.
    pub validated_by_exam: bool,
}

impl LessonProgress {
    pub fn lab_done(&self, lesson: &LessonRules) -> bool {
        !lesson.lab_required() || self.tasks_done.len() as u32 >= lesson.tasks
    }

    /// Marks the lesson as completed through the course exam. Returns `false` when it was already completed:
    /// progress earned by practising is never overwritten.
    pub fn validate_by_exam(&mut self, lesson: &LessonRules) -> bool {
        if self.completed {
            return false;
        }
        self.tasks_done = (0..lesson.tasks).collect();
        self.quiz_best = lesson.questions;
        self.completed = true;
        self.validated_by_exam = true;
        true
    }

    /// A quiz is passed with at least two thirds of correct answers.
    pub fn quiz_ok(&self, lesson: &LessonRules) -> bool {
        lesson.questions == 0 || self.quiz_best * 3 >= lesson.questions * 2
    }
}

/// Something a learner did in a lesson.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A lab step was validated (zero-based index).
    Task(u32),
    /// A quiz was submitted with this number of correct answers.
    Quiz(u32),
}

/// Who reports an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The learner's browser (simulated labs, quizzes).
    Browser,
    /// The server, after checking a step inside the learner's real environment.
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AwardKind {
    Task,
    Quiz,
    Lesson,
    Course,
    Exam,
}

/// XP granted for one achievement. `(kind, key)` is unique per learner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Award {
    pub kind: AwardKind,
    pub key: String,
    pub xp: u32,
}

impl Award {
    /// XP for finishing every lesson of a course.
    pub fn course(slug: &str) -> Self {
        Self { kind: AwardKind::Course, key: slug.to_string(), xp: XP_COURSE }
    }

    /// XP for validating a course through its exam.
    pub fn exam(slug: &str) -> Self {
        Self { kind: AwardKind::Exam, key: slug.to_string(), xp: XP_EXAM }
    }
}

/// Why an event was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordError {
    /// The course's prerequisites are not completed.
    CourseLocked,
    /// The steps of a real lab can only be reported by the server.
    ServerVerifiedLab,
    /// The step index is out of range.
    InvalidTask,
    /// The lab must be finished before the quiz.
    LabNotDone,
    /// The score is higher than the number of questions.
    InvalidScore,
}

/// Result of an accepted event.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Recorded {
    /// Awards to store; none when the event brought nothing new.
    pub awards: Vec<Award>,
    /// The lesson was completed by this event.
    pub lesson_completed: bool,
}

/// Applies `event` to `progress` and returns the awards it earns.
///
/// Replaying an event is harmless: a step already validated or a score not above the best one earns nothing.
pub fn record(lesson: &LessonRules, progress: &mut LessonProgress, event: Event, source: Source) -> Result<Recorded, RecordError> {
    let reference = lesson.reference();
    let mut awards = Vec::new();
    match event {
        Event::Task(_) if lesson.server_verified && source != Source::Server => return Err(RecordError::ServerVerifiedLab),
        Event::Task(index) => {
            if index >= lesson.tasks {
                return Err(RecordError::InvalidTask);
            }
            if progress.tasks_done.insert(index) {
                awards.push(Award { kind: AwardKind::Task, key: format!("{reference}/t{index}"), xp: XP_TASK });
            }
        }
        Event::Quiz(score) => {
            if !progress.lab_done(lesson) {
                return Err(RecordError::LabNotDone);
            }
            if score > lesson.questions {
                return Err(RecordError::InvalidScore);
            }
            // One award per additional correct answer: improving a score only pays the difference.
            for answer in progress.quiz_best + 1..=score {
                awards.push(Award { kind: AwardKind::Quiz, key: format!("{reference}/q{answer}"), xp: XP_QUIZ });
            }
            progress.quiz_best = progress.quiz_best.max(score);
        }
    }
    let lesson_completed = !progress.completed && progress.lab_done(lesson) && progress.quiz_ok(lesson);
    if lesson_completed {
        progress.completed = true;
        awards.push(Award { kind: AwardKind::Lesson, key: reference, xp: XP_LESSON });
    }
    Ok(Recorded { awards, lesson_completed })
}

/// What unlocking and completion need to know about a course.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseShape {
    pub slug: String,
    pub published: bool,
    /// Slugs of the courses to complete first.
    pub requires: Vec<String>,
    /// Slugs of the lessons, in order.
    pub lessons: Vec<String>,
}

impl CourseShape {
    /// A published course whose lessons are all in `lessons_done` (`course/lesson` references).
    pub fn is_completed(&self, lessons_done: &BTreeSet<String>) -> bool {
        self.published
            && !self.lessons.is_empty()
            && self.lessons.iter().all(|lesson| lessons_done.contains(&format!("{}/{lesson}", self.slug)))
    }

    /// A course is open once it is published and all its prerequisites are completed.
    pub fn is_unlocked(&self, courses_done: &BTreeSet<String>) -> bool {
        self.published && self.requires.iter().all(|required| courses_done.contains(required))
    }
}

/// Slugs of the completed courses.
pub fn completed_courses(courses: &[CourseShape], lessons_done: &BTreeSet<String>) -> BTreeSet<String> {
    courses.iter().filter(|course| course.is_completed(lessons_done)).map(|course| course.slug.clone()).collect()
}

/// Longest run of consecutive days with activity. `days` are day numbers (for instance days since the epoch,
/// in the learner's time zone), in any order, possibly repeated.
pub fn longest_streak(days: impl IntoIterator<Item = i64>) -> u32 {
    let days: BTreeSet<i64> = days.into_iter().collect();
    let (mut best, mut run, mut previous) = (0, 0, None);
    for day in days {
        run = if previous == Some(day - 1) { run + 1 } else { 1 };
        best = best.max(run);
        previous = Some(day);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lesson(tasks: u32, questions: u32) -> LessonRules {
        LessonRules { course: "git".into(), slug: "intro".into(), tasks, questions, server_verified: false, lab_available: true }
    }

    fn xp(recorded: &Recorded) -> u32 {
        recorded.awards.iter().map(|award| award.xp).sum()
    }

    #[test]
    fn a_step_is_awarded_once() {
        let (lesson, mut progress) = (lesson(2, 3), LessonProgress::default());
        let first = record(&lesson, &mut progress, Event::Task(0), Source::Browser).unwrap();
        assert_eq!(first.awards, vec![Award { kind: AwardKind::Task, key: "git/intro/t0".into(), xp: XP_TASK }]);
        let again = record(&lesson, &mut progress, Event::Task(0), Source::Browser).unwrap();
        assert!(again.awards.is_empty() && !again.lesson_completed);
    }

    #[test]
    fn step_out_of_range_is_refused() {
        assert_eq!(record(&lesson(2, 0), &mut LessonProgress::default(), Event::Task(2), Source::Browser), Err(RecordError::InvalidTask));
    }

    #[test]
    fn quiz_waits_for_the_lab() {
        let (lesson, mut progress) = (lesson(1, 3), LessonProgress::default());
        assert_eq!(record(&lesson, &mut progress, Event::Quiz(3), Source::Browser), Err(RecordError::LabNotDone));
        record(&lesson, &mut progress, Event::Task(0), Source::Browser).unwrap();
        assert!(record(&lesson, &mut progress, Event::Quiz(3), Source::Browser).unwrap().lesson_completed);
    }

    #[test]
    fn improving_a_quiz_score_pays_only_the_difference() {
        let (lesson, mut progress) = (lesson(0, 4), LessonProgress::default());
        let first = record(&lesson, &mut progress, Event::Quiz(2), Source::Browser).unwrap();
        assert_eq!((xp(&first), first.lesson_completed), (2 * XP_QUIZ, false)); // 2/4 is below two thirds
        let lower = record(&lesson, &mut progress, Event::Quiz(1), Source::Browser).unwrap();
        assert_eq!((xp(&lower), progress.quiz_best), (0, 2));
        let better = record(&lesson, &mut progress, Event::Quiz(3), Source::Browser).unwrap();
        assert_eq!(xp(&better), XP_QUIZ + XP_LESSON);
        assert_eq!(better.awards[0].key, "git/intro/q3");
        assert!(better.lesson_completed && progress.completed);
    }

    #[test]
    fn a_lesson_is_completed_only_once() {
        let (lesson, mut progress) = (lesson(0, 3), LessonProgress::default());
        assert!(record(&lesson, &mut progress, Event::Quiz(2), Source::Browser).unwrap().lesson_completed);
        let perfect = record(&lesson, &mut progress, Event::Quiz(3), Source::Browser).unwrap();
        assert!(!perfect.lesson_completed);
        assert_eq!(xp(&perfect), XP_QUIZ);
    }

    #[test]
    fn score_above_the_number_of_questions_is_refused() {
        assert_eq!(record(&lesson(0, 3), &mut LessonProgress::default(), Event::Quiz(4), Source::Browser), Err(RecordError::InvalidScore));
    }

    #[test]
    fn real_lab_steps_come_from_the_server_only() {
        let real = LessonRules { server_verified: true, lab_available: true, ..lesson(1, 0) };
        let mut progress = LessonProgress::default();
        assert_eq!(record(&real, &mut progress, Event::Task(0), Source::Browser), Err(RecordError::ServerVerifiedLab));
        assert!(record(&real, &mut progress, Event::Task(0), Source::Server).unwrap().lesson_completed);
    }

    #[test]
    fn a_real_lab_does_not_gate_the_quiz_without_real_environments() {
        let real = LessonRules { server_verified: true, lab_available: false, ..lesson(3, 3) };
        assert!(!real.lab_required());
        assert!(record(&real, &mut LessonProgress::default(), Event::Quiz(2), Source::Browser).unwrap().lesson_completed);
        let available = LessonRules { lab_available: true, ..real };
        assert_eq!(record(&available, &mut LessonProgress::default(), Event::Quiz(2), Source::Browser), Err(RecordError::LabNotDone));
    }

    fn course(slug: &str, published: bool, requires: &[&str], lessons: &[&str]) -> CourseShape {
        CourseShape {
            slug: slug.into(),
            published,
            requires: requires.iter().map(|s| s.to_string()).collect(),
            lessons: lessons.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn course_completion_and_unlocking() {
        let courses = [course("a", true, &[], &["a1", "a2"]), course("b", true, &["a"], &["b1"]), course("soon", false, &[], &[])];
        let partial: BTreeSet<String> = ["a/a1".to_string()].into();
        assert!(completed_courses(&courses, &partial).is_empty());
        assert!(!courses[1].is_unlocked(&completed_courses(&courses, &partial)));
        let all: BTreeSet<String> = ["a/a1".to_string(), "a/a2".to_string()].into();
        let done = completed_courses(&courses, &all);
        assert_eq!(done, ["a".to_string()].into());
        assert!(courses[1].is_unlocked(&done));
        // An unpublished course is neither completed (it has no lesson) nor open.
        assert!(!courses[2].is_unlocked(&done) && !courses[2].is_completed(&all));
    }

    #[test]
    fn exam_validation_fills_a_lesson_but_keeps_practised_ones() {
        let lesson = lesson(2, 3);
        let mut untouched = LessonProgress::default();
        assert!(untouched.validate_by_exam(&lesson));
        assert_eq!((untouched.tasks_done.len(), untouched.quiz_best), (2, 3));
        assert!(untouched.completed && untouched.validated_by_exam);

        let mut practised = LessonProgress { quiz_best: 2, completed: true, ..LessonProgress::default() };
        assert!(!practised.validate_by_exam(&lesson));
        assert_eq!((practised.quiz_best, practised.validated_by_exam), (2, false));
    }

    #[test]
    fn streak_counts_consecutive_days() {
        assert_eq!(longest_streak([]), 0);
        assert_eq!(longest_streak([10, 10, 10]), 1);
        assert_eq!(longest_streak([12, 10, 11, 20, 21, 30]), 3);
    }
}
