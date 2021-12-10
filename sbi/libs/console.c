/*
 * @Date: 2021-12-08 18:11:15
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 12:08:52
 * @FilePath: /tCore/sbi/libs/console.c
 */

#include "console.h"

static inline int __intr_save(void) {
  reg_size_t mstatus = read_csr(mstatus);
  if (mstatus & MSTATUS_MIE) {
    clear_csr(mstatus, MSTATUS_MIE);
    return 1;
  }
  return 0;
}

static inline void __intr_restore(int flag) {
  if (flag) set_csr(mstatus, MSTATUS_MIE);
}

#define intr_save(x)   \
  do {                 \
    x = __intr_save(); \
  } while (0)

#define intr_restore(x) __intr_restore(x)

int getchar(void) {
  int c;
  while ((c = uart_getc()) == -1)
    ;
  return c;
}

char* readline(const char* prompt) {
  static char buf[BUFSIZE];
  if (prompt != NULL) {
    uart_puts(prompt);
  }
  int c, i = 0;
  while (1) {
    c = getchar();
    if (c < 0)
      return NULL;
    else if (c >= ' ' && i < BUFSIZE - 1) {
      uart_putc(c);
      buf[i++] = c;
    } else if (c == '\b' && i > 0) {
      uart_putc(c);
      i--;
    } else if (c == '\n' || c == '\r') {
      uart_putc(c);
      buf[i] = '\0';
      return buf;
    }
  }
  return NULL;
}