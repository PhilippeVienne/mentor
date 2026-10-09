//! JSON API used by the lesson and exam pages: recording what a learner did.

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mentor_content::{Course, Exam};
use mentor_core::badges::{default_badges, BadgeDef, COURSE_BADGE_PREFIX};
use mentor_core::gamification::{level_info, DEFAULT_LEVEL_TITLES};
use mentor_core::progress::{completed_courses, Award, Event, RecordError, Source};
use mentor_db::exam::{Started, Submitted};
use mentor_db::{Error, TenantTx};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::learning::{done_positions, exam_pool, exam_settings, rules, OsRandom, EXAM_COOLDOWN_SECONDS, UTC_OFFSET_MINUTES};
use crate::site::{now, same_origin, Site};
use crate::{AppState, Content};

#[derive(Deserialize)]
pub struct ProgressRequest {
    course: String,
    lesson: String,
    #[serde(rename = "type")]
    kind: String,
    /// Identifier of a lab step. The browser can name one but never validate it: only the server does.
    task: Option<String>,
    score: Option<u32>,
}

fn refuse(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

/// What to show for a badge: one of the platform's, or the badge of a completed course.
fn describe_badge(slug: &str, definitions: &[BadgeDef], content: &Content) -> serde_json::Value {
    if let Some(badge) = definitions.iter().find(|badge| badge.slug == slug) {
        return json!({ "slug": slug, "name": badge.name, "emoji": badge.emoji, "tier": badge.tier, "description": badge.description });
    }
    let course = slug.strip_prefix(COURSE_BADGE_PREFIX).and_then(|course| content.catalogue.courses.iter().find(|c| c.slug == course));
    match course {
        Some(course) => {
            json!({ "slug": slug, "name": course.title, "emoji": course.icon, "tier": "gold", "description": format!("Termine le cours « {} ».", course.title) })
        }
        None => json!({ "slug": slug, "name": slug, "emoji": "🏅", "tier": "bronze", "description": "" }),
    }
}

/// `POST /api/progress`: a validated lab step or a quiz score. Answers with what it changed: XP, level,
/// badges, completion.
pub async fn progress(site: Site, State(state): State<AppState>, parts: Parts, Json(request): Json<ProgressRequest>) -> Response {
    let content = &*site.content;
    if !same_origin(&parts) {
        return refuse(StatusCode::FORBIDDEN, "Requête refusée : origine inattendue.");
    }
    let Some(viewer) = &site.viewer else { return refuse(StatusCode::UNAUTHORIZED, "Connecte-toi pour enregistrer ta progression.") };
    let learner = viewer.learner.id;
    let found = content.catalogue.courses.iter().find(|course| course.slug == request.course && course.published);
    let Some(course) = found else { return refuse(StatusCode::NOT_FOUND, "Leçon inconnue.") };
    let Some(lesson) = course.lessons.iter().find(|lesson| lesson.slug == request.lesson) else {
        return refuse(StatusCode::NOT_FOUND, "Leçon inconnue.");
    };
    let lesson_rules = rules(course, lesson);
    let event = match (request.kind.as_str(), request.task, request.score) {
        ("task", Some(task), _) => Event::Task(task),
        ("quiz", _, Some(score)) => Event::Quiz(score),
        _ => return refuse(StatusCode::BAD_REQUEST, "Type d'évènement inconnu."),
    };

    let outcome: Result<Response, Error> = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let lessons_before = tx.completed_lessons(learner).await?;
        let courses_before = completed_courses(&content.view.courses, &lessons_before);
        let shape = content.view.courses.iter().find(|shape| shape.slug == course.slug);
        if !shape.is_some_and(|shape| shape.is_unlocked(&courses_before)) {
            return Ok(refuse(StatusCode::FORBIDDEN, "Ce cours n'est pas encore débloqué."));
        }
        let level_before = viewer.level.level;

        // The browser reports this event: steps of a real lab are refused here, by the rules.
        let recorded = match tx.record_event(learner, &lesson_rules, event, Source::Browser).await {
            Ok(recorded) => recorded,
            Err(Error::Refused(reason)) => {
                let (status, message) = match reason {
                    RecordError::CourseLocked => (StatusCode::FORBIDDEN, "Ce cours n'est pas encore débloqué."),
                    RecordError::ServerVerifiedLab => {
                        (StatusCode::FORBIDDEN, "Les étapes de ce labo sont vérifiées par le serveur dans ton environnement réel.")
                    }
                    RecordError::InvalidTask => (StatusCode::BAD_REQUEST, "Étape invalide."),
                    RecordError::LabNotDone => (StatusCode::FORBIDDEN, "Termine d'abord le labo."),
                    RecordError::InvalidScore => (StatusCode::BAD_REQUEST, "Score invalide."),
                };
                return Ok(refuse(status, message));
            }
            Err(other) => return Err(other),
        };

        let mut xp_gained = recorded.xp_gained;
        let mut course_completed = false;
        if recorded.lesson_completed {
            let lessons_now = tx.completed_lessons(learner).await?;
            if shape.is_some_and(|shape| shape.is_completed(&lessons_now)) {
                course_completed = true;
                xp_gained += tx.grant(learner, &Award::course(&course.slug)).await?;
            }
        }
        let definitions = default_badges();
        let new_badges: Vec<serde_json::Value> = tx
            .award_badges(learner, &definitions, &content.view, UTC_OFFSET_MINUTES)
            .await?
            .iter()
            .map(|slug| describe_badge(slug, &definitions, content))
            .collect();
        let level = level_info(tx.total_xp(learner).await?, &DEFAULT_LEVEL_TITLES);
        tx.commit().await?;

        Ok(Json(json!({
            "xp_gained": xp_gained,
            "level_up": level.level > level_before,
            "level": level,
            "lesson_completed": recorded.lesson_completed,
            "course_completed": course_completed,
            "new_badges": new_badges,
            "progress": {
                "tasks_done": done_positions(&lesson_rules, &recorded.progress.tasks_done),
                "quiz_best": recorded.progress.quiz_best,
                "completed": recorded.progress.completed,
            },
        }))
        .into_response())
    }
    .await;
    outcome.unwrap_or_else(|err| {
        eprintln!("mentor-web: progress not recorded: {err}");
        refuse(StatusCode::SERVICE_UNAVAILABLE, "Progression non enregistrée : réessaie dans un instant.")
    })
}

