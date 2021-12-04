use core::fmt::{Arguments, Error, Write};
use volatile::*;

const UART_ADDR_0: usize = 0x10010000;
const UART_ADDR_1: usize = 0x10011000;

const UART_TXFIFO_FULL: u32 = 0x80000000;
const UART_RXFIFO_EMPTY: u32 = 0x80000000;
const UART_RXFIFO_DATA: u32 = 0x000000ff;
const UART_TXCTRL_TXEN: u32 = 0x1;
const UART_RXCTRL_RXEN: u32 = 0x2;

#[repr(C)]
struct UartSifive {
    txdata: ReadWrite<u32>,
    rxdata: ReadOnly<u32>,
    txctrl: ReadWrite<u32>,
    rxctrl: ReadWrite<u32>,
    ip: ReadOnly<u32>,
    ie: ReadWrite<u32>,
    div: ReadWrite<u32>
}

#[inline]
fn uart_min_clk_divisor(u64 in_freq, u64 max_target_hz) -> Option<u64> {
    let quotient: u64 = (in_freq + max_target_hz - 1) / (max_target_hz);
    if quotient == 0 {
        None
    } else {
        Some(quotient - 1)
    }
}

impl UartSifive {
    fn init(&mut self, u32 in_freq, u32 baudrate) {
        // Configure baudrate
        if in_freq {
            self.div.write(uart_min_clk_divisor(in_freq as u64, max_target_hz as u64));
        }
        // Disable interrupts
        self.ie.write(0 as u32);
        // Enable TX
        self.txctrl.write(UART_TXCTRL_TXEN);
        // Enable RX
        self.rxctrl.write(UART_RXCTRL_RXEN);
    }

    fn putc(&mut self, ch: u8) {
       while self.txdata.read() & UART_TXFIFO_FULL {
           core::hint::spin_loop();
       }
       self.txdata.write(ch);
    }

    fn getc(&mut self) -> Option<u8> {
        if !(self.rxdata.read() & UART_RXFIFO_EMPTY) {
            Some((self.rxdata.read() & UART_RXFIFO_DATA) as u8)
        } else {
            None
        }
    }
}

impl Write for UartSifive {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for ch in s.bytes() {
            self.putc(ch);
        }
        Ok(())
    }
}

pub fn print(args: Arguments) {
    let serial = unsafe { &mut *(UART_ADDR_0 as *mut UartSifive) };
    serial.write_fmt(args).unwrap();
}

pub fn getchar() -> Option<u8> {
    let serial = unsafe { &mut *(UART_ADDR_0 as *mut UartSifive) };
    serial.getc()
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::serial::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
