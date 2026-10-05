-- Brand of a tenant (name shown, colours, links): what v1 read from `branding.yml`. Missing keys take the
-- platform defaults, so an empty object is a valid, neutral brand.
ALTER TABLE tenant ADD COLUMN branding jsonb NOT NULL DEFAULT '{}';
