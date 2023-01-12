use std::{
    collections::{HashMap, HashSet},
    io::{stdin, stdout, BufRead, Write},
};

use itertools::Itertools;

use crate::{
    instructions::{
        parse, Instruction, Noop,
        Operand::{Literal, Reg},
        Set,
    },
    side_effects::{FileBackedEffects, SideEffects},
    VM,
};

pub(crate) struct Debugger {
    breakpoints: HashSet<u16>,
    single_step: bool,
    break_on_exhaust: bool,
    trace: bool,
    memory_patches: HashMap<u16, Box<dyn Instruction>>,
}

impl Debugger {
    pub(crate) fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            single_step: false,
            break_on_exhaust: true,
            trace: false,
            memory_patches: [
                (5451, Noop::new()),
                (5483, Set::new(Reg(0), Literal(6))),
                (5486, Set::new(Reg(7), Literal(25734))),
                (5489, Noop::new()),
            ]
            .into(),
        }
    }

    fn instruction_at_pc(&self, vm: &VM) -> (Box<dyn Instruction>, u16) {
        let (instruction, size) =
            parse(&vm.memory, vm.pc).expect(&format!("Invalid PC: {}", vm.pc));
        if let Some(instruction) = self.memory_patches.get(&vm.pc) {
            return (instruction.clone(), size);
        }
        (instruction, size)
    }

    pub(crate) fn run(&mut self, vm: &mut VM, side_effects: &mut dyn SideEffects) {
        loop {
            let (instruction, size) = self.instruction_at_pc(vm);
            vm.pc += size;
            instruction.execute(vm, side_effects);
        }
    }

    pub(crate) fn debug(&mut self, vm: &mut VM, side_effects: &mut FileBackedEffects) {
        loop {
            if self.break_on_exhaust && side_effects.exhausted() {
                self.break_on_exhaust = false;
                self.single_step = true;
            }
            if self.breakpoints.contains(&vm.pc) {
                self.single_step = true;
            }
            if self.single_step || self.trace {
                let (instruction, _) = self.instruction_at_pc(vm);
                println!("{}: {instruction}", vm.pc);
            }
            if self.single_step {
                self.shell(vm);
            }
            let (instruction, size) = self.instruction_at_pc(vm);
            vm.pc += size;
            instruction.execute(vm, side_effects);
        }
    }

    fn set_bp_command<'a, 'b>(&'a mut self, operands: impl Iterator<Item = &'b str>) {
        let operands = operands.collect_vec();
        if operands.is_empty() {
            if self.breakpoints.is_empty() {
                println!("No current breakpoints (add one with 'bp <address>')");
            } else {
                println!("Current breakpoints:");
                for bp in self.breakpoints.iter().sorted() {
                    println!("{bp}")
                }
            }
        } else {
            for operand in operands {
                let Ok(addr) = operand.parse() else {
                    println!("Expected integer, found: {operand}");
                    return;
                };
                if !self.breakpoints.insert(addr) {
                    // Breakpoint already present; toggle it off
                    self.breakpoints.remove(&addr);
                }
            }
        }
    }

    fn set_command<'a, 'b>(&'a mut self, vm: &'a mut VM, operands: impl Iterator<Item = &'b str>) {
        let Some((target, value)) = operands.collect_tuple() else {
            println!("Expected format: target value");
            return;
        };
        let Ok(value) = value.parse() else {
            println!("Expected integer, found: {value}");
            return;
        };
        if target == "pc" {
            vm.pc = value;
        } else if let Some(reg) = target.strip_prefix("reg") {
            let Ok(reg) = reg.parse::<usize>() else {
                println!("Invalid reg: {reg}");
                return;
            };
            if reg >= 8 {
                println!("Reg number must be 0..=7, got: {reg}");
                return;
            }
            vm.registers[reg] = value;
        } else if let Ok(addr) = target.parse::<usize>() {
            if addr >= vm.memory.len() {
                println!("Address out of bounds: {addr} (max {})", vm.memory.len());
                return;
            }
            vm.memory[addr] = value;
        } else {
            println!("Invalid command");
        }
    }

    fn shell(&mut self, vm: &mut VM) {
        let get_line = || {
            print!("# ");
            stdout().flush().expect("Failed to flush stdout");
            let reader = stdin();
            let mut handle = reader.lock();
            let mut line = String::new();
            handle.read_line(&mut line).expect("Failed to read line");
            line.trim().to_string()
        };
        loop {
            let line = get_line();
            let mut operands = line.split(' ');
            let Some(command) = operands.next() else {
                continue;
            };
            match command {
                "s" => {
                    self.single_step = true;
                    break;
                }
                "g" => {
                    self.single_step = false;
                    break;
                }
                "bp" => self.set_bp_command(operands),
                "regs" => {
                    println!("pc:   {}", vm.pc);
                    for reg in 0..8 {
                        println!("reg{reg}: {}", vm.registers[reg]);
                    }
                }
                "set" => self.set_command(vm, operands),
                "trace" => {
                    self.single_step = false;
                    self.trace = true;
                    break;
                }
                "help" => println!("Available commands:\n\
                    s                    - step a single instruction\n\
                    g                    - resume program execution\n\
                    bp                   - list the current breakpoints\n\
                    bp <address>...      - toggle a breakpoint at the given address (or addresses)\n\
                    regs                 - list pc and register values\n\
                    set <target> <value> - set the target register (or pc) to the given integer value\n\
                    trace                - resume program execution and print all instructions\
                "),
                "" => (),
                _ => println!("Unknown command (try 'help')"),
            }
        }
    }
}
