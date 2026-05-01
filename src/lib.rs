#![feature(abi_x86_interrupt)]
#![no_std]
pub mod interrupts;
pub mod vga_buffer;


pub fn init() {
    use crate::interrupts;
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable();
}
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
