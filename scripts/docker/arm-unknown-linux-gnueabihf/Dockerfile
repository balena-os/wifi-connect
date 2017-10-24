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
    pkg-config \
    xz-utils && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


# Setup the Raspberry Pi cross-compilation toolchain

ARG RPI_TOOLS=/root/rpi-tools

ARG SYSROOT=/root/sysroot

RUN git clone --depth 1 https://github.com/raspberrypi/tools.git $RPI_TOOLS

RUN ln -s $RPI_TOOLS/arm-bcm2708/arm-bcm2708hardfp-linux-gnueabi/arm-bcm2708hardfp-linux-gnueabi/sysroot $SYSROOT

ARG TOOLCHAIN=$RPI_TOOLS/arm-bcm2708/gcc-linaro-${CROSS_TRIPLE}-raspbian-x64/bin

RUN printf '#!/bin/bash\n\
arm-linux-gnueabihf-gcc --sysroot=/root/sysroot "$@"'\
>> $TOOLCHAIN/gcc-sysroot && \
chmod a+x $TOOLCHAIN/gcc-sysroot

RUN ln -s $TOOLCHAIN/${CROSS_TRIPLE}-strip $TOOLCHAIN/cross-strip

ENV PATH=$TOOLCHAIN:$PATH


# Install jq

RUN JQ_URL="https://circle-downloads.s3.amazonaws.com/circleci-images/cache/linux-amd64/jq-latest" \
    && curl --silent --show-error --location --fail --retry 3 --output /usr/bin/jq $JQ_URL \
    && chmod +x /usr/bin/jq \
    && jq --version


# Setup Rust for ARMv6 cross-compilation

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=/root/.cargo/bin:$PATH

RUN rustup target add arm-unknown-linux-gnueabihf

ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_LIBDIR=$SYSROOT/usr/lib/${CROSS_TRIPLE}/pkgconfig/ \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=gcc-sysroot \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_AR=${CROSS_TRIPLE}-gcc-ar


# Get the ARMv6 libdbus-1-dev package from Raspbian, extract it to sysroot and overwrite its pkg-config.

RUN cd $SYSROOT && \
    curl --remote-name http://mirrordirector.raspbian.org/raspbian/pool/main/d/dbus/libdbus-1-dev_1.8.22-0+deb8u1_armhf.deb && \
    ar p libdbus-1-dev_1.8.22-0+deb8u1_armhf.deb data.tar.xz | tar xJ

RUN printf 'prefix=/root/sysroot/usr\n\
libdir=${prefix}/lib/arm-linux-gnueabihf\n\
includedir=${prefix}/include\n\
Name: dbus\n\
Description: Free desktop message bus\n\
Version: 1.8.22\n\
Libs: -L${libdir} -ldbus-1\n\
Libs.private: -lpthread\n\
Cflags: -I${includedir}/dbus-1.0 -I${libdir}/dbus-1.0/include\n'\
> $SYSROOT/usr/lib/$CROSS_TRIPLE/pkgconfig/dbus-1.pc


VOLUME /work

WORKDIR /work
