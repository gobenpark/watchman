-- Your SQL goes here
create table public.articles
(
    id         varchar(200) not null
        constraint article_pk
            primary key,
    title      varchar(200),
    contents   text,
    embedding  vector(1536),
    url        varchar(200),
    site       varchar(200),
    created_at timestamp
);

alter table public.article
    owner to postgres;

create index article_embedding_idx
    on public.article using hnsw (embedding public.vector_l2_ops);