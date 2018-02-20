# Use an official Python runtime as a parent image
FROM rust

# Install any needed packages specified in requirements.txt
RUN rustup override set nightly-2018-02-10

# Install Nginx.
RUN \
  apt-get update && \
  apt-get install -y nginx && \
  rm -rf /var/lib/apt/lists/* && \
  echo "\ndaemon off;" >> /etc/nginx/nginx.conf && \
  chown -R www-data:www-data /var/lib/nginx

# Define mountable directories.
#VOLUME ["/etc/nginx/sites-enabled", "/etc/nginx/certs", "/etc/nginx/conf.d", "/var/log/nginx", "/var/www/html"]

# Define working directory.
WORKDIR /app

RUN rm /etc/nginx/sites-enabled/default

ADD nginx.conf /etc/nginx/nginx.conf

# Copy the current directory contents into the container at /app
ADD . /app

# Expose ports.
EXPOSE 80

# Define environment variable
# ENV NAME World

RUN ["make", "mercutio-sync-build"]

# Run app.py when the container launches
CMD service nginx restart; make mercutio-sync-nolog

#EXPOSE 8000