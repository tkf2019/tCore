/*
 * @Date: 2021-12-05 15:39:53
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 17:54:41
 * @FilePath: /tCore/sbi/devices/clint/clint.h
 */

#ifndef _SBI_CLINT_H
#define _SBI_CLINT_H

#include <include/riscv.h>
#include <include/mem.h>
#include <include/io.h>

#define CLINT_MSIP_OFFSET 0x0000
#define CLINT_MSIP0_OFFSET 0x0000
#define CLINT_MSIP1_OFFSET 0x0004
#define CLINT_MSIP2_OFFSET 0x0008
#define CLINT_MSIP3_OFFSET 0x000c
#define CLINT_MSIP4_OFFSET 0x0010
#define CLINT_MTIMECMP_OFFSET 0x4000
#define CLINT_MTIMECMP0_OFFSET 0x4000
#define CLINT_MTIMECMP1_OFFSET 0x4008
#define CLINT_MTIMECMP2_OFFSET 0x4010
#define CLINT_MTIMECMP3_OFFSET 0x4018
#define CLINT_MTIMECMP4_OFFSET 0x4020
#define CLINT_MTIME_OFFSET 0xbff8

#define CLINT_MSIP_SIZE 0x4
#define CLINT_MTIMECMP_SIZE 0x8

#define CLINT_SOFT(base, hartid) \
  base + hartid* CLINT_MSIP_SIZE + CLINT_MSIP_OFFSET

void clint_init(size_t base);

uint64_t clint_get_mtime();
void clint_set_timer(uint64_t hartid, uint64_t time);

uint32_t clint_check_soft(uint64_t hartid);
void clint_send_soft(uint64_t hartid);
void clint_clear_soft(uint64_t hartid);

#endif
