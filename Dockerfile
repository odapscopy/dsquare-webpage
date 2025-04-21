# FROM rust:1.71.0-slim AS builder
FROM rust:1.86.0-slim AS builder

WORKDIR /usr/src

# Create blank project. Remember to change the project's name to match the the name of your member-crate if you don't like the name `axum_postgres_docker`
RUN USER=root cargo new dsquare_webpage_docker

# We want dependencies cached, so copy those (Cargo.toml, Cargo.lock) first
## PLEASE NOTE: I did copied 'Cargo.lock' because this particular project is not a workspace member, and has its own 'Cargo.lock'
COPY Cargo.toml /usr/src/dsquare_webpage_docker/
COPY Cargo.lock /usr/src/dsquare_webpage_docker/

# Set the working directory
WORKDIR /usr/src/dsquare_webpage_docker

# Install 'musl-tools' to enable successful image build
RUN apt-get -y update 
RUN apt-get -y upgrade
RUN apt-get -y install musl-tools

# Verify the musl-tools installation is correctly set up
RUN musl-gcc --version

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the src code
COPY src /usr/src/dsquare_webpage_docker/src/

# Copy 'assets', 'pages', 'styles' directory, and 'index.html' file too
COPY assets /usr/src/dsquare_webpage_docker/assets
COPY pages /usr/src/dsquare_webpage_docker/pages
COPY styles /usr/src/dsquare_webpage_docker/styles
COPY index.html /usr/src/dsquare_webpage_docker

## Touch 'main.rs' to prevent cached release build
RUN touch /usr/src/dsquare_webpage_docker/src/main.rs

RUN cargo clean
# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release --verbose
RUN ls -l /usr/src/dsquare_webpage_docker/target/x86_64-unknown-linux-musl/release/

##### Runtime
FROM alpine:3.16.0 AS runtime

# Copy application binary from image 'builder'
COPY --from=builder /usr/src/dsquare_webpage_docker/target/x86_64-unknown-linux-musl/release/axum-webserver /usr/local/bin

EXPOSE 8090

# Run the application via 'dsquare_webpage_docker.exe'
CMD ["/usr/local/bin/dsquare_webpage_docker"]