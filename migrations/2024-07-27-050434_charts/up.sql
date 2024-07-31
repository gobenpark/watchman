-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.charts
(
    ticker   varchar(10) NOT NULL,
    open     double precision,
    high     double precision,
    low      double precision,
    close    double precision,
    volume   integer,
    datetime timestamp NOT NULL,
    PRIMARY KEY (ticker, datetime)
);



