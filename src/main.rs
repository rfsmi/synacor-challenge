use std::io::{stdin, Read};

use instructions::*;
use itertools::Itertools;

mod instructions;

const BINARY: &[u8] = include_bytes!("challenge.bin");

#[derive(Debug, PartialEq, Eq)]
enum SideEffect {
    Halt,
    Print(char),
    Read,
}

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    SetReg((usize, u16)),
    WriteMem((u16, u16)),
    Jump(u16),
    PushAddr,
    Push(u16),
    Pop,
    ConsumeInput,
}

struct VM {
    pc: u16,
    input: Option<u16>,
    registers: [u16; 8],
    stack: Vec<u16>,
    memory: [u16; 32768],
}

impl VM {
    fn new(binary: &[u8]) -> Self {
        let mut memory = [0; 32768];
        memory.iter_mut().set_from(
            binary
                .iter()
                .tuples()
                .map(|(l, r)| [*l, *r])
                .map(u16::from_le_bytes),
        );
        Self {
            pc: 0,
            input: None,
            registers: [0; 8],
            stack: Vec::new(),
            memory,
        }
    }

    fn step(&mut self) -> Vec<SideEffect> {
        let instruction = Instruction::decode(&self.memory, self.pc);
        let (effects, side_effects) = instruction.execute(self);
        let mut next_address = self.pc + instruction.size();
        for effect in effects {
            match effect {
                Effect::ConsumeInput => {
                    self.input.take();
                }
                Effect::Jump(address) => {
                    next_address = address;
                }
                Effect::Push(value) => {
                    self.stack.push(value);
                }
                Effect::Pop => {
                    self.stack.pop();
                }
                Effect::PushAddr => {
                    self.stack.push(next_address);
                }
                Effect::SetReg((reg, value)) => {
                    self.registers[reg] = value;
                }
                Effect::WriteMem((address, value)) => {
                    self.memory[address as usize] = value;
                }
            }
        }
        self.pc = next_address;
        side_effects
    }
}

fn main() {
    let mut vm = VM::new(BINARY);
    let reader = stdin();
    let mut handle = reader.lock();
    loop {
        for side_effect in vm.step() {
            match side_effect {
                SideEffect::Print(c) => print!("{c}"),
                SideEffect::Read => {
                    let mut buf = [0; 1];
                    match handle.read(&mut buf) {
                        Ok(1) => (),
                        _ => panic!("Failed to read a character"),
                    }
                    vm.input = Some(buf[0] as u16);
                }
                SideEffect::Halt => return,
            }
        }
    }
}
