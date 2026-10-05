//! Imports the data of a v1 portal (Django) into one tenant.
//!
//! The input is the JSON written by v1's own `manage.py dumpdata` for these models, so v1 needs no change:
//!
//! ```text
//! manage.py dumpdata auth.user training.lesson training.lessonprogress training.xpevent \
//!     training.userbadge training.cohort training.cohortmembership training.examattempt -o v1.json
//! ```
//!
//! The import runs in one transaction, with the platform role, and can be replayed: rows already present are
//! left alone. Dates are kept, which matters for streaks.
//!
//! Two things do not carry over as they are:
//!
//! - v1 does not store the identity provider's subject. Imported learners get the placeholder subject
//!   `v1:<username>`; the web tier must adopt such an account at the learner's first login.
//! - Exam question identifiers differ between v1 and v2 (they are digests of the rendered question). The
//!   caller provides the correspondence; identifiers without one are kept as they are and reported.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{Error, Result};

/// Prefix of the placeholder subject given to imported learners.
pub const SUBJECT_PREFIX: &str = "v1:";

#[derive(Deserialize)]
struct Object {
    model: String,
    pk: Value,
    fields: Value,
}

#[derive(Deserialize)]
struct User {
    username: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    is_staff: bool,
}

#[derive(Deserialize)]
struct Lesson {
    course: String,
    slug: String,
}

#[derive(Deserialize)]
struct Progress {
    user: i64,
    lesson: i64,
    #[serde(default)]
    tasks_done: Vec<i32>,
    #[serde(default)]
    quiz_best: i32,
    completed_at: Option<String>,
    #[serde(default)]
    validated_by_exam: bool,
}

#[derive(Deserialize)]
struct XpEvent {
    user: i64,
    kind: String,
    #[serde(rename = "ref")]
    key: String,
    xp: i32,
    created_at: String,
}

#[derive(Deserialize)]
struct UserBadge {
    user: i64,
    badge: String,
    awarded_at: String,
}

#[derive(Deserialize)]
struct Cohort {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    report_emails: String,
    created_at: String,
}

#[derive(Deserialize)]
struct Membership {
    cohort: String,
    user: i64,
    #[serde(default)]
    source: String,
    joined_at: String,
}

#[derive(Deserialize)]
struct ExamAttempt {
    user: i64,
    course: String,
    started_at: String,
    deadline: String,
    finished_at: Option<String>,
    #[serde(default)]
    question_ids: Vec<String>,
    #[serde(default)]
    option_order: BTreeMap<String, Vec<usize>>,
    #[serde(default)]
    answers: BTreeMap<String, usize>,
    #[serde(default)]
    score: i32,
    #[serde(default)]
    total: i32,
    #[serde(default)]
    passed: bool,
    #[serde(default)]
    expired: bool,
}

/// A parsed v1 export.
#[derive(Default)]
pub struct Dump {
    users: BTreeMap<i64, User>,
    lessons: BTreeMap<i64, Lesson>,
    progress: Vec<Progress>,
    events: Vec<XpEvent>,
    badges: Vec<UserBadge>,
    cohorts: BTreeMap<String, Cohort>,
    memberships: Vec<Membership>,
    attempts: Vec<ExamAttempt>,
}

