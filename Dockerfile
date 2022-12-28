FROM rustlang/rust:nightly AS builder
WORKDIR /usr/src/github.com/keypost-org/keypost-app/
COPY src/ src/
COPY static/ static/
COPY migrations/ migrations/
COPY scripts/deploy/ deploy/
COPY Cargo.toml .
RUN cargo build --release
RUN cargo install diesel_cli --no-default-features --features postgres --verbose

FROM debian:buster-slim
ARG APP=/usr/src/app
RUN apt-get update \
  && apt-get -y install ca-certificates libssl-dev libpq-dev
RUN mkdir -p ${APP} && mkdir -p ${APP}/migrations
COPY --from=builder /usr/src/github.com/keypost-org/keypost-app/target/release/keypost-app ${APP}/keypost-app
COPY --from=builder /usr/src/github.com/keypost-org/keypost-app/migrations/ ${APP}/migrations/
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /usr/src/github.com/keypost-org/keypost-app/deploy/start.sh ${APP}/deploy-start.sh
WORKDIR ${APP}
EXPOSE 8000
CMD ["./deploy-start.sh"]
