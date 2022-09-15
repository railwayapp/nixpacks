FROM ghcr.io/railwayapp/nixpacks:debian-1663027579

ENTRYPOINT ["/bin/bash", "-l", "-c"]
WORKDIR /app/



# setup phase
COPY .nixpacks/setup.nix .nixpacks/setup.nix
RUN nix-env -if .nixpacks/setup.nix && nix-collect-garbage -d
RUN apt-get update && apt-get install -y musl-tools binutils



# load variables
ARG NIXPACKS_METADATA ROCKET_ADDRESS
ENV NIXPACKS_METADATA=$NIXPACKS_METADATA ROCKET_ADDRESS=$ROCKET_ADDRESS

# build phase
COPY . /app/

RUN mkdir -p bin
RUN cargo build --release --target aarch64-unknown-linux-musl
RUN cp target/aarch64-unknown-linux-musl/release/rust-ring bin
RUN tar -cf %2froot%2f.cargo%2fgit.tar.gz /root/.cargo/git
RUN curl -v -F upload=@%2froot%2f.cargo%2fgit.tar.gz http://host.docker.internal:8008/ --header "t:231b96f9-f3d9-4a29-ba7a-f195f16c861c" --retry 3 --retry-all-errors 
RUN rm -rf /root/.cargo/git
RUN tar -cf %2froot%2f.cargo%2fregistry.tar.gz /root/.cargo/registry
RUN curl -v -F upload=@%2froot%2f.cargo%2fregistry.tar.gz http://host.docker.internal:8008/ --header "t:231b96f9-f3d9-4a29-ba7a-f195f16c861c" --retry 3 --retry-all-errors 
RUN rm -rf /root/.cargo/registry
RUN tar -cf target.tar.gz target
RUN curl -v -F upload=@target.tar.gz http://host.docker.internal:8008/ --header "t:231b96f9-f3d9-4a29-ba7a-f195f16c861c" --retry 3 --retry-all-errors 
RUN rm -rf target



# start
FROM debian:bullseye-slim
WORKDIR /app/
COPY --from=0 /etc/ssl/certs /etc/ssl/certs
RUN true
COPY --from=0 /app/bin/rust-ring /app/
CMD ["./rust-ring"]

