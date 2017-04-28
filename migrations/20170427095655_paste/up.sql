CREATE TABLE pastes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users (id),
    data TEXT NOT NULL
);
