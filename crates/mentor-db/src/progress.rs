//! Lesson progress and XP: applies the rules of `mentor-core` and persists their outcome.

use std::collections::{BTreeMap, BTreeSet};

use mentor_core::progress::{record, Award, AwardKind, Event, LessonProgress, LessonRules, Source};
use sqlx::Row;
use uuid::Uuid;

use crate::{Error, Result, TenantTx};

fn kind_name(kind: AwardKind) -> &'static str {
    match kind {
        AwardKind::Task => "task",
        AwardKind::Quiz => "quiz",
        AwardKind::Lesson => "lesson",
        AwardKind::Course => "course",
        AwardKind::Exam => "exam",
    }
}

/// What an accepted event changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// XP actually credited: an achievement already paid earns nothing, even under concurrent requests.
    pub xp_gained: u32,
    pub lesson_completed: bool,
    pub progress: LessonProgress,
}

/// Where a learner stands on a lesson, as lists and dashboards show it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LessonState {
    pub tasks_done: u32,
    pub quiz_best: u32,
    pub completed: bool,
    pub validated_by_exam: bool,
}

impl LessonState {
    /// Something was done on the lesson. Merely opening it does not count.
    pub fn started(&self) -> bool {
        self.tasks_done > 0 || self.quiz_best > 0
    }
}

impl TenantTx {
    /// A learner's state on every lesson they touched, by `course/lesson` reference.
    pub async fn lesson_states(&mut self, learner: Uuid) -> Result<BTreeMap<String, LessonState>> {
        let rows = sqlx::query(
            "SELECT course, lesson, cardinality(tasks_done) AS tasks, quiz_best, completed_at IS NOT NULL AS completed, validated_by_exam \
             FROM lesson_progress WHERE learner_id = $1",
        )
        .bind(learner)
        .fetch_all(&mut *self.tx)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let state = LessonState {
                    tasks_done: row.get::<i32, _>("tasks") as u32,
                    quiz_best: row.get::<i32, _>("quiz_best") as u32,
                    completed: row.get("completed"),
                    validated_by_exam: row.get("validated_by_exam"),
                };
                (format!("{}/{}", row.get::<String, _>("course"), row.get::<String, _>("lesson")), state)
            })
            .collect())
    }

    /// A learner's state on a lesson; the default (nothing done) when they never opened it.
    ///
    /// The row is locked until the end of the transaction, so two requests of the same learner on the same
    /// lesson are applied one after the other.
    pub async fn lesson_progress(&mut self, learner: Uuid, lesson: &LessonRules) -> Result<LessonProgress> {
        sqlx::query("INSERT INTO lesson_progress (tenant_id, learner_id, course, lesson) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
            .bind(self.tenant)
            .bind(learner)
            .bind(&lesson.course)
            .bind(&lesson.slug)
            .execute(&mut *self.tx)
            .await?;
        let row = sqlx::query(
            "SELECT tasks_done, quiz_best, completed_at IS NOT NULL AS completed, validated_by_exam \
             FROM lesson_progress WHERE learner_id = $1 AND course = $2 AND lesson = $3 FOR UPDATE",
        )
        .bind(learner)
        .bind(&lesson.course)
        .bind(&lesson.slug)
        .fetch_one(&mut *self.tx)
        .await?;
        let tasks: Vec<i32> = row.get("tasks_done");
        Ok(LessonProgress {
            tasks_done: tasks.into_iter().map(|index| index as u32).collect::<BTreeSet<u32>>(),
            quiz_best: row.get::<i32, _>("quiz_best") as u32,
            completed: row.get("completed"),
            validated_by_exam: row.get("validated_by_exam"),
        })
    }

    /// Stores an award unless the learner already has it; returns the XP credited.
    pub async fn grant(&mut self, learner: Uuid, award: &Award) -> Result<u32> {
        let inserted =
            sqlx::query("INSERT INTO award (tenant_id, learner_id, kind, key, xp) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING")
                .bind(self.tenant)
                .bind(learner)
                .bind(kind_name(award.kind))
                .bind(&award.key)
                .bind(award.xp as i32)
                .execute(&mut *self.tx)
                .await?
                .rows_affected();
        Ok(if inserted == 1 { award.xp } else { 0 })
    }

    /// Applies a learner event with the rules of `mentor-core` and persists the result.
    pub async fn record_event(&mut self, learner: Uuid, lesson: &LessonRules, event: Event, source: Source) -> Result<Recorded> {
        let mut progress = self.lesson_progress(learner, lesson).await?;
        let outcome = record(lesson, &mut progress, event, source).map_err(Error::Refused)?;
        let tasks: Vec<i32> = progress.tasks_done.iter().map(|&index| index as i32).collect();
        sqlx::query(
            "UPDATE lesson_progress SET tasks_done = $1, quiz_best = $2, \
             completed_at = CASE WHEN $3 AND completed_at IS NULL THEN now() ELSE completed_at END \
             WHERE learner_id = $4 AND course = $5 AND lesson = $6",
        )
        .bind(&tasks)
        .bind(progress.quiz_best as i32)
        .bind(progress.completed)
        .bind(learner)
        .bind(&lesson.course)
        .bind(&lesson.slug)
        .execute(&mut *self.tx)
        .await?;
        let mut xp_gained = 0;
        for award in &outcome.awards {
            xp_gained += self.grant(learner, award).await?;
        }
        Ok(Recorded { xp_gained, lesson_completed: outcome.lesson_completed, progress })
    }

    /// `course/lesson` references of the lessons a learner has completed.
    pub async fn completed_lessons(&mut self, learner: Uuid) -> Result<BTreeSet<String>> {
        let references: Vec<String> =
            sqlx::query_scalar("SELECT course || '/' || lesson FROM lesson_progress WHERE learner_id = $1 AND completed_at IS NOT NULL")
                .bind(learner)
                .fetch_all(&mut *self.tx)
                .await?;
        Ok(references.into_iter().collect())
    }

    /// Total XP of a learner.
    pub async fn total_xp(&mut self, learner: Uuid) -> Result<u32> {
        let total: i64 = sqlx::query_scalar("SELECT coalesce(sum(xp), 0)::bigint FROM award WHERE learner_id = $1")
            .bind(learner)
            .fetch_one(&mut *self.tx)
            .await?;
        Ok(total as u32)
    }
}
