-- Fix: FutureAuth expects metadata to be non-null JSONB
UPDATE "user" SET metadata = '{}'::jsonb WHERE metadata IS NULL;
ALTER TABLE "user" ALTER COLUMN metadata SET DEFAULT '{}'::jsonb;
ALTER TABLE "user" ALTER COLUMN metadata SET NOT NULL;
