CREATE TABLE clips (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP
);

ALTER TABLE clips ADD CONSTRAINT code_unique UNIQUE (code);