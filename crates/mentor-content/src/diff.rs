//! What changes for learners between two versions of a package.
//!
//! Stored progress refers to courses and lessons by name, to lab steps and exam questions by identifier. A new
//! version can therefore add freely, but what it removes leaves progress that refers to nothing, and what it
//! adds to a course makes that course incomplete again for those who had finished it. This module says what
//! was added and removed; how many learners are concerned is counted where their progress is stored.

use std::collections::BTreeSet;

use crate::catalogue::Course;
use crate::paths::LearningPath;

/// A lesson present in both versions whose lab or quiz differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonDiff {
    pub slug: String,
    /// Lab steps that are new: learners who had not completed the lesson have them to do.
    pub steps_added: usize,
    /// Lab steps that are gone; a reworded step without an `id` counts as one removed and one added.
    pub steps_removed: usize,
    pub questions_before: usize,
    pub questions_after: usize,
}

/// A course present in both versions that differs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CourseDiff {
    pub slug: String,
    /// Lessons of the previous version, to know who had completed the course.
    pub lessons_before: Vec<String>,
    pub lessons_added: Vec<String>,
    pub lessons_removed: Vec<String>,
    pub lessons_changed: Vec<LessonDiff>,
    /// Questions of the exam pool that are new, and that are gone.
    pub exam_questions_added: usize,
    pub exam_questions_removed: usize,
    /// The course was visible and no longer is, or the reverse.
    pub published_before: bool,
    pub published_after: bool,
}

impl CourseDiff {
    fn is_empty(&self) -> bool {
        self.lessons_added.is_empty()
            && self.lessons_removed.is_empty()
            && self.lessons_changed.is_empty()
            && self.exam_questions_added == 0
            && self.exam_questions_removed == 0
            && self.published_before == self.published_after
    }

    pub fn exam_changed(&self) -> bool {
        self.exam_questions_added > 0 || self.exam_questions_removed > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageDiff {
    pub courses_added: Vec<String>,
    pub courses_removed: Vec<String>,
    /// Courses kept that changed in a way that matters to learners. Rewording a text is not listed.
    pub courses_changed: Vec<CourseDiff>,
    pub paths_added: Vec<String>,
    pub paths_removed: Vec<String>,
}

impl PackageDiff {
    pub fn is_empty(&self) -> bool {
        self.courses_added.is_empty()
            && self.courses_removed.is_empty()
            && self.courses_changed.is_empty()
            && self.paths_added.is_empty()
            && self.paths_removed.is_empty()
    }

    /// Whether the new version takes away a course or a lesson: progress on them would refer to nothing, so
    /// this needs an explicit confirmation.
    pub fn removes_progress(&self) -> bool {
        !self.courses_removed.is_empty() || self.courses_changed.iter().any(|course| !course.lessons_removed.is_empty())
    }
}

fn added_and_removed<'a>(before: impl Iterator<Item = &'a str>, after: impl Iterator<Item = &'a str>) -> (Vec<String>, Vec<String>) {
    let (before, after): (Vec<&str>, Vec<&str>) = (before.collect(), after.collect());
    let (old, new): (BTreeSet<&str>, BTreeSet<&str>) = (before.iter().copied().collect(), after.iter().copied().collect());
    let added = after.iter().filter(|name| !old.contains(*name)).map(|name| name.to_string()).collect();
    let removed = before.iter().filter(|name| !new.contains(*name)).map(|name| name.to_string()).collect();
    (added, removed)
}

fn course_diff(before: &Course, after: &Course) -> CourseDiff {
    let (lessons_added, lessons_removed) = added_and_removed(
        before.lessons.iter().map(|lesson| lesson.slug.as_str()),
        after.lessons.iter().map(|lesson| lesson.slug.as_str()),
    );
    let mut lessons_changed = Vec::new();
    for old in &before.lessons {
        let Some(new) = after.lessons.iter().find(|lesson| lesson.slug == old.slug) else { continue };
        let steps = |lesson: &crate::catalogue::Lesson| -> Vec<String> {
            lesson.lab.as_ref().map_or_else(Vec::new, |lab| lab.steps.iter().map(|step| step.id.clone()).collect())
        };
        let (old_steps, new_steps) = (steps(old), steps(new));
        let (steps_added, steps_removed) = added_and_removed(old_steps.iter().map(String::as_str), new_steps.iter().map(String::as_str));
        if !steps_added.is_empty() || !steps_removed.is_empty() || old.quiz.len() != new.quiz.len() {
            lessons_changed.push(LessonDiff {
                slug: old.slug.clone(),
                steps_added: steps_added.len(),
                steps_removed: steps_removed.len(),
                questions_before: old.quiz.len(),
                questions_after: new.quiz.len(),
            });
        }
    }
    let pool = |course: &Course| -> Vec<String> {
        course.exam.as_ref().map_or_else(Vec::new, |exam| exam.questions.iter().map(|question| question.id.clone()).collect())
    };
    let (old_pool, new_pool) = (pool(before), pool(after));
    let (exam_added, exam_removed) = added_and_removed(old_pool.iter().map(String::as_str), new_pool.iter().map(String::as_str));
    CourseDiff {
        slug: before.slug.clone(),
        lessons_before: before.lessons.iter().map(|lesson| lesson.slug.clone()).collect(),
        lessons_added,
        lessons_removed,
        lessons_changed,
        exam_questions_added: exam_added.len(),
        exam_questions_removed: exam_removed.len(),
        published_before: before.published,
        published_after: after.published,
    }
}

/// Compares the installed version of a package (`before`) with the one about to replace it.
pub fn package_diff(before: (&[Course], &[LearningPath]), after: (&[Course], &[LearningPath])) -> PackageDiff {
    let (courses_added, courses_removed) =
        added_and_removed(before.0.iter().map(|course| course.slug.as_str()), after.0.iter().map(|course| course.slug.as_str()));
    let (paths_added, paths_removed) =
        added_and_removed(before.1.iter().map(|path| path.id.as_str()), after.1.iter().map(|path| path.id.as_str()));
    let courses_changed = before
        .0
        .iter()
        .filter_map(|old| after.0.iter().find(|new| new.slug == old.slug).map(|new| course_diff(old, new)))
        .filter(|diff| !diff.is_empty())
        .collect();
    PackageDiff { courses_added, courses_removed, courses_changed, paths_added, paths_removed }
}
