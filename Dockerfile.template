FROM balenalib/%%RESIN_MACHINE_NAME%%-debian

RUN install_packages dnsmasq wireless-tools

WORKDIR /usr/src/app

RUN curl https://api.github.com/repos/balena-io/wifi-connect/releases/latest -s \
    | grep -hoP 'browser_download_url": "\K.*%%RESIN_ARCH%%\.tar\.gz' \
    | xargs -n1 curl -Ls \
    | tar -xvz -C /usr/src/app/

COPY scripts/start.sh .

CMD ["bash", "start.sh"]
