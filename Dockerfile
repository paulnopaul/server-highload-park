FROM rust:1.55.0

WORKDIR /usr/src/server-highload-park
COPY ./src ./src
COPY ./httptest ./httptest
COPY ./Cargo.toml ./

RUN ls
RUN cargo build --release
RUN ulimit -n 3000

CMD ["./target/release/server-highload-park"]
