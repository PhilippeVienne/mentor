//! Cohorts mirrored from the identity provider.

use std::collections::BTreeMap;

use mentor_core::cohorts::{cohort_name, cohort_slug};
use uuid::Uuid;

use crate::{Result, TenantTx};

impl TenantTx {
    /// Aligns a learner's identity-provider cohorts with the groups of their token: missing cohorts and
    /// memberships are created, memberships of groups they left are removed. Memberships added by hand are
    /// never touched. Returns the slugs the learner now belongs to through the identity provider.
    pub async fn sync_idp_cohorts(&mut self, learner: Uuid, groups: &[&str]) -> Result<Vec<String>> {
        let wanted: BTreeMap<String, String> =
            groups.iter().map(|group| (cohort_slug(group), cohort_name(group))).filter(|(slug, _)| !slug.is_empty()).collect();
        for (slug, name) in &wanted {
            sqlx::query("INSERT INTO cohort (tenant_id, slug, name, source) VALUES ($1, $2, $3, 'idp') ON CONFLICT DO NOTHING")
                .bind(self.tenant)
                .bind(slug)
                .bind(name)
                .execute(&mut *self.tx)
                .await?;
            sqlx::query(
                "INSERT INTO cohort_member (tenant_id, cohort, learner_id, source) VALUES ($1, $2, $3, 'idp') ON CONFLICT DO NOTHING",
            )
            .bind(self.tenant)
            .bind(slug)
            .bind(learner)
            .execute(&mut *self.tx)
            .await?;
        }
        let slugs: Vec<String> = wanted.into_keys().collect();
        sqlx::query("DELETE FROM cohort_member WHERE learner_id = $1 AND source = 'idp' AND NOT (cohort = ANY ($2))")
            .bind(learner)
            .bind(&slugs)
            .execute(&mut *self.tx)
            .await?;
        Ok(slugs)
    }

    /// Adds a learner to a cohort by hand, creating the cohort when needed.
    pub async fn add_to_cohort(&mut self, learner: Uuid, slug: &str, name: &str) -> Result<()> {
        sqlx::query("INSERT INTO cohort (tenant_id, slug, name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(self.tenant)
            .bind(slug)
            .bind(name)
            .execute(&mut *self.tx)
            .await?;
        sqlx::query("INSERT INTO cohort_member (tenant_id, cohort, learner_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(self.tenant)
            .bind(slug)
            .bind(learner)
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }

    /// Slugs of the cohorts a learner belongs to, in alphabetical order.
    pub async fn cohorts_of(&mut self, learner: Uuid) -> Result<Vec<String>> {
        Ok(sqlx::query_scalar("SELECT cohort FROM cohort_member WHERE learner_id = $1 ORDER BY cohort")
            .bind(learner)
            .fetch_all(&mut *self.tx)
            .await?)
    }
}
