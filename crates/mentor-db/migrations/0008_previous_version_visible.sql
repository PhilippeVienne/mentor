-- The management page of a tenant tells whether a previous version of a package is kept (and can be brought
-- back by the platform's operator): the application may read that table, for its tenant, like the others.
-- The pictures of a previous version stay out of its reach: nothing serves them.

CREATE POLICY tenant_read ON package_previous FOR SELECT TO mentor_app USING (tenant_id = current_tenant());
GRANT SELECT ON package_previous TO mentor_app;
