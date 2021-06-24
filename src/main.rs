#![no_std]
#![feature(start)]
#![feature(panic_info_message)]
#[macro_use]
#[allow(unused_macros)]

mod print;
pub mod math;
mod write_to;
use core::panic::PanicInfo;
use volatile_register::{RO, RW};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        print::write_args(args);
    }
    loop {}
}

#[repr(C)]
pub struct ColorRegs {
    colpm0: RW<u8>,
    colpm1: RW<u8>,
    colpm2: RW<u8>,
    colpm3: RW<u8>,
    colpf0: RW<u8>,
    colpf1: RW<u8>,
    colpf2: RW<u8>,
    colpf3: RW<u8>,
    colbk: RW<u8>,
}

#[repr(align(4096))]
pub struct AlignedImage {
    pub data: [u8; 11328],
}
const FERRIS_DATA: AlignedImage = AlignedImage {
    data: *include_bytes!("ferris.dat"),
};

#[derive(Clone, Copy)]
pub struct DisplayListLine {
    pub mode: u8,
    pub lo_addr: u8,
    pub hi_addr: u8,
}

#[repr(align(1024))]
pub struct DisplayList {
    pub data: [u8; 64],
    pub lines: [DisplayListLine; 176],
    pub footer: DisplayListLine,
}

static mut DLIST: DisplayList = DisplayList {
    data: [0x00; 64],
    lines: [DisplayListLine {
        mode: 0x5e,
        lo_addr: 0,
        hi_addr: 0,
    }; 176],
    footer: DisplayListLine {
        mode: 0x41,
        lo_addr: 0,
        hi_addr: 0,
    },
};

const TIMER: *const RO<u8> = 0x14 as *const RO<u8>;
const SDMCTL: *mut RW<u8> = 0x22f as *mut RW<u8>;
const DLPTRS: *mut RW<u16> = 0x230 as *mut RW<u16>;
const HSCROLL: *mut RW<u8> = 0xd404 as *mut RW<u8>;
const PMBASE: *mut RW<u8> = 0xd407 as *mut RW<u8>;
const PMCTL: *mut RW<u8> = 0xd01d as *mut RW<u8>;
const HPOSP0: *mut RW<u8> = 0xd000 as *mut RW<u8>;
const HPOSP1: *mut RW<u8> = 0xd001 as *mut RW<u8>;

fn init_ferris(ferris_addr: u16) {
    unsafe {
        let mut addr = ferris_addr;
        for line in DLIST.lines.iter_mut() {
            line.lo_addr = (addr & 0xff) as u8;
            line.hi_addr = (addr >> 8) as u8;
            addr += 64;
        }
    }
}

fn set_ferris_position(x: i8, y: i8) {
    let x_offs = 128 as u8 - x as u8;
    unsafe {
        let dladdr = &mut DLIST as *mut DisplayList as u16 + 32 - (y as u16);
        DLIST.footer.lo_addr = (dladdr & 0xff) as u8;
        DLIST.footer.hi_addr = (dladdr >> 8) as u8;
        (*DLPTRS).write(dladdr);

        (*HSCROLL).write(15 - (x_offs & 3));
        for line in DLIST.lines.iter_mut() {
            line.lo_addr = line.lo_addr & 0xc0 | (x_offs >> 2);
        }
    }
}

#[start]
fn main(_argc: isize, _args: *const *const u8) -> isize {
    unsafe {
        let shadow_color_regs = &mut *(0x2c0 as *mut ColorRegs);
        let color_regs = &mut *(0xd012 as *mut ColorRegs);

        shadow_color_regs.colpm0.write(0xb4);
        shadow_color_regs.colpm1.write(0x84);
        shadow_color_regs.colbk.write(0);
        shadow_color_regs.colpf2.write(0xf);
        shadow_color_regs.colpf1.write(0x34);
        shadow_color_regs.colpf0.write(0x31);

        let ferris_addr = &FERRIS_DATA as *const AlignedImage as u16;

        init_ferris(ferris_addr);

        (*SDMCTL).write(0x39);
        (*PMCTL).write(3); // enable players
        (*PMBASE).write(0xd8);
        (*HPOSP0).write(0xcc - 6);
        (*HPOSP1).write(0x2c + 6);

        let mut alpha1: u16 = 0;
        let mut alpha2: u16 = 0;
        let mut x_offs: i8 = 0;
        loop {
            set_ferris_position(x_offs + math::sin((alpha1 >> 8) as u8) / 4, math::sin((alpha2 >> 8) as u8) / 4);

            color_regs.colpm0.write(0);
            color_regs.colpm1.write(0);

            let next_t = (*TIMER).read() + 1;
            while (*TIMER).read() != next_t {}

            alpha1 += 1400;
            alpha2 += 900;
            // x_offs -= 1;
        }
    }
}
