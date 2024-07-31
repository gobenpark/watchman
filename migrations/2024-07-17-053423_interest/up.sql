-- Your SQL goes here
create table interest
(
    id   uuid NOT NULL default uuid_generate_v4() primary key,
    sector_id uuid constraint id references sector,
    symbol varchar(200)
);
