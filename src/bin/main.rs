#![no_main]
#![no_std]
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

use blog_os::println;

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    blog_os::hlt_loop();
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{PageTable, Translate, Page};
    use blog_os::memory;
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::new(&boot_info.memory_map)
    };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    println!("It did not crash!");

    blog_os::hlt_loop();
}
