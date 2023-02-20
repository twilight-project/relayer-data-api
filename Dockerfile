FROM rust:1.67-alpine as builder

ARG SSH_KEY
RUN apk update && apk add alpine-sdk openssh git openssl-dev libpq-dev
RUN mkdir /root/.ssh
RUN echo "${SSH_KEY}" > /root/.ssh/id_rsa && \
    touch /root/.ssh/known_hosts && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts && \
    chmod 0600 /root/.ssh/id_rsa

COPY . ./twilight-relayerAPI

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=${PWD}/target \
    cd ./twilight-relayerAPI && \
    cargo --config "net.git-fetch-with-cli = true" b --release --bins

FROM alpine:3.17
RUN apk add --no-cache ca-certificates curl libpq-dev openssh
#RUN curl -LO http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb && \
#     DEBIAN_FRONTEND=noninteractive dpkg -i ./libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb

WORKDIR /app
COPY --from=builder ./twilight-relayerAPI/target/release/api ./
COPY --from=builder ./twilight-relayerAPI/target/release/archiver ./
COPY ./scripts/run.sh ./


ENTRYPOINT ["/app/run.sh"]

CMD ["archiver"]
