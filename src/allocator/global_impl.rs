use super::LinkedListAllocator;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A generic wrapper that provides interior mutability and thread-safety.
/// Since the GlobalAlloc trait only provides immutable references (&self),
/// we use a Spinlock (spin::Mutex) to safely wrap our allocator and allow
/// mutable operations on the free list.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    /// Wraps a given allocator in a Spinlock.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    /// Acquires the lock to gain mutable access to the underlying allocator.
    /// This will block (spin) if another core is currently allocating memory.
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Implementation of the GlobalAlloc trait.
/// This allows the Rust compiler to use our LinkedListAllocator for
/// heap allocations (Box, Vec, String, etc.).
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    /// The entry point for all heap allocations.
    /// It locks the allocator and searches the linked list for a suitable free region.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Acquire the lock to ensure exclusive access to the linked list.
        let mut allocator = self.lock();

        // Attempt to find a region that matches the requested size and alignment.
        match allocator.find_region(layout.size(), layout.align()) {
            Some(addr) => addr as *mut u8, // Success: return the raw pointer.
            None => ptr::null_mut(),       // Failure: return null pointer (OOM).
        }
    }

    /// The entry point for returning memory to the heap.
    /// It locks the allocator and puts the region back into the free list.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Acquire the lock before modifying the list nodes.
        let mut allocator = self.lock();

        // Safety: We trust the pointer and layout provided by the Rust compiler.
        // We cast the pointer back to a numerical address and re-add it as a free region.
        unsafe {
            allocator.add_free_region(ptr as usize, layout.size());
        }
    }
}