/// A published course and its exam.
fn course_with_exam<'a>(content: &'a Content, slug: &str) -> Option<(&'a Course, &'a Exam)> {
    let course = content.catalogue.courses.iter().find(|course| course.slug == slug && course.published)?;
    Some((course, course.exam.as_ref()?))
}

fn unavailable(err: Error) -> Response {
    eprintln!("mentor-web: exam request failed: {err}");
    refuse(StatusCode::SERVICE_UNAVAILABLE, "Service momentanément indisponible : réessaie dans un instant.")
}

/// `POST /api/exam/{course}/start`: starts an attempt or resumes the open one. The answer carries the drawn
/// questions in the order they are shown, without correct answers or explanations.
pub async fn exam_start(site: Site, State(state): State<AppState>, parts: Parts, Path(slug): Path<String>) -> Response {
    let content = &*site.content;
    if !same_origin(&parts) {
        return refuse(StatusCode::FORBIDDEN, "Requête refusée : origine inattendue.");
    }
    let Some(viewer) = &site.viewer else { return refuse(StatusCode::UNAUTHORIZED, "Connecte-toi pour passer l'examen.") };
    let learner = viewer.learner.id;
    let Some((course, exam)) = course_with_exam(content, &slug) else {
        return refuse(StatusCode::NOT_FOUND, "Ce cours n'a pas d'examen de validation.");
    };
    let at = now();
    let outcome: Result<Response, Error> = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let courses_done = completed_courses(&content.view.courses, &tx.completed_lessons(learner).await?);
        let shape = content.view.courses.iter().find(|shape| shape.slug == course.slug);
        if !shape.is_some_and(|shape| shape.is_unlocked(&courses_done)) {
            return Ok(refuse(StatusCode::FORBIDDEN, "Ce cours n'est pas encore débloqué."));
        }
        if courses_done.contains(&course.slug) {
            return Ok(refuse(StatusCode::CONFLICT, "Ce cours est déjà validé."));
        }
        let started = tx.start_exam(learner, &course.slug, &exam_pool(exam), exam_settings(exam), at, &mut OsRandom).await?;
        // Committed in every case: closing an attempt whose time ran out must stay recorded even when the
        // new attempt is refused.
        tx.commit().await?;
        let attempt = match started {
            Started::Attempt(attempt) => attempt,
            Started::RetryAfter(seconds) => {
                let body = json!({ "error": "Tu pourras retenter l'examen dans quelques instants.", "retry_after": seconds });
                return Ok((StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response());
            }
        };
        let questions: Vec<serde_json::Value> = attempt
            .questions
            .iter()
            .filter_map(|drawn| {
                let source = exam.questions.iter().find(|question| question.id == drawn.id)?;
                let options: Vec<serde_json::Value> =
                    drawn.option_order.iter().filter_map(|&original| Some(json!({ "html": source.options.get(original)?.html }))).collect();
                Some(json!({ "id": drawn.id, "question": source.question, "options": options }))
            })
            .collect();
        Ok(Json(json!({
            "attempt": attempt.id.to_string(),
            "title": exam.title,
            "seconds_left": (attempt.deadline - at).max(0),
            "pass_mark": exam.pass_mark,
            "total": questions.len(),
            "questions": questions,
        }))
        .into_response())
    }
    .await;
    outcome.unwrap_or_else(unavailable)
}

