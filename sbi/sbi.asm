
sbi.elf:     file format elf64-littleriscv


Disassembly of section .text:

0000000080000000 <_start>:
    80000000:	4081                	li	ra,0
    80000002:	4101                	li	sp,0
    80000004:	4181                	li	gp,0
    80000006:	4201                	li	tp,0
    80000008:	4281                	li	t0,0
    8000000a:	4301                	li	t1,0
    8000000c:	4381                	li	t2,0
    8000000e:	4401                	li	s0,0
    80000010:	4481                	li	s1,0
    80000012:	4601                	li	a2,0
    80000014:	4681                	li	a3,0
    80000016:	4701                	li	a4,0
    80000018:	4781                	li	a5,0
    8000001a:	4801                	li	a6,0
    8000001c:	4881                	li	a7,0
    8000001e:	4901                	li	s2,0
    80000020:	4981                	li	s3,0
    80000022:	4a01                	li	s4,0
    80000024:	4a81                	li	s5,0
    80000026:	4b01                	li	s6,0
    80000028:	4b81                	li	s7,0
    8000002a:	4c01                	li	s8,0
    8000002c:	4c81                	li	s9,0
    8000002e:	4d01                	li	s10,0
    80000030:	4d81                	li	s11,0
    80000032:	4e01                	li	t3,0
    80000034:	4e81                	li	t4,0
    80000036:	4f01                	li	t5,0
    80000038:	4f81                	li	t6,0
    8000003a:	34005073          	csrwi	mscratch,0
    8000003e:	00000297          	auipc	t0,0x0
    80000042:	3b828293          	addi	t0,t0,952 # 800003f6 <trap_entry>
    80000046:	30529073          	csrw	mtvec,t0
    8000004a:	00041117          	auipc	sp,0x41
    8000004e:	cc610113          	addi	sp,sp,-826 # 80040d10 <stack>
    80000052:	f14022f3          	csrr	t0,mhartid
    80000056:	00e29313          	slli	t1,t0,0xe
    8000005a:	40610133          	sub	sp,sp,t1
    8000005e:	4301                	li	t1,0
    80000060:	00628463          	beq	t0,t1,80000068 <_start+0x68>
    80000064:	2b20006f          	j	80000316 <other_main>
    80000068:	00001297          	auipc	t0,0x1
    8000006c:	89828293          	addi	t0,t0,-1896 # 80000900 <buf.0>
    80000070:	00001317          	auipc	t1,0x1
    80000074:	89030313          	addi	t1,t1,-1904 # 80000900 <buf.0>
    80000078:	02628063          	beq	t0,t1,80000098 <_start+0x98>
    8000007c:	00001397          	auipc	t2,0x1
    80000080:	88438393          	addi	t2,t2,-1916 # 80000900 <buf.0>
    80000084:	00737a63          	bgeu	t1,t2,80000098 <_start+0x98>
    80000088:	0002be03          	ld	t3,0(t0)
    8000008c:	01c33023          	sd	t3,0(t1)
    80000090:	02a1                	addi	t0,t0,8
    80000092:	0321                	addi	t1,t1,8
    80000094:	fe736ae3          	bltu	t1,t2,80000088 <_start+0x88>
    80000098:	00001297          	auipc	t0,0x1
    8000009c:	86828293          	addi	t0,t0,-1944 # 80000900 <buf.0>
    800000a0:	00001317          	auipc	t1,0x1
    800000a4:	c7030313          	addi	t1,t1,-912 # 80000d10 <ebss>
    800000a8:	0062f763          	bgeu	t0,t1,800000b6 <_start+0xb6>
    800000ac:	0002b023          	sd	zero,0(t0)
    800000b0:	02a1                	addi	t0,t0,8
    800000b2:	fe62ede3          	bltu	t0,t1,800000ac <_start+0xac>
    800000b6:	aaf9                	j	80000294 <main>

00000000800000b8 <puts>:
    800000b8:	1141                	addi	sp,sp,-16
    800000ba:	e022                	sd	s0,0(sp)
    800000bc:	842a                	mv	s0,a0
    800000be:	00000517          	auipc	a0,0x0
    800000c2:	6f250513          	addi	a0,a0,1778 # 800007b0 <etext>
    800000c6:	e406                	sd	ra,8(sp)
    800000c8:	464000ef          	jal	ra,8000052c <uart_puts>
    800000cc:	f1402573          	csrr	a0,mhartid
    800000d0:	0305051b          	addiw	a0,a0,48
    800000d4:	0ff57513          	andi	a0,a0,255
    800000d8:	3bc000ef          	jal	ra,80000494 <uart_putc>
    800000dc:	00000517          	auipc	a0,0x0
    800000e0:	6e450513          	addi	a0,a0,1764 # 800007c0 <etext+0x10>
    800000e4:	448000ef          	jal	ra,8000052c <uart_puts>
    800000e8:	8522                	mv	a0,s0
    800000ea:	6402                	ld	s0,0(sp)
    800000ec:	60a2                	ld	ra,8(sp)
    800000ee:	0141                	addi	sp,sp,16
    800000f0:	a935                	j	8000052c <uart_puts>

