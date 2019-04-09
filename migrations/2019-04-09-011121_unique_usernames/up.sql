CREATE TABLE user_new (
    id integer primary key,
    username text unique
);
INSERT INTO user_new SELECT id, username FROM user;
DROP TABLE user;
ALTER TABLE user_new RENAME TO user;
