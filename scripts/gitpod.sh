#!/bin/bash

docker pull postgres
docker run --name pgdb -e POSTGRES_PASSWORD=development -p 5432:5432 -d postgres
docker exec -it pgdb psql -U postgres -c "CREATE DATABASE interclip;"

# connect to the db
# docker exec -it pgdb psql -U postgres
# \c interclip

diesel setup
diesel migration run

cp scripts/gitpod.env .env