impl Dump {
    /// Parses the output of `dumpdata`. Models other than the ones listed in the module documentation are
    /// ignored.
    pub fn parse(json: &str) -> Result<Self> {
        let invalid = |what: &str, err: serde_json::Error| Error::InvalidExport(format!("{what}: {err}"));
        let objects: Vec<Object> = serde_json::from_str(json).map_err(|err| invalid("not a dumpdata export", err))?;
        let mut dump = Self::default();
        for object in objects {
            let Object { model, pk, fields } = object;
            let number = || pk.as_i64().ok_or_else(|| Error::InvalidExport(format!("{model}: non-numeric primary key {pk}")));
            match model.as_str() {
                "auth.user" => {
                    dump.users.insert(number()?, serde_json::from_value(fields).map_err(|err| invalid(&model, err))?);
                }
                "training.lesson" => {
                    dump.lessons.insert(number()?, serde_json::from_value(fields).map_err(|err| invalid(&model, err))?);
                }
                "training.lessonprogress" => dump.progress.push(serde_json::from_value(fields).map_err(|err| invalid(&model, err))?),
                "training.xpevent" => dump.events.push(serde_json::from_value(fields).map_err(|err| invalid(&model, err))?),
                "training.userbadge" => dump.badges.push(serde_json::from_value(fields).map_err(|err| invalid(&model, err))?),
                "training.cohort" => {
                    let slug =
                        pk.as_str().ok_or_else(|| Error::InvalidExport(format!("{model}: primary key {pk} is not a slug")))?.to_string();
                    dump.cohorts.insert(slug, serde_json::from_value(fields).map_err(|err| invalid(&model, err))?);
                }
                "training.cohortmembership" => dump.memberships.push(serde_json::from_value(fields).map_err(|err| invalid(&model, err))?),
                "training.examattempt" => dump.attempts.push(serde_json::from_value(fields).map_err(|err| invalid(&model, err))?),
                _ => {}
            }
        }
        Ok(dump)
    }
}

/// What an import wrote. Counts are rows actually inserted: replaying an import reports zeros.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub learners: u64,
    pub progress: u64,
    pub awards: u64,
    pub badges: u64,
    pub cohorts: u64,
    pub memberships: u64,
    pub attempts: u64,
    /// Rows left out, and identifiers that could not be translated, each with the reason.
    pub warnings: Vec<String>,
}

/// v1 named its sources after its identity provider and in French.
fn source(v1: &str) -> &'static str {
    if v1 == "keycloak" {
        "idp"
    } else {
        "manual"
    }
}

