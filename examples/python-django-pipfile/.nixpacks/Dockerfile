FROM ghcr.io/railwayapp/nixpacks:debian-1668470745

ENTRYPOINT ["/bin/bash", "-l", "-c"]
WORKDIR /app/

COPY .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix
RUN nix-env -if .nixpacks/nixpkgs-a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.nix && nix-collect-garbage -d


ARG NIXPACKS_METADATA PYTHONUNBUFFERED
ENV NIXPACKS_METADATA=$NIXPACKS_METADATA PYTHONUNBUFFERED=$PYTHONUNBUFFERED

# setup phase
# noop

# install phase
ENV PATH /opt/venv/bin:$PATH
RUN printf '\nPATH=/opt/venv/bin:$PATH' >> /root/.profile
COPY . /app/
RUN --mount=type=cache,id=afv03YOBZfk-/root/cache/pip,target=/root/.cache/pip python -m venv /opt/venv && . /opt/venv/bin/activate && pip install --upgrade build setuptools && pipenv install --system --deploy


# start
COPY . /app
CMD ["python manage.py migrate && gunicorn mysite.wsgi"]

