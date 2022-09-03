use itertools::Itertools;
use std::cmp::{min, Ordering};
use std::fmt;

pub struct Programs {
    programs: Vec<Program>,
    nums: usize
}

impl Programs {

    pub fn new(nums: usize) -> Self {
        let mut programs = Programs {
            programs: Vec::new(),
            nums
        };

        for num_cnt in 2..=nums {
            // Generate operator counts
            let op_count = op_counts(num_cnt);
    
            // Generate operator combintions
            let op_comb = op_combs(num_cnt);
    
            // Generate programs
            generate_num_programs(&mut programs, nums, num_cnt, &op_count, &op_comb);
        }
    
        programs
    }

    fn push(&mut self, program: Program) {
        self.programs.push(program)
    }

    pub fn run(&self, numbers: &Vec<u32>) -> Results {
        let mut results = Results::new();

        assert!(numbers.len() == self.nums);

        for program in &self.programs {
            match program.run(numbers) {
                Ok(ans) => {
                    if ans < 100 {
                        results.under_range += 1;
                    } else if ans > 999 {
                        results.above_range += 1;
                    } else {
                        results.solutions.push(Solution::new(program, ans));
                    }
                }
                Err(e) => {
                    match e {
                        ProgErr::Negative => results.negative += 1,
                        ProgErr::DivZero => results.div_zero += 1,
                        ProgErr::NonInteger => results.non_integer += 1,
                    }
                }
            }
        }

        results
    }

}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ProgOp {
    Number(u8),
    OpAdd,
    OpSub,
    OpMul,
    OpDiv
}

enum ProgErr {
    Negative,
    DivZero,
    NonInteger
}

#[derive(Debug, Eq, PartialEq)]
struct Program {
    instructions: Vec<ProgOp>
}

enum ProgFmt {
    Expr(String),
    Num(u32)
}

impl fmt::Display for ProgFmt {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgFmt::Expr(s) => write!(f, "({})", s),
            ProgFmt::Num(n) => write!(f, "{}", n)
        }
    }

}

impl Program {

    fn new(num_cnt: usize) -> Self {
        Program {
            instructions: Vec::with_capacity(num_cnt + (num_cnt - 1))
        }
    }

    fn push(&mut self, op: ProgOp) {
        self.instructions.push(op);
    } 

    fn run(&self, numbers: &[u32]) -> Result<u32, ProgErr> {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => stack.push(numbers[*x as usize]),
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(n2 + n1);
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    if n2 < n1 {
                        return Err(ProgErr::Negative)
                    }
                    stack.push(n2 - n1);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(n2 * n1);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    if n1 == 0 {
                        return Err(ProgErr::DivZero)
                    }
                    if n2 % n1 != 0 {
                        return Err(ProgErr::NonInteger)
                    }
                    stack.push(n2 / n1);
                },
            }
        }

        Ok(stack.pop().unwrap())
    }

    fn format(&self, numbers: &[u32]) -> String {
        let mut stack: Vec<ProgFmt> = Vec::new();

        for op in self.instructions.iter() {
            match op {
                ProgOp::Number(x) => stack.push(ProgFmt::Num(numbers[*x as usize])),
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(ProgFmt::Expr(format!("{} + {}", n2, n1)));
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(ProgFmt::Expr(format!("{} - {}", n2, n1)));
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(ProgFmt::Expr(format!("{} * {}", n2, n1)));
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(ProgFmt::Expr(format!("{} / {}", n2, n1)));
                },
            }
        }

        match stack.pop().unwrap() {
            ProgFmt::Expr(s) => s,
            ProgFmt::Num(n) => format!("{}", n),
        }
    }

}

#[derive(Default)]
pub struct Results<'a> {
    pub solutions: Vec<Solution<'a>>,
    pub under_range: usize,
    pub above_range: usize,
    pub negative: usize,
    pub div_zero: usize,
    pub non_integer: usize,
}

impl<'a> Results<'a> {

    fn new() -> Self {
        Results::default()
    }

}

#[derive(Eq)]
pub struct Solution<'a> {
    program: &'a Program,
    pub result: u32
}

impl<'a> Solution<'a> {

    fn new(program: &'a Program, result: u32) -> Self {
        Solution {
            program,
            result
        }
    }

    pub fn format(&self, numbers: &[u32]) -> String {
        format!("{} = {}", self.program.format(numbers), self.result)
    }

}

impl<'a> Ord for Solution<'a> {

    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord = self.result.cmp(&other.result);

        if ord == Ordering::Equal {
            ord = self.program.instructions.len().cmp(&other.program.instructions.len())
        }

        ord
    }

}

impl<'a> PartialOrd for Solution<'a> {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

}

impl<'a> PartialEq for Solution<'a> {

    fn eq(&self, other: &Self) -> bool {
        self.result == other.result && self.program.instructions.len() == other.program.instructions.len()
    }

}

fn generate_num_programs(programs: &mut Programs, nums: usize, num_cnt: usize, op_counts: &OpCounts, op_combs: &Vec<Vec<ProgOp>>) {
    for nums in (0..nums).permutations(num_cnt) {
        for op_count in op_counts {
            for op_comb in op_combs {
                let mut program = Program::new(num_cnt);
                let mut op_index = 0;

                // Push first number
                program.push(ProgOp::Number(nums[0] as u8));

                for i in 0..(num_cnt - 1) {
                    // Push number
                    program.push(ProgOp::Number(nums[i + 1] as u8));

                    // Push operators
                    for _ in 0..op_count[i] {
                        program.push(op_comb[op_index]);
                        op_index += 1;
                    }
                }

                programs.push(program);
            }
        }
    }
}

type OpCounts = Vec<Vec<usize>>;

fn op_counts(nums: usize) -> OpCounts {
    let mut results = Vec::new();

    op_counts_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, nums - 1, 2);

    results
}

fn op_counts_rec(results: &mut OpCounts, mut current: Vec<usize>, slot: usize, slots: usize, to_alloc: usize, stacked: usize) {
    if slot == slots - 1 {
        // Allocate all to the last slot
        current.push(to_alloc);
        results.push(current);

    } else {
        // How many can we allocate to this slot?
        let max_stack = stacked - 1;

        for i in 0..=min(to_alloc - 1, max_stack) {
            let mut next = current.clone();
            next.push(i);
            let next_stacked = stacked + 1 - i;
            op_counts_rec(results, next, slot + 1, slots, to_alloc - i, next_stacked);
        }

    }
}

type OpCombs = Vec<Vec<ProgOp>>;

fn op_combs(nums: usize) -> OpCombs {
    let mut results = Vec::new();

    op_combs_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1);

    results
}

fn op_combs_rec(results: &mut OpCombs, current: Vec<ProgOp>, slot: usize, slots: usize) {
    let mut add = |op: ProgOp| {
        let mut next = current.clone();
        next.push(op);
        if slot == slots - 1 {
            results.push(next)
        } else {
            op_combs_rec(results, next, slot + 1, slots)
        }
    };

    add(ProgOp::OpAdd);
    add(ProgOp::OpSub);
    add(ProgOp::OpMul);
    add(ProgOp::OpDiv);
}
