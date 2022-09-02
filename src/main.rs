use itertools::Itertools;
use std::cmp::min;

fn main() {
    let mut results = Results::new();

//    calc(vec![2, 3, 4, 5, 6, 7], &mut results);
    calc(vec![100, 75, 50, 25, 10, 10], &mut results);

    println!("{} results, {} negative, {} div by zero, {} non-integer, {} < 100, {} > 999",
        results.solutions.len(), results.negative, results.div_zero, results.non_integer,
        results.under_range, results.above_range);
}

fn calc(numbers: Vec<u32>, results: &mut Results)
{
    let nums = numbers.len();

    for num_cnt in 2..=nums {
        // Generate operator counts
        let op_count = op_counts(num_cnt);

        // Generate operator combintions
        let op_comb = op_combs(num_cnt);

        // Generate programs
        generate_programs(results, &numbers, num_cnt, &op_count, &op_comb);
    }
}

fn op_counts(nums: usize) -> Vec<Vec<usize>> {
    let mut results = Vec::new();

    op_counts_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, nums - 1, 2);

    results
}

fn op_counts_rec(results: &mut Vec<Vec<usize>>, mut current: Vec<usize>, slot: usize, slots: usize, to_alloc: usize, stacked: usize)
{
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

fn op_combs(nums: usize) -> Vec<Vec<ProgOp>> {
    let mut results = Vec::new();

    op_combs_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1);

    results
}

fn op_combs_rec(results: &mut Vec<Vec<ProgOp>>, current: Vec<ProgOp>, slot: usize, slots: usize)
{
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

fn generate_programs(results: &mut Results, numbers: &[u32], num_cnt: usize, op_counts: &Vec<Vec<usize>>, op_combs: &Vec<Vec<ProgOp>>)
{
    for nums in numbers.iter().permutations(num_cnt) {
        for op_count in op_counts {
            for op_comb in op_combs {
                let mut program = Program::new(num_cnt);
                let mut op_index = 0;

                // Push first number
                program.push(ProgOp::Number(*nums[0]));

                for i in 0..(num_cnt - 1) {
                    // Push number
                    program.push(ProgOp::Number(*nums[i + 1]));

                    // Push operators
                    for _ in 0..op_count[i] {
                        program.push(op_comb[op_index]);
                        op_index += 1;
                    }
                }

                match program.run() {
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ProgOp {
    Number(u32),
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

#[derive(Debug)]
struct Program {
    instructions: Vec<ProgOp>
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

    fn run(&self) -> Result<u32, ProgErr> {
        let mut stack: Vec<u32> = Vec::new();

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => stack.push(*x),
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

}

#[derive(Default)]
struct Results {
    solutions: Vec<Solution>,
    under_range: usize,
    above_range: usize,
    negative: usize,
    div_zero: usize,
    non_integer: usize,
}

impl Results {

    fn new() -> Self {
        Results::default()
    }

}

struct Solution {
    program: Program,
    result: u32
}

impl Solution {

    fn new(program: Program, result: u32) -> Self {
        Solution {
            program,
            result
        }
    }

}