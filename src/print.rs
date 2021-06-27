#![allow(unused_imports)]

extern "C" {
    fn __putchar(c: u8);
}

pub fn write(text: &str) {
    text.bytes().for_each(|b| unsafe { __putchar(b) });
}

pub struct WriteStr;

impl core::fmt::Write for WriteStr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(s);
        Ok(())
    }
}

#[allow(dead_code)]
pub fn write_args(args: &core::fmt::Arguments) {
    core::fmt::write(&mut crate::print::WriteStr, *args).unwrap();
}

macro_rules! print {
    ($($tts:tt)*) => {
        crate::print::write_args(&format_args!($($tts)*));
    }
}

macro_rules! println {
    ($($tts:tt)*) => {
        print!($($tts)*);
        crate::print::write("\n");
    }
}

pub(crate) use print;
pub(crate) use println;
