-- Add migration script here
CREATE TABLE IF NOT EXISTS users(
       id BIGSERIAL PRIMARY KEY,
       username TEXT NOT NULL UNIQUE,
       hashed_password TEXT NOT NULL,
       version BIGINT NOT NULL DEFAULT 1
);
