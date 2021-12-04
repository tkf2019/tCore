arch := riscv64
target := riscv64i-unknown-none-elf
kernel := target/$(target)/release/rbl

.PHONY: build $(kernel)

$(kernel):
	cargo build --release

build:
	$(kernel)

clean:
	rm -rf target