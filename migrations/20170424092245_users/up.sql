CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(16) UNIQUE NOT NULL,
    -- email is case-insensitive, but unique is case-sensitive
    -- should convert to lower case before save in database
    email TEXT UNIQUE NOT NULL,
    password_digest BYTEA NOT NULL,
    admin BOOLEAN NOT NULL DEFAULT 'f'
)
