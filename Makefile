OS := Eisen

ROOT := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
BOOT-BIN := $(ROOT)/target-bootloader/x86_64-unknown-uefi/debug
KERNEL-BIN := $(ROOT)/target-kernel/kernel-target/debug

IMG := $(OS).img

.ONESHELL:
SHELL = /bin/bash

all: build mkimg

build: build-boot build-kernel

build-boot:
	cd $(ROOT)/bootloader
	cargo build

build-kernel:
	cd $(ROOT)/kernel
	cargo build

mkimg:
	rm -f $(IMG)
	dd if=/dev/zero of=$(IMG) bs=1M count=64
	
	sudo gdisk $(IMG) < $(ROOT)/gdiskcmds
	
	LODEV=`sudo losetup -f`
	LOPRT=$$LODEV
	LOPRT+=p1
	LOMNT=$(ROOT)/$(OS)_mnt
	
	sudo losetup -P $$LODEV $(IMG)
	sudo mkfs.fat -F 32 $$LOPRT
	sudo mount --mkdir $$LOPRT $$LOMNT

	sudo mkdir -p $$LOMNT/efi/boot/
	sudo cp $(BOOT-BIN)/bootloader.efi $$LOMNT/efi/boot/bootx64.efi
	sudo mkdir -p $$LOMNT/sys/kernel/
	sudo cp $(KERNEL-BIN)/kernel $$LOMNT/sys/kernel/kernel

	sudo umount $$LOMNT
	sudo losetup -d $$LODEV
	sudo rm -rf $$LOMNT

run:
	qemu-system-x86_64 \
		-L OVMF/ \
		-pflash OVMF/OVMF.fd \
		-net none \
		-usb $(IMG) \
		-vga std \
		-m 256M