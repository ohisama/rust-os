use core::container::Container;
use core::mem::size_of;

use vga;
use io;
use util;
use util::range;

static IDT_SIZE: uint = 256;
type IdtTable = [IdtEntry, ..IDT_SIZE];

#[packed]
struct IdtEntry {
    handler_low: u16,
    selector: u16,
    always0: u8,
    flags: u8,
    handler_high: u16
}

#[packed]
struct IdtPtr {
    limit: u16,
    base: *IdtTable
}

#[packed]
pub struct Registers {
    ds: u32,
    edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32,
    int_no: u32, err_code: u32,
    eip: u32, cs: u32, eflags: u32, useresp: u32, ss: u32
}

impl IdtEntry {
    fn new(handler: u32, selector: u16, flags: u8) -> IdtEntry {
        IdtEntry {
            handler_low: (handler & 0xFFFF) as u16,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
            selector: selector,
            always0: 0,
            // We must uncomment the OR below when we get to using user-mode.
            // It sets the interrupt gate's privilege level to 3.
            flags: flags //| 0x60
        }
    }
}

impl IdtPtr {
    fn new(table: &IdtTable) -> IdtPtr {
        IdtPtr {
            limit: (size_of::<IdtEntry>() * table.len() - 1) as u16,
            base: table as *IdtTable
        }
    }
}

static mut entries: IdtTable = [
    IdtEntry {
        handler_low: 0,
        selector: 0,
        always0: 0,
        flags: 0,
        handler_high: 0
    }, ..IDT_SIZE
];

static mut table: IdtPtr = IdtPtr {
    limit: 0,
    base: 0 as *IdtTable
};

struct Handler {
    f: extern "Rust" fn(regs: &Registers)
}

fn dummy_isr_handler(regs: &Registers) {}
static mut interrupt_handlers: [Handler, ..IDT_SIZE] = [
    Handler { f: dummy_isr_handler }, ..IDT_SIZE
];
/*static mut interrupt_handlers: ['static |&Registers|, ..IDT_SIZE] = [
    dummy_isr_handler, ..IDT_SIZE
];*/

// Defined in handlers.s
extern { static isr_handler_array: [u32, ..IDT_SIZE]; }

pub fn init() {
    // Remap the irq table.
    io::out(0x20, 0x11);
    io::out(0xA0, 0x11);
    io::out(0x21, 0x20);
    io::out(0xA1, 0x28);
    io::out(0x21, 0x04);
    io::out(0xA1, 0x02);
    io::out(0x21, 0x01);
    io::out(0xA1, 0x01);
    io::out(0x21, 0x0);
    io::out(0xA1, 0x0);

    unsafe {
        range(0, IDT_SIZE, |i| {
            entries[i] = IdtEntry::new(isr_handler_array[i], 0x08, 0x8E);
        });

        table = IdtPtr::new(&entries);
        idt_flush(&table);
        idt_enable();
    }
}

pub fn register_irq_handler(which: uint, f: extern "Rust" fn(regs: &Registers)) {
    register_isr_handler(which + 32, f);
}

pub fn register_isr_handler(which: uint, f: extern "Rust" fn(regs: &Registers)) {
    unsafe {
        interrupt_handlers[which] = Handler { f: f };
    }
}

#[no_mangle]
pub extern fn isr_handler(regs: Registers) {
    if regs.int_no >= 32 && regs.int_no <= 47 {
        let irq = regs.int_no - 32;
        if irq <= 7 {
            io::out(0x20, 0x20); // Master
        }
        io::out(0xA0, 0x20); // Slave
    }

    unsafe {
        let f = interrupt_handlers[regs.int_no].f;
        f(&regs);
    }

    /*vga::puts("Interrupt! ");
    util::convert(regs.int_no, |c| vga::putch(c) );
    vga::puts(", error: ");
    util::convert(regs.err_code, |c| vga::putch(c) );
    vga::puts("\n");*/
}

unsafe fn idt_enable() {
    asm!("sti");
}

unsafe fn idt_flush(ptr: *IdtPtr) {
    asm!("lidt ($0)" :: "r"(ptr));
}
