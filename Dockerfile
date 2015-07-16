FROM resin/armv7hf-node:0.12.2

RUN apt-get update && apt-get install -y \
	bind9 \
	bridge-utils \
	connman \
	dropbear \
	iptables \
	libdbus-1-dev \
	libexpat-dev \
	nano \
	net-tools \
	sudo \
	usbutils \
	wireless-tools	

COPY ./assets/bind /etc/bind

RUN mkdir -p /app/src
COPY ./src/package.json /app/src/
RUN cd /app/src && JOBS=MAX npm install --unsafe-perm --production && npm cache clean

COPY . /app

RUN chmod a+x /app/start

VOLUME /var/lib/connman

CMD cd /app && ./start
