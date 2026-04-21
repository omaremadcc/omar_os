#![no_main]
#![no_std]
use core::arch::asm;
use core::panic::PanicInfo;

use blog_os::println;

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    blog_os::hlt_loop();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {

    blog_os::init();

    println!("Omar emad");

    blog_os::hlt_loop();
}
