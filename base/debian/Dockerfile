FROM debian:bullseye-20220622-slim

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get -y upgrade \
  && apt-get install --no-install-recommends -y sudo locales curl xz-utils ca-certificates openssl make git \
  && apt-get clean && rm -rf /var/lib/apt/lists/* \
  && mkdir -m 0755 /nix && mkdir -m 0755 /etc/nix && groupadd -r nixbld && chown root /nix \
  && printf 'sandbox = false \nfilter-syscalls = false' > /etc/nix/nix.conf \
  && for n in $(seq 1 10); do useradd -c "Nix build user $n" -d /var/empty -g nixbld -G nixbld -M -N -r -s "$(command -v nologin)" "nixbld$n"; done

SHELL ["/bin/bash", "-ol", "pipefail", "-c"]
RUN set -o pipefail && curl -L https://nixos.org/nix/install | bash \
    && /nix/var/nix/profiles/default/bin/nix-collect-garbage --delete-old \
    && printf 'if [ -d $HOME/.nix-profile/etc/profile.d ]; then\n for i in $HOME/.nix-profile/etc/profile.d/*.sh; do\n if [ -r $i ]; then\n . $i\n fi\n done\n fi\n' >> /root/.profile

ENV \
  ENV=/etc/profile \
  USER=root \
  PATH=/nix/var/nix/profiles/default/bin:/nix/var/nix/profiles/default/sbin:/bin:/sbin:/usr/bin:/usr/sbin \
  GIT_SSL_CAINFO=/etc/ssl/certs/ca-certificates.crt \
  NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
  NIX_PATH=/nix/var/nix/profiles/per-user/root/channels \
  NIXPKGS_ALLOW_BROKEN=1 \
  LD_LIBRARY_PATH=/usr/lib \
  CPATH=~/.nix-profile/include:$CPATH \
  LIBRARY_PATH=~/.nix-profile/lib:$LIBRARY_PATH \
  QTDIR=~/.nix-profile:$QTDIR

RUN nix-channel --update
