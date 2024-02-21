#![no_std]
#![feature(start)]
#![feature(panic_info_message)]
#![feature(concat_bytes)]
#![feature(const_mut_refs)]
#![allow(dead_code)]
#[macro_use]
#[allow(unused_macros)]

pub mod math;
mod utils;
use volatile_register::RW;

const TIMER: usize = 0x14;
const SDMCTL: usize = 0x22f;
const DLPTRS: usize = 0x230;
const HSCROLL: usize = 0xd404;
const PMBASE: usize = 0xd407;
const WSYNC: usize = 0xd40a;
const VCOUNT: usize = 0xd40b;
const PMCTL: usize = 0xd01d;
const HPOSP0: usize = 0xd000;
const HPOSP1: usize = 0xd001;
const RANDOM: usize = 0xd20a;

const COLPM0: usize = 0xd012;
const COLPM1: usize = 0xd013;
const COLPM2: usize = 0xd014;
const COLPM3: usize = 0xd015;
const COLPF0: usize = 0xd016;
const COLPF1: usize = 0xd017;
const COLPF2: usize = 0xd018;
const COLPF3: usize = 0xd019;
const COLBK: usize = 0xd01a;

const COLPM0S: usize = 0x2c0;
const COLPM1S: usize = 0x2c1;
const COLPM2S: usize = 0x2c2;
const COLPM3S: usize = 0x2c3;
const COLPF0S: usize = 0x2c4;
const COLPF1S: usize = 0x2c5;
const COLPF2S: usize = 0x2c6;
const COLPF3S: usize = 0x2c7;
const COLBKS: usize = 0x2c8;

// const SCOLOR_REGS: *mut ColorRegs = 0x2c0 as *mut ColorRegs;
// const COLOR_REGS: *mut ColorRegs = 0xd012 as *mut ColorRegs;

const TEXT: &[u8] = &atascii_bytes(&concat_bytes!(
    b"                                    ",
    b"Hello EuroLLVM!",
    b"                                ",
    b"Welcome to the first llvm-mos demo!",
    b"                                ",
    b"And first Rust demo for 6502 target",
    b"                                ",
    b"Greetings to llvm-mos hackers: Daniel Thornburgh (mysterymath), John Byrd, Jack Anderson",
    b"                                ",
    b"Music: Noisy Pillars by Jeroen Tel, Atari conversion: Miker",
    b"                                ",
    b"Demo code & rust-mos port: mrk",
    b"                                ",
    b"https://github.com/llvm-mos",
    b"                                ",
    b"https://github.com/mrk-its/rust-mos",
    b"                                ",
));

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        io_write_u8(COLBK, io_read_u8(RANDOM));
    }
}

pub fn io_write<T: Copy>(addr: usize, value: T) {
    unsafe {
        (*(addr as *const RW<T>)).write(value);
    }
}

pub fn io_write_u8(addr: usize, value: u8) {
    io_write(addr, value);
}

pub fn io_read<T: Copy>(addr: usize) -> T {
    unsafe { (*(addr as *const RW<T>)).read() }
}

pub fn io_read_u8(addr: usize) -> u8 {
    io_read(addr)
}

#[repr(align(4096))]
pub struct AlignedImage {
    pub data: [u8; 13312],
}

const FERRIS_HEIGHT: usize = 208;
const FERRIS_MARGIN: usize = 16;

const FERRIS_DATA: AlignedImage = AlignedImage {
    data: *include_bytes!(concat!(env!("OUT_DIR"), "/ferris.dat")),
};

const SCREEN_HEIGHT: usize = 232;

static mut FERRIS_LO_OFFSETS: [u8; FERRIS_HEIGHT] = [0; FERRIS_HEIGHT];
static mut FERRIS_HI_OFFSETS: [u8; FERRIS_HEIGHT * 3] = [0; FERRIS_HEIGHT * 3];

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DisplayListLine {
    pub mode: u8,
    pub addr: usize,
}

#[repr(align(1024))]
#[repr(C)]
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
        addr: 0,
    }; 208],
    text: DisplayListLine {
        mode: 0x52,
        addr: 0x0,
    },
    lines2: [DisplayListLine {
        mode: 0x5e,
        addr: 0,
    }; 16],
    footer: DisplayListLine {
        mode: 0x41,
        addr: 0,
    },
};

fn cpu_meter_init() {
    io_write_u8(PMCTL, 3); // GTIA: enable players
    io_write_u8(PMBASE, 0xd8);
    io_write_u8(HPOSP0, 0xcc - 6);
    io_write_u8(HPOSP1, 0x2c + 6);

    io_write_u8(COLPM0S, 0xb4);
    io_write_u8(COLPM1S, 0x84);
}

