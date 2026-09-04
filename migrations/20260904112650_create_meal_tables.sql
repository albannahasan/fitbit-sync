-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    auth_subject TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE MEALS (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    energy_kcal INTEGER NOT NULL,
    protein_g NUMERIC(7, 2),
    carbohydrate_g NUMERIC(7, 2),
    fat_g NUMERIC(7, 2),
    fiber_g NUMERIC(7, 2),
    meal_type TEXT NOT NULL DEFAULT 'UNKNOWN',
    eaten_at TIMESTAMPTZ NOT NULL,
    idempotency_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confidence DECIMAL(3, 2),
    CONSTRAINT not_null_check CHECK (CHAR_LENGTH(BTRIM(name)) > 0 AND CHAR_LENGTH(BTRIM(idempotency_key)) > 0),
    CONSTRAINT macro_check CHECK (protein_g >= 0 AND carbohydrate_g >= 0 AND fat_g >= 0 AND fiber_g >= 0),
    CONSTRAINT energy_kcal_check CHECK (energy_kcal > 0 AND energy_kcal <= 20000),
    CONSTRAINT unique_idempotency_key UNIQUE (user_id, idempotency_key),
    CONSTRAINT confidence_check CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT meal_type_check CHECK (meal_type IN ('unknown', 'breakfast', 'lunch', 'dinner', 'snack', 'other'))
);  

CREATE INDEX meals_user_eaten_at_idx ON MEALS(user_id, eaten_at DESC, id DESC);


CREATE TABLE MEAL_EVENTS (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meal_id UUID NOT NULL REFERENCES MEALS(id),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX event_lookup on MEAL_EVENTS(meal_id, created_at DESC, id DESC);