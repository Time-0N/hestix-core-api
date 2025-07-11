CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       keycloak_id UUID NOT NULL UNIQUE,
                       username TEXT NOT NULL,
                       email TEXT NOT NULL UNIQUE,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
