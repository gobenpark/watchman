-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.positions
(
    id   uuid NOT NULL default uuid_generate_v4() primary key,
    ticker   varchar(10) NOT NULL,
    price   double precision NOT NULL,
    amount     double precision NOT NULL,
    strategy_id varchar(10) NOT NULL,
    created_at timestamp NOT NULL default now()
);



