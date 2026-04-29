use crate::gdt;
use crate::halt_loop;
use crate::VGA_IRQ;
use core::fmt::Write;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

pub fn init() {
    gdt::init();
    init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let mut w = VGA_IRQ.lock();
    writeln!(w, "EXCEPTION: BREAKPOINT").unwrap();
    writeln!(w, "  rip:      {:?}", stack_frame.instruction_pointer).unwrap();
    writeln!(w, "{:#?}", stack_frame).unwrap();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    let cause = if error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
        "protection violation"
    } else {
        "page not present"
    };
    let access = if error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
        "instruction fetch"
    } else if error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) {
        "write"
    } else {
        "read"
    };
    let mode = if error_code.contains(PageFaultErrorCode::USER_MODE) {
        "user"
    } else {
        "kernel"
    };

    let mut w = VGA_IRQ.lock();
    writeln!(w, "EXCEPTION: PAGE FAULT").unwrap();
    writeln!(w, "  cause:   {}", cause).unwrap();
    writeln!(w, "  access:  {} from {} mode", access, mode).unwrap();
    match Cr2::read() {
        Ok(addr) => writeln!(w, "  address: {:?}", addr).unwrap(),
        Err(_) => writeln!(w, "  address: <invalid CR2>").unwrap(),
    }
    writeln!(w, "  raw:     {:?}", error_code).unwrap();
    writeln!(w, "{:#?}", stack_frame).unwrap();
    drop(w);

    halt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let mut w = VGA_IRQ.lock();
    writeln!(w, "EXCEPTION: GENERAL PROTECTION FAULT").unwrap();
    if error_code == 0 {
        writeln!(w, "  selector: <none>").unwrap();
    } else {
        let external = error_code & 0b1 != 0;
        let table = match (error_code >> 1) & 0b11 {
            0b00 => "GDT",
            0b01 | 0b11 => "IDT",
            0b10 => "LDT",
            _ => unreachable!(),
        };
        let index = (error_code >> 3) & 0x1FFF;
        writeln!(
            w,
            "  selector: index={} table={} external={}",
            index, table, external
        )
        .unwrap();
    }
    writeln!(w, "  raw:      {:#x}", error_code).unwrap();
    writeln!(w, "{:#?}", stack_frame).unwrap();
    drop(w);

    halt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    let mut w = VGA_IRQ.lock();
    writeln!(w, "EXCEPTION: DOUBLE FAULT").unwrap();
    writeln!(w, "  raw:      {:#x}", error_code).unwrap();
    writeln!(w, "{:#?}", stack_frame).unwrap();
    drop(w);

    halt_loop()
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // interrupts::without_interrupts(|| {
        // write!(VGA_IRQ.lock(), ".").unwrap();
    // });

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            if let DecodedKey::Unicode(character) = key {
                print!("{}", character);
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
