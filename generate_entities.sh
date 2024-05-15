#!/bin/sh
set -e

cleanup() {
    echo "=> Stopping Postgres database"
    docker stop db-gen
}

trap cleanup EXIT

echo "=> Installing sea-orm-cli"
cargo install sea-orm-cli

echo "=> Starting Postgres database"
docker run --rm --name db-gen -p 8999:5432 -e POSTGRES_PASSWORD=password -d postgres

echo "=> Generating schema"
cargo run -- postgresql://postgres:password@localhost:8999 --generate-schema

echo "=> Generating entities"
sea-orm-cli generate entity \
    -u postgresql://postgres:password@localhost:8999/mainnet \
    -o src/entities
