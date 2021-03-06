/*
 * @Date: 2021-12-05 15:39:37
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 21:39:40
 * @FilePath: /tCore/sbi/devices/uart/uart.c
 */

#include "uart.h"

static volatile void *uart_base;
static uint32_t uart_in_freq;
static uint32_t uart_baudrate;

static inline unsigned int uart_min_clk_divisor(uint64_t in_freq,
                                                uint64_t max_target_hz) {
  uint64_t quotient = (in_freq + max_target_hz - 1) / (max_target_hz);
  // Avoid underflow
  if (quotient == 0) {
    return 0;
  } else {
    return quotient - 1;
  }
}

static inline uint32_t get_reg(uint32_t i) {
  return readw(uart_base + (i << 2));
}

static inline void set_reg(uint32_t i, uint32_t v) {
  writew(v, uart_base + (i << 2));
}

void uart_putc(char ch) {
  while (get_reg(UART_REG_TXDATA) & UART_TXDATA_FULL)
    ;
  set_reg(UART_REG_TXDATA, ch);
}

int uart_getc(void) {
  uint32_t reg = get_reg(UART_REG_RXDATA);
  if (!(reg & UART_RXDATA_EMPTY)) return reg & UART_RXDATA_MASK;
  return -1;
}

void uart_init(size_t base, uint32_t in_freq, uint32_t baudrate) {
  uart_base = (volatile void *)base;
  uart_in_freq = in_freq;
  uart_baudrate = baudrate;

  if (in_freq) set_reg(UART_REG_DIV, uart_min_clk_divisor(in_freq, baudrate));
  set_reg(UART_REG_IE, 0);
  set_reg(UART_REG_TXCTRL, UART_TXCTRL_TXEN);
  set_reg(UART_REG_RXCTRL, UART_RXCTRL_RXEN);
}

void uart_puts(const char *s) {
  while (*s != '\0') uart_putc(*s++);
}

void uart_put_hex(uint32_t hex) {
  uart_puts("0x");
  int num = sizeof(hex) * 2;
  for (int idx = num - 1; idx >= 0; idx--) {
    char c = (hex >> (idx * 4)) & 0xf;
    uart_putc(c < 0xa ? ('0' + c) : ('a' + c - 0xa));
  }
}
