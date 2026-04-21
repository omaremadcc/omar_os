#![feature(abi_x86_interrupt)]
#![no_std]
pub mod interrupts;
pub mod vga_buffer;


pub fn init() {
    use crate::interrupts;
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable();
    unsafe { init_mouse(); }
}
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
pub unsafe fn init_mouse() {
    use x86_64::instructions::port::Port;

    let mut cmd_port = Port::new(0x64);
    let mut data_port = Port::new(0x60);

    // 1. Enable the mouse channel
    cmd_port.write(0xA8 as u8);

    // 2. Enable interrupts for the mouse
    // First, get the current config byte
    cmd_port.write(0x20 as u8);
    let mut status = data_port.read() | 2; // Bit 1 is mouse interrupt
    // Write it back
    cmd_port.write(0x60 as u8);
    data_port.write(status);

    // 3. Tell the mouse to start sending data
    // 0xD4 tells the controller the next byte goes to the mouse
    cmd_port.write(0xD4 as u8);
    data_port.write(0xF4 as u8); // 0xF4 is 'Enable Data Reporting'
}
