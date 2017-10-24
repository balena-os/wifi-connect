FROM debian:jessie

RUN apt-get update && \
    apt-get install  -y --no-install-recommends \
    ca-certificates \
    file \
    curl \
    git \
    openssh-client \
    gcc \
    g++ \
    libc6-dev \
    pkg-config \
    libdbus-1-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


RUN ln -s /usr/bin/strip /usr/bin/cross-strip


# Install jq

RUN JQ_URL="https://circle-downloads.s3.amazonaws.com/circleci-images/cache/linux-amd64/jq-latest" \
    && curl --silent --show-error --location --fail --retry 3 --output /usr/bin/jq $JQ_URL \
    && chmod +x /usr/bin/jq \
    && jq --version


# Setup Rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=/root/.cargo/bin:$PATH


VOLUME /work

WORKDIR /work
