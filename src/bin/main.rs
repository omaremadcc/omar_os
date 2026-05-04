#![no_main]
#![no_std]
use alloc::rc::Rc;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

use blog_os::{allocator::init_heap, println};

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    blog_os::hlt_loop();
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use blog_os::memory;
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = memory::BootInfoFrameAllocator::new(&boot_info.memory_map);

    let _ = init_heap(&mut mapper, &mut frame_allocator);
    use alloc::boxed::Box;
    use alloc::{vec, vec::{Vec}};
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec: Vec<i32> = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    println!("It did not crash!");

    blog_os::hlt_loop();
}

extern crate alloc;
