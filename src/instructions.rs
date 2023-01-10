use std::fmt::Debug;

use crate::{Effect, SideEffect, VM};

macro_rules! opcodes {
    // Final step: write out the whole match expression
    (@build $matcher:ident
        []
        [] []
        [] []
        [$($parsed:tt)*]) => {
        match $matcher {
            $($parsed)*
            (opcode, _, _, _) => panic!("Opcode {} not implemented", opcode),
        }
    };
    // Complete a match arm
    (@build $matcher:ident
        [=> $op:ident, $($rest:tt)*]
        [$($pat:tt)*] []
        [$op_size:expr] [$($args:tt)*]
        [$($parsed:tt)*]
    ) => {
        opcodes!(@build $matcher
            [$($rest)*]
            [] []
            [] []
            [$($parsed)* ($($pat)*) => Self {
                size: $op_size,
                op: Box::new($op { $($args)* })
            },]
        )
    };
    // Start parsing a new instruction
    (@build $matcher:ident
        [$opcode:literal $($rest:tt)*]
        [] []
        [] []
        [$($parsed:tt)*]
    ) => {
        opcodes!(@build $matcher
            [$($rest)*]
            [$opcode,] [_ _ _]
            [1] []
            [$($parsed)*]
        )
    };
    // Almost parsed the instruction; add any trailing '_'s to the pattern
    (@build $matcher:ident
        [=> $($rest:tt)*]
        [$($pat:tt)*] [_ $($unused_args:tt)*]
        [$op_size:expr] [$($args:tt)*]
        [$($parsed:tt)*]
    ) => {
        opcodes!(@build $matcher
            [=> $($rest)*]
            [$($pat)* _,] [$($unused_args)*]
            [$op_size] [$($args)*]
            [$($parsed)*]
        )
    };
    // Parsing the instruction; take the next argument
    (@build $matcher:ident
        [$arg:tt $($rest:tt)*]
        [$($pat:tt)*] [_ $($unused_args:tt)*]
        [$op_size:expr] [$($args:tt)*]
        [$($parsed:tt)*]
    ) => {
        opcodes!(@build $matcher
            [$($rest)*]
            [$($pat)* Some($arg),] [$($unused_args)*]
            [$op_size + 1] [$($args)* $arg: $arg.into(),]
            [$($parsed)*]
        )
    };
    // Entry-point
    [$matcher:ident, $($ops:tt)*] => {
        opcodes!(@build $matcher
            [$($ops)*]
            [] []
            [] []
            []
        )
    };
}

#[derive(Debug)]
pub(crate) struct Instruction {
    size: u16,
    op: Box<dyn Op>,
}

impl Instruction {
    pub(crate) fn decode(data: &[u16], address: u16) -> Instruction {
        let instruction = (
            data.get(address as usize)
                .expect(&format!("Invalid address: {}", address)),
            data.get(address as usize + 1).copied(),
            data.get(address as usize + 2).copied(),
            data.get(address as usize + 3).copied(),
        );
        opcodes![instruction,
            0 => Halt,
            1 a b => Set,
            2 a => Push,
            3 a => Pop,
            4 a b c => Eq,
            5 a b c => Gt,
            6 a => Jmp,
            7 a b => Jt,
            8 a b => Jf,
            9 a b c => Add,
            10 a b c => Mult,
            11 a b c => Mod,
            12 a b c => And,
            13 a b c => Or,
            14 a b => Not,
            15 a b => Rmem,
            16 a b => Wmem,
            17 a => Call,
            18 => Ret,
            19 a => Out,
            20 a => In,
            21 => Noop,
        ]
    }

    pub(crate) fn execute(&self, vm: &VM) -> (Vec<Effect>, Vec<SideEffect>) {
        let mut effects = Vec::new();
        let mut side_effects = Vec::new();
        self.op.execute(vm, &mut effects, &mut side_effects);
        (effects, side_effects)
    }

    pub(crate) fn size(&self) -> u16 {
        self.size
    }
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

    fn write(self, effects: &mut Vec<Effect>, value: u16) {
        match self {
            Operand::Reg(reg) => effects.push(Effect::SetReg((reg, value))),
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

trait Op: Debug {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, side_effects: &mut Vec<SideEffect>);
}

#[derive(Debug)]
struct Halt {}

impl Op for Halt {
    fn execute(&self, _vm: &VM, _effects: &mut Vec<Effect>, side_effects: &mut Vec<SideEffect>) {
        side_effects.push(SideEffect::Halt);
    }
}

#[derive(Debug)]
struct Set {
    a: Operand,
    b: Operand,
}

impl Op for Set {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        self.a.write(effects, self.b.value(vm));
    }
}

#[derive(Debug)]
struct Push {
    a: Operand,
}

impl Op for Push {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        effects.push(Effect::Push(self.a.value(vm)));
    }
}

