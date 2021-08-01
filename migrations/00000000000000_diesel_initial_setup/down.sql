-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

DROP INDEX IF EXISTS uid_index;
DROP INDEX IF EXISTS game_code_index;

DROP TABLE IF EXISTS playergame;
DROP TABLE IF EXISTS assignment;
DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS player;

DROP TYPE IF EXISTS game_status_t;
DROP TYPE IF EXISTS player_status_t;
DROP TYPE IF EXISTS target_status_t;
DROP TYPE IF EXISTS role_t;

DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();
