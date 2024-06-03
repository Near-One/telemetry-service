# Telemetry service

An application to collect NEAR clients telemetry data and store it in a relational database.

## Endpoints
Default port: `8080`

- `/nodes/mainnet`: POST node telemetry v1+
- `/nodes/testnet`: POST node telemetry v1+
- `/nodes`: POST node telemetry v2+
- `/metrics`: Prometheus metrics
- `/healthz`: health check

## Telemetry versions

- `v1`: legacy telemetry format
- `v2`: new telemetry format post [#11444](https://github.com/near/nearcore/pull/11444)

## Development

### Requirements
- docker
- cargo

### Connect to a local database

Run a local database: `docker run --rm --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=password -d postgres`

Database connection URL: `postgresql://postgres:password@localhost:5432`

### Regenerate entities definition

Required when introducing schema changes (through a migration!).

`./generate_entities.sh`

### CI
The CI checks that the project compiles successfully at every commit. Docker images are pushed to the registry only by tagged builds.

## Usage
Run locally with `cargo` or build and run as a docker image:
```
docker build -t telemetry-service .

docker run --rm -p 8080:8080 telemetry-service postgresql://postgres:password@host.docker.internal:5432
```

Alternatively, you can start both the database and the service through docker compose:
```
docker-compose up
```

### Logs
Logs are printed to `stdout`. Log level can be controlled through the environment variable `RUST_LOG`.
