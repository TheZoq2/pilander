BUILD_TARGET=arm-unknown-linux-gnueabihf

IP_ADDRESS=192.168.0.104

upload:
	make build
	scp ./target/${BUILD_TARGET}/debug/lander ${IP_ADDRESS}:~/build/lander

build:
	cargo build --target=${BUILD_TARGET}

