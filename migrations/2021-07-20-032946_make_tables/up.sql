CREATE TABLE users (
    name        varchar(16) CONSTRAINT firstkey PRIMARY KEY,
    hash        varchar(128) NOT NULL, -- what is the max output length?
    expiration  timestamp NOT NULL,
    enabled     bool NOT NULL,
    scopes      text[] NOT NULL
);

INSERT INTO users(name, hash, expiration, enabled, scopes) VALUES 
    ('admin', '$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tc2FsdA$HXrbCSqkWTwH9W4z4JTyyJuuhEX/DNDs5tgTDfo+dHI', '2030-01-01 00:00:01', true, ARRAY[ 'authadmin' ])
    ;

CREATE TABLE scopes (
    name        varchar(16) PRIMARY KEY
);

INSERT INTO scopes(name) VALUES
    ('authadmin')
    ;
