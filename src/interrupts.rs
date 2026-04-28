use crate::gdt;
use crate::VGA_IRQ;
use core::fmt::Write;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    unsafe {
      idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
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

    loop {
        x86_64::instructions::hlt();
    }
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

    loop {
        x86_64::instructions::hlt();
    }
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

    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}
