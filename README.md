# Database setup

## install diesel_cli (Only necessary for development)
```command
sudo apt install libpq-dev
cargo install diesel_cli --no-default-features --features postgres
```

## running a local postgres container

`docker run --rm -ePOSTGRES_DATABASE=relayer -ePOSTGRES_USER=relayer -ePOSTGRES_PASSWORD=relayer -p 5432:5432 postgres`

## Initialize the database

Check the .env file for the DATABASE_URL environment variable, be sure it's set up to point at the postgres docker instance.

`diesel migration run`

## Run the archiver

`cargo r --release --bin archiver`

## Testing
Tests are using uuid features that require additional compiler flags:

```command
./init_test_db.sh
cargo test
```
