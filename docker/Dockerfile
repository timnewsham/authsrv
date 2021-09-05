FROM rust:1.52 as builder

RUN USER=root cargo new --bin authsrv
WORKDIR ./authsrv
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/authsrv*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/authsrv

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 postgresql redis \
    && rm -rf /var/lib/apt/lists/*

# XXX gen RANDPW
# openssl rand -hex 8
ENV TZ=Etc/UTC \
    APP_USER=appuser \
    DB=oauth \
    DBUSER=oauthuser \
    DBPW=be0a2cb343921d16

EXPOSE 8000

ENV DATABASE_URL=postgres://$DBUSER:$DBPW@localhost/$DB \
    ROCKET_DATABASES={diesel={url=postgres://$DBUSER:$DBPW@localhost/$DB}} \
    ROCKET_ADDRESS=0.0.0.0

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

RUN /etc/init.d/postgresql start \
    && /etc/init.d/redis-server start \
    && su -c "createuser oauthuser" postgres \
    && su -c "createdb oauth" postgres \
    && echo "alter user $DBUSER with encrypted password '$DBPW';" |su -c psql postgres \
    && echo "grant all privileges on database $DB to $DBUSER ;" | su -c psql postgres

COPY ./migrations ./migrations/
COPY --from=builder /authsrv/target/release/authsrv ${APP}/authsrv
COPY --from=builder /authsrv/Rocket.toml ${APP}/Rocket.toml
COPY --from=builder /authsrv/docker/start.sh /bin/start.sh

RUN chown -R $APP_USER:$APP_USER ${APP}

#USER $APP_USER
#WORKDIR ${APP}
#CMD ["./authsrv"]

USER root
WORKDIR ${APP}
CMD ["/bin/start.sh"]
