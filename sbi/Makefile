ARCH := riscv64
ifndef GCCPREFIX
GCCPREFIX := riscv64-unknown-elf-
endif
CC := $(GCCPREFIX)gcc
# IMAC & No floating point
CFLAGS := -march=rv64imac -mabi=lp64
CFLAGS += -mcmodel=medany -mexplicit-relocs
CFLAGS += -I. -O2 -std=gnu99 -nostdinc 
CFLAGS += -g -Wall -Wno-unused
CFLAGS += -fno-builtin -fno-stack-protector -ffunction-sections -fdata-sections
CCASFLAGS := -I. -mcmodel=medany -mexplicit-relocs
LD := $(GCCPREFIX)ld
LDFLAGS := -nostdlib -nostartfiles
LDSCRIPT := linker.ld
OBJCOPY := $(GCCPREFIX)objcopy
OBJDUMP := $(GCCPREFIX)objdump

BINDIR := ../bin
INCLUDE := include
TARGET := sbi
ELF := $(TARGET).elf
BIN := $(TARGET).bin
ASM := $(TARGET).asm
HEADER := $(foreach sdir, $(INCLUDE), $(wildcard $(sdir)/*.h))

LIB_SBI1_O := start.o
LIB_SBI2_O := $(LIB_SBI1_O) \
	trap.o \
	devices/uart/uart.o \
	devices/clint/clint.o \
	libs/string.o \
	libs/console.o

QEMU := qemu-system-$(ARCH)
QEMUFLAGS := -M sifive_u -smp 5 -m 4G
QEMUFLAGS += -display none -serial stdio
QEMUFLAGS += -bios ../bin/u-boot-spl.bin
QEMUFLAGS += -kernel sbi.bin

SD_IMAGE := $(BINDIR)/sd-image.dts
TARGET_IMAGE := $(BINDIR)/sd-part-2.img

all: $(BIN) $(ASM)

image: $(BIN) $(ASM)
	@mkimage -f $(SD_IMAGE) $(TARGET_IMAGE)
	@sudo dd if=$(TARGET_IMAGE) of=/dev/sda seek=2082

qemu: 
	@$(QEMU) $(QEMUFLAGS)

elf: $(ELF)

$(ELF): $(LIB_SBI1_O) main.o $(LIB_SBI2_O) $(LDSCRIPT)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $(filter %.o, $^) -T$(filter %.ld, $^)

%.bin: %.elf
	$(OBJCOPY) -S -R .comment -R .note.gnu.build-id -O binary $^ $@

%.asm: %.elf
	$(OBJDUMP) -d $^ > $@

%.o: %.c $(HEADER)
	$(CC) $(CFLAGS) -o $@ -c $<

%.o: %.S $(HEADER)
	$(CC) $(CFLAGS) $(CCASFLAGS) -c $< -o $@

.PHONY: clean
clean:
	rm -rf $(BIN) $(ELF) $(ASM) */*.o */*/*.o *.o