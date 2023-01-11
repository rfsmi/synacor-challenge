use std::fmt::Debug;

use crate::{SideEffects, VM};

macro_rules! make_parser {
    // Generate the instruction getter
    [$fn_name:ident
        []
        [] [] [] [] []
        [$($arms:tt)*]
    ] => {
        pub(crate) fn $fn_name(
            data: &[u16], address: u16,
        ) -> (Box<dyn Instruction>, u16) {
            let operands = (
                data.get(address as usize)
                    .copied()
                    .expect(&format!("Invalid address: {}", address)),
                data.get(address as usize + 1).copied(),
                data.get(address as usize + 2).copied(),
                data.get(address as usize + 3).copied(),
            );
            match operands {
                $($arms)*
                (opcode, _, _, _) => panic!("Opcode {} not implemented", opcode),
            }
        }
    };
    // Ending an instruction
    [$fn_name:ident
        [, $($rest:tt)*]
        [$op:ident] [$code:literal] [$($args:ident,)*] [$($rem:tt)*] [$size:expr]
        [$($arms:tt)*]
    ] => {
        make_parser![$fn_name
            [$($rest)*]
            [] [] [] [] []
            [$($arms)* ($code, $(Some($args),)* $($rem)*) => (
                Box::new($op { $($args: $args.into(),)* }),
                $size,
            ),]
        ];
        #[derive(Debug)]
        struct $op {
            $($args: Operand,)*
        }
    };
    // Starting a new instruction
    [$fn_name:ident
        [$op:ident : $code:literal $($rest:tt)*]
        [] [] [] [] []
        [$($arms:tt)*]
    ] => {
        make_parser![$fn_name
            [$($rest)*]
            [$op] [$code] [] [_, _, _,] [1]
            [$($arms)*]
        ];
    };
    // Parsing an operand
    [$fn_name:ident
        [$arg:tt $($rest:tt)*]
        [$op:ident] [$($code:tt)*] [$($args:tt)*] [_, $($rem:tt)*] [$size:expr]
        [$($arms:tt)*]
    ] => {
        make_parser![$fn_name
            [$($rest)*]
            [$op] [$($code)*] [$($args)* $arg,] [$($rem)*] [1 + $size]
            [$($arms)*]
        ];
    };
    // Entry point
    ($fn_name:ident, $($ops:tt)*) => {
        make_parser![$fn_name
            [$($ops)*]
            [] [] [] [] []
            []
        ];
    };
}

#[derive(Debug, Clone, Copy)]
enum Operand {
    Literal(u16),
    Reg(usize),
}

impl Operand {
    fn value(self, vm: &VM) -> u16 {
        match self {
            Operand::Literal(value) => value,
            Operand::Reg(reg) => vm.registers[reg],
        }
    }

    fn write(self, vm: &mut VM, value: u16) {
        match self {
            Operand::Reg(reg) => vm.registers[reg] = value,
            _ => panic!("Invalid write target: {self:?}"),
        }
    }
}

impl From<u16> for Operand {
    fn from(value: u16) -> Self {
        match value {
            0..=32767 => Operand::Literal(value),
            32768..=32775 => Operand::Reg(value as usize - 32768),
            _ => panic!("Invalid number: {value}"),
        }
    }
}

make_parser![parse,
    Halt: 0,
    Set: 1 a b,
    Push: 2 a,
    Pop: 3 a,
    Eq: 4 a b c,
    Gt: 5 a b c,
    Jmp: 6 a,
    Jt: 7 a b,
    Jf: 8 a b,
    Add: 9 a b c,
    Mult: 10 a b c,
    Mod: 11 a b c,
    And: 12 a b c,
    Or: 13 a b c,
    Not: 14 a b,
    Rmem: 15 a b,
    Wmem: 16 a b,
    Call: 17 a,
    Ret: 18,
    Out: 19 a,
    In: 20 a,
    Noop: 21,
];

pub(crate) trait Instruction: Debug {
    fn execute(&self, vm: &mut VM, side_effects: &mut dyn SideEffects);
}

