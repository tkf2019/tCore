/*
 * @Date: 2021-12-08 11:13:12
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 16:08:33
 * @FilePath: /tCore/sbi/trap.S
 */

#include <include/common.h>
#include <include/smp.h>
#include <include/riscv.h>

.altmacro
.macro STORE_SP n
  STORE x\n, \n*XLEN(sp)
.endm
.macro LOAD_SP n
  LOAD x\n, \n*XLEN(sp)
.endm

.section .text
.globl trap_entry
trap_entry:
  csrrw sp, mscratch, sp
  # save the context
  addi sp, sp, -32*XLEN
  STORE x0, 0*XLEN(sp)
  STORE x1, 1*XLEN(sp)
  # get sp from mscratch
  csrr x1, mscratch
  STORE x1, 2*XLEN(sp)
  # store x3 to x31
  .set n, 3
  .rept 29
    STORE_SP %n
    .set n, n+1
  .endr
  # pass sp to trap handler
  mv a0, sp
  jal trap_handler
  # update mepc after handler
  csrw mepc, a0
  # save sp to mscratch 
  addi t0, sp, 32*XLEN
  csrw mscratch, t0
  # restore the context
  LOAD x1, 1*XLEN(sp)
  .set n, 3
  .rept 29
    LOAD_SP %n
    .set n, n+1
  .endr
  LOAD x2, 2*XLEN(sp)
  # return
  mret
