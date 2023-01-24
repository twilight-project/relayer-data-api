# Twilight relayer API

## install diesel_cli (Only necessary for development)
```command
sudo apt install libpq-dev
cargo install diesel_cli --no-default-features --features postgres
```

## running with docker-compose
Build the container, you will need both twilight-relayer and twilight-relayerAPI at the same dir level for this:

```console
DOCKER_BUILDKIT=1 docker build . -t relayer-api -f twilight-relayerAPI/Dockerfile
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

## Testing
Tests are using uuid features that require additional compiler flags:

```command
./init_test_db.sh
cargo test
```
