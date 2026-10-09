-- Course packages installed for a tenant.
--
-- A tenant sees the courses of the packages installed for it, and nothing else: no catalogue is shared by
-- default. A package is stored compiled (its courses and training paths as JSON, its pictures as bytes), so
-- that serving a tenant needs neither the package's directory nor a compiler.
--
-- The application role only READS these tables: installing, updating and removing a package is an act of
-- platform administration (`mentor package-install`), done by the owning role.

CREATE TABLE package (
    tenant_id    uuid NOT NULL REFERENCES tenant (id) ON DELETE CASCADE,
    -- The `name` of the manifest: what identifies the package for this tenant, whatever its source.
    name         text NOT NULL,
    version      text NOT NULL,
    title        text NOT NULL,
    -- Where it was installed from, as given to the command; informative.
    source       text NOT NULL,
    -- Order of the packages in the tenant's catalogue.
    position     integer NOT NULL,
    manifest     jsonb NOT NULL,
    -- Compiled courses, in the order of the manifest.
    courses      jsonb NOT NULL,
    -- Compiled training paths of the package.
    paths        jsonb NOT NULL DEFAULT '[]',
    installed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, name)
);

-- What each package brings, so that two packages of a tenant can never define the same course or the same
-- training path: stored progress and addresses refer to them by these names.
CREATE TABLE package_item (
    tenant_id uuid NOT NULL,
    kind      text NOT NULL CHECK (kind IN ('course', 'path')),
    slug      text NOT NULL,
    package   text NOT NULL,
    PRIMARY KEY (tenant_id, kind, slug),
    FOREIGN KEY (tenant_id, package) REFERENCES package (tenant_id, name) ON DELETE CASCADE
);

-- Pictures of the courses (their `images/` folders): the only files of a package that are ever served.
CREATE TABLE package_image (
    tenant_id  uuid NOT NULL,
    package    text NOT NULL,
    course     text NOT NULL,
    path       text NOT NULL,
    media_type text NOT NULL,
    content    bytea NOT NULL,
    PRIMARY KEY (tenant_id, course, path),
    FOREIGN KEY (tenant_id, package) REFERENCES package (tenant_id, name) ON DELETE CASCADE
);

DO $$
DECLARE
    name text;
BEGIN
    FOREACH name IN ARRAY ARRAY['package', 'package_item', 'package_image'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', name);
        EXECUTE format('CREATE POLICY tenant_read ON %I FOR SELECT TO mentor_app USING (tenant_id = current_tenant())', name);
        EXECUTE format('GRANT SELECT ON %I TO mentor_app', name);
        EXECUTE format('CREATE POLICY platform_all ON %I TO mentor_platform USING (true) WITH CHECK (true)', name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO mentor_platform', name);
    END LOOP;
END
$$;
