CREATE TABLE users (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR(16) NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX users_name_idx ON users ((LOWER(name)));

CREATE TABLE posts (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content VARCHAR(240) NOT NULL,
    create_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX posts_user_id_idx ON posts (user_id);
