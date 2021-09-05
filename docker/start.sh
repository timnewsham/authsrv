#!/bin/sh

/etc/init.d/postgresql start
/etc/init.d/redis-server start
su -c "./authsrv" $APP_USER

