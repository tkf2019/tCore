/*
 * @Date: 2021-12-04 23:41:36
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-07 23:32:39
 * @FilePath: /tCore/sbi/include/common.h
 */

#ifndef _SBI_COMMON_H
#define _SBI_COMMON_H

#define BITS 64
#define STORE sd
#define LOAD ld
#define XLEN 8

#define BIT(nr) (1UL << (nr))
#define BIT_MASK(nr) (1UL << ((nr) % BITS))
#define BIT_WORD(bit) ((bit) / BITS)

#define LOG_STACK_SIZE 12
#define STACK_SIZE (1UL << LOG_STACK_SIZE)

#if XLEN == 8
#define int64_t long
#define uint64_t unsigned long
#elif XLEN == 4
#define int64_t long long
#define uint64_t unsigned long long
#else
#error "unexpected XLEN"
#endif

#define size_t unsigned long
#define int32_t int
#define uint32_t unsigned int
#define in16_t short
#define uint16_t unsigned short
#define int8_t char
#define uint8_t unsigned char

#define DEFAULT_UART 0
#define DEFAULT_UART_FREQ 0
#define DEFAULT_UART_BAUDRATE 115200

#endif