FROM resin/armv7hf-node:0.12.2

RUN apt-get update && apt-get install -y \
	dropbear \
	usbutils\
	wireless-tools\
	sudo \
	net-tools \
	iptables \
	libdbus-1-dev \
	libexpat-dev

COPY . /app

RUN chmod a+x /app/start

#RUN cd /app/src && npm install

EXPOSE 8080
VOLUME /var/lib/connman:/var/lib/connman

CMD cd /app && ./start
