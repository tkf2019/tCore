#!/bin/bash

SBI=plat/qemu/rustsbi.bin
KERNEL=target/riscv64gc-unknown-none-elf/release/tcore-kernel
IMG=easy-fs.img

tmux new-session -d "qemu-system-riscv64 -machine virt -nographic -bios $SBI -kernel $KERNEL -drive file=$IMG,if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -s -S" && \
tmux split-window -h "riscv64-unknown-elf-gdb -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
tmux -2 attach-session -d
