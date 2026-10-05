//! What the catalogue means for progress: the rules of each lesson, and the view badges are computed from.

use std::collections::BTreeMap;

use mentor_content::{Catalogue, Course, Lesson};
use mentor_core::progress::{CourseShape, LessonRules};
use mentor_db::badges::CatalogueView;

/// Engine name of labs that run in a real environment.
const REAL_ENGINE: &str = "real";

/// Time zone that cuts activity into days and dates badges, until tenants have their own setting.
pub const UTC_OFFSET_MINUTES: i32 = 60;

/// This server does not run labs yet, simulated or real. A lab that cannot be done does not gate its quiz.
const LABS_AVAILABLE: bool = false;

/// Progress rules of a lesson.
pub fn rules(course: &Course, lesson: &Lesson) -> LessonRules {
    LessonRules {
        course: course.slug.clone(),
        slug: lesson.slug.clone(),
        tasks: lesson.lab.as_ref().map_or(0, |lab| lab.steps.len() as u32),
        questions: lesson.quiz.len() as u32,
        server_verified: lesson.lab.as_ref().is_some_and(|lab| lab.engine == REAL_ENGINE),
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