00000000800000f2 <smp_memcpy>:
    800000f2:	ce19                	beqz	a2,80000110 <smp_memcpy+0x1e>
    800000f4:	962e                	add	a2,a2,a1
    800000f6:	87aa                	mv	a5,a0
    800000f8:	0140000f          	fence	w,o
    800000fc:	00058703          	lb	a4,0(a1)
    80000100:	0820000f          	fence	i,r
    80000104:	00e78023          	sb	a4,0(a5)
    80000108:	0585                	addi	a1,a1,1
    8000010a:	0785                	addi	a5,a5,1
    8000010c:	fec596e3          	bne	a1,a2,800000f8 <smp_memcpy+0x6>
    80000110:	8082                	ret

0000000080000112 <test_ipi>:
    80000112:	7179                	addi	sp,sp,-48
    80000114:	e44e                	sd	s3,8(sp)
    80000116:	f406                	sd	ra,40(sp)
    80000118:	f022                	sd	s0,32(sp)
    8000011a:	ec26                	sd	s1,24(sp)
    8000011c:	e84a                	sd	s2,16(sp)
    8000011e:	e052                	sd	s4,0(sp)
    80000120:	89aa                	mv	s3,a0
    80000122:	508000ef          	jal	ra,8000062a <clint_clear_soft>
    80000126:	304467f3          	csrrsi	a5,mie,8
    8000012a:	00801937          	lui	s2,0x801
    8000012e:	448d                	li	s1,3
    80000130:	0922                	slli	s2,s2,0x8
    80000132:	00000517          	auipc	a0,0x0
    80000136:	67e50513          	addi	a0,a0,1662 # 800007b0 <etext>
    8000013a:	3f2000ef          	jal	ra,8000052c <uart_puts>
    8000013e:	f1402573          	csrr	a0,mhartid
    80000142:	0305051b          	addiw	a0,a0,48
    80000146:	0ff57513          	andi	a0,a0,255
    8000014a:	34a000ef          	jal	ra,80000494 <uart_putc>
    8000014e:	00000517          	auipc	a0,0x0
    80000152:	67250513          	addi	a0,a0,1650 # 800007c0 <etext+0x10>
    80000156:	3d6000ef          	jal	ra,8000052c <uart_puts>
    8000015a:	00000517          	auipc	a0,0x0
    8000015e:	66e50513          	addi	a0,a0,1646 # 800007c8 <etext+0x18>
    80000162:	3ca000ef          	jal	ra,8000052c <uart_puts>
    80000166:	4501                	li	a0,0
    80000168:	5a2000ef          	jal	ra,8000070a <readline>
    8000016c:	00054403          	lbu	s0,0(a0)
    80000170:	fd04041b          	addiw	s0,s0,-48
    80000174:	fff40793          	addi	a5,s0,-1
    80000178:	02f4fd63          	bgeu	s1,a5,800001b2 <test_ipi+0xa0>
    8000017c:	00000517          	auipc	a0,0x0
    80000180:	63450513          	addi	a0,a0,1588 # 800007b0 <etext>
    80000184:	3a8000ef          	jal	ra,8000052c <uart_puts>
    80000188:	f1402573          	csrr	a0,mhartid
    8000018c:	0305051b          	addiw	a0,a0,48
    80000190:	0ff57513          	andi	a0,a0,255
    80000194:	300000ef          	jal	ra,80000494 <uart_putc>
    80000198:	00000517          	auipc	a0,0x0
    8000019c:	62850513          	addi	a0,a0,1576 # 800007c0 <etext+0x10>
    800001a0:	38c000ef          	jal	ra,8000052c <uart_puts>
    800001a4:	00000517          	auipc	a0,0x0
    800001a8:	64c50513          	addi	a0,a0,1612 # 800007f0 <etext+0x40>
    800001ac:	380000ef          	jal	ra,8000052c <uart_puts>
    800001b0:	b749                	j	80000132 <test_ipi+0x20>
    800001b2:	00000517          	auipc	a0,0x0
    800001b6:	5fe50513          	addi	a0,a0,1534 # 800007b0 <etext>
    800001ba:	372000ef          	jal	ra,8000052c <uart_puts>
    800001be:	f1402573          	csrr	a0,mhartid
    800001c2:	0305051b          	addiw	a0,a0,48
    800001c6:	0ff57513          	andi	a0,a0,255
    800001ca:	2ca000ef          	jal	ra,80000494 <uart_putc>
    800001ce:	00000517          	auipc	a0,0x0
    800001d2:	5f250513          	addi	a0,a0,1522 # 800007c0 <etext+0x10>
    800001d6:	356000ef          	jal	ra,8000052c <uart_puts>
    800001da:	00000517          	auipc	a0,0x0
    800001de:	62e50513          	addi	a0,a0,1582 # 80000808 <etext+0x58>
    800001e2:	34a000ef          	jal	ra,8000052c <uart_puts>
    800001e6:	4501                	li	a0,0
    800001e8:	522000ef          	jal	ra,8000070a <readline>
    800001ec:	8a2a                	mv	s4,a0
    800001ee:	45c000ef          	jal	ra,8000064a <strlen>
    800001f2:	00150613          	addi	a2,a0,1
    800001f6:	85d2                	mv	a1,s4
    800001f8:	854a                	mv	a0,s2
    800001fa:	4bc000ef          	jal	ra,800006b6 <memcpy>
    800001fe:	00000517          	auipc	a0,0x0
    80000202:	5b250513          	addi	a0,a0,1458 # 800007b0 <etext>
    80000206:	326000ef          	jal	ra,8000052c <uart_puts>
    8000020a:	f1402573          	csrr	a0,mhartid
    8000020e:	0305051b          	addiw	a0,a0,48
    80000212:	0ff57513          	andi	a0,a0,255
    80000216:	27e000ef          	jal	ra,80000494 <uart_putc>
    8000021a:	00000517          	auipc	a0,0x0
    8000021e:	5a650513          	addi	a0,a0,1446 # 800007c0 <etext+0x10>
    80000222:	30a000ef          	jal	ra,8000052c <uart_puts>
    80000226:	00000517          	auipc	a0,0x0
    8000022a:	5f250513          	addi	a0,a0,1522 # 80000818 <etext+0x68>
    8000022e:	2fe000ef          	jal	ra,8000052c <uart_puts>
    80000232:	8522                	mv	a0,s0
    80000234:	32a000ef          	jal	ra,8000055e <uart_put_hex>
    80000238:	8522                	mv	a0,s0
    8000023a:	3da000ef          	jal	ra,80000614 <clint_send_soft>
    8000023e:	344027f3          	csrr	a5,mip
    80000242:	8ba1                	andi	a5,a5,8
    80000244:	e799                	bnez	a5,80000252 <test_ipi+0x140>
    80000246:	10500073          	wfi
    8000024a:	344027f3          	csrr	a5,mip
    8000024e:	8ba1                	andi	a5,a5,8
    80000250:	dbfd                	beqz	a5,80000246 <test_ipi+0x134>
    80000252:	854e                	mv	a0,s3
    80000254:	3d6000ef          	jal	ra,8000062a <clint_clear_soft>
    80000258:	00000517          	auipc	a0,0x0
    8000025c:	55850513          	addi	a0,a0,1368 # 800007b0 <etext>
    80000260:	2cc000ef          	jal	ra,8000052c <uart_puts>
    80000264:	f1402573          	csrr	a0,mhartid
    80000268:	0305051b          	addiw	a0,a0,48
    8000026c:	0ff57513          	andi	a0,a0,255
    80000270:	224000ef          	jal	ra,80000494 <uart_putc>
    80000274:	00000517          	auipc	a0,0x0
    80000278:	54c50513          	addi	a0,a0,1356 # 800007c0 <etext+0x10>
    8000027c:	2b0000ef          	jal	ra,8000052c <uart_puts>
    80000280:	00000517          	auipc	a0,0x0
    80000284:	5c050513          	addi	a0,a0,1472 # 80000840 <etext+0x90>
    80000288:	2a4000ef          	jal	ra,8000052c <uart_puts>
    8000028c:	8522                	mv	a0,s0
    8000028e:	2d0000ef          	jal	ra,8000055e <uart_put_hex>
    80000292:	b545                	j	80000132 <test_ipi+0x20>

