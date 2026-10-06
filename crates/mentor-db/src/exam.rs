//! Exam attempts: drawing, deadline, grading, and validation of the course when the exam is passed.
//!
//! The rules are in `mentor_core::exam`; this module stores attempts and applies the outcome. Whether the
//! learner may take the exam at all (course published, unlocked, not already validated) is decided by the
//! caller, who knows the catalogue. Time is in seconds since the Unix epoch.

use std::collections::BTreeMap;

use mentor_core::exam::{draw, grade, is_expired, retry_after, DrawnQuestion, Grade, PoolQuestion, Randomness};
use mentor_core::progress::{Award, LessonRules};
use sqlx::types::Json;
use sqlx::Row;
use uuid::Uuid;

use crate::{Result, TenantTx};

/// Exam parameters of a course, from its catalogue entry and the platform configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExamSettings {
    /// Number of questions drawn from the pool.
    pub draw: usize,
    pub shuffle: bool,
    pub minutes: i64,
    /// Required percentage of correct answers.
    pub pass_mark: u32,
    /// Delay before a new attempt after a failed or expired one.
    pub cooldown_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub id: Uuid,
    pub deadline: i64,
    pub questions: Vec<DrawnQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Started {
    /// A new attempt, or the one already open: starting twice never draws twice.
    Attempt(Attempt),
    /// The last attempt failed too recently; seconds to wait.
    RetryAfter(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Submitted {
    /// No such attempt for this learner and course.
    Unknown,
    AlreadyFinished,
    /// Submitted after the deadline: the attempt is closed as failed.
    Expired {
        retry_after: i64,
    },
    Graded {
        grade: Grade,
        /// The questions as they were shown, to present the correction in the same order.
        questions: Vec<DrawnQuestion>,
        /// XP credited for passing; 0 when failed, or when the course was already completed by practising.
        xp_gained: u32,
        /// Lessons this exam marked as completed.
        lessons_validated: u32,
    },
}

/// Where a learner stands with the exam of a course, before starting anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExamStatus {
    /// An attempt is running and its time is not over.
    pub open: bool,
    /// Seconds to wait before a new attempt; 0 when one is open or allowed.
    pub retry_after: i64,
}

impl TenantTx {
    /// Reads the state of the exam without changing anything. An attempt whose time ran out is only closed
    /// by the next start, from which the delay will run: it counts here as failing now.
    pub async fn exam_status(&mut self, learner: Uuid, course: &str, cooldown_seconds: i64, now: i64) -> Result<ExamStatus> {
        let row = sqlx::query(
            "SELECT coalesce(bool_or(finished_at IS NULL AND deadline > to_timestamp($3)), false) AS open, \
                    extract(epoch FROM max(CASE WHEN finished_at IS NULL THEN to_timestamp($3) ELSE finished_at END) \
                                       FILTER (WHERE NOT passed AND (finished_at IS NOT NULL OR deadline <= to_timestamp($3))))::bigint AS failed_at \
             FROM exam_attempt WHERE learner_id = $1 AND course = $2",
        )
        .bind(learner)
        .bind(course)
        .bind(now as f64)
        .fetch_one(&mut *self.tx)
        .await?;
        let open: bool = row.get("open");
        let wait = if open { 0 } else { retry_after(row.get::<Option<i64>, _>("failed_at"), cooldown_seconds, now) };
        Ok(ExamStatus { open, retry_after: wait })
    }

    /// Starts an attempt, or returns the open one. Attempts whose time ran out are closed first.
    pub async fn start_exam(
        &mut self,
        learner: Uuid,
        course: &str,
        pool: &[PoolQuestion],
        settings: ExamSettings,
        now: i64,
        rng: &mut impl Randomness,
    ) -> Result<Started> {
        sqlx::query(
            "UPDATE exam_attempt SET finished_at = to_timestamp($3), expired = true, passed = false, score = 0, total = jsonb_array_length(questions) \
             WHERE learner_id = $1 AND course = $2 AND finished_at IS NULL AND deadline < to_timestamp($3)",
        )
        .bind(learner)
        .bind(course)
        .bind(now as f64)
        .execute(&mut *self.tx)
        .await?;

        let open = sqlx::query("SELECT id, extract(epoch FROM deadline)::bigint AS deadline, questions FROM exam_attempt WHERE learner_id = $1 AND course = $2 AND finished_at IS NULL")
            .bind(learner)
            .bind(course)
            .fetch_optional(&mut *self.tx)
            .await?;
        if let Some(row) = open {
            let Json(questions) = row.get::<Json<Vec<DrawnQuestion>>, _>("questions");
            return Ok(Started::Attempt(Attempt { id: row.get("id"), deadline: row.get("deadline"), questions }));
        }

        let last_failure: Option<i64> = sqlx::query_scalar(
            "SELECT extract(epoch FROM max(finished_at))::bigint FROM exam_attempt WHERE learner_id = $1 AND course = $2 AND finished_at IS NOT NULL AND NOT passed",
        )
        .bind(learner)
        .bind(course)
        .fetch_one(&mut *self.tx)
        .await?;
        let wait = retry_after(last_failure, settings.cooldown_seconds, now);
        if wait > 0 {
            return Ok(Started::RetryAfter(wait));
        }

        let questions = draw(pool, settings.draw, settings.shuffle, rng);
        let deadline = now + settings.minutes * 60;
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO exam_attempt (tenant_id, learner_id, course, started_at, deadline, questions, total) \
             VALUES ($1, $2, $3, to_timestamp($4), to_timestamp($5), $6, $7) RETURNING id",
        )
        .bind(self.tenant)
        .bind(learner)
        .bind(course)
        .bind(now as f64)
        .bind(deadline as f64)
        .bind(Json(&questions))
        .bind(questions.len() as i32)
        .fetch_one(&mut *self.tx)
        .await?;
        Ok(Started::Attempt(Attempt { id, deadline, questions }))
    }

    /// Grades an attempt. When it is passed, every lesson of the course not yet completed is marked as
    /// validated by the exam, and the exam XP is credited unless the course was already completed.
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_exam(
        &mut self,
        learner: Uuid,
        course: &str,
        attempt: Uuid,
        answers: &BTreeMap<String, usize>,
        pool: &[PoolQuestion],
        settings: ExamSettings,
        lessons: &[LessonRules],
        now: i64,
    ) -> Result<Submitted> {
        let row = sqlx::query(
            "SELECT finished_at IS NOT NULL AS finished, extract(epoch FROM deadline)::bigint AS deadline, questions \
             FROM exam_attempt WHERE id = $1 AND learner_id = $2 AND course = $3 FOR UPDATE",
        )
        .bind(attempt)
        .bind(learner)
        .bind(course)
        .fetch_optional(&mut *self.tx)
        .await?;
        let Some(row) = row else { return Ok(Submitted::Unknown) };
        if row.get("finished") {
            return Ok(Submitted::AlreadyFinished);
        }
        let Json(questions) = row.get::<Json<Vec<DrawnQuestion>>, _>("questions");
        if is_expired(row.get("deadline"), now) {
            sqlx::query("UPDATE exam_attempt SET finished_at = to_timestamp($2), expired = true, passed = false, score = 0 WHERE id = $1")
                .bind(attempt)
                .bind(now as f64)
                .execute(&mut *self.tx)
                .await?;
            return Ok(Submitted::Expired { retry_after: settings.cooldown_seconds });
        }

        let result = grade(pool, &questions, answers, settings.pass_mark);
        let kept: BTreeMap<&str, usize> = result.results.iter().filter_map(|r| r.chosen.map(|chosen| (r.id.as_str(), chosen))).collect();
        sqlx::query(
            "UPDATE exam_attempt SET finished_at = to_timestamp($2), answers = $3, score = $4, total = $5, passed = $6 WHERE id = $1",
        )
        .bind(attempt)
        .bind(now as f64)
        .bind(Json(&kept))
        .bind(result.score as i32)
        .bind(result.total as i32)
        .bind(result.passed)
        .execute(&mut *self.tx)
        .await?;
        if !result.passed {
            return Ok(Submitted::Graded { grade: result, questions, xp_gained: 0, lessons_validated: 0 });
        }

        let mut lessons_validated = 0;
        for lesson in lessons {
            let mut progress = self.lesson_progress(learner, lesson).await?;
            if progress.validate_by_exam(lesson) {
                let tasks: Vec<&str> = progress.tasks_done.iter().map(String::as_str).collect();
                sqlx::query(
                    "UPDATE lesson_progress SET tasks_done = $1, quiz_best = $2, completed_at = to_timestamp($3), validated_by_exam = true \
                     WHERE learner_id = $4 AND course = $5 AND lesson = $6",
                )
                .bind(&tasks)
                .bind(progress.quiz_best as i32)
                .bind(now as f64)
                .bind(learner)
                .bind(&lesson.course)
                .bind(&lesson.slug)
                .execute(&mut *self.tx)
                .await?;
                lessons_validated += 1;
            }
        }
        // A course finished by practising already paid its own XP: the exam adds nothing to it.
        let xp_gained = if lessons_validated > 0 { self.grant(learner, &Award::exam(course)).await? } else { 0 };
        Ok(Submitted::Graded { grade: result, questions, xp_gained, lessons_validated })
    }
}
