#!/bin/sh

curl http://localhost:8000/test
echo

curl -X POST -H "Content-type: application/json" http://localhost:8000/auth -d '{"name": "admin", "secret": "bad", "scopes": ["authadmin"] }'
echo

curl -X POST -H "Content-type: application/json" http://localhost:8000/auth -d '{"name": "admin", "secret": "adminadmin", "scopes": ["authadmin", "donthave"] }'
echo

curl -X POST -H "Content-type: application/json" http://localhost:8000/auth -d '{"name": "admin", "secret": "adminadmin", "scopes": ["authadmin"] }'
echo

