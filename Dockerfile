FROM resin/rpi-node

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

RUN cd /app/src && npm install

EXPOSE 8080

CMD cd /app && ./start
