FROM docker.io/rust:1.81 AS build

# empty project
RUN USER=root cargo new --bin orca-bot
WORKDIR /orca-bot

# copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# dependencies
COPY ./contrib ./contrib

# cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source tree
COPY ./src ./src

# build release version
RUN rm -f ./target/release/orca-bot ./target/release/deps/orca_bot-*
RUN cargo build --release

# stage 2 - actual image
FROM docker.io/ubuntu:24.04

ARG user=1000

RUN apt update
RUN apt install -y ffmpeg
RUN mkdir /roms
RUN mkdir /app
VOLUME /log

COPY --from=build /orca-bot/target/release/orca-bot /app
COPY contrib/orca.rom /roms

RUN chown -R $user:$user /app
RUN chown -R $user:$user /roms

USER $user

CMD ["/app/orca-bot", "run", "/roms/orca.rom", "--history-file", "/log/history.csv"]
