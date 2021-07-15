-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS account;

DROP TYPE IF EXISTS game_status_t;
DROP TYPE IF EXISTS player_status_t;
DROP TYPE IF EXISTS target_status_t;
DROP TYPE IF EXISTS role_t;

DROP INDEX IF EXISTS uid_index;

DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();
