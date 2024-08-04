-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.orders
(
    id         integer               not null
        primary key,
    ticker varchar(10),
    price      double precision,
    quantity     double precision,
    order_type  INTEGER,
    created_at timestamp(6) default now() not null
);