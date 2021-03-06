/*
 * @Date: 2021-12-08 11:43:00
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 12:08:51
 * @FilePath: /tCore/sbi/trap/trap.h
 */

#ifndef _SBI_TRAP_H
#define _SBI_TRAP_H

#include <include/common.h>

struct trap_regs {
  // Normal registers from x0 to x31
  reg_size_t regs[REG_NUM];
  reg_size_t mepc;
  reg_size_t mstatus;
};

struct trap_info {
  // Trap program counter
  reg_size_t epc;
  // Trap exception cause
  reg_size_t cause;
  // Trap value
  reg_size_t tval;
  // Trap value 2
  reg_size_t tval2;
  // Trap instruction
  reg_size_t tinst;
};

int trap_redirect(struct trap_regs* regs, struct trap_info* trap);

struct trap_regs* trap_handler(struct trap_regs* regs);

#endif