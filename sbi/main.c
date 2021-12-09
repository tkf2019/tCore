/*
 * @Date: 2021-12-05 15:38:01
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 11:47:15
 * @FilePath: /tCore/sbi/main.c
 */

#include <include/common.h>
#include <include/mem.h>
#include <include/smp.h>
#include <include/riscv.h>

#include "devices/uart/uart.h"
#include "devices/clint/clint.h"

#include "libs/string.h"
#include "libs/console.h"

#define DEFAULT_UART UART0_ADDR
#define DEFAULT_UART_FREQ 0
#define DEFAULT_UART_BAUDRATE 115200

void trap_handler();
void test_ipi(size_t hartid);
int wait_ipi(size_t hartid);

void puts(const char* s) {
  uart_puts("\r\n[HART ");
  char hartid = read_csr(mhartid);
  uart_putc('0' + hartid);
  uart_puts("] ");
  uart_puts(s);
}

void* smp_memcpy(void* dst, const void* src, size_t n) {
  const char* s = src;
  char* d = dst;
  while (n-- > 0) {
    writeb(readb(s), d);
    s++, d++;
  }
  return dst;
}

int main(size_t hartid, size_t fdt) {
  // init uart0
  uart_init(DEFAULT_UART, DEFAULT_UART_FREQ, DEFAULT_UART_BAUDRATE);
  // init clint
  clint_init(CLINT_CTRL_ADDR);

  puts("Running SBI!");
  puts("Test put hexadecimal: ");
  uart_put_hex(0x12345678);

  // Test console
  char* line;
  puts("Test console: ");
  if ((line = readline(NULL)) != NULL) {
    puts("Test console OK: ");
    uart_puts(line);
  }
  // Test sending and receiving IPI
  puts("Test IPI");
  test_ipi(hartid);

  return 0;  // dead code
}

void test_ipi(size_t hartid) {
  // clear clint msip
  clint_clear_soft(hartid);
  // set software interrupt in mie
  set_csr(mie, MIP_MSIP);
  // begin test loop
  while (1) {
    // hartid = 0, to_hartid = 1 ~ 4
    size_t to_hartid;
    while (1) {
      puts("Input hartid to wake up target hart: ");
      to_hartid = readline(NULL)[0] - '0';
      if (to_hartid >= ZERO_HART + 1 && to_hartid <= MAX_HARTS - 1) {
        break;
      } else {
        puts("Hartid out of range!");
      }
    }
    // Data tighted memory
    puts("Input message: ");
    char* m = readline(NULL);
    memcpy(SMP_ADDR, m, strlen(m) + 1);
    // uart_put_hex(read_csr(mcause));

    puts("Send software interrupt. Hartid=");
    uart_put_hex(to_hartid);
    clint_send_soft(to_hartid);
    wait_ipi(hartid);
    puts("Finished receiving. Hartid=");
    uart_put_hex(to_hartid);
  }
}

void trap_handler() {}

int other_main(size_t hartid, size_t fdt) {
  // init uart0
  uart_init(DEFAULT_UART, DEFAULT_UART_FREQ, DEFAULT_UART_BAUDRATE);
  // init clint
  clint_init(CLINT_CTRL_ADDR);
  // clear clint msip
  clint_clear_soft(hartid);
  // set software interrupt in mie
  set_csr(mie, MIP_MSIP);
  while (1) {
    wait_ipi(hartid);
    puts("Software interrupt from Hart 0");
    puts("Message from Hart 0: ");
    uart_puts(SMP_ADDR);
    clint_send_soft(ZERO_HART);
  }
  return 0;
}

int wait_ipi(size_t hartid) {
  // loop and wait for software interrupt
  while (!(read_csr(mip) & MIP_MSIP)) wfi();
  // receive software interrupt now
  clint_clear_soft(hartid);
  return 0;
}