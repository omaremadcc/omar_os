use core::ops::Add;
use crate::{
    print, println,
    vga_buffer::{backspace, ctrl_backspace},
};
use core::arch::asm;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // 32
    Keyboard,
    Mouse = PIC_1_OFFSET + 12, // 44
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Timer.as_u8()]
            .set_handler_fn(timer_interrupt_handler); // new
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

static IS_ALT: spin::Mutex<bool> = spin::Mutex::new(false);
static IS_SHIFT: spin::Mutex<bool> = spin::Mutex::new(false);
static IS_CTRL: spin::Mutex<bool> = spin::Mutex::new(false);

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let port = 0x60;
    let dbg = false;
    let byte = inb(port);
    if dbg {
        println!("byte: {}", byte);
    }
    if byte <= 59 {
        match byte {
            0x1d | 0xe0 => *IS_CTRL.lock() = true,
            0x3a | 0x38 => *IS_ALT.lock() = true,
            0x2a | 0x36 => *IS_SHIFT.lock() = true,
            _ => {}
        }
    } else {
        let make_code = get_make_code(byte);
        if make_code <= 59 {
            match make_code {
                0x1d | 0xe0 => *IS_CTRL.lock() = false,
                0x3a | 0x38 => *IS_ALT.lock() = false,
                0x2a | 0x36 => *IS_SHIFT.lock() = false,
                28 => println!(""),
                0x0e if *IS_CTRL.lock() => ctrl_backspace(),
                0x0e => backspace(),
                _ => {
                    let key = if *IS_SHIFT.lock() {
                        SHIFT_COLEMAK_MAP[(make_code - 1) as usize]
                    } else {
                        COLEMAK_MAP[(make_code - 1) as usize]
                    };
                    print!("{}", key);
                }
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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