/// Imports `dump` into `tenant`. `question_ids` maps v1 exam question identifiers to v2 ones.
pub async fn import(owner: &PgPool, tenant: Uuid, dump: &Dump, question_ids: &BTreeMap<String, String>) -> Result<Report> {
    let mut tx = owner.begin().await?;
    let mut report = Report::default();

    let mut learners: BTreeMap<i64, Uuid> = BTreeMap::new();
    for (pk, user) in &dump.users {
        let subject = format!("{SUBJECT_PREFIX}{}", user.username);
        let display_name = format!("{} {}", user.first_name, user.last_name).trim().to_string();
        report.learners += sqlx::query(
            "INSERT INTO learner (tenant_id, subject, username, display_name, email, is_admin) VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (tenant_id, subject) DO NOTHING",
        )
        .bind(tenant)
        .bind(&subject)
        .bind(&user.username)
        .bind(&display_name)
        .bind(&user.email)
        .bind(user.is_staff)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        let id = sqlx::query_scalar("SELECT id FROM learner WHERE tenant_id = $1 AND subject = $2")
            .bind(tenant)
            .bind(&subject)
            .fetch_one(&mut *tx)
            .await?;
        learners.insert(*pk, id);
    }
    let unknown_user = |what: &str, user: i64, warnings: &mut Vec<String>| warnings.push(format!("{what} skipped: unknown user {user}"));

    for row in &dump.progress {
        let (Some(learner), Some(lesson)) = (learners.get(&row.user), dump.lessons.get(&row.lesson)) else {
            report.warnings.push(format!("lesson progress skipped: unknown user {} or lesson {}", row.user, row.lesson));
            continue;
        };
        report.progress += sqlx::query(
            "INSERT INTO lesson_progress (tenant_id, learner_id, course, lesson, tasks_done, quiz_best, completed_at, validated_by_exam) \
             VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8) ON CONFLICT DO NOTHING",
        )
        .bind(tenant)
        .bind(learner)
        .bind(&lesson.course)
        .bind(&lesson.slug)
        .bind(&row.tasks_done)
        .bind(row.quiz_best)
        .bind(&row.completed_at)
        .bind(row.validated_by_exam)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }

    for event in &dump.events {
        let Some(learner) = learners.get(&event.user) else {
            unknown_user("XP event", event.user, &mut report.warnings);
            continue;
        };
        report.awards += sqlx::query(
            "INSERT INTO award (tenant_id, learner_id, kind, key, xp, created_at) VALUES ($1, $2, $3, $4, $5, $6::timestamptz) ON CONFLICT DO NOTHING",
        )
        .bind(tenant)
        .bind(learner)
        .bind(&event.kind)
        .bind(&event.key)
        .bind(event.xp)
        .bind(&event.created_at)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }

    for badge in &dump.badges {
        let Some(learner) = learners.get(&badge.user) else {
            unknown_user("badge", badge.user, &mut report.warnings);
            continue;
        };
        report.badges +=
            sqlx::query("INSERT INTO learner_badge (tenant_id, learner_id, badge, awarded_at) VALUES ($1, $2, $3, $4::timestamptz) ON CONFLICT DO NOTHING")
                .bind(tenant)
                .bind(learner)
                .bind(&badge.badge)
                .bind(&badge.awarded_at)
                .execute(&mut *tx)
                .await?
                .rows_affected();
    }

    for (slug, cohort) in &dump.cohorts {
        let emails: Vec<String> =
            cohort.report_emails.split([',', '\n']).map(str::trim).filter(|email| !email.is_empty()).map(str::to_string).collect();
        report.cohorts += sqlx::query(
            "INSERT INTO cohort (tenant_id, slug, name, description, source, report_emails, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz) \
             ON CONFLICT DO NOTHING",
        )
        .bind(tenant)
        .bind(slug)
        .bind(&cohort.name)
        .bind(&cohort.description)
        .bind(source(&cohort.source))
        .bind(&emails)
        .bind(&cohort.created_at)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }

    for member in &dump.memberships {
        let Some(learner) = learners.get(&member.user) else {
            unknown_user("cohort membership", member.user, &mut report.warnings);
            continue;
        };
        if !dump.cohorts.contains_key(&member.cohort) {
            report.warnings.push(format!("cohort membership skipped: unknown cohort {}", member.cohort));
            continue;
        }
        report.memberships += sqlx::query(
            "INSERT INTO cohort_member (tenant_id, cohort, learner_id, source, joined_at) VALUES ($1, $2, $3, $4, $5::timestamptz) ON CONFLICT DO NOTHING",
        )
        .bind(tenant)
        .bind(&member.cohort)
        .bind(learner)
        .bind(source(&member.source))
        .bind(&member.joined_at)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }

    let mut untranslated = 0usize;
    for attempt in &dump.attempts {
        let Some(learner) = learners.get(&attempt.user) else {
            unknown_user("exam attempt", attempt.user, &mut report.warnings);
            continue;
        };
        let mut translate = |id: &String| {
            question_ids.get(id).cloned().unwrap_or_else(|| {
                untranslated += 1;
                id.clone()
            })
        };
        let questions: Vec<Value> = attempt
            .question_ids
            .iter()
            .map(|id| serde_json::json!({ "id": translate(id), "option_order": attempt.option_order.get(id).cloned().unwrap_or_default() }))
            .collect();
        let answers: BTreeMap<String, usize> =
            attempt.answers.iter().map(|(id, chosen)| (question_ids.get(id).cloned().unwrap_or_else(|| id.clone()), *chosen)).collect();
        // An attempt has no natural key in v1's export: learner, course and start time identify it.
        report.attempts += sqlx::query(
            "INSERT INTO exam_attempt (tenant_id, learner_id, course, started_at, deadline, finished_at, questions, answers, score, total, passed, expired) \
             SELECT $1, $2, $3, $4::timestamptz, $5::timestamptz, $6::timestamptz, $7, $8, $9, $10, $11, $12 \
             WHERE NOT EXISTS (SELECT FROM exam_attempt WHERE tenant_id = $1 AND learner_id = $2 AND course = $3 AND started_at = $4::timestamptz)",
        )
        .bind(tenant)
        .bind(learner)
        .bind(&attempt.course)
        .bind(&attempt.started_at)
        .bind(&attempt.deadline)
        .bind(&attempt.finished_at)
        .bind(Json(&questions))
        .bind(Json(&answers))
        .bind(attempt.score)
        .bind(attempt.total)
        .bind(attempt.passed)
        .bind(attempt.expired)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }
    if untranslated > 0 {
        report.warnings.push(format!("{untranslated} exam question identifier(s) had no v2 equivalent and were kept as they are"));
    }

    tx.commit().await?;
    Ok(report)
}