0000000080000294 <main>:
    80000294:	1101                	addi	sp,sp,-32
    80000296:	6671                	lui	a2,0x1c
    80000298:	20060613          	addi	a2,a2,512 # 1c200 <n+0x1c1e0>
    8000029c:	4581                	li	a1,0
    8000029e:	e426                	sd	s1,8(sp)
    800002a0:	84aa                	mv	s1,a0
    800002a2:	10010537          	lui	a0,0x10010
    800002a6:	ec06                	sd	ra,24(sp)
    800002a8:	e822                	sd	s0,16(sp)
    800002aa:	22a000ef          	jal	ra,800004d4 <uart_init>
    800002ae:	02000537          	lui	a0,0x2000
    800002b2:	38e000ef          	jal	ra,80000640 <clint_init>
    800002b6:	00000517          	auipc	a0,0x0
    800002ba:	5aa50513          	addi	a0,a0,1450 # 80000860 <etext+0xb0>
    800002be:	dfbff0ef          	jal	ra,800000b8 <puts>
    800002c2:	00000517          	auipc	a0,0x0
    800002c6:	5ae50513          	addi	a0,a0,1454 # 80000870 <etext+0xc0>
    800002ca:	defff0ef          	jal	ra,800000b8 <puts>
    800002ce:	12345537          	lui	a0,0x12345
    800002d2:	67850513          	addi	a0,a0,1656 # 12345678 <n+0x12345658>
    800002d6:	288000ef          	jal	ra,8000055e <uart_put_hex>
    800002da:	00000517          	auipc	a0,0x0
    800002de:	5ae50513          	addi	a0,a0,1454 # 80000888 <etext+0xd8>
    800002e2:	dd7ff0ef          	jal	ra,800000b8 <puts>
    800002e6:	4501                	li	a0,0
    800002e8:	422000ef          	jal	ra,8000070a <readline>
    800002ec:	c919                	beqz	a0,80000302 <main+0x6e>
    800002ee:	842a                	mv	s0,a0
    800002f0:	00000517          	auipc	a0,0x0
    800002f4:	5a850513          	addi	a0,a0,1448 # 80000898 <etext+0xe8>
    800002f8:	dc1ff0ef          	jal	ra,800000b8 <puts>
    800002fc:	8522                	mv	a0,s0
    800002fe:	22e000ef          	jal	ra,8000052c <uart_puts>
    80000302:	00000517          	auipc	a0,0x0
    80000306:	5ae50513          	addi	a0,a0,1454 # 800008b0 <etext+0x100>
    8000030a:	dafff0ef          	jal	ra,800000b8 <puts>
    8000030e:	8526                	mv	a0,s1
    80000310:	e03ff0ef          	jal	ra,80000112 <test_ipi>

0000000080000314 <trap_handler>:
    80000314:	8082                	ret

