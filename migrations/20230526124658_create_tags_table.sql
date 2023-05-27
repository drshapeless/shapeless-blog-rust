-- Add migration script here
CREATE TABLE IF NOT EXISTS tags (
       name TEXT NOT NULL,
       blog_id BIGINT NOT NULL
);
