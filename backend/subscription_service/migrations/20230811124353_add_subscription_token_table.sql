-- Create subscription tokens table
CREATE table subscription_tokens(
    subscription_token text not null ,
    subscription_id uuid not null
        references subscriptions(id),
    primary key (subscription_token)
)