-- Cohorts and exam attempts. Same isolation model as the first migration.

CREATE TABLE cohort (
    tenant_id     uuid NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    slug          text NOT NULL,
    name          text NOT NULL,
    description   text NOT NULL DEFAULT '',
    -- 'idp': mirrored from the identity provider's groups at each login; 'manual': managed by an administrator.
    source        text NOT NULL DEFAULT 'manual' CHECK (source IN ('idp', 'manual')),
    report_emails text[] NOT NULL DEFAULT '{}',
    created_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, slug)
);

CREATE TABLE cohort_member (
    tenant_id  uuid NOT NULL,
    cohort     text NOT NULL,
    learner_id uuid NOT NULL,
    source     text NOT NULL DEFAULT 'manual' CHECK (source IN ('idp', 'manual')),
    joined_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, cohort, learner_id),
    FOREIGN KEY (tenant_id, cohort) REFERENCES cohort (tenant_id, slug) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, learner_id) REFERENCES learner (tenant_id, id) ON DELETE CASCADE
);

CREATE TABLE exam_attempt (
    tenant_id   uuid NOT NULL,
    id          uuid NOT NULL DEFAULT gen_random_uuid(),
    learner_id  uuid NOT NULL,
    course      text NOT NULL,
    started_at  timestamptz NOT NULL,
    deadline    timestamptz NOT NULL,
    finished_at timestamptz,
    -- Drawn questions in shown order, each with the shown order of its options. Correct answers are not
    -- stored here: they stay in the catalogue.
    questions   jsonb NOT NULL,
    -- Question id → shown position chosen.
    answers     jsonb NOT NULL DEFAULT '{}',
    score       integer NOT NULL DEFAULT 0,
    total       integer NOT NULL DEFAULT 0,
    passed      boolean NOT NULL DEFAULT false,
    expired     boolean NOT NULL DEFAULT false,
    PRIMARY KEY (tenant_id, id),
    FOREIGN KEY (tenant_id, learner_id) REFERENCES learner (tenant_id, id) ON DELETE CASCADE
);

-- At most one open attempt per learner and course, whatever the application does.
CREATE UNIQUE INDEX exam_attempt_one_open ON exam_attempt (tenant_id, learner_id, course) WHERE finished_at IS NULL;

DO $$
DECLARE
    name text;
BEGIN
    FOREACH name IN ARRAY ARRAY['cohort', 'cohort_member', 'exam_attempt'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', name);
        EXECUTE format(
            'CREATE POLICY tenant_isolation ON %I TO mentor_app '
            'USING (tenant_id = current_tenant()) WITH CHECK (tenant_id = current_tenant())', name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO mentor_app', name);
    END LOOP;
END
$$;
