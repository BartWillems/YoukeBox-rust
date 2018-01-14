CREATE TABLE rooms (
    "id"            SERIAL  PRIMARY KEY,
    "name"          VARCHAR NOT NULL UNIQUE,
    "description"   VARCHAR,
    "is_public"     BOOLEAN NOT NULL DEFAULT 't'
);

CREATE UNIQUE INDEX unique_roomname_on_rooms ON rooms (lower(name));

CREATE TABLE videos (
    "id"            SERIAL      PRIMARY KEY,
    "video_id"      VARCHAR     NOT NULL,
    "title"         VARCHAR     NOT NULL,
    "description"   VARCHAR,
    "room_id"       SERIAL      REFERENCES rooms (id),
    "duration"      VARCHAR     NOT NULL,
    "played"        BOOLEAN     NOT NULL DEFAULT 'f',
    "added_on"      TIMESTAMP   NOT NULL DEFAULT now(),
    "started_on"    TIMESTAMP   DEFAULT NULL
);

CREATE TABLE users (
    "id"            SERIAL      PRIMARY KEY,
    "username"      VARCHAR     NOT NULL UNIQUE,
    "password"      VARCHAR     NOT NULL,
    "added_on"      TIMESTAMP   NOT NULL DEFAULT now(),
    "updated_at"    TIMESTAMP   DEFAULT NULL
);

CREATE UNIQUE INDEX unique_username_on_users ON users (lower(username));