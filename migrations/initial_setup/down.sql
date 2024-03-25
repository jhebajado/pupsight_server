-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();

-- Drop tables
DROP TABLE pets CASCADE;
DROP TABLE results CASCADE;
DROP TABLE samples CASCADE;
DROP TABLE users CASCADE;
DROP TYPE classification;
