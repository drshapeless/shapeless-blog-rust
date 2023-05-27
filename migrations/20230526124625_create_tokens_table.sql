-- Add migration script here
CREATE TABLE IF NOT EXISTS tokens (
    user_id BIGINT NOT NULL,
    token TEXT NOT NULL,
    expired_time TIMESTAMP(0) WITH TIME ZONE NOT NULL
);
