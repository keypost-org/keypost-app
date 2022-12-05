FROM rustlang/rust:nightly AS builder
WORKDIR /usr/src/github.com/keypost-org/keypost-app/
COPY src/ src/
COPY static/ static/
COPY Cargo.toml .
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app
RUN apt-get update \
  && apt-get -y install ca-certificates libssl-dev libpq-dev
RUN mkdir -p ${APP}
COPY --from=builder /usr/src/github.com/keypost-org/keypost-app/target/release/keypost-app ${APP}/keypost-app
WORKDIR ${APP}
EXPOSE 8000
CMD ["./keypost-app"]
