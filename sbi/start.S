/*
 * @Date: 2021-12-05 15:46:21
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 18:33:52
 * @FilePath: /tCore/sbi/start.S
 */

#include <include/common.h>
#include <include/smp.h>
#include <include/riscv.h>

.altmacro
.macro LI_ZERO n
  li x\n, 0
.endm

.section .text.entry
.globl _start
_start:
  // flush the instruction cache
  # fence.i
  // clear registers
  .set n, 1
  .rept 9
    LI_ZERO %n
    .set n, n+1
  .endr
  // skip a0 = hartid, a1 = dtb ptr
  .set n, 12
  .rept 20
    LI_ZERO %n
    .set n, n+1
  .endr

  // clear mscratch
  csrw mscratch, 0 

  // initialize trap entry
  la t0, trap_entry
  csrw mtvec, t0

  // Leaving HART 0 do the works following
  # smp_pause(s1, s2)

  // Resume by IPI from HART 0 
  # smp_resume(s1, s2)

  // Allocate 4KB stack for each hart
  la sp, stack
  csrr t0, mhartid
  slli t1, t0, LOG_STACK_SIZE
  sub sp, sp, t1

  li t1, ZERO_HART
  bne t0, t1, other_main

  // HART 0
  // Load data section
  la t0, ldata
  la t1, sdata
  beq t0, t1, 2f
  la t2, edata
  bgeu t1, t2, 2f
1:
  LOAD t3, 0(t0)
  STORE t3, 0(t1)
  addi t0, t0, XLEN
  addi t1, t1, XLEN
  bltu t1, t2, 1b
2:

  // Clear bss section
  la t0, sbss
  la t1, ebss
  bgeu t0, t1, 2f
1:
  STORE zero, 0(t0)
  addi t0, t0, XLEN
  bltu t0, t1, 1b
2:
  tail main