0000000080000316 <other_main>:
    80000316:	1101                	addi	sp,sp,-32
    80000318:	6671                	lui	a2,0x1c
    8000031a:	20060613          	addi	a2,a2,512 # 1c200 <n+0x1c1e0>
    8000031e:	4581                	li	a1,0
    80000320:	e822                	sd	s0,16(sp)
    80000322:	842a                	mv	s0,a0
    80000324:	10010537          	lui	a0,0x10010
    80000328:	ec06                	sd	ra,24(sp)
    8000032a:	e426                	sd	s1,8(sp)
    8000032c:	1a8000ef          	jal	ra,800004d4 <uart_init>
    80000330:	02000537          	lui	a0,0x2000
    80000334:	30c000ef          	jal	ra,80000640 <clint_init>
    80000338:	8522                	mv	a0,s0
    8000033a:	2f0000ef          	jal	ra,8000062a <clint_clear_soft>
    8000033e:	304467f3          	csrrsi	a5,mie,8
    80000342:	008014b7          	lui	s1,0x801
    80000346:	04a2                	slli	s1,s1,0x8
    80000348:	a019                	j	8000034e <other_main+0x38>
    8000034a:	10500073          	wfi
    8000034e:	344027f3          	csrr	a5,mip
    80000352:	8ba1                	andi	a5,a5,8
    80000354:	dbfd                	beqz	a5,8000034a <other_main+0x34>
    80000356:	8522                	mv	a0,s0
    80000358:	2d2000ef          	jal	ra,8000062a <clint_clear_soft>
    8000035c:	00000517          	auipc	a0,0x0
    80000360:	45450513          	addi	a0,a0,1108 # 800007b0 <etext>
    80000364:	1c8000ef          	jal	ra,8000052c <uart_puts>
    80000368:	f1402573          	csrr	a0,mhartid
    8000036c:	0305051b          	addiw	a0,a0,48
    80000370:	0ff57513          	andi	a0,a0,255
    80000374:	120000ef          	jal	ra,80000494 <uart_putc>
    80000378:	00000517          	auipc	a0,0x0
    8000037c:	44850513          	addi	a0,a0,1096 # 800007c0 <etext+0x10>
    80000380:	1ac000ef          	jal	ra,8000052c <uart_puts>
    80000384:	00000517          	auipc	a0,0x0
    80000388:	53c50513          	addi	a0,a0,1340 # 800008c0 <etext+0x110>
    8000038c:	1a0000ef          	jal	ra,8000052c <uart_puts>
    80000390:	00000517          	auipc	a0,0x0
    80000394:	42050513          	addi	a0,a0,1056 # 800007b0 <etext>
    80000398:	194000ef          	jal	ra,8000052c <uart_puts>
    8000039c:	f1402573          	csrr	a0,mhartid
    800003a0:	0305051b          	addiw	a0,a0,48
    800003a4:	0ff57513          	andi	a0,a0,255
    800003a8:	0ec000ef          	jal	ra,80000494 <uart_putc>
    800003ac:	00000517          	auipc	a0,0x0
    800003b0:	41450513          	addi	a0,a0,1044 # 800007c0 <etext+0x10>
    800003b4:	178000ef          	jal	ra,8000052c <uart_puts>
    800003b8:	00000517          	auipc	a0,0x0
    800003bc:	52850513          	addi	a0,a0,1320 # 800008e0 <etext+0x130>
    800003c0:	16c000ef          	jal	ra,8000052c <uart_puts>
    800003c4:	8526                	mv	a0,s1
    800003c6:	166000ef          	jal	ra,8000052c <uart_puts>
    800003ca:	4501                	li	a0,0
    800003cc:	248000ef          	jal	ra,80000614 <clint_send_soft>
    800003d0:	bfbd                	j	8000034e <other_main+0x38>

00000000800003d2 <wait_ipi>:
    800003d2:	1141                	addi	sp,sp,-16
    800003d4:	e406                	sd	ra,8(sp)
    800003d6:	344027f3          	csrr	a5,mip
    800003da:	8ba1                	andi	a5,a5,8
    800003dc:	e799                	bnez	a5,800003ea <wait_ipi+0x18>
    800003de:	10500073          	wfi
    800003e2:	344027f3          	csrr	a5,mip
    800003e6:	8ba1                	andi	a5,a5,8
    800003e8:	dbfd                	beqz	a5,800003de <wait_ipi+0xc>
    800003ea:	240000ef          	jal	ra,8000062a <clint_clear_soft>
    800003ee:	60a2                	ld	ra,8(sp)
    800003f0:	4501                	li	a0,0
    800003f2:	0141                	addi	sp,sp,16
    800003f4:	8082                	ret

