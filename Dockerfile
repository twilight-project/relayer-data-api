FROM rust:latest as builder

ARG SSH_KEY
RUN mkdir /root/.ssh
RUN echo "${SSH_KEY}" > /root/.ssh/id_rsa && \
    touch /root/.ssh/known_hosts && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts && \
    chmod 0600 /root/.ssh/id_rsa

#COPY ./twilight-relayer ./twilight-relayer
COPY . ./twilight-relayerAPI

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=${PWD}/target \
    cd ./twilight-relayerAPI && \
    cargo --config "net.git-fetch-with-cli = true" b --release --bins

FROM ubuntu:latest
RUN apt-get update && apt-get install -y ca-certificates libssl-dev curl libpq-dev
RUN curl -LO http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb && \
     DEBIAN_FRONTEND=noninteractive dpkg -i ./libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb

WORKDIR /app
COPY --from=builder ./twilight-relayerAPI/target/release/api ./
COPY --from=builder ./twilight-relayerAPI/target/release/archiver ./
COPY ./scripts/run.sh ./


ENTRYPOINT ["/app/run.sh"]

CMD ["archiver"]
