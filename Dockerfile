FROM resin/armv7hf-node:0.12.2

RUN apt-get update && apt-get install -y \
	dropbear \
	usbutils\
	wireless-tools\
	sudo \
	net-tools \
	iptables \
	libdbus-1-dev \
	libexpat-dev \
	nano \
	connman \
	bridge-utils \
	bind9

COPY ./assets/bind /etc/bind

RUN mkdir -p /app/src
COPY ./src/package.json /app/src/
RUN cd /app/src && JOBS=MAX npm install --unsafe-perm --production && npm cache clean

COPY . /app

RUN chmod a+x /app/start

VOLUME /var/lib/connman

CMD cd /app && ./start
