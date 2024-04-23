FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

WORKDIR /workspace/sbio-wrapper
COPY ./ /workspace/sbio-wrapper

RUN apt-get update && apt-get install -y python3-pip
RUN pip install pre-commit
RUN apt-get -y install curl gnupg
RUN curl -sL https://deb.nodesource.com/setup_20.x  | bash -
RUN apt-get install -y nodejs
RUN npm install @commitlint/cli @commitlint/config-conventional --save-dev
RUN npm install --save-dev semantic-release
