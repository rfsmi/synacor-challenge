use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Operation {
    Add,
    Sub,
    Mult,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operation::Add => '+',
                Operation::Mult => '*',
                Operation::Sub => '-',
            }
        )
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct State {
    coord: (usize, usize),
    value: u16,
    op: Option<Operation>,
}

#[derive(Clone, Copy)]
enum Node {
    Op(Operation),
    Value(u16),
    Start,
    End(u16),
}

pub(crate) struct Maze {
    nodes: Vec<Vec<Node>>,
    width: isize,
    height: isize,
}

impl Maze {
    pub(crate) fn solve() {
        let start = State {
            coord: (0, 3),
            op: None,
            value: 22,
        };
        let end = State {
            coord: (3, 0),
            op: None,
            value: 30,
        };
        let maze = Maze::new();
        let mut queue: VecDeque<(State, Option<State>)> = [(start, None)].into();
        let mut parents: HashMap<State, State> = HashMap::new();
        while let Some((state, parent)) = queue.pop_front() {
            if parents.contains_key(&state) {
                continue;
            }
            if let Some(parent) = parent {
                parents.insert(state, parent);
            }
            if state == end {
                break;
            }
            if let Node::End(_) = &maze.nodes[state.coord.1][state.coord.0] {
                continue; // Can't go anywhere else from the end
            }
            for (x, y) in maze.neighbours(state.coord) {
                let mut child = State {
                    coord: (x, y),
                    op: None,
                    value: state.value,
                };
                match &maze.nodes[y][x] {
                    Node::Start => continue,
                    Node::Op(op) => child.op = Some(*op),
                    Node::End(val) | Node::Value(val) => {
                        child.value = match state.op {
                            Some(Operation::Add) => state.value + val,
                            Some(Operation::Mult) => state.value * val,
                            Some(Operation::Sub) => state.value - val,
                            None => unreachable!(),
                        }
                    }
                }
                queue.push_back((child, Some(state)));
            }
        }
        let mut moves = Vec::new();
        let mut state = end;
        while let Some(&parent) = parents.get(&state) {
            let instruction = if parent.coord.0 < state.coord.0 {
                "go east"
            } else if parent.coord.0 > state.coord.0 {
                "go west"
            } else if parent.coord.1 < state.coord.1 {
                "go south"
            } else {
                "go north"
            };
            let node = &maze.nodes[state.coord.1][state.coord.0];
            let description = match node {
                Node::Start => unreachable!(),
                Node::Op(op) => op.to_string(),
                Node::End(val) | Node::Value(val) => format!("{val} = {}", state.value),
            };
            moves.push(format!("{instruction} # {description}"));
            state = parent;
        }
        for instruction in moves.iter().rev() {
            println!("{instruction}");
        }
    }

    fn new() -> Self {
        use Node::{End, Op, Start, Value};
        use Operation::{Add, Mult, Sub};
        Self {
            nodes: vec![
                vec![Op(Mult), Value(8), Op(Sub), End(1)],
                vec![Value(4), Op(Mult), Value(11), Op(Mult)],
                vec![Op(Add), Value(4), Op(Sub), Value(18)],
                vec![Start, Op(Sub), Value(9), Op(Mult)],
            ],
            width: 4,
            height: 4,
        }
    }

    fn neighbours(&self, (x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> + '_ {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let (x, y) = (x as isize + dx, y as isize + dy);
                if x >= 0 && x < self.width && y >= 0 && y < self.height {
                    Some((x as usize, y as usize))
                } else {
                    None
                }
            })
    }
}
