FROM debian:jessie

ARG CROSS_TRIPLE=arm-linux-gnueabihf

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    file \
    curl \
    git \
    openssh-client \
    gcc \
    libc6-dev \
    pkg-config

RUN printf 'deb http://emdebian.org/tools/debian/ jessie main' >> /etc/apt/sources.list

RUN curl http://emdebian.org/tools/debian/emdebian-toolchain-archive.key | apt-key add -

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc-${CROSS_TRIPLE} \
    g++-${CROSS_TRIPLE} \
    libdbus-1-dev:armhf && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN ln -s /usr/bin/${CROSS_TRIPLE}-strip /usr/bin/cross-strip


# Install jq

RUN JQ_URL="https://circle-downloads.s3.amazonaws.com/circleci-images/cache/linux-amd64/jq-latest" \
    && curl --silent --show-error --location --fail --retry 3 --output /usr/bin/jq $JQ_URL \
    && chmod +x /usr/bin/jq \
    && jq --version


# Setup Rust for ARMv7 cross-compilation

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=/root/.cargo/bin:$PATH

RUN rustup target add armv7-unknown-linux-gnueabihf

ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_LIBDIR=/usr/lib/${CROSS_TRIPLE}/pkgconfig/ \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc


VOLUME /work

WORKDIR /work
