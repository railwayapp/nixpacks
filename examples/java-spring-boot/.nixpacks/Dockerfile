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
RUN --mount=type=cache,id=cvr3po0lGc-gradle,target=.gradle ./gradlew build -x check



# start
CMD ["java $JAVA_OPTS -jar -Dserver.port=$PORT build/libs/*.jar"]

