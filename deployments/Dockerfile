FROM rust:1.77-slim-buster as build

RUN USER=root cargo new --bin discord
WORKDIR /watchman

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/discord*
RUN cargo build --release

FROM rust:1.77-slim-buster

COPY --from=build /watchman/target/release/watchman .

CMD ["./watchman"]