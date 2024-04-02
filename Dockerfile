FROM rust:1.77.0-slim-buster as builder

ARG SSH_KEY
RUN apt-get update && apt-get install -y openssh-client git libssl-dev build-essential libpq-dev pkg-config
RUN mkdir /root/.ssh
RUN echo "${SSH_KEY}" > /root/.ssh/id_rsa && \
    touch /root/.ssh/known_hosts && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts && \
    chmod 0600 /root/.ssh/id_rsa

COPY . ./twilight-relayerAPI

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=${PWD}/target \
    cd ./twilight-relayerAPI && \
    git config --global url."ssh://git@github.com/".insteadOf ssh://github.com/ && \
    cargo --config "net.git-fetch-with-cli = true" b --release --bins

FROM debian:10-slim
RUN apt-get update && apt-get install -y ca-certificates curl libpq-dev libssl-dev

WORKDIR /app
COPY --from=builder ./twilight-relayerAPI/target/release/api ./
COPY --from=builder ./twilight-relayerAPI/target/release/archiver ./
COPY --from=builder ./twilight-relayerAPI/target/release/auth ./
COPY ./scripts/run.sh ./
COPY ./.compose.env .env


ENTRYPOINT ["/app/run.sh"]

CMD ["archiver"]
