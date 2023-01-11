use std::{
    io::{stdin, Read},
    process::exit,
};

pub(crate) trait SideEffects {
    fn print(&mut self, value: u16);
    fn read(&mut self) -> u16;
    fn halt(&mut self);
}

#[derive(Default)]
pub(crate) struct MockSideEffects {
    pub(crate) halted: bool,
    pub(crate) printed: Vec<char>,
    pub(crate) input: Vec<char>,
}

impl SideEffects for MockSideEffects {
    fn print(&mut self, value: u16) {
        let Some(c) = char::from_u32(value as u32) else {
            panic!("Value is not an ascii character: {value}");
        };
        self.printed.push(c);
    }

    fn read(&mut self) -> u16 {
        self.input.remove(0) as u16
    }

    fn halt(&mut self) {
        self.halted = true;
    }
}

#[derive(Default)]
pub(crate) struct DefaultSideEffects {}

impl SideEffects for DefaultSideEffects {
    fn print(&mut self, value: u16) {
        let Some(c) = char::from_u32(value as u32) else {
            panic!("Value is not an ascii character: {value}");
        };
        print!("{c}");
    }

    fn read(&mut self) -> u16 {
        let reader = stdin();
        let mut handle = reader.lock();
        let mut buf = [0; 1];
        match handle.read(&mut buf) {
            Ok(1) => (),
            _ => panic!("Failed to read a character"),
        }
        buf[0] as u16
    }

    fn halt(&mut self) {
        exit(0);
    }
}
