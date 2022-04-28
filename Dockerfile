FROM rust:1.60 as builder

WORKDIR /app
RUN cargo install --locked --branch master \
--git https://github.com/eeff/cargo-build-deps

COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod


FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
#cleanup
&& apt-get autoremove -y \
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]

