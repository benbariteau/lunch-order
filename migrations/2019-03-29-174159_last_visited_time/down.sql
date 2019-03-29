CREATE TABLE restaurant_old (
    id integer primary key,
    name text not null,
    last_visit_date string not null
);

INSERT INTO restaurant_old (id, name, last_visit_date) SELECT id, name, last_visit_time FROM restaurant;
DROP TABLE restaurant;
ALTER TABLE restaurant_old RENAME TO restaurant;
