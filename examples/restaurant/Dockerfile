## Rust container

FROM alpine:3.14 as CARGO_INSTALL

RUN apk add tree curl build-base
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y

## Build container

FROM CARGO_INSTALL

RUN apk add protoc
RUN mkdir app
ADD Cargo.toml app/
ADD proto app/proto
ADD src app/src
WORKDIR /app/
RUN export PROTOC=/usr/bin/protoc && \
  export PROTOC_INCLUDE=/usr/include && \
  source $HOME/.cargo/env && \
  cargo build --release

## Application container

FROM alpine:3.14
COPY --from=1 /app/target/release/restaurant .
EXPOSE 31600
ENTRYPOINT [ "./restaurant" ]