#[derive(Debug)]
struct Pop {
    a: Operand,
}

impl Op for Pop {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let Some(&value) = vm.stack.last() else {
            panic!("Cannot pop from empty stack");
        };
        self.a.write(effects, value);
        effects.push(Effect::Pop);
    }
}

#[derive(Debug)]
struct Eq {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Eq {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = match self.b.value(vm) == self.c.value(vm) {
            true => 1,
            false => 0,
        };
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Gt {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Gt {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = match self.b.value(vm) > self.c.value(vm) {
            true => 1,
            false => 0,
        };
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Jmp {
    a: Operand,
}

impl Op for Jmp {
    fn execute(&self, _vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        match self.a {
            Operand::Literal(value) => effects.push(Effect::Jump(value)),
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

#[derive(Debug)]
struct Jt {
    a: Operand,
    b: Operand,
}

impl Op for Jt {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        if self.a.value(vm) == 0 {
            return;
        }
        match self.b {
            Operand::Literal(value) => effects.push(Effect::Jump(value)),
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

#[derive(Debug)]
struct Jf {
    a: Operand,
    b: Operand,
}

impl Op for Jf {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        if self.a.value(vm) != 0 {
            return;
        }
        match self.b {
            Operand::Literal(value) => effects.push(Effect::Jump(value)),
            _ => panic!("Invalid jump target: {:?}", self.a),
        }
    }
}

#[derive(Debug)]
struct Add {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Add {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = (self.b.value(vm) + self.c.value(vm)) % 32768;
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Mult {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Mult {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = (self.b.value(vm) as usize * self.c.value(vm) as usize) % 32768;
        self.a.write(effects, value as u16);
    }
}

#[derive(Debug)]
struct Mod {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Mod {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = (self.b.value(vm) % self.c.value(vm)) % 32768;
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct And {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for And {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = self.b.value(vm) & self.c.value(vm);
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Or {
    a: Operand,
    b: Operand,
    c: Operand,
}

impl Op for Or {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = self.b.value(vm) | self.c.value(vm);
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Not {
    a: Operand,
    b: Operand,
}

impl Op for Not {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = !self.b.value(vm) & ((1 << 15) - 1);
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Rmem {
    a: Operand,
    b: Operand,
}

impl Op for Rmem {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        let value = vm.memory[self.b.value(vm) as usize];
        self.a.write(effects, value);
    }
}

#[derive(Debug)]
struct Wmem {
    a: Operand,
    b: Operand,
}

impl Op for Wmem {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        effects.push(Effect::WriteMem((self.a.value(vm), self.b.value(vm))));
    }
}

#[derive(Debug)]
struct Call {
    a: Operand,
}

impl Op for Call {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {
        effects.push(Effect::PushAddr);
        effects.push(Effect::Jump(self.a.value(vm)));
    }
}

#[derive(Debug)]
struct Ret {}

impl Op for Ret {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, side_effects: &mut Vec<SideEffect>) {
        let Some(&value) = vm.stack.last() else {
            side_effects.push(SideEffect::Halt);
            return;
        };
        effects.push(Effect::Jump(value));
        effects.push(Effect::Pop);
    }
}

#[derive(Debug)]
struct Out {
    a: Operand,
}

impl Op for Out {
    fn execute(&self, vm: &VM, _effects: &mut Vec<Effect>, side_effects: &mut Vec<SideEffect>) {
        let ord = self.a.value(vm);
        let c = char::from_u32(ord as u32).expect(&format!("Value is not ascii: {ord}"));
        side_effects.push(SideEffect::Print(c));
    }
}

#[derive(Debug)]
struct In {
    a: Operand,
}

impl Op for In {
    fn execute(&self, vm: &VM, effects: &mut Vec<Effect>, side_effects: &mut Vec<SideEffect>) {
        let Some(ord) = vm.input else {
            side_effects.push(SideEffect::Read);
            effects.push(Effect::Jump(vm.pc));
            return;
        };
        effects.push(Effect::ConsumeInput);
        self.a.write(effects, ord);
    }
}

#[derive(Debug)]
struct Noop {}

impl Op for Noop {
    fn execute(&self, _vm: &VM, _effects: &mut Vec<Effect>, _side_effects: &mut Vec<SideEffect>) {}
}

mod test {
    use super::*;

    #[test]
    fn test_decode_out() {
        let instr = Instruction::decode(&[19, 1], 0);
        assert_eq!(
            format!("{instr:?}"),
            "Instruction { size: 2, op: Out { a: Literal(1) } }"
        );
    }

    #[test]
    fn test_execute_out() {
        let vm = VM::new(&[]);
        let instr = Instruction::decode(&[19, 65], 0);
        let (effects, side_effects) = instr.execute(&vm);
        assert_eq!(effects, vec![]);
        assert_eq!(side_effects, vec![SideEffect::Print('A')]);
    }
}
