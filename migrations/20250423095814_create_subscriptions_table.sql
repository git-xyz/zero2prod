-- Active: 1745401898401@@127.0.0.1@5432@rust_01
-- Add migration script here
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email text NOT NULL UNIQUE,
    name text NOT NULL,
    subscribed_at timestamptz NOT NULL
)