-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.daily_cap
(
    ticker         varchar(10) not null
        primary key,
    market_cap     bigint,
    trading_volume bigint,
    change         numeric,
    datetime       timestamp   not null
);

alter table public.daily_cap
    owner to postgres;

