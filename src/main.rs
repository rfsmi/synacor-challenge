mod instructions;
mod side_effects;

use instructions::parse;
use itertools::Itertools;
use side_effects::{RealSideEffects, SideEffects};

#[derive(Debug, PartialEq, Eq)]
struct VM {
    pc: u16,
    registers: [u16; 8],
    stack: Vec<u16>,
    memory: [u16; 32768],
}

impl Default for VM {
    fn default() -> Self {
        Self {
            pc: 0,
            registers: [0; 8],
            stack: Vec::new(),
            memory: [0; 32768],
        }
    }
}

fn main() {
    let mut vm = VM::default();
    vm.memory.iter_mut().set_from(
        include_bytes!("challenge.bin")
            .iter()
            .tuples()
            .map(|(l, r)| [*l, *r])
            .map(u16::from_le_bytes),
    );
    let mut side_effects = RealSideEffects::default();
    loop {
        let (instruction, size) = parse(&vm.memory, vm.pc);
        vm.pc += size;
        instruction.execute(&mut vm, &mut side_effects);
    }
}
