use core::fmt::{self, Write};

use uart_16550::MmioSerialPort;

/// 关机。
#[inline]
pub unsafe fn shutdown(code: u16) {
    const TEST_DEV: usize = 0x100000;
    let code = if code == 0 {
        0x5555
    } else {
        ((code as u32) << 16) | 0x3333
    };
    core::arch::asm!("sw {}, ({})", in(reg) code, in(reg) TEST_DEV);
}

// 打印到串口。

struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut uart = unsafe { MmioSerialPort::new(0x1000_0000) };
        for c in s.bytes() {
            uart.send(c);
        }
        Ok(())
    }
}

#[inline]
pub fn _print(args: fmt::Arguments) {
    Uart.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::hal::_print(core::format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::hal::_print(core::format_args!($($arg)*));
        $crate::println!();
    }}
}
