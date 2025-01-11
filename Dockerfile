# will only work fine if you're compiling using the same distro
FROM docker.io/ubuntu:24.04

RUN apt update
RUN apt install -y ffmpeg
RUN mkdir /roms

COPY target/release/orca-bot .
COPY contrib/orca.rom /roms

ENV LOG=debug

CMD ["./orca-bot", "/roms/orca.rom"]
