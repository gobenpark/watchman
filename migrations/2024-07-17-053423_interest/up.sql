-- Your SQL goes here
create table interest
(
    id   varchar(20) primary key,
    sector_id varchar(20) constraint id references sector,
    symbol varchar(200)
);