fn cpu_meter_done() {
    io_write_u8(COLPM0, 0);
    io_write_u8(COLPM1, 0);
}

fn wait_vbl() {
    let next_t = io_read_u8(TIMER) + 1;
    while io_read_u8(TIMER) != next_t {}
}

const fn atascii(c: u8) -> u8 {
    match c {
        0x00..=0x1f => c + 0x40,
        0x20..=0x5f => c - 0x20,
        _ => c,
    }
}

const fn atascii_bytes<const N: usize>(text: &[u8; N]) -> [u8; N] {
    let mut ret = [0; N];
    let mut i = 0;
    while i < N {
        ret[i] = atascii(text[i]);
        i += 1;
    }
    ret
}

fn scroll_text(pos: usize) {
    unsafe {
        DLIST.text.addr = TEXT.as_ptr() as usize + pos;
    }
}

fn ferris_init(ferris_start_addr: usize) {
    io_write_u8(COLBKS, 0);
    io_write_u8(COLPF2S, 0xf);
    io_write_u8(COLPF1S, 0x24);
    io_write_u8(COLPF0S, 0x20);

    unsafe {
        let dladdr = core::ptr::addr_of_mut!(DLIST) as usize;
        DLIST.footer.addr = dladdr;

        io_write(DLPTRS, dladdr);

        // (*DLPTRS).write(dladdr);

        let mut addr: usize = ferris_start_addr;

        for i in 0..FERRIS_HEIGHT {
            FERRIS_LO_OFFSETS[i] = addr as u8;
            FERRIS_HI_OFFSETS[i * 3] = (addr >> 8) as u8;
            FERRIS_HI_OFFSETS[i * 3 + 1] = (addr >> 8) as u8;
            FERRIS_HI_OFFSETS[i * 3 + 2] = (addr >> 8) as u8;
            addr += 64;
        }
    }
}

#[allow(unused_assignments)]
unsafe fn update_dlist(index: &mut i16, lines: &mut [DisplayListLine], byte_offs: u8) {
    let lo0 = FERRIS_LO_OFFSETS[(*index as usize) & 3] + byte_offs;
    let lo1 = lo0 + 64;
    let lo2 = lo0 + 128;
    let lo3 = lo0 + 192;
    // let mut optr: usize = &(FERRIS_HI_OFFSETS[*index as usize * 3]) as *const u8 as usize + 1;

    for lines in lines.chunks_mut(16) {
        let mut i = *index as usize;
        if i >= FERRIS_HEIGHT - FERRIS_MARGIN {
            i = 0;
        }
        let optr: usize = &(FERRIS_HI_OFFSETS[i * 3]) as *const u8 as usize + 1;
        let ptr = &(lines[0].addr) as *const usize as *const u8 as usize;

        *((ptr + 0 + 0 * 3) as *mut u8) = lo0;
        *((ptr + 0 + 4 * 3) as *mut u8) = lo0;
        *((ptr + 0 + 8 * 3) as *mut u8) = lo0;
        *((ptr + 0 + 12 * 3) as *mut u8) = lo0;

        *((ptr + 0 + 1 * 3) as *mut u8) = lo1;
        *((ptr + 0 + 5 * 3) as *mut u8) = lo1;
        *((ptr + 0 + 9 * 3) as *mut u8) = lo1;
        *((ptr + 0 + 13 * 3) as *mut u8) = lo1;

        *((ptr + 0 + 2 * 3) as *mut u8) = lo2;
        *((ptr + 0 + 6 * 3) as *mut u8) = lo2;
        *((ptr + 0 + 10 * 3) as *mut u8) = lo2;
        *((ptr + 0 + 14 * 3) as *mut u8) = lo2;

        *((ptr + 0 + 3 * 3) as *mut u8) = lo3;
        *((ptr + 0 + 7 * 3) as *mut u8) = lo3;
        *((ptr + 0 + 11 * 3) as *mut u8) = lo3;
        *((ptr + 0 + 15 * 3) as *mut u8) = lo3;

        *((ptr + 1 + 0 * 3) as *mut u8) = *((optr + 1 + 0 * 3) as *mut u8);
        *((ptr + 1 + 1 * 3) as *mut u8) = *((optr + 1 + 1 * 3) as *mut u8);
        *((ptr + 1 + 2 * 3) as *mut u8) = *((optr + 1 + 2 * 3) as *mut u8);
        *((ptr + 1 + 3 * 3) as *mut u8) = *((optr + 1 + 3 * 3) as *mut u8);
        *((ptr + 1 + 4 * 3) as *mut u8) = *((optr + 1 + 4 * 3) as *mut u8);
        *((ptr + 1 + 5 * 3) as *mut u8) = *((optr + 1 + 5 * 3) as *mut u8);
        *((ptr + 1 + 6 * 3) as *mut u8) = *((optr + 1 + 6 * 3) as *mut u8);
        *((ptr + 1 + 7 * 3) as *mut u8) = *((optr + 1 + 7 * 3) as *mut u8);
        *((ptr + 1 + 8 * 3) as *mut u8) = *((optr + 1 + 8 * 3) as *mut u8);
        *((ptr + 1 + 9 * 3) as *mut u8) = *((optr + 1 + 9 * 3) as *mut u8);
        *((ptr + 1 + 10 * 3) as *mut u8) = *((optr + 1 + 10 * 3) as *mut u8);
        *((ptr + 1 + 11 * 3) as *mut u8) = *((optr + 1 + 11 * 3) as *mut u8);
        *((ptr + 1 + 12 * 3) as *mut u8) = *((optr + 1 + 12 * 3) as *mut u8);
        *((ptr + 1 + 13 * 3) as *mut u8) = *((optr + 1 + 13 * 3) as *mut u8);
        *((ptr + 1 + 14 * 3) as *mut u8) = *((optr + 1 + 14 * 3) as *mut u8);
        *((ptr + 1 + 15 * 3) as *mut u8) = *((optr + 1 + 15 * 3) as *mut u8);

        // ptr += 48;
        // optr += 48;
        *index += 16;
    }
}

