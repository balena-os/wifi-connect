#!/bin/bash

set -ev

if [ -z "$1" ]; then
	printf 'Rust compilation target not specified'
	exit 1
fi

TARGET=$1

cross() {
    docker run -it --rm -v $PWD:/work $TARGET "$@"
}

docker build -t $TARGET scripts/docker/$TARGET

cross cargo build --release --target=$TARGET

cross cross-strip target/$TARGET/release/resin-wifi-connect
