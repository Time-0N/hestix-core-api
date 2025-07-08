CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       username TEXT NOT NULL,
                       email TEXT NOT NULL UNIQUE,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
