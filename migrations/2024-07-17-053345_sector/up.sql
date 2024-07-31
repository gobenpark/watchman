-- Your SQL goes here
create table sector
(
    id   uuid NOT NULL default uuid_generate_v4() primary key,
    name varchar(200)
);

