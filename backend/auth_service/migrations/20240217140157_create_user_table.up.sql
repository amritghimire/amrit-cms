-- Add up migration script here
-- Add migration script here

CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       name VARCHAR(100) NOT NULL,
                       email VARCHAR(150) UNIQUE NOT NULL,
                       username varchar(150) unique not null ,
                       normalized_username varchar(150) unique not null ,
                       password_hash VARCHAR(128) NOT NULL,
                       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
                       is_active bool default true not null ,
                       is_confirmed bool default false not null
);