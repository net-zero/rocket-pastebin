ALTER TABLE users DROP roles;
ALTER TABLE users ADD admin BOOLEAN NOT NULL DEFAULT 'f';