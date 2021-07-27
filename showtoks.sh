#!/bin/sh

psql -d oauth -c "select * from tokens;" |cat

