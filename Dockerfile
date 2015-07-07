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

COPY ./assets/namedb /etc/namedb
COPY . /app

RUN chmod a+x /app/start

#RUN cd /app/src && npm install

VOLUME /var/lib/connman

CMD cd /app && ./start
