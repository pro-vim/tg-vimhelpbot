FROM rust:1.49 AS build
WORKDIR /app
ADD Cargo.toml Cargo.lock /app/
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
RUN apt-get install -y curl
ADD vim-hash /
RUN curl -fSsLo /vimtags "https://raw.githubusercontent.com/vim/vim/$(cat /vim-hash)/runtime/doc/tags"

FROM base-debian
COPY --from=build /app/target/release/vimhelp /usr/bin/vimhelpbot
COPY --from=build-vim-tags /vimtags /usr/share/vimtags
COPY --from=build-nvim-tags /neovim/build/runtime/doc/tags /usr/share/nvimtags
COPY customtags /usr/share/customtags
ENV VIM_DB_PATH=/usr/share/vimtags
ENV NEOVIM_DB_PATH=/usr/share/nvimtags
ENV CUSTOM_DB_PATH=/usr/share/customtags
RUN rm -rf /var/lib/apt/lists/*
CMD ["/usr/bin/vimhelpbot"]
