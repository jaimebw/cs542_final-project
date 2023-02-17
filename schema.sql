-- This file outlines the base structure of the test sqlite database. The test database will be recreated when this
-- file is changed.

CREATE TABLE "users" (
    uid BINARY(16),
    email VARCHAR(100),
    password_hash BINARY(32),
    PRIMARY KEY (uid),
    UNIQUE (email)
);