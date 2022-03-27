FROM nixos/nix

RUN nix-channel --update

COPY . /app
WORKDIR /app

# Install
RUN nix-shell environment.nix --run 'npm install'

# Build
RUN nix-shell environment.nix --run 'npm run build'

# Start
CMD nix-shell environment.nix --run 'npm run start'
