FROM rust:1.87 as builder

ARG SSH_KEY
RUN apt-get update && apt-get install -y openssh-client git libssl-dev build-essential libpq-dev pkg-config
RUN mkdir /root/.ssh
RUN echo "${SSH_KEY}" > /root/.ssh/id_ed25519 && \
    touch /root/.ssh/known_hosts && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts && \
    chmod 0600 /root/.ssh/id_ed25519
COPY . ./twilight-relayerAPI
WORKDIR /twilight-relayerAPI

RUN git config --global url."ssh://git@github.com/".insteadOf https://github.com/
RUN git config --global url."ssh://git@github.com/".insteadOf ssh://github.com/

RUN eval `ssh-agent -s` && ssh-add /root/.ssh/id_ed25519 && \
    git config --global url."ssh://git@github.com/".insteadOf https://github.com/ && \
    git config --global url."ssh://git@github.com/".insteadOf ssh://github.com/ && \
    git config --global url."ssh://git@github.com/twilight-project/twilight-relayer-sdk.git".insteadOf https://github.com/twilight-project/twilight-relayer-sdk.git && \
    cargo --config "net.git-fetch-with-cli = true" b --release --bins

FROM debian:10-slim
RUN apt-get update && apt-get install -y ca-certificates curl libpq-dev libssl-dev

WORKDIR /app
COPY --from=builder ./twilight-relayerAPI/target/release/api ./
COPY --from=builder ./twilight-relayerAPI/target/release/archiver ./
COPY --from=builder ./twilight-relayerAPI/target/release/auth ./
COPY ./scripts/run.sh ./
COPY ./.env ./.env


ENTRYPOINT ["/app/run.sh"]

CMD ["archiver"]
