-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    name citext NOT NULL UNIQUE,
    description VARCHAR
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    video_id VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR,
    room_id SERIAL REFERENCES rooms (id),
    duration VARCHAR NOT NULL,
    played BOOLEAN NOT NULL DEFAULT 'f',
    added_on TIMESTAMP NOT NULL DEFAULT now(),
    played_on TIMESTAMP DEFAULT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username citext NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    added_on TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP DEFAULT NULL
);