#[derive(Deserialize)]
pub struct ExamSubmission {
    attempt: String,
    /// Question id → shown position chosen. Anything that is not a position counts as no answer.
    #[serde(default)]
    answers: BTreeMap<String, serde_json::Value>,
}

/// `POST /api/exam/{course}/submit`: grades an attempt on the server and, when it is passed, validates the
/// course. The correction is only given here, once the attempt is closed.
pub async fn exam_submit(
    site: Site,
    State(state): State<AppState>,
    parts: Parts,
    Path(slug): Path<String>,
    Json(submission): Json<ExamSubmission>,
) -> Response {
    let content = &*site.content;
    if !same_origin(&parts) {
        return refuse(StatusCode::FORBIDDEN, "Requête refusée : origine inattendue.");
    }
    let Some(viewer) = &site.viewer else { return refuse(StatusCode::UNAUTHORIZED, "Connecte-toi pour passer l'examen.") };
    let learner = viewer.learner.id;
    let Some((course, exam)) = course_with_exam(content, &slug) else {
        return refuse(StatusCode::NOT_FOUND, "Ce cours n'a pas d'examen de validation.");
    };
    let Ok(attempt) = Uuid::parse_str(&submission.attempt) else { return refuse(StatusCode::NOT_FOUND, "Tentative inconnue.") };
    let answers: BTreeMap<String, usize> =
        submission.answers.iter().filter_map(|(id, value)| Some((id.clone(), value.as_u64()? as usize))).collect();
    let lessons: Vec<_> = course.lessons.iter().map(|lesson| rules(course, lesson)).collect();

    let outcome: Result<Response, Error> = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let submitted =
            tx.submit_exam(learner, &course.slug, attempt, &answers, &exam_pool(exam), exam_settings(exam), &lessons, now()).await?;
        let (grade, shown, xp_gained, lessons_validated) = match submitted {
            Submitted::Unknown => return Ok(refuse(StatusCode::NOT_FOUND, "Tentative inconnue.")),
            Submitted::AlreadyFinished => return Ok(refuse(StatusCode::CONFLICT, "Cette tentative est déjà terminée.")),
            Submitted::Expired { retry_after } => {
                tx.commit().await?;
                let body = json!({
                    "expired": true, "passed": false, "score": 0, "total": exam.draw.min(exam.questions.len() as u32),
                    "retry_after": retry_after, "error": "Temps écoulé : la tentative est expirée.",
                });
                return Ok((StatusCode::GONE, Json(body)).into_response());
            }
            Submitted::Graded { grade, questions, xp_gained, lessons_validated } => (grade, questions, xp_gained, lessons_validated),
        };
        let results: Vec<serde_json::Value> = grade
            .results
            .iter()
            .filter_map(|result| {
                let source = exam.questions.iter().find(|question| question.id == result.id)?;
                let order = &shown.iter().find(|drawn| drawn.id == result.id)?.option_order;
                let options: Vec<&str> = order.iter().filter_map(|&original| Some(source.options.get(original)?.html.as_str())).collect();
                Some((result, source, options))
            })
            .map(|(result, source, options)| {
                json!({
                    "id": result.id,
                    "question": source.question,
                    "options": options,
                    "chosen": result.chosen,
                    "correct_position": result.correct_position,
                    "correct": result.correct,
                    "explanation": source.explanation,
                })
            })
            .collect();
        let mut body = json!({
            "expired": false,
            "passed": grade.passed,
            "score": grade.score,
            "total": grade.total,
            "pass_mark": exam.pass_mark,
            "results": results,
        });
        if grade.passed {
            let definitions = default_badges();
            let new_badges: Vec<serde_json::Value> = tx
                .award_badges(learner, &definitions, &content.view, UTC_OFFSET_MINUTES)
                .await?
                .iter()
                .map(|slug| describe_badge(slug, &definitions, content))
                .collect();
            let level = level_info(tx.total_xp(learner).await?, &DEFAULT_LEVEL_TITLES);
            body["course_validated"] = json!(true);
            body["lessons_validated"] = json!(lessons_validated);
            body["xp_gained"] = json!(xp_gained);
            body["level_up"] = json!(level.level > viewer.level.level);
            body["level"] = json!(level);
            body["new_badges"] = json!(new_badges);
        } else {
            body["retry_after"] = json!(EXAM_COOLDOWN_SECONDS);
        }
        tx.commit().await?;
        Ok(Json(body).into_response())
    }
    .await;
    outcome.unwrap_or_else(unavailable)
}
