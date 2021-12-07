/*
 * @Date: 2021-12-05 15:39:53
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-06 16:28:07
 * @FilePath: /tCore/sbi/devices/clint/clint.h
 */

#ifndef _SBI_CLINT_H
#define _SBI_CLINT_H

// TODO
void clint_init();
void clint_get_mtime();
void clint_set_timer();
void clint_send_soft();
void clint_clear_soft();

#endif
