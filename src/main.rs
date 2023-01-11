mod instructions;
mod side_effects;

use instructions::parse;
use itertools::Itertools;
use side_effects::{DefaultSideEffects, SideEffects};

const BINARY: &[u8] = include_bytes!("challenge.bin");

#[derive(Debug, PartialEq, Eq)]
struct VM {
    pc: u16,
    registers: [u16; 8],
    stack: Vec<u16>,
    memory: [u16; 32768],
}

impl VM {
    fn new(binary: impl Iterator<Item = u16>) -> Self {
        let mut memory = [0; 32768];
        memory.iter_mut().set_from(binary);
        Self {
            pc: 0,
            registers: [0; 8],
            stack: Vec::new(),
            memory,
        }
    }

    fn step(&mut self, side_effects: &mut dyn SideEffects) {
        let (instruction, size) = parse(&self.memory, self.pc);
        self.pc += size;
        instruction.execute(self, side_effects);
    }
}

fn main() {
    let binary = BINARY
        .iter()
        .tuples()
        .map(|(l, r)| [*l, *r])
        .map(u16::from_le_bytes);
    let mut vm = VM::new(binary);
    let mut side_effects = DefaultSideEffects::default();
    loop {
        vm.step(&mut side_effects);
    }
}
