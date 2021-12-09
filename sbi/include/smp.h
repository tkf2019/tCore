/*
 * @Date: 2021-12-05 15:58:32
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 10:46:30
 * @FilePath: /tCore/sbi/include/smp.h
 */

#ifndef _SBI_SMP_H
#define _SBI_SMP_H

#include <include/mem.h>
#include <include/riscv.h>

#define MAX_HARTS 5
#define ZERO_HART 0  // S7 Core

// end of msip for Inter-Processor Interrupt
#define CLINT_END_HART_IPI CLINT_CTRL_ADDR + (MAX_HARTS * CLINT_MSIP_SIZE)

// disable multiple processors
#define smp_disable(reg1, reg2) \
  csrr reg1, mhartid;           \
  li reg2, ZERO_HART;           \
  beq reg1, reg2, hart0_entry;  \
  42:;                          \
  wfi;                          \
  j 42b;                        \
  hart0_entry:

// block
#define smp_pause(reg1, reg2) \
  li reg2, MIP_MSIP;          \
  csrw mie, reg2;             \
  li reg1, ZERO_HART;         \
  csrr reg2, mhartid;         \
  bne reg1, reg2, 42f

// Resume other cores by IPI
// After receiving an interrupt:
// 1. Check MSIP bit in msip
// 2. Clear msip in clint

#define smp_resume(reg1, reg2) \
  li reg1, CLINT_CTRL_ADDR;    \
  41:;                         \
  li reg2, 1;                  \
  sw reg2, 0(reg1);            \
  addi reg1, reg1, 4;          \
  li reg2, CLINT_END_HART_IPI; \
  blt reg1, reg2, 41b;         \
  42:;                         \
  wfi;                         \
  csrr reg2, mip;              \
  andi reg2, reg2, MIP_MSIP;   \
  beqz reg2, 42b;              \
  li reg1, CLINT_CTRL_ADDR;    \
  csrr reg2, mhartid;          \
  slli reg2, reg2, 2;          \
  add reg2, reg2, reg1;        \
  sw zero, 0(reg2);            \
  41:;                         \
  lw reg2, 0(reg1);            \
  bnez reg2, 41b;              \
  addi reg1, reg1, 4;          \
  li reg2, CLINT_END_HART_IPI; \
  blt reg1, reg2, 41b;

#endif