00000000800003f6 <trap_entry>:
    800003f6:	34011173          	csrrw	sp,mscratch,sp
    800003fa:	7111                	addi	sp,sp,-256
    800003fc:	e002                	sd	zero,0(sp)
    800003fe:	e406                	sd	ra,8(sp)
    80000400:	340020f3          	csrr	ra,mscratch
    80000404:	e806                	sd	ra,16(sp)
    80000406:	ec0e                	sd	gp,24(sp)
    80000408:	f012                	sd	tp,32(sp)
    8000040a:	f416                	sd	t0,40(sp)
    8000040c:	f81a                	sd	t1,48(sp)
    8000040e:	fc1e                	sd	t2,56(sp)
    80000410:	e0a2                	sd	s0,64(sp)
    80000412:	e4a6                	sd	s1,72(sp)
    80000414:	e8aa                	sd	a0,80(sp)
    80000416:	ecae                	sd	a1,88(sp)
    80000418:	f0b2                	sd	a2,96(sp)
    8000041a:	f4b6                	sd	a3,104(sp)
    8000041c:	f8ba                	sd	a4,112(sp)
    8000041e:	fcbe                	sd	a5,120(sp)
    80000420:	e142                	sd	a6,128(sp)
    80000422:	e546                	sd	a7,136(sp)
    80000424:	e94a                	sd	s2,144(sp)
    80000426:	ed4e                	sd	s3,152(sp)
    80000428:	f152                	sd	s4,160(sp)
    8000042a:	f556                	sd	s5,168(sp)
    8000042c:	f95a                	sd	s6,176(sp)
    8000042e:	fd5e                	sd	s7,184(sp)
    80000430:	e1e2                	sd	s8,192(sp)
    80000432:	e5e6                	sd	s9,200(sp)
    80000434:	e9ea                	sd	s10,208(sp)
    80000436:	edee                	sd	s11,216(sp)
    80000438:	f1f2                	sd	t3,224(sp)
    8000043a:	f5f6                	sd	t4,232(sp)
    8000043c:	f9fa                	sd	t5,240(sp)
    8000043e:	fdfe                	sd	t6,248(sp)
    80000440:	850a                	mv	a0,sp
    80000442:	ed3ff0ef          	jal	ra,80000314 <trap_handler>
    80000446:	34151073          	csrw	mepc,a0
    8000044a:	10010293          	addi	t0,sp,256
    8000044e:	34029073          	csrw	mscratch,t0
    80000452:	60a2                	ld	ra,8(sp)
    80000454:	61e2                	ld	gp,24(sp)
    80000456:	7202                	ld	tp,32(sp)
    80000458:	72a2                	ld	t0,40(sp)
    8000045a:	7342                	ld	t1,48(sp)
    8000045c:	73e2                	ld	t2,56(sp)
    8000045e:	6406                	ld	s0,64(sp)
    80000460:	64a6                	ld	s1,72(sp)
    80000462:	6546                	ld	a0,80(sp)
    80000464:	65e6                	ld	a1,88(sp)
    80000466:	7606                	ld	a2,96(sp)
    80000468:	76a6                	ld	a3,104(sp)
    8000046a:	7746                	ld	a4,112(sp)
    8000046c:	77e6                	ld	a5,120(sp)
    8000046e:	680a                	ld	a6,128(sp)
    80000470:	68aa                	ld	a7,136(sp)
    80000472:	694a                	ld	s2,144(sp)
    80000474:	69ea                	ld	s3,152(sp)
    80000476:	7a0a                	ld	s4,160(sp)
    80000478:	7aaa                	ld	s5,168(sp)
    8000047a:	7b4a                	ld	s6,176(sp)
    8000047c:	7bea                	ld	s7,184(sp)
    8000047e:	6c0e                	ld	s8,192(sp)
    80000480:	6cae                	ld	s9,200(sp)
    80000482:	6d4e                	ld	s10,208(sp)
    80000484:	6dee                	ld	s11,216(sp)
    80000486:	7e0e                	ld	t3,224(sp)
    80000488:	7eae                	ld	t4,232(sp)
    8000048a:	7f4e                	ld	t5,240(sp)
    8000048c:	7fee                	ld	t6,248(sp)
    8000048e:	6142                	ld	sp,16(sp)
    80000490:	30200073          	mret

0000000080000494 <uart_putc>:
    80000494:	00001717          	auipc	a4,0x1
    80000498:	86c70713          	addi	a4,a4,-1940 # 80000d00 <uart_base>
    8000049c:	631c                	ld	a5,0(a4)
    8000049e:	439c                	lw	a5,0(a5)
    800004a0:	0820000f          	fence	i,r
    800004a4:	2781                	sext.w	a5,a5
    800004a6:	fe07cbe3          	bltz	a5,8000049c <uart_putc+0x8>
    800004aa:	0140000f          	fence	w,o
    800004ae:	631c                	ld	a5,0(a4)
    800004b0:	c388                	sw	a0,0(a5)
    800004b2:	8082                	ret

00000000800004b4 <uart_getc>:
    800004b4:	00001797          	auipc	a5,0x1
    800004b8:	84c7b503          	ld	a0,-1972(a5) # 80000d00 <uart_base>
    800004bc:	0511                	addi	a0,a0,4
    800004be:	4108                	lw	a0,0(a0)
    800004c0:	0820000f          	fence	i,r
    800004c4:	2501                	sext.w	a0,a0
    800004c6:	00054563          	bltz	a0,800004d0 <uart_getc+0x1c>
    800004ca:	0ff57513          	andi	a0,a0,255
    800004ce:	8082                	ret
    800004d0:	557d                	li	a0,-1
    800004d2:	8082                	ret

