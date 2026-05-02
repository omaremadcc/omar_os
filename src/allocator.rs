use core::alloc::{GlobalAlloc, Layout};

struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, size: Layout) -> *mut u8 {
        let ptr = 0;
        return ptr as *mut u8;
    }
    unsafe fn dealloc(&self, ptr: *mut u8, size: Layout) {

    }
}

#[global_allocator]
static ALLOC: Alloc = Alloc {};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB, FrameAllocator, Mapper};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)

    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator) }?.flush();
    };

    Ok(())
}
