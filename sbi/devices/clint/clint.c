/*
 * @Date: 2021-12-05 15:39:57
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 17:55:02
 * @FilePath: /tCore/sbi/devices/clint/clint.c
 */

#include "clint.h"

static volatile void* clint_base;

uint64_t clint_get_mtime() { return readd(clint_base + CLINT_MTIME_OFFSET); }

void clint_set_timecmp(uint64_t hartid, uint64_t time) {
  writed(time,
         clint_base + hartid * CLINT_MTIMECMP_SIZE + CLINT_MTIMECMP_OFFSET);
}

uint32_t clint_check_soft(uint64_t hartid) {
  return readw(CLINT_SOFT(clint_base, hartid));
}

void clint_send_soft(uint64_t hartid) {
  writew(1, CLINT_SOFT(clint_base, hartid));
}

void clint_clear_soft(uint64_t hartid) {
  writew(0, CLINT_SOFT(clint_base, hartid));
}

void clint_init(size_t base) { clint_base = (volatile void*)base; }
