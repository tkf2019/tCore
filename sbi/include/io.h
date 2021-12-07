/*
 * @Date: 2021-12-06 10:27:57
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-06 19:47:06
 * @FilePath: /tCore/sbi/include/io.h
 */

#ifndef _SBI_IO_H
#define _SBI_IO_H

#include <include/common.h>

#define RISCV_FENCE(p, s) \
  __asm__ __volatile__("fence " #p "," #s : : : "memory")

#define __io_br() \
  do {            \
  } while (0)
#define __io_ar() RISCV_FENCE(i, r)
#define __io_bw() RISCV_FENCE(w, o)
#define __io_aw() \
  do {            \
  } while (0)

static inline void __raw_writeb(uint8_t v, volatile void* a) {
  asm volatile("sb %0, 0(%1)" : : "r"(v), "r"(a));
}
static inline void __raw_writeh(uint16_t v, volatile void* a) {
  asm volatile("sh %0, 0(%1)" : : "r"(v), "r"(a));
}
static inline void __raw_writew(uint32_t v, volatile void* a) {
  asm volatile("sw %0, 0(%1)" : : "r"(v), "r"(a));
}
#if XLEN != 4
static inline void __raw_writed(uint64_t v, volatile void* a) {
  asm volatile("sd %0, 0(%1)" : : "r"(v), "r"(a));
}
#endif

static inline uint8_t __raw_readb(const volatile void* a) {
  uint8_t v;
  asm volatile("lb %0, 0(%1)" : "=r"(v) : "r"(a));
  return v;
}
static inline uint16_t __raw_readh(const volatile void* a) {
  uint16_t v;
  asm volatile("lh %0, 0(%1)" : "=r"(v) : "r"(a));
  return v;
}
static inline uint32_t __raw_readw(const volatile void* a) {
  uint32_t v;
  asm volatile("lw %0, 0(%1)" : "=r"(v) : "r"(a));
  return v;
}
#if XLEN != 4
static inline uint64_t __raw_readd(const volatile void* a) {
  uint64_t v;
  asm volatile("ld %0, 0(%1)" : "=r"(v) : "r"(a));
  return v;
}
#endif

#define readb(a)        \
  ({                    \
    uint8_t v;          \
    __io_br();          \
    v = __raw_readb(a); \
    __io_ar();          \
    v;                  \
  })
#define readh(a)        \
  ({                    \
    uint16_t v;         \
    __io_br();          \
    v = __raw_readh(a); \
    __io_ar();          \
    v;                  \
  })
#define readw(a)        \
  ({                    \
    uint32_t v;         \
    __io_br();          \
    v = __raw_readw(a); \
    __io_ar();          \
    v;                  \
  })
#if XLEN != 4
#define readd(a)        \
  ({                    \
    uint64_t v;         \
    __io_br();          \
    v = __raw_readd(a); \
    __io_ar();          \
    v;                  \
  })
#endif

#define writeb(v, a)        \
  ({                        \
    __io_bw();              \
    __raw_writeb((v), (a)); \
    __io_aw();              \
  })
#define writeh(v, a)        \
  ({                        \
    __io_bw();              \
    __raw_writeh((v), (a)); \
    __io_aw();              \
  })
#define writew(v, a)        \
  ({                        \
    __io_bw();              \
    __raw_writew((v), (a)); \
    __io_aw();              \
  })
#if XLEN != 4
#define writed(v, a)        \
  ({                        \
    __io_bw();              \
    __raw_writed((v), (a)); \
    __io_aw();              \
  })
#endif

#endif