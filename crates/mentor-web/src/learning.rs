//! What the catalogue means for progress: the rules of each lesson, and the view badges are computed from.

use std::collections::BTreeMap;

use mentor_content::{Catalogue, Course, Exam, Lesson};
use mentor_core::exam::{PoolQuestion, Randomness};
use mentor_core::progress::{CourseShape, LessonRules};
use mentor_db::badges::CatalogueView;
use mentor_db::exam::ExamSettings;

/// Time zone that cuts activity into days and dates badges, until tenants have their own setting.
pub const UTC_OFFSET_MINUTES: i32 = 60;

/// This server is not connected to an execution plane yet. A lab that cannot be done does not gate its quiz.
const LABS_AVAILABLE: bool = false;

/// Delay before a new exam attempt after a failed or expired one.
pub const EXAM_COOLDOWN_SECONDS: i64 = 600;

/// How the exam of a course is drawn, timed and graded.
pub fn exam_settings(exam: &Exam) -> ExamSettings {
    ExamSettings {
        draw: exam.draw as usize,
        shuffle: exam.shuffle,
        minutes: i64::from(exam.minutes),
        pass_mark: exam.pass_mark,
        cooldown_seconds: EXAM_COOLDOWN_SECONDS,
    }
}

/// The question pool of an exam, as the rules see it.
pub fn exam_pool(exam: &Exam) -> Vec<PoolQuestion> {
    exam.questions
        .iter()
        .map(|question| PoolQuestion {
            id: question.id.clone(),
            options: question.options.len(),
            // The compiler guarantees exactly one correct option.
            correct: question.options.iter().position(|option| option.correct).unwrap_or(0),
        })
        .collect()
}

/// Randomness of the operating system: a learner cannot predict a draw.
pub struct OsRandom;

impl Randomness for OsRandom {
    fn below(&mut self, upper: usize) -> usize {
        let upper = upper as u64;
        // Rejection sampling: values above the last full multiple of `upper` would favour small results.
        let limit = u64::MAX - u64::MAX % upper;
        loop {
            let mut bytes = [0u8; 8];
            getrandom::fill(&mut bytes).expect("the operating system provides randomness");
            let value = u64::from_le_bytes(bytes);
            if value < limit {
                return (value % upper) as usize;
            }
        }
    }
}

/// Progress rules of a lesson.
pub fn rules(course: &Course, lesson: &Lesson) -> LessonRules {
    LessonRules {
        course: course.slug.clone(),
        slug: lesson.slug.clone(),
        tasks: lesson.lab.as_ref().map_or(0, |lab| lab.steps.len() as u32),
        questions: lesson.quiz.len() as u32,
        lab_available: LABS_AVAILABLE,
    }
}

/// The catalogue as the progress rules see it.
pub fn view(catalogue: &Catalogue) -> CatalogueView {
    let mut questions = BTreeMap::new();
    let mut courses = Vec::with_capacity(catalogue.courses.len());
    for course in &catalogue.courses {
        for lesson in &course.lessons {
            questions.insert(format!("{}/{}", course.slug, lesson.slug), lesson.quiz.len() as u32);
        }
        courses.push(CourseShape {
            slug: course.slug.clone(),
            published: course.published,
            requires: course.requires.clone(),
            lessons: course.lessons.iter().map(|lesson| lesson.slug.clone()).collect(),
        });
    }
    CatalogueView { courses, questions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_randomness_stays_in_range_and_reaches_every_value() {
        let mut seen = [false; 5];
        for _ in 0..400 {
            seen[OsRandom.below(5)] = true;
        }
        assert!(seen.iter().all(|&hit| hit));
        assert_eq!(OsRandom.below(1), 0);
    }
}
