FROM ubuntu:noble

ARG DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC

RUN apt-get update && apt-get -y upgrade \
  && apt-get install --no-install-recommends -y sudo locales curl tzdata xz-utils ca-certificates openssl make git pkg-config \
  && apt-get clean && rm -rf /var/lib/apt/lists/* \
  && rm -rf /usr/share/doc/* \
  && mkdir -m 0755 /nix && mkdir -m 0755 /etc/nix

SHELL ["/bin/bash", "-ol", "pipefail", "-c"]
RUN set -o pipefail && curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux --no-confirm --init none \
    --extra-conf "sandbox = false" \
    --extra-conf "filter-syscalls = false" \
    --extra-conf "experimental-features = nix-command flakes" \
    && /nix/var/nix/profiles/default/bin/nix-channel --remove nixpkgs \
    && /nix/var/nix/profiles/default/bin/nix-collect-garbage --delete-old

# Set up the shell profile to source Nix
RUN echo 'if [ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then' >> /root/.profile && \
    echo '  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' >> /root/.profile && \
    echo 'fi' >> /root/.profile

ENV \
  ENV=/etc/profile \
  USER=root \
  PATH=/nix/var/nix/profiles/default/bin:/nix/var/nix/profiles/default/sbin:/bin:/sbin:/usr/bin:/usr/sbin \
  GIT_SSL_CAINFO=/etc/ssl/certs/ca-certificates.crt \
  NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
  NIX_PATH=/nix/var/nix/profiles/per-user/root/channels \
  NIXPKGS_ALLOW_BROKEN=1 \
  NIXPKGS_ALLOW_UNFREE=1 \
  NIXPKGS_ALLOW_INSECURE=1 \
  LD_LIBRARY_PATH=/usr/lib
