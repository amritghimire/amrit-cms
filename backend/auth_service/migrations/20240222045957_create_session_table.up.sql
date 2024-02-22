CREATE TABLE sessions (
    identifier uuid NOT NULL constraint sessions_pk primary key,
    verifier_hash VARCHAR(225) NOT NULL unique ,
    expiration_date timestamptz NOT NULL,
    user_id         integer     not null
        constraint confirmations_users_id_fk
            references users on delete cascade ,
    extra_info json default '{}'::json NOT NULL
);