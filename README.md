# Twilight Relayer Data API

The **Relayer Data API** is a read-only JSON-RPC service that provides structured access to real-time and historical data from the Twilight Relayer.

This API layer is optimized for querying:
- Open and historical orders
- Account balances and margin state
- Price feeds and funding rates
- Realized and unrealized PnL

It is designed to operate independently from the order submission system (`relayer-order-api`), ensuring uninterrupted access to trading data even under high load. This separation supports scalable data consumption by external clients such as trading dashboards, explorers, analytics engines, and monitoring tools.

> ⚠️ This API does not support transaction submission or state changes. For placing or canceling orders, use the [Relayer Order API](https://github.com/twilight-project/relayer-order-api).

For subscriptions and rpcs, see [here](./docs/API.md)

## install diesel_cli (Only necessary for development)

```command
sudo apt install libpq-dev
cargo install diesel_cli --no-default-features --features postgres
```

## running with docker-compose

Build the container, you will need an ssh-key with read-only access to the twilight-relayer repo for this:

```console
DOCKER_BUILDKIT=1 docker build . -t relayer-api --build-arg SSH_KEY="$(cat ../.ssh/build_key)"
docker-compose -f twilight-relayerAPI/docker-compose.yaml up -d
```

# Running outside of docker environment

## running a local postgres container

`docker run --rm -ePOSTGRES_DATABASE=relayer -ePOSTGRES_USER=relayer -ePOSTGRES_PASSWORD=relayer -p 5432:5432 postgres`

## Initialize the database

Check the .env file for the DATABASE_URL environment variable, be sure it's set up to point at the postgres docker instance.

## Run the archiver

`cargo r --release --bin archiver`

## Run the api server

`cargo r --release --bin api`

## Run the auth server

`cargo r --release --bin auth`

## Testing

Tests are using uuid features that require additional compiler flags:

```command
./init_test_db.sh
cargo test
```
