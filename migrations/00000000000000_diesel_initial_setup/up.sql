-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

CREATE TYPE game_status_t AS ENUM (
    'WAITING_FOR_PLAYERS',
    'ACTIVE',
    'FINISHED',
    'PAUSED'
);

CREATE TYPE player_status_t AS ENUM (
    'DEAD',
    'ALIVE',
    'LEFT_GAME'
);

CREATE TYPE target_status_t AS ENUM (
    'CURRENT',
    'KILL_SUCCESS',
    'TARGET_LEFT',
    'REASSIGNED',
    'GAME_END'
);

CREATE TYPE role_t AS ENUM (
    'ADMIN',
    'USER'
);

CREATE TABLE account (
    id              INT GENERATED ALWAYS AS IDENTITY,
    nickname        VARCHAR NOT NULL UNIQUE,
    email           VARCHAR NOT NULL UNIQUE,
    uid     VARCHAR NOT NULL UNIQUE,
    role            role_t NOT NULL DEFAULT 'USER',
    registered_at   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX uid_index ON account(uid);

CREATE TABLE game (
    id              INT GENERATED ALWAYS AS IDENTITY,
    lobby_name      VARCHAR,
    lobby_owner     INT NOT NULL
                    REFERENCES account(id)
                        ON UPDATE CASCADE ON DELETE NO ACTION,
    code            VARCHAR(6) UNIQUE NOT NULL,
    -- Insert here game settings
    game_status     game_status_t NOT NULL,
    PRIMARY KEY (id)
);


-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
