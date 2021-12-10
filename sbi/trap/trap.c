/*
 * @Date: 2021-12-08 15:51:18
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-10 11:21:49
 * @FilePath: /tCore/sbi/trap/trap.c
 */

#include "trap.h"

void puts(const char* s) {
  uart_puts("\r\n[HART ");
  char hartid = read_csr(mhartid);
  uart_putc('0' + hartid);
  uart_puts(" TRAP] ");
  uart_puts(s);
}

struct trap_regs* trap_handler(struct trap_regs* regs) {
  int result = SBI_NOT_SUPPORTED;
  const char* msg = "trap handler failed";
  reg_size_t mcause = read_csr(mcause);
  reg_size_t mtval = read_csr(mtval), mtval2 = 0, mtinst = 0;
  struct trap_info trap;
  if (mcause & CAUSE_INT_MASK) {
    mcause &= ~CAUSE_INT_MASK;
    switch (mcause) {
      case IRQ_M_TIMER:
        result = timer_handler();
        break;
      case IRQ_M_SOFT:
        result = ipi_handler();
        break;
      default:
        msg = "unhandled external interrupt";
        goto trap_error;
    };
    return regs;
  }
  switch (mcause) {
    case CAUSE_MACHINE_ECALL:
      result = ecall_handler(regs);
      msg = "ecall handler failed";
      break;
    default:
      msg = "unhandled exception";
      goto trap_error;
  };
trap_error:
  puts(msg);
  return regs;
}