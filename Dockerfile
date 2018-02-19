# Use an official Python runtime as a parent image
FROM rust

# Install any needed packages specified in requirements.txt
RUN rustup override set nightly-2018-02-10

# Make port 80 available to the world outside this container
EXPOSE 8000
EXPOSE 8001
EXPOSE 8002

# Set the working directory to /app
WORKDIR /app

# Copy the current directory contents into the container at /app
ADD . /app

# Define environment variable
# ENV NAME World

RUN ["make", "mercutio-sync-build"]

# Run app.py when the container launches
CMD ["make", "mercutio-sync"]