00000000800004d4 <uart_init>:
    800004d4:	00001797          	auipc	a5,0x1
    800004d8:	82c78793          	addi	a5,a5,-2004 # 80000d00 <uart_base>
    800004dc:	e388                	sd	a0,0(a5)
    800004de:	c185                	beqz	a1,800004fe <uart_init+0x2a>
    800004e0:	1602                	slli	a2,a2,0x20
    800004e2:	9201                	srli	a2,a2,0x20
    800004e4:	1582                	slli	a1,a1,0x20
    800004e6:	fff60713          	addi	a4,a2,-1
    800004ea:	9181                	srli	a1,a1,0x20
    800004ec:	95ba                	add	a1,a1,a4
    800004ee:	4681                	li	a3,0
    800004f0:	02c5f963          	bgeu	a1,a2,80000522 <uart_init+0x4e>
    800004f4:	0140000f          	fence	w,o
    800004f8:	6398                	ld	a4,0(a5)
    800004fa:	0761                	addi	a4,a4,24
    800004fc:	c314                	sw	a3,0(a4)
    800004fe:	0140000f          	fence	w,o
    80000502:	6398                	ld	a4,0(a5)
    80000504:	4681                	li	a3,0
    80000506:	0741                	addi	a4,a4,16
    80000508:	c314                	sw	a3,0(a4)
    8000050a:	0140000f          	fence	w,o
    8000050e:	6398                	ld	a4,0(a5)
    80000510:	4685                	li	a3,1
    80000512:	0721                	addi	a4,a4,8
    80000514:	c314                	sw	a3,0(a4)
    80000516:	0140000f          	fence	w,o
    8000051a:	639c                	ld	a5,0(a5)
    8000051c:	07b1                	addi	a5,a5,12
    8000051e:	c394                	sw	a3,0(a5)
    80000520:	8082                	ret
    80000522:	02c5d5b3          	divu	a1,a1,a2
    80000526:	fff5869b          	addiw	a3,a1,-1
    8000052a:	b7e9                	j	800004f4 <uart_init+0x20>

000000008000052c <uart_puts>:
    8000052c:	00054683          	lbu	a3,0(a0)
    80000530:	c695                	beqz	a3,8000055c <uart_puts+0x30>
    80000532:	00000717          	auipc	a4,0x0
    80000536:	7ce70713          	addi	a4,a4,1998 # 80000d00 <uart_base>
    8000053a:	631c                	ld	a5,0(a4)
    8000053c:	0505                	addi	a0,a0,1
    8000053e:	a011                	j	80000542 <uart_puts+0x16>
    80000540:	631c                	ld	a5,0(a4)
    80000542:	439c                	lw	a5,0(a5)
    80000544:	0820000f          	fence	i,r
    80000548:	2781                	sext.w	a5,a5
    8000054a:	fe07cbe3          	bltz	a5,80000540 <uart_puts+0x14>
    8000054e:	0140000f          	fence	w,o
    80000552:	631c                	ld	a5,0(a4)
    80000554:	c394                	sw	a3,0(a5)
    80000556:	00054683          	lbu	a3,0(a0)
    8000055a:	f2ed                	bnez	a3,8000053c <uart_puts+0x10>
    8000055c:	8082                	ret

000000008000055e <uart_put_hex>:
    8000055e:	00000717          	auipc	a4,0x0
    80000562:	7a270713          	addi	a4,a4,1954 # 80000d00 <uart_base>
    80000566:	631c                	ld	a5,0(a4)
    80000568:	00000697          	auipc	a3,0x0
    8000056c:	03000613          	li	a2,48
    80000570:	39068693          	addi	a3,a3,912 # 800008f8 <etext+0x148>
    80000574:	0685                	addi	a3,a3,1
    80000576:	a011                	j	8000057a <uart_put_hex+0x1c>
    80000578:	631c                	ld	a5,0(a4)
    8000057a:	439c                	lw	a5,0(a5)
    8000057c:	0820000f          	fence	i,r
    80000580:	2781                	sext.w	a5,a5
    80000582:	fe07cbe3          	bltz	a5,80000578 <uart_put_hex+0x1a>
    80000586:	0140000f          	fence	w,o
    8000058a:	631c                	ld	a5,0(a4)
    8000058c:	c390                	sw	a2,0(a5)
    8000058e:	0006c603          	lbu	a2,0(a3)
    80000592:	f26d                	bnez	a2,80000574 <uart_put_hex+0x16>
    80000594:	4671                	li	a2,28
    80000596:	48a5                	li	a7,9
    80000598:	5871                	li	a6,-4
    8000059a:	00c556bb          	srlw	a3,a0,a2
    8000059e:	8abd                	andi	a3,a3,15
    800005a0:	05768593          	addi	a1,a3,87
    800005a4:	00d8e663          	bltu	a7,a3,800005b0 <uart_put_hex+0x52>
    800005a8:	03068593          	addi	a1,a3,48
    800005ac:	a011                	j	800005b0 <uart_put_hex+0x52>
    800005ae:	631c                	ld	a5,0(a4)
    800005b0:	439c                	lw	a5,0(a5)
    800005b2:	0820000f          	fence	i,r
    800005b6:	2781                	sext.w	a5,a5
    800005b8:	fe07cbe3          	bltz	a5,800005ae <uart_put_hex+0x50>
    800005bc:	0140000f          	fence	w,o
    800005c0:	631c                	ld	a5,0(a4)
    800005c2:	c38c                	sw	a1,0(a5)
    800005c4:	3671                	addiw	a2,a2,-4
    800005c6:	fd061ae3          	bne	a2,a6,8000059a <uart_put_hex+0x3c>
    800005ca:	8082                	ret

00000000800005cc <clint_get_mtime>:
    800005cc:	00000797          	auipc	a5,0x0
    800005d0:	73c7b503          	ld	a0,1852(a5) # 80000d08 <clint_base>
    800005d4:	67b1                	lui	a5,0xc
    800005d6:	17e1                	addi	a5,a5,-8
    800005d8:	953e                	add	a0,a0,a5
    800005da:	6108                	ld	a0,0(a0)
    800005dc:	0820000f          	fence	i,r
    800005e0:	8082                	ret

