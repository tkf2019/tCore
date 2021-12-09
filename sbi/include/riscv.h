/*
 * @Date: 2021-12-08 16:05:40
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-08 22:36:38
 * @FilePath: /tCore/sbi/include/riscv.h
 */

#ifndef _SBI_RISCV_H
#define _SBI_RSICV_H

#include <include/common.h>

#define REG_NUM 32
#define reg_size_t size_t

#define REG_ZERO 0
#define REG_RA 1
#define REG_SP 2
#define REG_GP 3
#define REG_TP 4
#define REG_T0 5
#define REG_T1 6
#define REG_T2 7
#define REG_S0 8
#define REG_S1 9
#define REG_A0 10
#define REG_A1 11
#define REG_A2 12
#define REG_A3 13
#define REG_A4 14
#define REG_A5 15
#define REG_A6 16
#define REG_A7 17
#define REG_S2 18
#define REG_S3 19
#define REG_S4 20
#define REG_S5 21
#define REG_S6 22
#define REG_S7 23
#define REG_S8 24
#define REG_S9 25
#define REG_S10 26
#define REG_S11 27
#define REG_T3 28
#define REG_T4 29
#define REG_T5 30
#define REG_T6 31

#define MODE_PREFIX(__suffix) m##__suffix

#define MSTATUS_UIE 0x00000001
#define MSTATUS_SIE 0x00000002
#define MSTATUS_HIE 0x00000004
#define MSTATUS_MIE 0x00000008
#define MSTATUS_UPIE 0x00000010
#define MSTATUS_SPIE 0x00000020
#define MSTATUS_HPIE 0x00000040
#define MSTATUS_MPIE 0x00000080
#define MSTATUS_SPP 0x00000100
#define MSTATUS_HPP 0x00000600
#define MSTATUS_MPP 0x00001800
#define MSTATUS_FS 0x00006000
#define MSTATUS_XS 0x00018000
#define MSTATUS_MPRV 0x00020000
#define MSTATUS_PUM 0x00040000
#define MSTATUS_VM 0x1F000000
#define MSTATUS32_SD 0x80000000
#define MSTATUS64_SD 0x8000000000000000

#define MCAUSE32_CAUSE 0x7FFFFFFF
#define MCAUSE64_CAUSE 0x7FFFFFFFFFFFFFFF
#define MCAUSE32_INT 0x80000000
#define MCAUSE64_INT 0x8000000000000000

#define SSTATUS_UIE 0x00000001
#define SSTATUS_SIE 0x00000002
#define SSTATUS_UPIE 0x00000010
#define SSTATUS_SPIE 0x00000020
#define SSTATUS_SPP 0x00000100
#define SSTATUS_FS 0x00006000
#define SSTATUS_XS 0x00018000
#define SSTATUS_PUM 0x00040000
#define SSTATUS32_SD 0x80000000
#define SSTATUS64_SD 0x8000000000000000

#define IRQ_S_SOFT 1
#define IRQ_VS_SOFT 2
#define IRQ_M_SOFT 3
#define IRQ_S_TIMER 5
#define IRQ_VS_TIMER 6
#define IRQ_M_TIMER 7
#define IRQ_S_EXT 9
#define IRQ_VS_EXT 10
#define IRQ_M_EXT 11
#define IRQ_S_GEXT 12
#define IRQ_PMU_OVF 13

#define MIP_SSIP BIT(IRQ_S_SOFT)
#define MIP_MSIP BIT(IRQ_M_SOFT)
#define MIP_STIP BIT(IRQ_S_TIMER)
#define MIP_MTIP BIT(IRQ_M_TIMER)
#define MIP_SEIP BIT(IRQ_S_EXT)
#define MIP_MEIP BIT(IRQ_M_EXT)

#define SIP_SSIP MIP_SSIP
#define SIP_STIP MIP_STIP

#define PRV_U 0
#define PRV_S 1
#define PRV_H 2
#define PRV_M 3

#define VM_MBARE 0
#define VM_MBB 1
#define VM_MBBID 2
#define VM_SV32 8
#define VM_SV39 9
#define VM_SV48 10

#define CAUSE_MISALIGNED_FETCH 0
#define CAUSE_FETCH_ACCESS 1
#define CAUSE_ILLEGAL_INSTRUCTION 2
#define CAUSE_BREAKPOINT 3
#define CAUSE_MISALIGNED_LOAD 4
#define CAUSE_LOAD_ACCESS 5
#define CAUSE_MISALIGNED_STORE 6
#define CAUSE_STORE_ACCESS 7
#define CAUSE_USER_ECALL 8
#define CAUSE_SUPERVISOR_ECALL 9
#define CAUSE_MACHINE_ECALL 11
#define CAUSE_FETCH_PAGE_FAULT 12
#define CAUSE_LOAD_PAGE_FAULT 13
#define CAUSE_STORE_PAGE_FAULT 15

#define DEFAULT_RSTVEC 0x00001000
#define DEFAULT_NMIVEC 0x00001004
#define DEFAULT_MTVEC 0x00001010
#define CONFIG_STRING_ADDR 0x0000100C
#define EXT_IO_BASE 0x40000000
#define DRAM_BASE 0x80000000

#define fence(p, s) __asm__ __volatile__("fence " #p "," #s : : : "memory")

#define wfi() __asm__ __volatile("wfi")

#define read_csr(r)                         \
  ({                                        \
    reg_size_t v;                           \
    asm volatile("csrr %0, " #r : "=r"(v)); \
    v;                                      \
  })

#define write_csr(reg, val) ({ asm volatile("csrw " #reg ", %0" ::"rK"(val)); })

#define swap_csr(reg, val)                                            \
  ({                                                                  \
    reg_size_t __tmp;                                                 \
    asm volatile("csrrw %0, " #reg ", %1" : "=r"(__tmp) : "rK"(val)); \
    __tmp;                                                            \
  })

#define set_csr(reg, bit)                                             \
  ({                                                                  \
    reg_size_t __tmp;                                                 \
    asm volatile("csrrs %0, " #reg ", %1" : "=r"(__tmp) : "rK"(bit)); \
    __tmp;                                                            \
  })

#define clear_csr(reg, bit)                                           \
  ({                                                                  \
    reg_size_t __tmp;                                                 \
    asm volatile("csrrc %0, " #reg ", %1" : "=r"(__tmp) : "rK"(bit)); \
    __tmp;                                                            \
  })

#endif