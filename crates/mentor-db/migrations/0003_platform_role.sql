-- Platform administration role.
--
-- Row-level security is FORCED, so it also binds the role that owns the tables: without a policy of its own,
-- that role could not even create a tenant. `mentor_platform` is the explicit, auditable way to act across
-- tenants (tenant administration, data migration, global reports). The role running the migrations becomes a
-- member; application logins must never be.

DO $$
BEGIN
    CREATE ROLE mentor_platform NOLOGIN;
EXCEPTION
    WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

DO $$
DECLARE
    name text;
BEGIN
    FOREACH name IN ARRAY ARRAY['tenant', 'learner', 'lesson_progress', 'award', 'learner_badge', 'cohort', 'cohort_member', 'exam_attempt'] LOOP
        EXECUTE format('CREATE POLICY platform_all ON %I TO mentor_platform USING (true) WITH CHECK (true)', name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO mentor_platform', name);
    END LOOP;
END
$$;

-- The role running the migrations must hold the PRIVILEGES of `mentor_platform`, which is what policies look
-- at. Since PostgreSQL 16, creating a role only gives its creator the right to administer it, not to inherit
-- from it: the grant has to be explicit. Membership is per cluster, so another database may be granting it at
-- the same moment; retry rather than fail.
DO $$
BEGIN
    FOR attempt IN 1..5 LOOP
        EXIT WHEN pg_has_role(current_user, 'mentor_platform', 'USAGE');
        BEGIN
            EXECUTE format('GRANT mentor_platform TO %I WITH INHERIT TRUE', current_user);
        EXCEPTION
            WHEN OTHERS THEN PERFORM pg_sleep(0.1);
        END;
    END LOOP;
    IF NOT pg_has_role(current_user, 'mentor_platform', 'USAGE') THEN
        RAISE EXCEPTION 'role % could not be granted the privileges of mentor_platform', current_user;
    END IF;
END
$$;
