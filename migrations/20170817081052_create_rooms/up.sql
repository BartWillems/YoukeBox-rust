-- Your SQL goes here
CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE CONSTRAINT lowercase CHECK (name = lower(name)),
    description VARCHAR
)