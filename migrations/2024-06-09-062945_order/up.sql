-- Your SQL goes here
create table orders
(
    id         varchar(200)               not null
        primary key,
    price      double precision,
    amount     double precision,
    created_at timestamp(6) default now() not null
);