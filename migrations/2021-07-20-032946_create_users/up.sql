CREATE TABLE users (
    name        varchar(16) CONSTRAINT firstkey PRIMARY KEY,
    hash        varchar(128) NOT NULL, -- what is the max output length?
    scopes      text[] NOT NULL
    -- XXX expiration time?
);

INSERT INTO users(name, hash, scopes) VALUES 
    ('admin', '$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tc2FsdA$HXrbCSqkWTwH9W4z4JTyyJuuhEX/DNDs5tgTDfo+dHI', ARRAY[ 'authadmin' ]);

