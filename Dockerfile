FROM rust:1.87.0 as builder
RUN USER=root apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install git curl g++ build-essential libssl-dev pkg-config && \
    apt-get -y install software-properties-common protobuf-compiler && \
    apt-get update 

# RUN git clone -b v0.1.0 https://github.com/twilight-project/relayer-data-api.git
COPY . /relayer-data-api
WORKDIR /relayer-data-api
RUN cargo build --release --bins    

FROM rust
RUN apt-get update && apt-get install -y ca-certificates curl libpq-dev libssl-dev postgresql-client redis-tools netcat-openbsd procps
EXPOSE 5000
EXPOSE 8989
EXPOSE 8990
EXPOSE 8987

WORKDIR /app

ARG BINARY
COPY --from=builder ./relayer-data-api/target/release/${BINARY} ./main
COPY --from=builder ./relayer-data-api/target/release/${BINARY}.d ./main.d
# COPY ./healthcheck.sh /usr/local/bin/healthcheck.sh
# RUN chmod +x /usr/local/bin/healthcheck.sh
ENTRYPOINT ["/app/main"]