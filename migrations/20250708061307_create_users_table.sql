CREATE TABLE users (
                       id          UUID PRIMARY KEY,
                       idp_issuer  TEXT NOT NULL,
                       idp_subject TEXT NOT NULL,
                       username    TEXT NOT NULL,
                       email       TEXT NOT NULL UNIQUE,
                       created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                       CONSTRAINT users_idp_identity_unique UNIQUE (idp_issuer, idp_subject)
);