fn set_ferris_position(x: i8, y: i8) {
    let x_offs = 128 as u8 + x as u8;
    let mut index = (FERRIS_HEIGHT as i16 - SCREEN_HEIGHT as i16) / 2 + y as i16;

    unsafe {
        let byte_offs = x_offs >> 2;
        update_dlist(&mut index, &mut DLIST.lines, byte_offs);
        index += 8;
        update_dlist(&mut index, &mut DLIST.lines2, byte_offs);
    }
}

type Fptr = unsafe extern "C" fn() -> ();
const RMT_INIT: *const Fptr = &0x8e00usize as *const usize as *const Fptr;
const RMT_PLAY: *const Fptr = &0x8e03usize as *const usize as *const Fptr;

#[start]
fn main(_argc: isize, _args: *const *const u8) -> isize {
    io_write_u8(SDMCTL, 0);
    wait_vbl();

    let ferris_start_addr = &FERRIS_DATA as *const AlignedImage as usize;

    cpu_meter_init();
    ferris_init(ferris_start_addr);

    let mut alpha1: usize = 0;
    let mut alpha2: usize = 0;
    let mut text_pos: usize = 0;

    let mut frame_cnt: u16 = 0;

    io_write_u8(SDMCTL, 0x18 | 0x21);

    unsafe { (*RMT_INIT)() }
    wait_vbl();
    const START_DELAY: u16 = 128;
    const EFF2_DELAY: u16 = 896;
    loop {
        unsafe { (*RMT_PLAY)() }

        let (x, y) = if frame_cnt < START_DELAY {
            (127, 0)
        } else if frame_cnt - START_DELAY < 512 {
            (((frame_cnt - START_DELAY) / 2 - 128) as i8, 0)
        } else {
            let x_offs = if frame_cnt - START_DELAY < EFF2_DELAY {
                (frame_cnt - START_DELAY - EFF2_DELAY) as i16
            } else {
                0
            };
            let x = x_offs / 2 + (math::sin((alpha1 >> 8) as u8) / 4) as i16;
            let x = x.max(-128).min(127) as i8;
            let y = math::sin((alpha2 >> 8) as u8) / 4;
            (x, y)
        };

        let ferris_hscr = 15 - (x as u8 & 3);

        io_write_u8(HSCROLL, ferris_hscr);

        set_ferris_position(x, y);
        alpha1 += 1365; // 65536 / (6 * 8)  6 is speed of rmt tune
        alpha2 += 997;
        cpu_meter_done();
        scroll_text(text_pos / 4);

        let test_hscr = 15 - text_pos as u8 & 3;
        let colpf1_save = io_read_u8(COLPF1S);
        let colpf2_save = io_read_u8(COLPF2S);

        while io_read_u8(VCOUNT) < 218 / 2 {}

        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);

        io_write_u8(HSCROLL, test_hscr);

        io_write_u8(COLPF1, 0x0c);
        io_write_u8(COLPF2, 0x0);

        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);
        io_write_u8(WSYNC, test_hscr);

        io_write_u8(HSCROLL, ferris_hscr);

        io_write_u8(COLPF1, colpf1_save);
        io_write_u8(COLPF2, colpf2_save);

        text_pos = (text_pos + 1) % ((TEXT.len() - 32) * 4);
        wait_vbl();

        frame_cnt += 1;
    }
}
