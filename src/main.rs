mod debugger;
mod instructions;
mod orb_maze;
mod side_effects;
mod teleporter;

use clap::Parser;
use debugger::Debugger;
use instructions::parse;
use itertools::Itertools;
use orb_maze::Maze;
use side_effects::{BasicSideEffects, FileBackedEffects, SideEffects};
use teleporter::Teleporter;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
enum Command {
    Run,
    Debug,
    DumpBinary,
    CalculateTeleporterNumber,
    SolveMaze,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_enum, default_value_t=Command::Run)]
    command: Command,
}

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

fn dump(binary: &[u16]) {
    let mut pos = 0;
    let mut ops = Vec::new();
    while pos < binary.len() as u16 {
        let Some((instruction, size)) = parse(binary, pos) else {
            pos += 1;
            continue;
        };
        ops.push((instruction.to_string(), pos));
        pos += size;
    }
    for (_, group) in &ops.into_iter().group_by(|(text, _)| text.clone()) {
        let items = group.collect_vec();
        let print = |(text, size): &(String, u16)| println!("{size}: {text}");
        if items.len() > 3 {
            print(items.first().unwrap());
            println!("...");
            print(items.last().unwrap());
            continue;
        }
        for item in &items {
            print(item);
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
    let mut debugger = Debugger::new();
    match Args::parse().command {
        Command::Run => {
            let mut side_effects = BasicSideEffects::default();
            debugger.run(&mut vm, &mut side_effects);
        }
        Command::Debug => {
            let mut side_effects = FileBackedEffects::new("replay.txt");
            debugger.debug(&mut vm, &mut side_effects);
        }
        Command::DumpBinary => dump(&vm.memory),
        Command::CalculateTeleporterNumber => Teleporter::run(),
        Command::SolveMaze => Maze::solve(),
    }
}
