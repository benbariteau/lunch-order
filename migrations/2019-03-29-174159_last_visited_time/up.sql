CREATE TABLE restaurant_new (
    id integer primary key,
    name text not null,
    last_visit_time string not null
);

INSERT INTO restaurant_new (id, name, last_visit_time) SELECT id, name, last_visit_date FROM restaurant;
DROP TABLE restaurant;
ALTER TABLE restaurant_new RENAME TO restaurant;
