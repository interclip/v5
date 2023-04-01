#!/bin/sh

# Start MySQL in the background
docker-entrypoint.sh mysqld &

# Wait for MySQL to start
until mysqladmin ping -h127.0.0.1 -P3306 --silent; do
  echo "Waiting for MySQL to start..."
  sleep 1
done

# Start the interclip-server
interclip-server
