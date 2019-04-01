CREATE TABLE user_new (
    id integer primary key,
    name text
);
INSERT INTO user_new (id, name) SELECT id, email FROM user;
DROP TABLE user;
ALTER TABLE user_new RENAME TO user;
