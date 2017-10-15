-- Your SQL goes here
-- Citext is a case-insensitive text field, so the database sees "foobar" and "FOOBAR" as equals
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    name citext NOT NULL UNIQUE,
    description VARCHAR
)