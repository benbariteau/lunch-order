CREATE TABLE user_new (
    id integer primary key,
    email text
);
INSERT INTO user_new (id, email) SELECT id, name FROM user;
DROP TABLE user;
ALTER TABLE user_new RENAME TO user;
