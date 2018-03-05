# Use an official Python runtime as a parent image
FROM nginx

# Define mountable directories.
#VOLUME ["/etc/nginx/sites-enabled", "/etc/nginx/certs", "/etc/nginx/conf.d", "/var/log/nginx", "/var/www/html"]

# Copy the current directory contents into the container at /app
ADD . /app

# Define working directory.
WORKDIR /app

# RUN rm /etc/nginx/sites-enabled/default || true

ADD nginx.conf /etc/nginx/nginx.conf

# Expose ports.
EXPOSE 80

RUN mkdir -p mercutio/log; mkdir -p log;

RUN apt-get update; apt-get install sqlite3 libsqlite3-dev -y

# Run app.py when the container launches
CMD service nginx restart; RUST_BACKTRACE=1 MERCUTIO_SYNC_LOG=0 ./mercutio-sync
