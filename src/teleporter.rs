use std::collections::HashMap;

pub(crate) struct Teleporter {
    memo: HashMap<(u16, u16), u16>,
    magic: u16,
}

impl Teleporter {
    pub(crate) fn run() {
        for magic in 1..32768 {
            if Teleporter::new(magic).check(4, 1) == 6 {
                println!("Magic number is: {magic}");
            }
        }
        // Magic number is: 25734
    }

    fn new(magic: u16) -> Self {
        Self {
            memo: HashMap::new(),
            magic,
        }
    }

    fn check(&mut self, a: u16, b: u16) -> u16 {
        if let Some(result) = self.memo.get(&(a, b)) {
            return *result;
        }
        let result = if a == 0 {
            (b + 1) % 32768
        } else if b == 0 {
            self.check((a + 32767) % 32768, self.magic)
        } else {
            let b = self.check(a, (b + 32767) % 32768);
            self.check((a + 32767) % 32768, b)
        };
        self.memo.insert((a, b), result);
        result
    }
}
