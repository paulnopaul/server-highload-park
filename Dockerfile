FROM rust:1.31

WORKDIR /usr/src/server-highload-park
COPY . .

RUN cargo install --path .

CMD ["server-highload-park"]
