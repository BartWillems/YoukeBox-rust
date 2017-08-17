-- Your SQL goes here
CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
)