.PHONY: run

BIN   := target/debug/input-hub

run:
	cargo build
	sudo ./${BIN}

AARCH64_TARGET := aarch64-unknown-linux-gnu
AARCH64_CC := aarch64-linux-gnu-gcc
AARCH64_CXX := aarch64-linux-gnu-g++
AARCH64_AR := aarch64-linux-gnu-ar

ARMV7_TARGET := armv7-unknown-linux-gnueabihf
ARMV7_CC := arm-linux-gnueabihf-gcc
ARMV7_CXX := arm-linux-gnueabihf-g++
ARMV7_AR := arm-linux-gnueabihf-ar

build-aarch64:
	PKG_CONFIG_ALLOW_CROSS=1 \
	PKG_CONFIG_SYSROOT_DIR=/ \
	PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
	CC_aarch64_unknown_linux_gnu=$(AARCH64_CC) \
	CXX_aarch64_unknown_linux_gnu=$(AARCH64_CXX) \
	AR_aarch64_unknown_linux_gnu=$(AARCH64_AR) \
	RUSTFLAGS="-C linker=$(AARCH64_CC)" \
	cargo build --release \
		--target=$(AARCH64_TARGET)

cross-aarch64:
	cross build --target aarch64-unknown-linux-gnu --release

build-armv7:
	CC_armv7_unknown_linux_gnueabihf=$(ARMV7_CC) \
	CXX_armv7_unknown_linux_gnueabihf=$(ARMV7_CXX) \
	AR_armv7_unknown_linux_gnueabihf=$(ARMV7_AR) \
	RUSTFLAGS="-C linker=$(ARMV7_CC)" \
	PKG_CONFIG_ALLOW_CROSS=1 \
	cargo build --release \
		--target=$(ARMV7_TARGET) \
