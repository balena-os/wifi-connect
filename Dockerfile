FROM resin/rpi-node

RUN apt-get update && apt-get install -y dropbear usbutils wireless-tools sudo

COPY . /app

RUN chmod a+x /app/start

RUN cd /app/src && npm install

EXPOSE 8080

CMD cd /app && ./start
