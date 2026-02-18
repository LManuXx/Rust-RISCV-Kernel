// SPDX-License-Identifier: MIT

use core::fmt::{self, Write};
use spin::Mutex;

/// UART Register offsets and bitmasks for a standard 16550a device.
const LSR: usize = 5; // Line Status Register: gives us the state of the FIFO.
const LSR_DATA_READY: u8 = 1 << 0; // Bit 0: Set when there is a byte waiting to be read.
const LSR_TX_EMPTY: u8 = 1 << 5; // Bit 5: Set when the transmitter is ready for a new byte.

/// Core structure for the Universal Asynchronous Receiver-Transmitter.
pub struct Uart {
    /// Base MMIO address where the UART device is mapped in the memory space.
    base_address: usize,
}

impl Uart {
    /// Creates a new UART driver instance pointing to a specific MMIO address.
    pub const fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    /// Sends a single byte to the UART.
    /// It polls the Line Status Register until the hardware is ready to transmit.
    pub fn put_byte(&self, byte: u8) {
        let lsr_ptr = (self.base_address + LSR) as *mut u8;

        unsafe {
            // Wait for the Transmit Holding Register to be empty.
            // We use volatile reads to ensure the CPU actually checks the hardware status every time.
            while (core::ptr::read_volatile(lsr_ptr) & LSR_TX_EMPTY) == 0 {
                core::hint::spin_loop();
            }
        }

        let data_ptr = self.base_address as *mut u8;
        unsafe {
            // Write the data byte to the transmitter register.
            core::ptr::write_volatile(data_ptr, byte);
        }
    }

    /// Receives a single byte from the UART.
    /// This is a blocking operation that polls until data is available.
    pub fn get_byte(&self) -> u8 {
        let lsr_ptr = (self.base_address + LSR) as *mut u8;

        unsafe {
            // Block until the 'Data Ready' bit is set.
            while (core::ptr::read_volatile(lsr_ptr) & LSR_DATA_READY) == 0 {
                core::hint::spin_loop();
            }
        }

        let data_ptr = self.base_address as *mut u8;
        unsafe {
            // Read the received byte from the data register.
            core::ptr::read_volatile(data_ptr)
        }
    }
}

/// Implementation of core::fmt::Write allows us to use the print!/println! macros
/// by piping formatted strings into our put_byte function.
impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.put_byte(byte);
        }
        Ok(())
    }
}

/// Global UART instance wrapped in a Mutex.
/// In QEMU's RISC-V 'virt' machine, the UART is mapped at address 0x10000000.
pub static UART: Mutex<Uart> = Mutex::new(Uart::new(0x1000_0000));

/// Internal helper for the print macros.
/// It acquires the global UART lock and writes the formatted arguments.
pub fn _print(args: fmt::Arguments) {
    UART.lock()
        .write_fmt(args)
        .expect("Failed to write to UART");
}

/// Standard print! macro, exported for global use.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

/// Standard println! macro, exported for global use.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
