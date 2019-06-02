name ?= chip8book
mode ?= debug
target ?= x86_64-unknown-uefi
target_dir := target/$(target)/$(mode)
esp_dir := $(shell pwd)/$(target_dir)/esp
efi_dir := $(esp_dir)/EFI/Boot

default: build

build:
	cargo xbuild --target $(target)
	mkdir -p $(efi_dir)
	cp -f $(target_dir)/$(name).efi $(efi_dir)/BootX64.efi

run: build
	qemu-system-x86_64 -serial stdio -L . --bios OVMF.fd -drive format=raw,file=fat:rw:$(esp_dir) -m 128M -net none -vga std
