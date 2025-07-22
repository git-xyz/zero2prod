-- Add migration script here
CREATE TABLE subscriptions_tokens (
    subscription_token TEXT NOT NULL,
    subscription_id UUID NOT NULL
        REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_token)
);
COMMIT;