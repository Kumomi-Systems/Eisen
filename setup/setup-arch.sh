#!/bin/bash

sudo pacman -S dosfstools ovmf qemu-desktop rustup --needed --noconfirm

sudo cp -r /usr/share/OVMF/x64 OVMF/
sudo chown -R --reference=.git/ OVMF/

rustup default nightly
rustup component add rust-src
rustup target install x86_64-unknown-uefi
