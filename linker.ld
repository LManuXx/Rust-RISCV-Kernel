/* Set the target architecture to RISC-V */
OUTPUT_ARCH( "riscv" )

/* Define the hardware entry point; this matches the label in boot.s */
ENTRY( _start )

SECTIONS
{
    /* * Base address for RAM on QEMU's 'virt' machine.
     * All memory addresses will be calculated relative to this start point.
     */
    . = 0x80000000;

    /* * Executable code goes here. 
     * We ensure the init code (entry point) is placed at the very beginning.
     */
    .text : {
        *(.text.init)
        *(.text .text.*)
    }

    /* Read-only data like string literals and constants */
    .rodata : { *(.rodata .rodata.*) }

    /* Initialized global variables */
    .data : { *(.data .data.*) }

    /* Uninitialized global variables (zero-initialized by the bootloader/kernel) */
    .bss : { *(.bss .bss.*) }

    /* * Stack allocation:
     * We align to 8 bytes to comply with the RISC-V 64-bit calling convention.
     * We reserve 16KB (0x4000) for the initial kernel stack.
     */
    . = ALIGN(8);
    . += 0x4000;
    _stack_top = .;

    /* * Heap initialization:
     * We align the start of the heap to a 4096-byte page boundary.
     * This is crucial if we decide to implement hardware paging later.
     * The symbol _heap_start is exported to Rust to define the allocator's boundary.
     */
    . = ALIGN(4096);
    _heap_start = .;
}