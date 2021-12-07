/*
 * @Date: 2021-12-05 15:38:01
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-07 23:51:47
 * @FilePath: /tCore/sbi/main.c
 */

#include <include/common.h>
#include <include/mem.h>
#include <include/encoding.h>

// #include "libs/string.h"
#include "devices/uart/uart.h"
#include "devices/clint/clint.h"

int main(int hartid, unsigned long fdt) {
  // init uart0
  // uart_init(DEFAULT_UART, DEFAULT_UART_FREQ, DEFAULT_UART_BAUDRATE);
  uart_puts("\r\nRunning on S7 core! Hartid=");
  uart_put_dec(hartid);
  uart_puts("\r\nTest put decimal:");
  uart_put_dec(12345678);
  uart_puts("\r\n");

  while (1)
    ;
  return 0;  // dead code
}