chip8book
=========================

Bare-metal chip8 for macbook based on [libchip8](https://github.com/YushiOMOTE/libchip8).

### Prerequisites

* [QEMU](https://www.qemu.org/)
* [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF)
    * Needs `OVMF.fd` in the same directory as `Makefile`.

### Try

* To build

    ```
    $ make build
    ```

* To run on QEMU

    ```
    $ make run
    ```
