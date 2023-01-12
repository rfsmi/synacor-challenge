use indicatif::{ProgressBar, ProgressStyle};
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use std::{collections::HashMap, sync::Mutex};

pub(crate) struct Teleporter {
    memo: HashMap<(u16, u16), u16>,
    magic: u16,
}

impl Teleporter {
    pub(crate) fn run() {
        // Magic number is: 25734
        ThreadPoolBuilder::default()
            .stack_size(100_000_000)
            .build_global()
            .unwrap();
        println!("Trying all possible values: 0-32767");

        let style = ProgressStyle::default_bar().progress_chars("#>-");
        let pb = Mutex::new(ProgressBar::new(32768).with_style(style));

        (0..32768)
            .into_par_iter()
            .filter_map(|magic| {
                pb.lock().unwrap().inc(1);
                if Teleporter::new(magic).check(4, 1) == 6 {
                    Some(magic)
                } else {
                    None
                }
            })
            .for_each(|num| {
                pb.lock()
                    .unwrap()
                    .println(format!("Found magic number: {num}"))
            });
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