00000000800005e2 <clint_set_timecmp>:
    800005e2:	0140000f          	fence	w,o
    800005e6:	00000797          	auipc	a5,0x0
    800005ea:	6705                	lui	a4,0x1
    800005ec:	7227b783          	ld	a5,1826(a5) # 80000d08 <clint_base>
    800005f0:	80070713          	addi	a4,a4,-2048 # 800 <n+0x7e0>
    800005f4:	953a                	add	a0,a0,a4
    800005f6:	050e                	slli	a0,a0,0x3
    800005f8:	953e                	add	a0,a0,a5
    800005fa:	e10c                	sd	a1,0(a0)
    800005fc:	8082                	ret

00000000800005fe <clint_check_soft>:
    800005fe:	00000797          	auipc	a5,0x0
    80000602:	70a7b783          	ld	a5,1802(a5) # 80000d08 <clint_base>
    80000606:	050a                	slli	a0,a0,0x2
    80000608:	953e                	add	a0,a0,a5
    8000060a:	4108                	lw	a0,0(a0)
    8000060c:	2501                	sext.w	a0,a0
    8000060e:	0820000f          	fence	i,r
    80000612:	8082                	ret

0000000080000614 <clint_send_soft>:
    80000614:	0140000f          	fence	w,o
    80000618:	00000797          	auipc	a5,0x0
    8000061c:	6f07b783          	ld	a5,1776(a5) # 80000d08 <clint_base>
    80000620:	050a                	slli	a0,a0,0x2
    80000622:	953e                	add	a0,a0,a5
    80000624:	4785                	li	a5,1
    80000626:	c11c                	sw	a5,0(a0)
    80000628:	8082                	ret

000000008000062a <clint_clear_soft>:
    8000062a:	0140000f          	fence	w,o
    8000062e:	00000797          	auipc	a5,0x0
    80000632:	6da7b783          	ld	a5,1754(a5) # 80000d08 <clint_base>
    80000636:	050a                	slli	a0,a0,0x2
    80000638:	953e                	add	a0,a0,a5
    8000063a:	4781                	li	a5,0
    8000063c:	c11c                	sw	a5,0(a0)
    8000063e:	8082                	ret

0000000080000640 <clint_init>:
    80000640:	00000797          	auipc	a5,0x0
    80000644:	6ca7b423          	sd	a0,1736(a5) # 80000d08 <clint_base>
    80000648:	8082                	ret

000000008000064a <strlen>:
    8000064a:	00054783          	lbu	a5,0(a0)
    8000064e:	872a                	mv	a4,a0
    80000650:	4501                	li	a0,0
    80000652:	cb81                	beqz	a5,80000662 <strlen+0x18>
    80000654:	0505                	addi	a0,a0,1
    80000656:	00a707b3          	add	a5,a4,a0
    8000065a:	0007c783          	lbu	a5,0(a5)
    8000065e:	fbfd                	bnez	a5,80000654 <strlen+0xa>
    80000660:	8082                	ret
    80000662:	8082                	ret

0000000080000664 <memset>:
    80000664:	ca01                	beqz	a2,80000674 <memset+0x10>
    80000666:	962a                	add	a2,a2,a0
    80000668:	87aa                	mv	a5,a0
    8000066a:	0785                	addi	a5,a5,1
    8000066c:	feb78fa3          	sb	a1,-1(a5)
    80000670:	fec79de3          	bne	a5,a2,8000066a <memset+0x6>
    80000674:	8082                	ret

0000000080000676 <memmove>:
    80000676:	02a5f263          	bgeu	a1,a0,8000069a <memmove+0x24>
    8000067a:	00c587b3          	add	a5,a1,a2
    8000067e:	00f57e63          	bgeu	a0,a5,8000069a <memmove+0x24>
    80000682:	00c50733          	add	a4,a0,a2
    80000686:	c615                	beqz	a2,800006b2 <memmove+0x3c>
    80000688:	fff7c683          	lbu	a3,-1(a5)
    8000068c:	17fd                	addi	a5,a5,-1
    8000068e:	177d                	addi	a4,a4,-1
    80000690:	00d70023          	sb	a3,0(a4)
    80000694:	fef59ae3          	bne	a1,a5,80000688 <memmove+0x12>
    80000698:	8082                	ret
    8000069a:	00c586b3          	add	a3,a1,a2
    8000069e:	87aa                	mv	a5,a0
    800006a0:	ca11                	beqz	a2,800006b4 <memmove+0x3e>
    800006a2:	0005c703          	lbu	a4,0(a1)
    800006a6:	0585                	addi	a1,a1,1
    800006a8:	0785                	addi	a5,a5,1
    800006aa:	fee78fa3          	sb	a4,-1(a5)
    800006ae:	fed59ae3          	bne	a1,a3,800006a2 <memmove+0x2c>
    800006b2:	8082                	ret
    800006b4:	8082                	ret

00000000800006b6 <memcpy>:
    800006b6:	ca19                	beqz	a2,800006cc <memcpy+0x16>
    800006b8:	962e                	add	a2,a2,a1
    800006ba:	87aa                	mv	a5,a0
    800006bc:	0005c703          	lbu	a4,0(a1)
    800006c0:	0585                	addi	a1,a1,1
    800006c2:	0785                	addi	a5,a5,1
    800006c4:	fee78fa3          	sb	a4,-1(a5)
    800006c8:	fec59ae3          	bne	a1,a2,800006bc <memcpy+0x6>
    800006cc:	8082                	ret

