FROM rust:1.49 as build
WORKDIR /app
ADD Cargo.toml Cargo.toml /app/
ADD src /app/src/
RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /app/target/release/vimhelp /usr/bin/vimhelpbot
COPY vimtags /usr/share/vimtags
COPY nvimtags /usr/share/nvimtags
ENV VIM_DB_PATH=/usr/share/vimtags
ENV NEOVIM_DB_PATH=/usr/share/nvimtags
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
CMD ["/usr/bin/vimhelpbot"]
