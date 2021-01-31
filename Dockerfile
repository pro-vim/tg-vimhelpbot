FROM rust:1.49 as build
WORKDIR /app
ADD Cargo.toml Cargo.toml /app/
ADD src /app/src/
RUN cargo build --release

FROM debian:buster-slim AS base-debian
RUN apt-get update && apt-get install -y openssl ca-certificates

FROM base-debian AS build-nvim-tags
RUN apt-get install -y \
	git ninja-build gettext libtool libtool-bin autoconf automake cmake g++ pkg-config unzip
ADD nvim-hash /
RUN git clone https://github.com/neovim/neovim /neovim
WORKDIR /neovim
RUN git checkout "$(cat /nvim-hash)"
RUN make helptags

FROM base-debian AS build-vim-tags
RUN apt-get install -y git
ADD vim-hash /
RUN git clone https://github.com/vim/vim /vim
WORKDIR /vim
RUN git checkout "$(cat /vim-hash)"

FROM base-debian
COPY --from=build /app/target/release/vimhelp /usr/bin/vimhelpbot
COPY --from=build-vim-tags /vim/runtime/doc/tags /usr/share/vimtags
COPY --from=build-nvim-tags /neovim/build/runtime/doc/tags /usr/share/nvimtags
ENV VIM_DB_PATH=/usr/share/vimtags
ENV NEOVIM_DB_PATH=/usr/share/nvimtags
RUN rm -rf /var/lib/apt/lists/*
CMD ["/usr/bin/vimhelpbot"]