00000000800006ce <memcmp>:
    800006ce:	c205                	beqz	a2,800006ee <memcmp+0x20>
    800006d0:	962e                	add	a2,a2,a1
    800006d2:	a019                	j	800006d8 <memcmp+0xa>
    800006d4:	00c58d63          	beq	a1,a2,800006ee <memcmp+0x20>
    800006d8:	00054783          	lbu	a5,0(a0)
    800006dc:	0005c703          	lbu	a4,0(a1)
    800006e0:	0505                	addi	a0,a0,1
    800006e2:	0585                	addi	a1,a1,1
    800006e4:	fee788e3          	beq	a5,a4,800006d4 <memcmp+0x6>
    800006e8:	40e7853b          	subw	a0,a5,a4
    800006ec:	8082                	ret
    800006ee:	4501                	li	a0,0
    800006f0:	8082                	ret

00000000800006f2 <getchar>:
    800006f2:	1141                	addi	sp,sp,-16
    800006f4:	e022                	sd	s0,0(sp)
    800006f6:	e406                	sd	ra,8(sp)
    800006f8:	547d                	li	s0,-1
    800006fa:	dbbff0ef          	jal	ra,800004b4 <uart_getc>
    800006fe:	fe850ee3          	beq	a0,s0,800006fa <getchar+0x8>
    80000702:	60a2                	ld	ra,8(sp)
    80000704:	6402                	ld	s0,0(sp)
    80000706:	0141                	addi	sp,sp,16
    80000708:	8082                	ret

000000008000070a <readline>:
    8000070a:	715d                	addi	sp,sp,-80
    8000070c:	e486                	sd	ra,72(sp)
    8000070e:	e0a2                	sd	s0,64(sp)
    80000710:	fc26                	sd	s1,56(sp)
    80000712:	f84a                	sd	s2,48(sp)
    80000714:	f44e                	sd	s3,40(sp)
    80000716:	f052                	sd	s4,32(sp)
    80000718:	ec56                	sd	s5,24(sp)
    8000071a:	e85a                	sd	s6,16(sp)
    8000071c:	e45e                	sd	s7,8(sp)
    8000071e:	c119                	beqz	a0,80000724 <readline+0x1a>
    80000720:	e0dff0ef          	jal	ra,8000052c <uart_puts>
    80000724:	4901                	li	s2,0
    80000726:	54fd                	li	s1,-1
    80000728:	49fd                	li	s3,31
    8000072a:	4aa1                	li	s5,8
    8000072c:	4b29                	li	s6,10
    8000072e:	4bb5                	li	s7,13
    80000730:	3fe00a13          	li	s4,1022
    80000734:	d81ff0ef          	jal	ra,800004b4 <uart_getc>
    80000738:	fe950ee3          	beq	a0,s1,80000734 <readline+0x2a>
    8000073c:	06054563          	bltz	a0,800007a6 <readline+0x9c>
    80000740:	02a9d263          	bge	s3,a0,80000764 <readline+0x5a>
    80000744:	ff2a48e3          	blt	s4,s2,80000734 <readline+0x2a>
    80000748:	0ff57413          	andi	s0,a0,255
    8000074c:	8522                	mv	a0,s0
    8000074e:	d47ff0ef          	jal	ra,80000494 <uart_putc>
    80000752:	00000797          	auipc	a5,0x0
    80000756:	1ae78793          	addi	a5,a5,430 # 80000900 <buf.0>
    8000075a:	97ca                	add	a5,a5,s2
    8000075c:	00878023          	sb	s0,0(a5)
    80000760:	2905                	addiw	s2,s2,1
    80000762:	bfc9                	j	80000734 <readline+0x2a>
    80000764:	01551863          	bne	a0,s5,80000774 <readline+0x6a>
    80000768:	fc0906e3          	beqz	s2,80000734 <readline+0x2a>
    8000076c:	d29ff0ef          	jal	ra,80000494 <uart_putc>
    80000770:	397d                	addiw	s2,s2,-1
    80000772:	b7c9                	j	80000734 <readline+0x2a>
    80000774:	03650b63          	beq	a0,s6,800007aa <readline+0xa0>
    80000778:	fb751ee3          	bne	a0,s7,80000734 <readline+0x2a>
    8000077c:	4535                	li	a0,13
    8000077e:	d17ff0ef          	jal	ra,80000494 <uart_putc>
    80000782:	00000517          	auipc	a0,0x0
    80000786:	17e50513          	addi	a0,a0,382 # 80000900 <buf.0>
    8000078a:	992a                	add	s2,s2,a0
    8000078c:	00090023          	sb	zero,0(s2) # 801000 <n+0x800fe0>
    80000790:	60a6                	ld	ra,72(sp)
    80000792:	6406                	ld	s0,64(sp)
    80000794:	74e2                	ld	s1,56(sp)
    80000796:	7942                	ld	s2,48(sp)
    80000798:	79a2                	ld	s3,40(sp)
    8000079a:	7a02                	ld	s4,32(sp)
    8000079c:	6ae2                	ld	s5,24(sp)
    8000079e:	6b42                	ld	s6,16(sp)
    800007a0:	6ba2                	ld	s7,8(sp)
    800007a2:	6161                	addi	sp,sp,80
    800007a4:	8082                	ret
    800007a6:	4501                	li	a0,0
    800007a8:	b7e5                	j	80000790 <readline+0x86>
    800007aa:	4529                	li	a0,10
    800007ac:	bfc9                	j	8000077e <readline+0x74>
	...
