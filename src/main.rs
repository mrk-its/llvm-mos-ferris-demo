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

const TIMER: *const RO<u8> = 0x14 as *const RO<u8>;
const SDMCTL: *mut RW<u8> = 0x22f as *mut RW<u8>;
const DLPTRS: *mut RW<usize> = 0x230 as *mut RW<usize>;
const HSCROLL: *mut RW<u8> = 0xd404 as *mut RW<u8>;
const PMBASE: *mut RW<u8> = 0xd407 as *mut RW<u8>;
const WSYNC: *mut RW<u8> = 0xd40a as *mut RW<u8>;
const VCOUNT: *mut RO<u8> = 0xd40b as *mut RO<u8>;
const PMCTL: *mut RW<u8> = 0xd01d as *mut RW<u8>;
const HPOSP0: *mut RW<u8> = 0xd000 as *mut RW<u8>;
const HPOSP1: *mut RW<u8> = 0xd001 as *mut RW<u8>;
const RANDOM: *mut RW<u8> = 0xd20a as *mut RW<u8>;

const SCOLOR_REGS: *mut ColorRegs = 0x2c0 as *mut ColorRegs;
const COLOR_REGS: *mut ColorRegs = 0xd012 as *mut ColorRegs;

const TEXT: &[u8] = b"                                    \
                       https://github.com/llvm-mos                                    \
                       https://github.com/mrk-its/rust                                    ";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // if let Some(args) = info.message() {
    //     print::write_args(args);
    // }
    loop {
        unsafe {
            (*COLOR_REGS).colbk.write((*RANDOM).read());
        }
    }
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
    pub data: [u8; 13312],
}

const FERRIS_HEIGHT: usize = 208;
const FERRIS_MARGIN: usize = 16;

const FERRIS_DATA: AlignedImage = AlignedImage {
    data: *include_bytes!("ferris.dat"),
};

const SCREEN_HEIGHT: usize = 232;

static mut FERRIS_LO_OFFSETS: [u8; FERRIS_HEIGHT] = [0; FERRIS_HEIGHT];
static mut FERRIS_HI_OFFSETS: [u8; FERRIS_HEIGHT] = [0; FERRIS_HEIGHT];

#[derive(Clone, Copy)]
pub struct DisplayListLine {
    pub mode: u8,
    pub lo_addr: u8,
    pub hi_addr: u8,
}

#[repr(align(1024))]
pub struct DisplayList {
    pub data: [u8; 1],
    pub lines: [DisplayListLine; 208],
    pub text: DisplayListLine,
    pub lines2: [DisplayListLine; 16],
    pub footer: DisplayListLine,
}

static mut DLIST: DisplayList = DisplayList {
    data: [0x30; 1],
    lines: [DisplayListLine {
        mode: 0x5e,
        lo_addr: 0,
        hi_addr: 0,
    }; 208],
    text: DisplayListLine {
        mode: 0x52,
        lo_addr: 0x0,
        hi_addr: 0x0,
    },
    lines2: [DisplayListLine {
        mode: 0x5e,
        lo_addr: 0,
        hi_addr: 0,
    }; 16],
    footer: DisplayListLine {
        mode: 0x41,
        lo_addr: 0,
        hi_addr: 0,
    },
};

fn cpu_meter_init() {
    unsafe {
        (*PMCTL).write(3); // GTIA: enable players
        (*PMBASE).write(0xd8);
        (*HPOSP0).write(0xcc - 6); // right
        (*HPOSP1).write(0x2c + 6); // left
        (*SCOLOR_REGS).colpm0.write(0xb4);
        (*SCOLOR_REGS).colpm1.write(0x84);
    }
}

fn cpu_meter_done() {
    unsafe {
        (*COLOR_REGS).colpm0.write(0);
        (*COLOR_REGS).colpm1.write(0);
    }
}

fn wait_vbl() {
    unsafe {
        let next_t = (*TIMER).read() + 1;
        while (*TIMER).read() != next_t {}
    }
}

fn atascii(c: u8) -> u8 {
    match c {
        0x00..=0x1f => c + 0x40,
        0x20..=0x5f => c - 0x20,
        _ => c,
    }
}

fn text_init() {
    let text_addr = TEXT.as_ptr() as usize;
    for i in 0..TEXT.len() {
        unsafe {
            *((text_addr + i) as *const u8 as *mut u8) = atascii(TEXT[i]);
        }
    }
}

fn scroll_text(pos: usize) {
    let text_addr = TEXT.as_ptr() as usize + pos;
    unsafe {
        DLIST.text.lo_addr = text_addr as u8;
        DLIST.text.hi_addr = (text_addr >> 8) as u8;
    }
}

