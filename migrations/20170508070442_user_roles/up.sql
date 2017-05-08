ALTER TABLE users DROP admin;
ALTER TABLE users ADD roles TEXT[] NOT NULL DEFAULT '{"user"}';