impl Instruction for Halt {
    fn execute(&self, _vm: &mut VM, side_effects: &mut dyn SideEffects) {
        side_effects.halt();
    }
}

impl Instruction for Set {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        self.a.write(vm, self.b.value(vm));
    }
}

impl Instruction for Push {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        vm.stack.push(self.a.value(vm));
    }
}

impl Instruction for Pop {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        match vm.stack.pop() {
            Some(value) => self.a.write(vm, value),
            None => panic!("Cannot pop from empty stack"),
        }
    }
}

impl Instruction for Eq {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = match self.b.value(vm) == self.c.value(vm) {
            true => 1,
            false => 0,
        };
        self.a.write(vm, value);
    }
}

impl Instruction for Gt {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = match self.b.value(vm) > self.c.value(vm) {
            true => 1,
            false => 0,
        };
        self.a.write(vm, value);
    }
}

impl Instruction for Jmp {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        match self.a {
            Operand::Literal(value) => vm.pc = value,
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

impl Instruction for Jt {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        if self.a.value(vm) == 0 {
            return;
        }
        match self.b {
            Operand::Literal(value) => vm.pc = value,
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

impl Instruction for Jf {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        if self.a.value(vm) != 0 {
            return;
        }
        match self.b {
            Operand::Literal(value) => vm.pc = value,
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

impl Instruction for Add {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = (self.b.value(vm) + self.c.value(vm)) % 32768;
        self.a.write(vm, value);
    }
}

impl Instruction for Mult {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = (self.b.value(vm) as usize * self.c.value(vm) as usize) % 32768;
        self.a.write(vm, value as u16);
    }
}

impl Instruction for Mod {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = (self.b.value(vm) % self.c.value(vm)) % 32768;
        self.a.write(vm, value);
    }
}

impl Instruction for And {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = self.b.value(vm) & self.c.value(vm);
        self.a.write(vm, value);
    }
}

impl Instruction for Or {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = self.b.value(vm) | self.c.value(vm);
        self.a.write(vm, value);
    }
}

impl Instruction for Not {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = !self.b.value(vm) & ((1 << 15) - 1);
        self.a.write(vm, value);
    }
}

impl Instruction for Rmem {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        let value = vm.memory[self.b.value(vm) as usize];
        self.a.write(vm, value);
    }
}

impl Instruction for Wmem {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        vm.memory[self.a.value(vm) as usize] = self.b.value(vm);
    }
}

impl Instruction for Call {
    fn execute(&self, vm: &mut VM, _side_effects: &mut dyn SideEffects) {
        vm.stack.push(vm.pc);
        vm.pc = self.a.value(vm);
    }
}

impl Instruction for Ret {
    fn execute(&self, vm: &mut VM, side_effects: &mut dyn SideEffects) {
        match vm.stack.pop() {
            Some(value) => vm.pc = value,
            None => side_effects.halt(),
        }
    }
}

impl Instruction for Out {
    fn execute(&self, vm: &mut VM, side_effects: &mut dyn SideEffects) {
        side_effects.print(self.a.value(vm));
    }
}

impl Instruction for In {
    fn execute(&self, vm: &mut VM, side_effects: &mut dyn SideEffects) {
        self.a.write(vm, side_effects.read());
    }
}

impl Instruction for Noop {
    fn execute(&self, _vm: &mut VM, _side_effects: &mut dyn SideEffects) {}
}

mod test {
    use super::*;
    use crate::side_effects::MockSideEffects;

    #[test]
    fn test_out() {
        let (instr, size) = parse(&[19, 65], 0);
        assert_eq!(size, 2);
        assert_eq!(format!("{instr:?}"), "Out { a: Literal(65) }");

        let mut vm = VM::new(&[]);
        let mut side_effects = MockSideEffects::default();
        instr.execute(&mut vm, &mut side_effects);
        assert_eq!(vm, VM::new(&[])); // No effect on vm
        assert_eq!(side_effects.printed, vec!['A']);
    }
}
