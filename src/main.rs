#![no_main]
#![no_std]
use core::arch::asm;
use core::panic::PanicInfo;

use crate::vga_buffer::{backspace, ctrl_backspace};

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let port = 0x60;
    let mut last_code = 250;
    let dbg = false;
    let mut is_alt = false;
    let mut is_shift = false;
    let mut is_ctrl = false;
    loop {
        let byte = inb(port);
        if byte != last_code {
            if dbg {
                println!("byte: {}", byte);
            }
            if byte <= 59 {
                match byte {
                    0x1d | 0xe0 => is_ctrl = true,
                    0x3a | 0x38 => is_alt = true,
                    0x2a | 0x36 => is_shift = true,
                    _ => {}
                }


            } else {
                let make_code = get_make_code(byte);
                if make_code <= 59 {
                    match make_code {
                        0x1d | 0xe0 => is_ctrl = false,
                        0x3a | 0x38 => is_alt = false,
                        0x2a | 0x36 => is_shift = false,
                        28 => println!(""),
                        0x0e if is_ctrl => ctrl_backspace(),
                        0x0e => backspace(),
                        _ => {
                            let key = if is_shift { SHIFT_COLEMAK_MAP[(make_code - 1) as usize] } else { COLEMAK_MAP[(make_code - 1) as usize] };
                            print!("{}", key);
                        }
                    }
                }
            }

            last_code = byte;
        }
    }
}
pub fn inb(port: u16) -> u8 {
    let result: u8;
    unsafe {
        asm!(
            "in al, dx",      // The x86 instruction
            out("al") result, // Tell Rust the output is in the 'al' register
            in("dx") port,    // Tell Rust to put the 'port' variable in 'dx'
            options(nomem, nostack, preserves_flags)
        );
    }
    result
}
fn get_make_code(break_code: u8) -> u8 {
    break_code & 0x7F // Clears the 7th bit
}
fn get_break_code(make_code: u8) -> u8 {
    make_code | 0x80 // Sets the 7th bit
}

static COLEMAK_MAP: [&str; 59] = [
    "Esc", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", r"\b", /* backspace */
    r"\t", "q", "w", "f", "p", "g", "j", "l", "u", "y", ";", "[", "]", r"\n", "LCtrl", "a", "r",
    "s", "t", "d", "h", "n", "e", "i", "o", "'", "`", "LShift", r"\\", "z", "x", "c", "v", "b",
    "k", "m", ",", ".", "/", "RShift", "Super", "LAlt", " ", /* Space */
    "RAlt", "Caps",
];

static SHIFT_COLEMAK_MAP: [&str; 59] = [
    "Esc", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "_", "+", r"\b", /* backspace */
    r"\t", "Q", "W", "f", "P", "G", "J", "L", "U", "Y", ":", "{", "}", r"\n", "LCtrl", "A", "R",
    "S", "T", "D", "H", "N", "E", "I", "O", r#"\""#, "~", "LShift", r"|", "Z", "X", "C", "V", "B",
    "K", "M", "<", ">", "?", "RShift", "Super", "LAlt", " ", /* Space */
    "RAlt", "Caps",
];

mod vga_buffer;
