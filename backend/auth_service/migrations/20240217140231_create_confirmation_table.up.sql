-- Add migration script here
create table confirmations
(
    confirmation_id uuid        not null
        constraint confirmations_pk
            primary key,
    details         json default '{}'::json,
    verifier_hash   varchar(20) not null,
    user_id         integer     not null
        constraint confirmations_users_id_fk
            references users,
    created_at      timestamptz not null,
    expires_at      timestamptz not null,
    action_type     varchar(25) not null
);

comment on table confirmations is 'Universal confirmation table';

