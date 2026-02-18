// SPDX-License-Identifier: MIT

pub mod global_impl;

pub use self::global_impl::Locked;
use core::mem;
use core::ptr;

/// Alignment constant for 64-bit architecture (8 bytes).
const WORD_64: usize = 8;

/// Represents a hole in the heap memory.
/// These nodes are stored physically inside the free memory blocks themselves.
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

/// Helper enum to categorize how a requested size fits into a free block.
enum FitResult {
    /// The block matches the request perfectly (after alignment).
    Perfect(usize),
    /// The block is larger than needed; it can be split into used and free parts.
    Split(usize),
}

pub struct LinkedListAllocator {
    /// Sentinel head node that simplifies list operations (insertion/deletion).
    /// Its size is always 0 and it never gets allocated.
    head: ListNode,
}

impl LinkedListAllocator {
    /// Creates an empty allocator with no managed memory.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initializes the allocator with a raw memory range.
    /// # Safety
    /// The caller must ensure the memory range is valid and not used elsewhere.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size);
        }
    }

    /// Searches for a suitable free region that meets size and alignment requirements.
    /// Returns the starting address if found.
    fn find_region(&mut self, size: usize, align: usize) -> Option<usize> {
        let mut current_node = &mut self.head;

        // Iterate through the linked list of free blocks
        while let Some(ref mut next_node) = current_node.next {
            match next_node.can_fit(size, align) {
                Some(FitResult::Perfect(addr)) => {
                    // Perfect match: remove this node from the list entirely.
                    // We connect the previous node to whatever follows the current one.
                    current_node.next = next_node.next.take();
                    return Some(addr);
                }

                Some(FitResult::Split(addr)) => {
                    // Split match: user gets the start, and the remainder becomes a new node.
                    let alloc_end = addr + size;

                    let new_free_addr = align_up(alloc_end, mem::align_of::<ListNode>());

                    let new_free_size = next_node.end_addr() - new_free_addr;

                    unsafe {
                        let next_ptr = next_node.next.take();
                        let new_node_ptr = new_free_addr as *mut ListNode;

                        // Ahora new_node_ptr SIEMPRE será múltiplo de 8
                        ptr::write_volatile(
                            new_node_ptr,
                            ListNode {
                                size: new_free_size,
                                next: next_ptr,
                            },
                        );

                        current_node.next = Some(&mut *new_node_ptr);
                    }
                    return Some(addr);
                }

                None => {
                    // No fit in this block, move to the next one.
                    current_node = current_node.next.as_mut().unwrap();
                }
            }
        }
        None
    }

    /// Adds a chunk of memory to the free list, maintaining address order.
    /// # Safety
    /// The memory range must be valid and its ownership must be transferred here.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        let aligned_addr = align_up(addr, WORD_64);
        let current_size = addr + size - aligned_addr;
        let mut current = &mut self.head;

        // Discard the region if it's too small to even hold a ListNode header.
        if current_size < mem::size_of::<ListNode>() {
            return;
        }

        let new_node_ptr = aligned_addr as *mut ListNode;
        let new_node = unsafe {
            // Write the new node into the start of the free region.
            ptr::write(
                new_node_ptr,
                ListNode {
                    size: current_size,
                    next: None,
                },
            );
            &mut *new_node_ptr
        };

        // Traverse the list to find the correct insertion point (sorted by address).
        while let Some(ref mut node) = current.next {
            if node.start_addr() > aligned_addr {
                break;
            }
            current = current.next.as_mut().unwrap();
        }

        // Insert the new node between 'current' and 'current.next'.
        new_node.next = current.next.take();
        current.next = Some(new_node);
    }
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Returns the starting memory address of this node.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Returns the end address (exclusive) of the memory block managed by this node.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }

    /// Checks if a request for 'size' with 'align' can be satisfied by this block.
    fn can_fit(&self, size: usize, align: usize) -> Option<FitResult> {
        let aligned_address = align_up(self.start_addr(), align);
        let end_alloc = aligned_address + size;

        if end_alloc == self.end_addr() {
            // Fits exactly till the end of the block.
            Some(FitResult::Perfect(aligned_address))
        } else if end_alloc + mem::size_of::<ListNode>() <= self.end_addr() {
            // Fits with enough space left over to store a new ListNode header.
            Some(FitResult::Split(aligned_address))
        } else {
            // Either doesn't fit or the leftover fragment is too small to manage.
            None
        }
    }

    /// Relocates a node to a new address. Useful when splitting a block.
    /// # Safety
    /// Caller must ensure 'new_addr' is valid and doesn't overlap with active allocations.
    unsafe fn update_node(&mut self, new_size: usize, new_addr: usize) -> usize {
        let ptr_addr = new_addr as *mut ListNode;
        let next_node = self.next.take();
        let new_node = unsafe {
            core::ptr::write_volatile(
                ptr_addr,
                ListNode {
                    size: new_size,
                    next: next_node,
                },
            );
            &mut *ptr_addr
        };

        new_node.start_addr()
    }
}

/// Aligns the given address upwards to the nearest multiple of 'align'.
/// 'align' must be a power of two.
pub fn align_up(addr: usize, align: usize) -> usize {
    let reminder = addr % align;
    if reminder == 0 {
        addr
    } else {
        addr + align - reminder
    }
}
