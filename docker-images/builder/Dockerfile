FROM rust

RUN apt-get update
RUN apt install sudo
RUN sudo apt-get install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release -y

RUN sudo mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
RUN echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

RUN sudo apt-get update
RUN sudo apt-get install docker-ce-cli 
RUN curl -sSL https://nixpacks.com/install.sh | bash


COPY ./read-port.sh read-port.sh
