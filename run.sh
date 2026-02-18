#!/bin/bash

# --- Kernel Build & Run Script ---

# 1. Compile the project
# We use the standard cargo build. The target and linker settings
# are automatically pulled from .cargo/config.toml.
echo "Building kernel..."
cargo build

# 2. Execute in QEMU
# This launches the emulator with the following configuration:
# -machine virt: Emulates the RISC-V VirtIO board.
# -nographic: Disables the VGA window, routing all output to the terminal.
# -serial mon:stdio: Combines the UART output and QEMU monitor in this console.
# -kernel: Points to the newly compiled ELF binary.
echo "Launching QEMU..."
qemu-system-riscv64 \
  -machine virt \
  -nographic \
  -serial mon:stdio \
  -kernel target/riscv64gc-unknown-none-elf/debug/mi_kernel