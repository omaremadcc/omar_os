use core::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use spin::Mutex;


struct Alloc {
    inner_heap: Mutex<InnerHeap>,
}
struct InnerHeap {
    head: Option<*mut FreeListNode>,
    size: usize,
    used: usize,
}
unsafe impl Send for InnerHeap {}
unsafe impl Sync for InnerHeap {}

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, size: Layout) -> *mut u8 {
        // println!("Allocating");
        let mut heap = self.inner_heap.lock();
        let requested_size = ((size.size() + 7) & !7) + mem::size_of::<FreeListNode>();
        let min_split_size = requested_size + mem::size_of::<FreeListNode>();
        // println!("heap size: {}, used: {}", heap.size, heap.used);
        if let Some(head_ptr) = heap.head
            && heap.size - heap.used >= requested_size
        {
            let mut prev_ptr = None;
            let mut ptr = head_ptr;
            let mut node: FreeListNode = FreeListNode::from_data_ptr(head_ptr);
            while node.size < requested_size {
                if let Some(next) = node.next {
                    prev_ptr = Some(ptr);
                    ptr = next;
                    node = FreeListNode::from_data_ptr(next);
                } else {
                    return 0 as *mut u8;
                }
            }

            let aligned_offset = requested_size;
            if node.size >= min_split_size {
                let node = Some(FreeListNode {
                    size: node.size - aligned_offset,
                    next: node.next,
                });
                unsafe {
                    let new_ptr = ((ptr as *mut FreeListNode) as usize) + aligned_offset;
                    if let Some(prev_ptr) = prev_ptr {
                        let reference = &mut *prev_ptr;
                        reference.next = Some(new_ptr as *mut FreeListNode);
                    } else {
                        heap.head = Some(new_ptr as *mut FreeListNode);
                    }
                    ptr::write(new_ptr as *mut FreeListNode, node.unwrap());
                }
            } else {
                if let Some(prev_ptr) = prev_ptr {
                    unsafe {
                        let reference = &mut *prev_ptr;
                        reference.next = node.next;
                    }
                } else {
                    heap.head = node.next;
                }
            }
            heap.used += requested_size;
            let new_ptr = (ptr as usize) + mem::size_of::<FreeListNode>();
            return new_ptr as *mut u8;
        } else {
            return 0 as *mut u8;
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, size: Layout) {
        println!(
            "Looking like some mf want to dealloc, with size {}",
            size.size()
        );
        let mut heap = self.inner_heap.lock();
        let mut ptr = ptr as usize - mem::size_of::<FreeListNode>();
        let mut ptr = ptr as *mut FreeListNode;
        let node: FreeListNode = unsafe { *ptr };
        let mut current = heap.head;
        while let Some(node) = current {
            let node: FreeListNode = unsafe { *node };
            if let Some(next) = node.next {
                if next > (ptr as *mut FreeListNode) {
                    break;
                }
            }
            current = node.next as Option<*mut FreeListNode>;
        }
        let current_node: FreeListNode = unsafe { *(current.unwrap()) };
        current_node.next = Some(ptr as *mut FreeListNode);
        let new_node = FreeListNode {
            size: size.size(),
            next: node.next,
        };
        unsafe {
            ptr::write(ptr as *mut FreeListNode, new_node);
        }
    }
}

impl Alloc {
    fn init(&self, heap_start: usize, heap_size: usize) {
        let head_free_list_node = FreeListNode {
            size: heap_size,
            next: None,
        };
        unsafe {
            ptr::write(heap_start as *mut FreeListNode, head_free_list_node);
        }
        let free_list_head = Some(heap_start as *mut FreeListNode);
        let mut inner_heap = self.inner_heap.lock();

        inner_heap.head = free_list_head;
        inner_heap.size = heap_size;
        inner_heap.used = 0;
    }
}

#[global_allocator]
static ALLOC: Alloc = Alloc {
    inner_heap: Mutex::new(InnerHeap {
        head: None,
        size: 0,
        used: 0,
    }),
};

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 + 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        let page_range = Page::range_inclusive(heap_start_page, heap_end_page);
        page_range
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator) }?.flush();
    }

    ALLOC.init(HEAP_START, HEAP_SIZE);

    Ok(())
}

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

use crate::println;
