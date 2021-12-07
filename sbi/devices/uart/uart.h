/*
 * @Date: 2021-12-05 15:39:32
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-06 10:47:20
 * @FilePath: /tCore/sbi/devices/uart/uart.h
 */

#ifndef _SBI_UART_H
#define _SBI_UART_H

#include <include/common.h>
#include <include/io.h>
#include <include/mem.h>

#define UART_REG_TXDATA 0
#define UART_REG_RXDATA 1
#define UART_REG_TXCTRL 2
#define UART_REG_RXCTRL 3
#define UART_REG_IE 4
#define UART_REG_IP 5
#define UART_REG_DIV 6

#define UART_TXDATA_FULL 0x80000000
#define UART_RXDATA_EMPTY 0x80000000
#define UART_RXDATA_MASK 0x000000ff
#define UART_TXCTRL_TXEN 0x1
#define UART_RXCTRL_RXEN 0x1

void uart_init(unsigned long base, uint32_t in_freq, uint32_t baudrate);

void uart_puts(const char* s);
void uart_put_hex(uint32_t hex);
void uart_put_dec(int32_t dec);

#endif