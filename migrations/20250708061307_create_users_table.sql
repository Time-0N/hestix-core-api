CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       keycloak_id TEXT NOT NULL,
                       username TEXT NOT NULL,
                       email TEXT NOT NULL UNIQUE,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
