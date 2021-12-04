/*
    RISC-V Boot
*/
#define LREG			ld
#define SREG			sd
#define REGBYTES		8
#define RELOC_TYPE		R_RISCV_64
#define SYM_INDEX		0x20
#define SYM_SIZE		0x18

.section .text
.globl _start
_start:
	li x1, 0
	li x2, 0
	li x3, 0
	li x4, 0
	li x5, 0
	li x6, 0
	li x7, 0
	li x8, 0
	li x9, 0
	# skip a0 and a1
	# a0 is hart id
	# a1 is dtb address
	li x12, 0
	li x13, 0
	li x14, 0
	li x15, 0
	li x16, 0
	li x17, 0
	li x18, 0
	li x19, 0
	li x20, 0
	li x21, 0
	li x22, 0
	li x23, 0
	li x24, 0
	li x25, 0
	li x26, 0
	li x27, 0
	li x28, 0
	li x29, 0
	li x30, 0
	li x31, 0
    
    # setup PMP config
    # 0x00000000-0xffffffff RWX
    li t0, 0b00001111
    csrw pmpcfg0, t0
    li t0, 0xffffffff
    csrw pmpaddr0, t0

    csrw mscratch, x0

    la t0, trap_vector
    csrw mtvec, t0
try:
    csrr t1, mtvec
    bne t0, t1, try

    # setup boot stack space
    la sp, bootstacktop

    # save sp to mscratch
    csrw mscratch, sp

    # first hart first
    bnez a0, other_hart
    j boot_first_hart

other_hart:
    # we have only one hart now
    wfi
    j other_hart

.section .bss.stack
.align 12 # PAGE_SIZE
bootstack:
    # 4KB stack
    .space 0x1000
bootstacktop: