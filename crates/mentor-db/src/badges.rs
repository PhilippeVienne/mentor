//! Badges earned: computes a learner's statistics from stored progress and grants what they newly earned.

use std::collections::{BTreeMap, BTreeSet};

use mentor_core::badges::{newly_earned, BadgeDef, Stats};
use mentor_core::gamification::{level_info, DEFAULT_LEVEL_TITLES};
use mentor_core::progress::{completed_courses, longest_streak, CourseShape};
use sqlx::Row;
use uuid::Uuid;

use crate::{Result, TenantTx};

/// What the statistics need to know about the tenant's catalogue.
#[derive(Debug, Clone, Default)]
pub struct CatalogueView {
    pub courses: Vec<CourseShape>,
    /// Number of quiz questions of each lesson, by `course/lesson` reference.
    pub questions: BTreeMap<String, u32>,
}

impl TenantTx {
    /// A learner's achievements so far. `utc_offset_minutes` is the tenant's time zone, used to cut activity
    /// into days for the streak.
    pub async fn learner_stats(&mut self, learner: Uuid, catalogue: &CatalogueView, utc_offset_minutes: i32) -> Result<Stats> {
        let rows = sqlx::query(
            "SELECT course, lesson, cardinality(tasks_done) AS tasks, quiz_best, completed_at IS NOT NULL AS completed, validated_by_exam \
             FROM lesson_progress WHERE learner_id = $1",
        )
        .bind(learner)
        .fetch_all(&mut *self.tx)
        .await?;
        let mut stats = Stats::default();
        for row in rows {
            let reference = format!("{}/{}", row.get::<String, _>("course"), row.get::<String, _>("lesson"));
            // Lessons validated through an exam count as done, but not as practice.
            if !row.get::<bool, _>("validated_by_exam") {
                stats.tasks += row.get::<i32, _>("tasks") as u32;
                let questions = catalogue.questions.get(&reference).copied().unwrap_or(0);
                if questions > 0 && row.get::<i32, _>("quiz_best") as u32 == questions {
                    stats.perfect_quizzes += 1;
                }
            }
            if row.get("completed") {
                stats.lessons_done.insert(reference);
            }
        }
        stats.courses_done = completed_courses(&catalogue.courses, &stats.lessons_done);
        stats.level = level_info(self.total_xp(learner).await?, &DEFAULT_LEVEL_TITLES).level;
        stats.exams_passed =
            sqlx::query_scalar::<_, i64>("SELECT count(DISTINCT course) FROM exam_attempt WHERE learner_id = $1 AND passed")
                .bind(learner)
                .fetch_one(&mut *self.tx)
                .await? as u32;
        let days: Vec<i64> = sqlx::query_scalar(
            "SELECT DISTINCT floor((extract(epoch FROM created_at) + $2::bigint * 60) / 86400)::bigint FROM award WHERE learner_id = $1",
        )
        .bind(learner)
        .bind(i64::from(utc_offset_minutes))
        .fetch_all(&mut *self.tx)
        .await?;
        stats.streak = longest_streak(days);
        Ok(stats)
    }

    /// Badges the learner already owns.
    pub async fn badges_of(&mut self, learner: Uuid) -> Result<BTreeSet<String>> {
        let slugs: Vec<String> =
            sqlx::query_scalar("SELECT badge FROM learner_badge WHERE learner_id = $1").bind(learner).fetch_all(&mut *self.tx).await?;
        Ok(slugs.into_iter().collect())
    }

    /// When the learner was awarded each badge they own, in seconds since the Unix epoch.
    pub async fn badge_dates(&mut self, learner: Uuid) -> Result<BTreeMap<String, i64>> {
        let rows = sqlx::query("SELECT badge, extract(epoch FROM awarded_at)::bigint AS awarded FROM learner_badge WHERE learner_id = $1")
            .bind(learner)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows.into_iter().map(|row| (row.get("badge"), row.get("awarded"))).collect())
    }

    /// Grants every badge the learner has earned and does not own yet; returns the new ones, in order.
    pub async fn award_badges(
        &mut self,
        learner: Uuid,
        definitions: &[BadgeDef],
        catalogue: &CatalogueView,
        utc_offset_minutes: i32,
    ) -> Result<Vec<String>> {
        let stats = self.learner_stats(learner, catalogue, utc_offset_minutes).await?;
        let owned = self.badges_of(learner).await?;
        let mut granted = Vec::new();
        for slug in newly_earned(definitions, &owned, &stats) {
            let inserted =
                sqlx::query("INSERT INTO learner_badge (tenant_id, learner_id, badge) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
                    .bind(self.tenant)
                    .bind(learner)
                    .bind(&slug)
                    .execute(&mut *self.tx)
                    .await?
                    .rows_affected();
            if inserted == 1 {
                granted.push(slug);
            }
        }
        Ok(granted)
    }
}
