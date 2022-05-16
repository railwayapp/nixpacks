FROM debian:bullseye-20220509-slim

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get -y upgrade \
  && apt-get install --no-install-recommends -y sudo locales curl xz-utils ca-certificates openssl build-essential \
  && apt-get clean && rm -rf /var/lib/apt/lists/* \
  && mkdir -m 0755 /nix && mkdir -m 0755 /etc/nix && groupadd -r nixbld && chown root /nix \
  && printf 'sandbox = false \nfilter-syscalls = false' > /etc/nix/nix.conf \
  && for n in $(seq 1 10); do useradd -c "Nix build user $n" -d /var/empty -g nixbld -G nixbld -M -N -r -s "$(command -v nologin)" "nixbld$n"; done

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN set -o pipefail && curl -L https://nixos.org/nix/install | bash \
    && /nix/var/nix/profiles/default/bin/nix-collect-garbage --delete-old

ENV \
  ENV=/etc/profile \
  USER=root \
  PATH=/nix/var/nix/profiles/default/bin:/nix/var/nix/profiles/default/sbin:/bin:/sbin:/usr/bin:/usr/sbin \
  GIT_SSL_CAINFO=/etc/ssl/certs/ca-certificates.crt \
  NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
  NIX_PATH=/nix/var/nix/profiles/per-user/root/channels

RUN nix-channel --update
