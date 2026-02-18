# Target architecture: RISC-V 64-bit with General and Compressed extensions.
.attribute arch, "rv64gc" 

# We place this code in a special section defined in our linker script 
# to ensure it sits at the very beginning of the executable.
.section .text.init

# Disable compressed instructions for the boot sequence to ensure 
# predictable instruction alignment.
.option norvc 

.global _start

_start:
    # --- Multicore Management ---
    # Read the 'mhartid' (Machine Hart ID) Control and Status Register.
    # Each CPU core (hart) has a unique ID, starting from 0.
    csrr t0, mhartid
    
    # If the ID is not 0, send the core to the 'park' loop.
    # We only want the primary core to perform system initialization.
    bnez t0, park

    # --- Stack Setup ---
    # Load the address of '_stack_top' into the Stack Pointer (sp).
    # This symbol is defined at the end of our linker script.
    la sp, _stack_top
    
    # --- Transfer to Rust ---
    # Jump to the 'kmain' function in our Rust code.
    # Since 'kmain' never returns, we don't use 'call' (which would save a return address).
    j kmain

park:
    # Core parking loop: Wait For Interrupt (wfi) to save power, 
    # then jump back to itself if it ever wakes up.
    wfi
    j park