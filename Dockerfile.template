ARG BALENA_ARCH=%%BALENA_ARCH%%

FROM balenalib/$BALENA_ARCH-debian
ARG BALENA_ARCH=%%BALENA_ARCH%%


RUN install_packages dnsmasq wireless-tools

# use latest version. If specific version is required, it should be provided as vX.Y.Z, e.g v4.11.37
ARG VERSION="latest"

WORKDIR /usr/src/app

RUN \
    export BASE_URL="https://github.com/balena-os/wifi-connect/releases" &&\    
    case $BALENA_ARCH in \
        "aarch64") \
            BINARY_ARCH_NAME="aarch64-unknown-linux-gnu" ;; \
        "amd64") \
            BINARY_ARCH_NAME="x86_64-unknown-linux-gnu" ;;\
        "armv7hf") \
            BINARY_ARCH_NAME="armv7-unknown-linux-gnueabihf" ;;\
        *) \
            echo >&2 "error: unsupported architecture ($BALENA_ARCH)"; exit 1 ;; \ 
    esac;\
    if [ ${VERSION} = "latest" ]; then \
        export URL_PARTIAL="latest/download" ; \
    else \
        export URL_PARTIAL="download/${VERSION}" ; \
    fi; \
    curl -Ls "$BASE_URL/$URL_PARTIAL/wifi-connect-$BINARY_ARCH_NAME.tar.gz" \
    | tar -xvz -C  /usr/src/app/

COPY scripts/start.sh .

CMD ["bash", "start.sh"]