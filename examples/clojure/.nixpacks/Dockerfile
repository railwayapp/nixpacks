FROM ghcr.io/railwayapp/nixpacks:debian-1665446966

ENTRYPOINT ["/bin/bash", "-l", "-c"]
WORKDIR /app/

COPY .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix
RUN nix-env -if .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix && nix-collect-garbage -d


ARG NIXPACKS_METADATA
ENV NIXPACKS_METADATA=$NIXPACKS_METADATA

# setup phase
COPY . /app/

# build phase
COPY . /app/
RUN  lein uberjar; if [ -f /app/target/uberjar/*standalone.jar ]; then  mv /app/target/uberjar/*standalone.jar /app/target/*standalone.jar; fi


# start
CMD ["bash -c \"java $JAVA_OPTS -jar /app/target/*standalone.jar\""]

