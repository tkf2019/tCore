all: kernel

kernel:
	@cargo xtask make --oscomp
	@cp target/riscv64gc-unknown-none-elf/release/tcore-kernel.bin kernel-qemu