use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;

pub(crate) struct Teleporter {
    memo: HashMap<(u16, u16), u16>,
    magic: u16,
}

impl Teleporter {
    pub(crate) fn run() {
        // Magic number is: 25734
        println!("Trying all possible values: 0-32767");
        let style = ProgressStyle::default_bar().progress_chars("##-");
        let pb = ProgressBar::new(32768).with_style(style);
        for magic in 0..32768 {
            if Teleporter::new(magic).check(4, 1) == 6 {
                pb.println(format!("Found magic number: {magic}"));
            }
            pb.inc(1);
        }
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
