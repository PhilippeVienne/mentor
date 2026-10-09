-- The version a package had before it was last replaced, kept so that an update can be undone.
--
-- One version back only: replacing a package again discards the older copy. Undoing swaps the two, so undoing
-- twice gives the update back.

CREATE TABLE package_previous (
    tenant_id   uuid NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    name        text NOT NULL,
    version     text NOT NULL,
    title       text NOT NULL,
    source      text NOT NULL,
    manifest    jsonb NOT NULL,
    courses     jsonb NOT NULL,
    paths       jsonb NOT NULL,
    replaced_at timestamptz NOT NULL DEFAULT now(),
    -- No reference to `package`: replacing a package deletes and recreates its row, and the copy must
    -- survive that. It is removed with the package by `mentor package-remove`.
    PRIMARY KEY (tenant_id, name)
);

CREATE TABLE package_previous_image (
    tenant_id  uuid NOT NULL,
    package    text NOT NULL,
    course     text NOT NULL,
    path       text NOT NULL,
    media_type text NOT NULL,
    content    bytea NOT NULL,
    PRIMARY KEY (tenant_id, package, course, path),
    FOREIGN KEY (tenant_id, package) REFERENCES package_previous (tenant_id, name) ON DELETE CASCADE
);

-- Nothing here is ever read by the application: only the platform role has a policy.
DO $$
DECLARE
    name text;
BEGIN
    FOREACH name IN ARRAY ARRAY['package_previous', 'package_previous_image'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', name);
        EXECUTE format('CREATE POLICY platform_all ON %I TO mentor_platform USING (true) WITH CHECK (true)', name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO mentor_platform', name);
    END LOOP;
END
$$;
