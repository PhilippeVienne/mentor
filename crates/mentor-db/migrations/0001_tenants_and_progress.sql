-- Tenants, learners and their progress.
--
-- Isolation model. Every tenant-owned table carries `tenant_id` and has row-level security ENABLED and
-- FORCED: a query only sees, and can only write, rows of the tenant named by the transaction setting
-- `app.tenant_id`. With no setting, it sees nothing. The application connects with a role that is a member of
-- `mentor_app`, which owns nothing and cannot bypass these policies. Migrations and platform administration
-- use the owning role.
--
-- Foreign keys include `tenant_id`, so a row can never point at another tenant's learner even if a policy
-- were wrong.

-- Roles belong to the whole cluster, not to one database: the role may already exist, or be created at the
-- same moment by a migration running in another database.
DO $$
BEGIN
    CREATE ROLE mentor_app NOLOGIN;
EXCEPTION
    WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

-- The tenant of the current transaction, or NULL when none was set.
CREATE FUNCTION current_tenant() RETURNS uuid
    LANGUAGE sql STABLE
    AS $$ SELECT nullif(current_setting('app.tenant_id', true), '')::uuid $$;

CREATE TABLE tenant (
    id         uuid PRIMARY KEY,
    slug       text NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9][a-z0-9-]{0,62}$'),
    name       text NOT NULL,
    -- Host names served for this tenant (`acme.mentor.example`, a custom domain).
    hostnames  text[] NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

-- A host name belongs to at most one tenant.
CREATE FUNCTION tenant_hostnames_are_unique() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF EXISTS (SELECT FROM tenant WHERE id <> NEW.id AND hostnames && NEW.hostnames) THEN
        RAISE EXCEPTION 'a host name of tenant % already belongs to another tenant', NEW.slug
            USING ERRCODE = 'unique_violation';
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER tenant_hostnames_unique BEFORE INSERT OR UPDATE OF hostnames ON tenant
    FOR EACH ROW EXECUTE FUNCTION tenant_hostnames_are_unique();

-- Finding the tenant of a request happens before any tenant is set, so it cannot go through the policies:
-- this function runs with its owner's rights and reveals nothing but the identifier.
CREATE FUNCTION resolve_tenant(host text) RETURNS uuid
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public
    AS $$ SELECT id FROM tenant WHERE lower(host) = ANY (hostnames) $$;

CREATE TABLE learner (
    id           uuid NOT NULL DEFAULT gen_random_uuid(),
    tenant_id    uuid NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    -- Subject of the tenant's identity provider (the OIDC `sub` claim).
    subject      text NOT NULL,
    username     text NOT NULL,
    display_name text NOT NULL DEFAULT '',
    email        text NOT NULL DEFAULT '',
    is_admin     boolean NOT NULL DEFAULT false,
    created_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, id),
    UNIQUE (tenant_id, subject)
);

CREATE TABLE lesson_progress (
    tenant_id         uuid NOT NULL,
    learner_id        uuid NOT NULL,
    course            text NOT NULL,
    lesson            text NOT NULL,
    -- Zero-based indices of validated lab steps.
    tasks_done        integer[] NOT NULL DEFAULT '{}',
    quiz_best         integer NOT NULL DEFAULT 0 CHECK (quiz_best >= 0),
    completed_at      timestamptz,
    validated_by_exam boolean NOT NULL DEFAULT false,
    PRIMARY KEY (tenant_id, learner_id, course, lesson),
    FOREIGN KEY (tenant_id, learner_id) REFERENCES learner (tenant_id, id) ON DELETE CASCADE
);

-- XP journal. The primary key is what makes XP idempotent: an achievement is paid once per learner.
CREATE TABLE award (
    tenant_id  uuid NOT NULL,
    learner_id uuid NOT NULL,
    kind       text NOT NULL CHECK (kind IN ('task', 'quiz', 'lesson', 'course', 'exam')),
    key        text NOT NULL,
    xp         integer NOT NULL CHECK (xp >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, learner_id, kind, key),
    FOREIGN KEY (tenant_id, learner_id) REFERENCES learner (tenant_id, id) ON DELETE CASCADE
);

CREATE TABLE learner_badge (
    tenant_id  uuid NOT NULL,
    learner_id uuid NOT NULL,
    badge      text NOT NULL,
    awarded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, learner_id, badge),
    FOREIGN KEY (tenant_id, learner_id) REFERENCES learner (tenant_id, id) ON DELETE CASCADE
);

ALTER TABLE tenant ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant FORCE ROW LEVEL SECURITY;
-- The application may read its own tenant row, and nothing else of this table.
CREATE POLICY own_tenant ON tenant FOR SELECT TO mentor_app USING (id = current_tenant());

DO $$
DECLARE
    name text;
BEGIN
    FOREACH name IN ARRAY ARRAY['learner', 'lesson_progress', 'award', 'learner_badge'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', name);
        EXECUTE format(
            'CREATE POLICY tenant_isolation ON %I TO mentor_app '
            'USING (tenant_id = current_tenant()) WITH CHECK (tenant_id = current_tenant())', name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO mentor_app', name);
    END LOOP;
END
$$;

GRANT SELECT ON tenant TO mentor_app;
GRANT EXECUTE ON FUNCTION resolve_tenant(text), current_tenant() TO mentor_app;
