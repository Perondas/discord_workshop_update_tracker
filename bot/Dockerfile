FROM rust:latest

WORKDIR /usr/src/bot
COPY . .

RUN cargo install --path .

CMD ["bot"]