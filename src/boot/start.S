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
    csrr    a0, CSR_MHARTID
    /*
	 * Save hart id and dtb pointer. The thread pointer register is not
	 * modified by C code. It is used by secondary_hart_loop.
	 */
    mv  tp, a0
    mv  s1, a1
    /*
	 * Set the global data pointer to a known value in case we get a very
	 * early trap. The global data pointer will be set its actual value only
	 * after it has been initialized.
	 */
	mv	gp, zero
    /*
	 * Set the trap handler. This must happen after initializing gp because
	 * the handler may use it.
	 */
	la  t0, trap_entry
	csrw    mtvec, t0
    /*
	 * Mask all interrupts. Interrupts are disabled globally (in m/sstatus)
	 * for U-Boot, but we will need to read m/sip to determine if we get an
	 * IPI
	 */
	csrw	mie, zero

    /* set xSIE bit to receive IPIs */
    li  t0, MIE_MSIE

/*
 * Set stackpointer in internal/ex RAM to call board_init_f
 */
call_board_init_f:
    li  t0, -16
    