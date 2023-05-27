-- Add migration script here
CREATE TABLE IF NOT EXISTS blogs(
       id BIGSERIAL PRIMARY KEY,
       user_id BIGINT NOT NULL,
       url TEXT NOT NULL UNIQUE,
       title TEXT NOT NULL,
       preview TEXT NOT NULL,
       content TEXT NOT NULL,
       create_time TIMESTAMP(0) WITH TIME ZONE NOT NULL DEFAULT NOW(),
       edit_time TIMESTAMP(0) WITH TIME ZONE NOT NULL DEFAULT NOW(),
       version BIGINT NOT NULL DEFAULT 1
);
