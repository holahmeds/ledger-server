create table public.transaction_tags
(
    transaction_id serial
        references public.transactions
            on delete cascade,
    tag            varchar not null,
    primary key (transaction_id, tag)
);
