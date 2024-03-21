CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- Create tables

CREATE TABLE users (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    login_name VARCHAR(24) NOT NULL,
    first_name VARCHAR(48) NOT NULL,
    last_name VARCHAR(48) NOT NULL,
    argon2 BYTEA NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT NULL,

    UNIQUE(login_name),
    PRIMARY KEY (id)
);

CREATE TABLE pets (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    name VARCHAR(32) NOT NULL,
    birthday TIMESTAMP DEFAULT NULL,
    owner_id UUID NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT NULL,

    PRIMARY KEY (id),
    CONSTRAINT fk_owner
        FOREIGN KEY(owner_id)
            REFERENCES users(id)
);


CREATE TABLE samples (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    label VARCHAR(32) NOT NULL,
    bytes BYTEA NOT NULL,
    owner_id UUID NOT NULL,
    pet_id UUID DEFAULT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT NULL,


    PRIMARY KEY (id),
    CONSTRAINT fk_owner
        FOREIGN KEY(owner_id)
            REFERENCES users(id),
    CONSTRAINT fk_pet
        FOREIGN KEY(pet_id)
            REFERENCES pets(id)
);

CREATE TYPE Classification AS ENUM ('normal', 'incipient');

CREATE TABLE results (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    certainity REAL NOT NULL,
    classification Classification NOT NULL,

    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,

    iris_x REAL DEFAULT NULL,
    iris_y REAL DEFAULT NULL,
    iris_a REAL DEFAULT NULL,
    iris_b REAL DEFAULT NULL,
    precentage REAL DEFAULT NULL,


    sample_id UUID NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT NULL,

    PRIMARY KEY (id),
    CONSTRAINT fk_sample
        FOREIGN KEY(sample_id)
            REFERENCES samples(id)
);

----

CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
