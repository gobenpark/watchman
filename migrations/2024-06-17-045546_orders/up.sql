-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.orders
(
    id         integer               not null
        primary key,
    ticker varchar(10) not null,
    strategy_id varchar(10),
    price      double precision not null,
    quantity     integer not null,
    order_action  varchar(1) not null,
    created_at timestamp(6) default now() not null
    accepted     boolean      default false not null
);