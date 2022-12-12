# Database setup

## install diesel_cli (Only necessary for development)
```command
sudo apt install libpq-dev
cargo install diesel_cli --no-default-features --features postgres
```

## running a local postgres container

`docker run --rm -ePOSTGRES_DATABASE=relayer -ePOSTGRES_USER=relayer -ePOSTGRES_PASSWORD=relayer -p 5432:5432 postgres`
