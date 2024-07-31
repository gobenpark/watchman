-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.orders
(
    id         integer               not null
        primary key,
    price      double precision,
    amount     double precision,
    order_type  INTEGER,
    created_at timestamp(6) default now() not null
);