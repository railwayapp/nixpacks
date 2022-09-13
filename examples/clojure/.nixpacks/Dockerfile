FROM ghcr.io/railwayapp/nixpacks:debian-1663027579

ENTRYPOINT ["/bin/bash", "-l", "-c"]
WORKDIR /app/



# setup phase
COPY .nixpacks/setup.nix .nixpacks/setup.nix
RUN nix-env -if .nixpacks/setup.nix && nix-collect-garbage -d

# load variables
ARG NIXPACKS_METADATA
ENV NIXPACKS_METADATA=$NIXPACKS_METADATA

# build phase
COPY . /app/
RUN  lein uberjar; if [ -f /app/target/uberjar/*standalone.jar ]; then  mv /app/target/uberjar/*standalone.jar /app/target/*standalone.jar; fi



# start
CMD ["bash -c \"java $JAVA_OPTS -jar /app/target/*standalone.jar\""]

