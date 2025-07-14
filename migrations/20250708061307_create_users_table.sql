-- sqlx-up
CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       keycloak_id UUID NOT NULL UNIQUE,
                       username TEXT NOT NULL,
                       email TEXT NOT NULL UNIQUE,
                       created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- sqlx-down
DROP TABLE users;