```text
  _____  _    _  _____ _______ __      __
 |  __ \| |  | |/ ____|__   __|\ \    / /
 | |__) | |  | | (___    | |    \ \  / / 
 |  _  /| |  | |\___ \   | |     \ \/ /  
 | | \ \| |__| |____) |  | |      \  /   
 |_|  \_\\____/|_____/   |_|       \/    
                                         
 [ RISC-V 64-BIT BARE-METAL KERNEL ]
``` 
![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)
![Target](https://img.shields.io/badge/arch-RISCV--64-blue?logo=riscv)
![Status](https://img.shields.io/badge/status-active-success)

# Rust RISC-V Mini-Kernel

A minimalist, bare-metal kernel written in Rust for the RISC-V architecture (RV64GC). This project explores low-level systems programming, memory management, and hardware abstraction without a standard library (`no_std`).

## Current Features

- **Custom Bootloader**: A minimal `boot.s` assembly entry point that handles multi-core parking (hart management) and stack initialization.
- **UART Driver**: A functional driver for the 16550a UART device, allowing serial communication (input/output) through QEMU's virtual console.
- **Dynamic Memory Management**: 
    - Implementation of a **Linked List Allocator** from scratch.
    - Support for `FitResult` logic (Perfect vs. Split fits) to minimize fragmentation.
    - Integration with Rust's `GlobalAlloc` trait, enabling the use of `alloc` types like `Box`, `Vec`, and `String`.
- **Thread Safety**: All global resources (UART and Allocator) are wrapped in a `spin::Mutex` to ensure safe access in a concurrent environment.
- **Flexible Memory Layout**: Utilizes a custom Linker Script (`linker.ld`) to define memory sections and dynamically export heap boundaries to the kernel.

## Project Structure

* `src/main.rs`: Kernel entry point and hardware initialization logic.
* `src/allocator.rs`: Core logic for the Linked List memory allocator.
* `src/allocator/global_impl.rs`: Rust `GlobalAlloc` interface and Spinlock wrapper.
* `src/uart.rs`: UART 16550a driver and logging macros (`print!`, `println!`).
* `boot.s`: RISC-V assembly for initial CPU setup.
* `linker.ld`: Memory layout definition.

## How to Run

Ensure you have the `riscv64gc-unknown-none-elf` target installed via rustup.

1. **Build and Run**:
   ```bash
   chmod +x run.sh
   ./run.sh
   ```

Alternatively, use `cargo run` if the `.cargo/config.toml` is configured. 

## 🚀 Boot Sequence
1. **QEMU** loads the binary at `0x8000_0000`.
2. **`boot.s`**:
    - Disables multicore harts (parking them) to avoid race conditions.
    - Sets up the initial Stack Pointer (`sp`).
    - Jumps to Rust's `kmain`.
3. **`main.rs`**:
    - Initializes the **UART** for console output.
    - Resolves the heap boundaries using **Linker Symbols**.
    - Initializes the **LinkedListAllocator**.
4. **Kernel Space**: Enters the main echo loop and waits for user input.

## 📦 Key Dependencies
- [`spin`](https://crates.io/crates/spin): Providing the `Mutex` wrapper for safe access to hardware in a `no_std` environment (Spinlocks).
- [`alloc`](https://doc.rust-lang.org/alloc/): The core Rust library for heap-allocated structures, enabled by our custom allocator.

## 🛠 Hardware Support
| Feature | Status | Description |
| :--- | :---: | :--- |
| **RV64GC ISA** | ✅ | Base instructions & Compressed sets |
| **UART 16550a** | ✅ | Serial communication & Logging |
| **Dynamic Allocator** | ✅ | Linked List Heap management |
| **PLICS/CLINT** | 🚧 | Interrupt Controller (Work in Progress) |
| **Paging/MMU** | 📅 | Planned: Sv39 Virtual Memory |
| **Multicore** | ❌ | Planned: SMP Bootstrapping |

## 🗺 Memory Map Layout
```text
0x8000_0000 +------------------+
            |  .text (Kernel)  |
            +------------------+
            |  .rodata / .data |
            +------------------+
            |  .bss            |
            +------------------+
            |  Stack (16 KiB)  |
_heap_start +------------------+
            |                  |
            |  Dynamic Heap    |
            |                  |
0x8800_0000 +------------------+
```

## 🖥 Quick Look
When you run `cargo run`, you should see the kernel bootstrapping:
```bash
RISC-V Kernel Booting...
Memory Allocator initialized:
  Start Address: 0x8000c000
  Total Size:    131024 KB
Running heap tests...
  Box value: 42 at 0x8000c008
Heap tests passed! Memory freed.
```

### Disclaimer
**Please note:** While the core logic and implementation of this code were developed exclusively by me, the comments and documentation provided within the source files were generated with the assistance of an AI. Although they have been carefully reviewed for accuracy, they may still contain errors or inconsistencies.

If you spot any bugs, technical inaccuracies, or have improvements for the documentation, please feel free to open a Pull Request or an issue. Contributions are more than welcome!