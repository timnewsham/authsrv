
-- $ psql -d oauth -f create.sql

DROP TABLE IF EXISTS users;

CREATE TABLE users (
    name        varchar(16) CONSTRAINT firstkey PRIMARY KEY,
    hash        varchar(128) NOT NULL, -- what is the max output length?
    scopes      varchar(128) NOT NULL
    -- XXX expiration time?
);

INSERT INTO users(name, hash, scopes) VALUES 
    ('admin', '$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tc2FsdA$HXrbCSqkWTwH9W4z4JTyyJuuhEX/DNDs5tgTDfo+dHI', 'authadmin');


DROP TABLE IF EXISTS tokens;

CREATE TABLE tokens (
    token       char(40) PRIMARY KEY,
    title       varchar(40) NOT NULL,
    scopes      varchar(128) NOT NULL,
    expire      timestamp NOT NULL
);
