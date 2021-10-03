FROM rust:1.31

WORKDIR /usr/src/server-highload-park
COPY ./src ./src
COPY ./static ./static
COPY ./Cargo.toml ./

RUN ls
RUN cargo build --release

CMD ["./target/release/server-highload-park"]
