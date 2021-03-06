use core::mem::{size_of, transmute};

use arch::RING3;

static GDT_SIZE: uint = 6;
type GdtTable = [GdtEntry, ..GDT_SIZE];

static GRANULARITY: u8 = 0xc0; // 4kb blocks and 32-bit protected

static ACCESSED: u8 = 1 << 0;
static RW: u8 = 1 << 1;
static EXECUTE: u8 = 1 << 3;
static ALWAYS1: u8 = 1 << 4;
static PRESENT: u8 = 1 << 7;

static USER: u8 = RING3 << 5; // Ring 3

static CODE: u8 = PRESENT | ALWAYS1 | EXECUTE | RW;
static DATA: u8 = PRESENT | ALWAYS1 | RW;

#[allow(dead_code)]
#[packed]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8
}

#[allow(dead_code)]
#[packed]
struct GdtPtr {
    limit: u16,
    base: *const GdtTable
}

#[allow(dead_code)]
#[packed]
struct TssEntry {
    prev_tss: u32,
    esp0: u32,
    ss0: u32,
    unused: [u32, ..15],
    es: u32,
    cs: u32,
    ss: u32,
    ds: u32,
    fs: u32,
    gs: u32,
    ldt: u32,
    trap: u16,
    iomap_base: u16
}

impl GdtEntry {
    fn new(base: uint, limit: uint, access: u8, granularity: u8) -> GdtEntry {
        GdtEntry {
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            base_high: ((base >> 24) & 0xFF) as u8,
            limit_low: (limit & 0xFFFF) as u16,
            granularity: (((limit >> 16) & 0x0F) as u8) | (granularity & 0xF0),
            access: access
        }
    }

    fn null() -> GdtEntry {
        GdtEntry::new(0, 0, 0, 0)
    }

    fn flat(access: u8, granularity: u8) -> GdtEntry {
        GdtEntry::new(0, 0xFFFFFFFF, access, granularity)
    }
}

impl GdtPtr {
    fn new(table: *const GdtTable) -> GdtPtr {
        GdtPtr {
            limit: size_of::<GdtTable>() as u16,
            base: table
        }
    }
}

impl TssEntry {
    fn as_gdt_entry(&self) -> GdtEntry {
        let base: uint = unsafe { transmute(self) };
        let limit = size_of::<TssEntry>();
        GdtEntry::new(base, limit, PRESENT | EXECUTE | ACCESSED, 0)
    }
}

static mut entries: GdtTable = [
    GdtEntry {
        base_low: 0,
        base_middle: 0,
        base_high: 0,
        limit_low: 0,
        granularity: 0,
        access: 0
    }, ..GDT_SIZE
];

static mut table: GdtPtr = GdtPtr { limit: 0, base: 0 as *const GdtTable };

static mut tss: TssEntry = TssEntry {
    prev_tss: 0,
    esp0: 0,
    ss0: 0,
    unused: [0, ..15],
    es: 0,
    cs: 0,
    ss: 0,
    ds: 0,
    fs: 0,
    gs: 0,
    ldt: 0,
    trap: 0,
    iomap_base: 0
};

pub fn init() {
    unsafe {
        entries[0] = GdtEntry::null();
        entries[1] = GdtEntry::flat(CODE, GRANULARITY);
        entries[2] = GdtEntry::flat(DATA, GRANULARITY);
        entries[3] = GdtEntry::flat(USER | CODE, GRANULARITY);
        entries[4] = GdtEntry::flat(USER | DATA, GRANULARITY);
        entries[5] = write_tss(0x10, 0x0);

        table = GdtPtr::new(&entries);

        gdt_flush(&table);
        tss_flush(0x28 | 0x3);
    }
}

pub fn set_kernel_stack(stack_top: u32) {
    unsafe {
        tss.esp0 = stack_top;
    }
}

unsafe fn write_tss(ss0: u32, esp0: u32) -> GdtEntry {
    tss.ss0 = ss0;
    tss.esp0 = esp0;

    tss.iomap_base = size_of::<TssEntry>() as u16;

    tss.as_gdt_entry()
}

pub fn set_segments(dataseg: u16) {
    unsafe {
        asm!("mov %ax, %ds;
              mov %ax, %es;
              mov %ax, %fs;
              mov %ax, %gs;" :: "{ax}"(dataseg) :: "volatile");
    }
}

fn set_all_segments(dataseg: u16) {
    unsafe {
        asm!("mov %ax, %ds;
              mov %ax, %es;
              mov %ax, %fs;
              mov %ax, %gs;
              mov %ax, %ss;" :: "{ax}"(dataseg) :: "volatile");
    }
}

unsafe fn gdt_flush(ptr: *const GdtPtr) {
    asm!("lgdt ($0)" :: "r"(ptr) :: "volatile");

    set_all_segments(0x10);
    extern { fn load_code_segment(); }
    load_code_segment();
}

unsafe fn tss_flush(seg: u16) {
    asm!("ltr %ax" :: "{ax}"(seg) :: "volatile");
}
