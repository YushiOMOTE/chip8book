chip8book
=========================

Bare-metal chip8 for macbook based on [libchip8](https://github.com/YushiOMOTE/libchip8).

### Prerequisites

* [QEMU](https://www.qemu.org/)
* [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF)
    * Needs `OVMF.fd` in the same directory as `Makefile`.

### Try

* Get [uefi-rs](https://github.com/rust-osdev/uefi-rs/)

    ```
    $ git submodule update --init
    ```

* To build

    ```
    $ make build
    ```

* To run on QEMU

    ```
    $ make run
    ```
