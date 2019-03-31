CREATE TABLE user (
    id integer primary key,
    name text
);
CREATE TABLE user_private (
    id integer primary key,
    user_id integer,
    password_hash text
);
