.PHONY: run run-local run-remote build-aarch64 cross-aarch64 cross-aarch64-remote build-armv7

BIN := target/debug/gmpad

run: run-local

run-local:
	cargo build
	sudo ./${BIN} local

run-remote:
	cargo build
	sudo ./${BIN} remote

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

# Default cross-build (host -> aarch64). Builds with all features — requires
# libclang>=9 in the cross container for uhid-virt/bindgen.
cross-aarch64:
	cross build --target $(AARCH64_TARGET) --release

# Remote-only cross-build. Skips uhid-virt (and its bindgen build script),
# producing a Bluetooth-HID-capable binary suitable for handhelds (Trimui
# Smart Pro, RG35XX, etc.). This is the recommended target until the cross
# container ships a newer libclang.
cross-aarch64-remote:
	cross build --target $(AARCH64_TARGET) --release \
		--no-default-features --features remote

build-armv7:
	CC_armv7_unknown_linux_gnueabihf=$(ARMV7_CC) \
	CXX_armv7_unknown_linux_gnueabihf=$(ARMV7_CXX) \
	AR_armv7_unknown_linux_gnueabihf=$(ARMV7_AR) \
	RUSTFLAGS="-C linker=$(ARMV7_CC)" \
	PKG_CONFIG_ALLOW_CROSS=1 \
	cargo build --release \
		--target=$(ARMV7_TARGET)
