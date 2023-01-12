use std::{
    fs::File,
    io::{stdin, Read, Seek, Write},
    process::exit,
};

pub(crate) trait SideEffects {
    fn print(&mut self, value: u16);
    fn read(&mut self) -> u16;
    fn halt(&mut self);
}

pub(crate) struct FileBackedEffects {
    file_path: String,
    pos: u64,
}

impl FileBackedEffects {
    pub(crate) fn new(path: &str) -> Self {
        Self {
            file_path: path.into(),
            pos: 0,
        }
    }

    pub(crate) fn exhausted(&self) -> bool {
        let file_size = match std::fs::metadata(&self.file_path) {
            Ok(md) => md.len(),
            _ => 0,
        };
        self.pos >= file_size
    }
}

impl SideEffects for FileBackedEffects {
    fn print(&mut self, value: u16) {
        let Some(c) = char::from_u32(value as u32) else {
            panic!("Value is not an ascii character: {value}");
        };
        print!("{c}");
    }

    fn read(&mut self) -> u16 {
        let mut buf = [0; 1];
        if !self.exhausted() {
            // Read from the file
            let Ok(mut file) = File::open(&self.file_path) else {
                panic!("Failed to open file for reading: {}", self.file_path);
            };
            file.seek(std::io::SeekFrom::Start(self.pos))
                .expect("Failed to seek to pos");
            file.read(&mut buf).expect("Failed to read from file");
        } else {
            // Read from stdin
            let reader = stdin();
            let mut handle = reader.lock();
            match handle.read(&mut buf) {
                Ok(1) => (),
                _ => panic!("Failed to read a character from stdin"),
            }
            let Ok(mut file) = File::options().append(true).open(&self.file_path) else {
                panic!("Failed to open file for writing: {}", self.file_path);
            };
            file.write(&buf).expect("Failed to write to file");
        }
        self.pos += 1;
        return buf[0] as u16;
    }

    fn halt(&mut self) {
        exit(0);
    }
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
