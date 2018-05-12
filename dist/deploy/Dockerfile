# Dockerfile for running the edit-server

FROM nginx

RUN rm /etc/nginx/conf.d/default.conf || true
RUN rm /etc/nginx/conf.d/examplessl.conf || true

RUN apt-get update; apt-get install sqlite3 libsqlite3-dev -y

ADD nginx.conf /etc/nginx/nginx.conf
ADD . /app

WORKDIR /app
EXPOSE 80

RUN mkdir -p edit/log; mkdir -p log;

CMD service nginx restart; RUST_BACKTRACE=1 MERCUTIO_SYNC_LOG=0 ./edit-server
