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
    use x86_64::structures::paging::{PageTable, Translate};
    use blog_os::memory;
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { mapper.translate_addr(virt) };
        println!("{:?} -> {:?}", virt, phys);
    }    println!("It did not crash!");
    blog_os::hlt_loop();
}
