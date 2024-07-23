FROM rust:1.79-slim-buster as build

RUN apt-get update && apt-get install -y python3 python3-pip pkg-config libssl-dev
RUN USER=root cargo new --bin watchman
WORKDIR /watchman

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/watchman*
RUN cargo build --release

FROM rust:1.79-slim-buster
RUN apt-get update && apt-get install -y python3 python3-pip pkg-config libssl-dev
RUN pip install
COPY --from=build /watchman/target/release/watchman .

CMD ["./watchman"]