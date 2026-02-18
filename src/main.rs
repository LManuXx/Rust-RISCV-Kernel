// SPDX-License-Identifier: MIT

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Use the 'alloc' crate to enable dynamic memory types like Box, Vec, and String.
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::ptr;

// Load our low-level boot assembly.
core::arch::global_asm!(include_str!("../boot.s"));

mod allocator;
mod uart;

use crate::allocator::{LinkedListAllocator, Locked};
use crate::uart::UART;

// These symbols are defined in our linker script (linker.ld).
// We don't care about their value, only their address in memory.

unsafe extern "C" {
    static _heap_start: u8;
}

/// The global memory allocator. We wrap our LinkedListAllocator in a
/// spinlock-based Locked wrapper to ensure thread-safe access (essential for multicore).
#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

/// This handler is called when the allocator fails to find a suitable memory region.
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

/// Standard panic handler for our kernel.
/// Prints the error info to the UART before halting.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel Panic! -> {}", info);
    loop {}
}

/// The main entry point for the kernel after boot.s has set up the stack.
#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    println!("RISC-V Kernel Booting...");

    // Initialize the allocator using the heap boundary defined by the linker.
    unsafe {
        // Get the address of the _heap_start symbol.
        let heap_start = ptr::addr_of!(_heap_start) as usize;
        let aligned_start = (heap_start + 7) & !7;
        // Define the end of available RAM. For QEMU 'virt' machine,
        // 0x8800_0000 gives us 128MB of total space from the start of RAM.
        let memory_end = 0x8800_0000;
        let heap_size = memory_end - aligned_start;

        // Populate the free list with the available heap region.
        ALLOCATOR.lock().init(aligned_start, heap_size);

        println!("Memory Allocator initialized:");
        println!("  Start Address: 0x{:x}", heap_start);
        println!("  Total Size:    {} KB", heap_size / 1024);
    }

    // Run a quick stress test to ensure pointers and list logic are working.
    test_dynamic_allocation();

    println!("System ready. Echo mode active:");
    loop {
        // Simple UART echo loop.
        let c = UART.lock().get_byte();
        match c {
            13 => println!(), // Handle Carriage Return
            _ => print!("{}", c as char),
        }
    }
}

/// Verification function to test Box, Vec, and String allocation/deallocation.
fn test_dynamic_allocation() {
    println!("Running heap tests...");

    // Testing Box (Single allocation)
    let x = Box::new(42);
    println!("  Box value: {} at {:p}", *x, x);

    // Testing Vec (Multiple allocations and potential resizing)
    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }
    println!("  Vector: {:?}", v);

    // Testing String (Heap-allocated byte buffer)
    let s = String::from("Hello from the Heap!");
    println!("  String: '{}'", s);

    // As 'x', 'v', and 's' go out of scope here, the 'dealloc' calls
    // will trigger, returning those regions to the linked list.
    println!("Heap tests completed successfully.");
}
