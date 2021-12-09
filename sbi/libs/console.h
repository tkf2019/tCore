/*
 * @Date: 2021-12-08 18:11:20
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 10:36:29
 * @FilePath: /tCore/sbi/libs/console.h
 */

#ifndef _SBI_CONSOLE_H
#define _SBI_CONSOLE_H

#include <include/common.h>

#include "devices/uart/uart.h"

#define BUFSIZE 1024
#define WHITESPACE " \t\r\n"

int getchar(void);

char* readline(const char* promt);

#endif