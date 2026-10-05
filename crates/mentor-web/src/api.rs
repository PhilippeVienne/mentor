//! JSON API used by the lesson page: recording what a learner did.

use axum::extract::State;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mentor_core::badges::{default_badges, BadgeDef, COURSE_BADGE_PREFIX};
use mentor_core::gamification::{level_info, DEFAULT_LEVEL_TITLES};
use mentor_core::progress::{completed_courses, Award, Event, RecordError, Source};
use mentor_db::{Error, TenantTx};
use serde::Deserialize;
use serde_json::json;

use crate::learning::{rules, UTC_OFFSET_MINUTES};
use crate::site::{same_origin, Site};
use crate::AppState;

#[derive(Deserialize)]
pub struct ProgressRequest {
    course: String,
    lesson: String,
    #[serde(rename = "type")]
    kind: String,
    task: Option<u32>,
    score: Option<u32>,
}

fn refuse(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

/// What to show for a badge: one of the platform's, or the badge of a completed course.
fn describe_badge(slug: &str, definitions: &[BadgeDef], state: &AppState) -> serde_json::Value {
    if let Some(badge) = definitions.iter().find(|badge| badge.slug == slug) {
        return json!({ "slug": slug, "name": badge.name, "emoji": badge.emoji, "tier": badge.tier, "description": badge.description });
    }
    let course = slug.strip_prefix(COURSE_BADGE_PREFIX).and_then(|course| state.catalogue.courses.iter().find(|c| c.slug == course));
    match course {
        Some(course) => {
            json!({ "slug": slug, "name": course.title, "emoji": course.icon, "tier": "gold", "description": format!("Termine le parcours « {} ».", course.title) })
        }
        None => json!({ "slug": slug, "name": slug, "emoji": "🏅", "tier": "bronze", "description": "" }),
    }
}

/// `POST /api/progress`: a validated lab step or a quiz score. Answers with what it changed: XP, level,
/// badges, completion.
pub async fn progress(site: Site, State(state): State<AppState>, parts: Parts, Json(request): Json<ProgressRequest>) -> Response {
    if !same_origin(&parts) {
        return refuse(StatusCode::FORBIDDEN, "Requête refusée : origine inattendue.");
    }
    let Some(viewer) = &site.viewer else { return refuse(StatusCode::UNAUTHORIZED, "Connecte-toi pour enregistrer ta progression.") };
    let learner = viewer.learner.id;
    let found = state.catalogue.courses.iter().find(|course| course.slug == request.course && course.published);
    let Some(course) = found else { return refuse(StatusCode::NOT_FOUND, "Leçon inconnue.") };
    let Some(lesson) = course.lessons.iter().find(|lesson| lesson.slug == request.lesson) else {
        return refuse(StatusCode::NOT_FOUND, "Leçon inconnue.");
    };
    let event = match (request.kind.as_str(), request.task, request.score) {
        ("task", Some(task), _) => Event::Task(task),
        ("quiz", _, Some(score)) => Event::Quiz(score),
        _ => return refuse(StatusCode::BAD_REQUEST, "Type d'évènement inconnu."),
    };

    let outcome: Result<Response, Error> = async {
        let mut tx = TenantTx::begin(&state.db, site.tenant).await?;
        let lessons_before = tx.completed_lessons(learner).await?;
        let courses_before = completed_courses(&state.view.courses, &lessons_before);
        let shape = state.view.courses.iter().find(|shape| shape.slug == course.slug);
        if !shape.is_some_and(|shape| shape.is_unlocked(&courses_before)) {
            return Ok(refuse(StatusCode::FORBIDDEN, "Ce parcours n'est pas encore débloqué."));
        }
        let level_before = viewer.level.level;

        // The browser reports this event: steps of a real lab are refused here, by the rules.
        let recorded = match tx.record_event(learner, &rules(course, lesson), event, Source::Browser).await {
            Ok(recorded) => recorded,
            Err(Error::Refused(reason)) => {
                let (status, message) = match reason {
                    RecordError::CourseLocked => (StatusCode::FORBIDDEN, "Ce parcours n'est pas encore débloqué."),
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
            .award_badges(learner, &definitions, &state.view, UTC_OFFSET_MINUTES)
            .await?
            .iter()
            .map(|slug| describe_badge(slug, &definitions, &state))
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
                "tasks_done": recorded.progress.tasks_done,
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