fn ferris_init(ferris_start_addr: usize) {
    unsafe {
        (*SCOLOR_REGS).colbk.write(0);
        (*SCOLOR_REGS).colpf2.write(0xf);
        (*SCOLOR_REGS).colpf1.write(0x24);
        (*SCOLOR_REGS).colpf0.write(0x20);

        let dladdr = &mut DLIST as *mut DisplayList as usize;
        DLIST.footer.lo_addr = (dladdr & 0xff) as u8;
        DLIST.footer.hi_addr = (dladdr >> 8) as u8;
        (*DLPTRS).write(dladdr);

        let mut addr: usize = ferris_start_addr;

        for i in 0..FERRIS_HEIGHT {
            FERRIS_LO_OFFSETS[i] = addr as u8;
            FERRIS_HI_OFFSETS[i] = (addr >> 8) as u8;
            addr += 64;
        }
    }
}

fn update_dlist(index: &mut usize, lines: &mut [DisplayListLine], byte_offs: u8) {
    unsafe {
        let lo0 = FERRIS_LO_OFFSETS[(*index as usize) & 3] + byte_offs;
        let lo1 = lo0 + 64;
        let lo2 = lo0 + 128;
        let lo3 = lo0 + 192;
        let mut ptr = &(lines[0].lo_addr) as *const u8 as usize;
        for lines in lines.chunks_mut(16) {
            let mut i = *index as usize;
            if i >= FERRIS_HEIGHT - FERRIS_MARGIN {
                i = 0;
            }
            let mut optr: usize = &(FERRIS_HI_OFFSETS[i]) as *const u8 as usize;

            *(ptr as *mut u8) = lo0;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo1;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo2;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo3;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;

            *(ptr as *mut u8) = lo0;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo1;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo2;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo3;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo0;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo1;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo2;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo3;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo0;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo1;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo2;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 2;
            optr += 1;
            *(ptr as *mut u8) = lo3;
            ptr += 1;
            *(ptr as *mut u8) = *(optr as *mut u8);
            ptr += 1;
            ptr += 1;
            optr += 1;
            *index += 16;
        }
    }
}

fn set_ferris_position(x: i8, y: i8) {
    let x_offs = 128 as u8 + x as u8;

    let mut index = ((FERRIS_HEIGHT as i16 - SCREEN_HEIGHT as i16) / 2 + y as i16) as usize;

    unsafe {
        let byte_offs = x_offs >> 2;
        update_dlist(&mut index, &mut DLIST.lines, byte_offs);
        index += 8;
        update_dlist(&mut index, &mut DLIST.lines2, byte_offs);
    }
}

#[start]
fn main(_argc: isize, _args: *const *const u8) -> isize {
    unsafe {
        (*SDMCTL).write(0);
        wait_vbl();
    }

    let ferris_start_addr = &FERRIS_DATA as *const AlignedImage as usize;

    cpu_meter_init();
    text_init();
    ferris_init(ferris_start_addr);

    let mut alpha1: usize = 0;
    let mut alpha2: usize = 0;
    let mut x_offs: i8 = 0;
    let mut text_pos: usize = 0;

    unsafe {
        (*SDMCTL).write(0x18 | 0x21);
        wait_vbl();
    }

    loop {
        let x = x_offs + math::sin((alpha1 >> 8) as u8) / 4;
        let y = math::sin((alpha2 >> 8) as u8) / 4;
        let ferris_hscr = 15 - (x as u8 & 3);
        unsafe {
            (*HSCROLL).write(ferris_hscr);
        }

        set_ferris_position(x, y);
        alpha1 += 1377;
        alpha2 += 997;
        x_offs += 0;
        cpu_meter_done();
        scroll_text(text_pos / 4);
        let test_hscr = 15 - text_pos as u8 & 3;
        unsafe {
            let colpf1_save = (*SCOLOR_REGS).colpf1.read();
            let colpf2_save = (*SCOLOR_REGS).colpf2.read();
            while (*VCOUNT).read() < 218 / 2 {}
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);

            (*HSCROLL).write(test_hscr);
            (*COLOR_REGS).colpf1.write(0x0c);
            (*COLOR_REGS).colpf2.write(0x0);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*WSYNC).write(test_hscr);
            (*HSCROLL).write(ferris_hscr);
            (*COLOR_REGS).colpf2.write(colpf2_save);
            (*COLOR_REGS).colpf1.write(colpf1_save);
        }
        text_pos = (text_pos + 1) % ((TEXT.len() - 32) * 4);
        wait_vbl();
    }
}
