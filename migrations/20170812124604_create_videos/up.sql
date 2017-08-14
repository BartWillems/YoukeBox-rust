-- Your SQL goes here
CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    video_id VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR,
    duration VARCHAR NOT NULL,
    played BOOLEAN NOT NULL DEFAULT 'f',
    added_on TIMESTAMP NOT NULL DEFAULT now(),
    played_on TIMESTAMP DEFAULT NULL
)