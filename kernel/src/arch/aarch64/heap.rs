use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the global heap allocator.
///
/// # Safety
/// The caller must ensure that the memory range `[heap_start, heap_start + heap_size)`
/// is valid, mapped, and not used by anything else.
pub unsafe fn init(heap_start: usize, heap_size: usize) {
    ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
}
