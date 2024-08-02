-- Your SQL goes here
create table if not exists public.positions
(
    id          uuid default uuid_generate_v4() not null
        primary key,
    ticker      varchar(10)                     not null,
    price       double precision                not null,
    amount      double precision                not null,
    strategy_id varchar(10)                     not null,
    created_at  timestamp                       not null
);

alter table public.positions
    owner to postgres;

