use crate::VGA_IRQ;
use core::fmt::Write;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.double_fault.set_handler_fn(double_fault_handler); // new

    idt
  };


}

pub fn init() {
    init_idt();
}

fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    writeln!(VGA_IRQ.lock(), "EXCEPTION: BREAKPOINT {:#?}", stack_frame).unwrap();
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

    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
