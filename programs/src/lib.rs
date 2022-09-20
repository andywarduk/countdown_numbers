pub mod infix;
pub mod progop;
pub mod program;
pub mod duplicates;

use itertools::Itertools;
use std::cmp::{min, Ordering};
use std::collections::HashSet;
use colored::*;

use crate::progop::*;
use crate::program::*;
use crate::infix::*;

/// Collection of RPN program to run for a set of numbers
pub struct Programs {
    programs: Vec<Program>,
    nums: usize
}

impl Programs {

    /// Create a new Programs struct
    pub fn new(nums: usize, inc_commutative: bool) -> Self {
        let operators = vec![ProgOp::OpAdd, ProgOp::OpSub, ProgOp::OpMul, ProgOp::OpDiv];

        Self::new_with_operators(nums, inc_commutative, operators)
    }

    /// Create a new Programs struct with a given set of valid operators
    fn new_with_operators(nums: usize, inc_commutative: bool, operators: Vec<ProgOp>) -> Self {
        let mut programs = Programs {
            programs: Vec::new(),
            nums
        };

        for num_cnt in 1..=nums {
            // Generate operator counts
            let op_count = op_counts(num_cnt);
    
            // Generate operator combintions
            let op_comb = op_combs(num_cnt, &operators);

            // Generate programs
            generate_num_programs(&mut programs, nums, num_cnt, &op_count, &op_comb, inc_commutative);
        }
    
        programs
    }

    /// Returns number of programs contained in the programs collection
    pub fn len(&self) -> usize {
        self.programs.len()
    }

    /// Returns true if the programs collection is empty
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    /// Runs all of the programs in the programs collection with a given set of numbers and returns the results
    pub fn run(&self, numbers: &Vec<u32>) -> Results {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut results = Results::new();

        assert!(numbers.len() == self.nums);

        for program in &self.programs {
            match program.run(numbers, &mut stack) {
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
                        ProgErr::Zero => results.zero += 1,
                        ProgErr::Negative => results.negative += 1,
                        ProgErr::DivZero => results.div_zero += 1,
                        ProgErr::NonInteger => results.non_integer += 1,
                        ProgErr::Mul1 => results.mult_by_1 += 1,
                        ProgErr::Div1 => results.div_by_1 += 1,
                    }
                }
            }
        }

        results
    }

    /// Runs all of the programs in the programs collection with a given set of numbers and a target and returns the solutions
    pub fn run_target(&self, target: u32, numbers: &Vec<u32>) -> Vec<Solution> {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut solutions = Vec::new();

        assert!(numbers.len() == self.nums);

        for program in &self.programs {
            if let Ok(ans) = program.run(numbers, &mut stack) {
                if ans == target {
                    solutions.push(Solution::new(program, ans));
                }
            }
        }

        solutions
    }

    /// Adds a program to the programs collection
    fn push(&mut self, program: Program) {
        self.programs.push(program)
    }
    
}

/// Holds the results of running all programs with a set of numbers
#[derive(Default)]
pub struct Results<'a> {
    pub solutions: Vec<Solution<'a>>, // Valid solution collection
    pub under_range: usize,           // Number of programs with answer below valid range
    pub above_range: usize,           // Number of programs with answer above valid range
    pub zero: usize,                  // Number of programs with zero intermediate result
    pub negative: usize,              // Number of programs with negative intermediate result
    pub div_zero: usize,              // Number of programs encountering division by zero
    pub non_integer: usize,           // Number of programs with non-integer intermediate result
    pub mult_by_1: usize,             // Number of programs containing a multipy by 1
    pub div_by_1: usize               // Number of programs containing a divide by 1
}

impl<'a> Results<'a> {

    /// Create new Result
    fn new() -> Self {
        Results::default()
    }

}

/// Holds the result of running a program
#[derive(Eq)]
pub struct Solution<'a> {
    program: &'a Program,
    pub result: u32
}

impl<'a> Solution<'a> {

    /// Creates a new Solution struct
    fn new(program: &'a Program, result: u32) -> Self {
        Solution {
            program,
            result
        }
    }

    /// Returns the program equation in infix style
    pub fn program_infix(&self, numbers: &[u32], mode: InfixGrpMode) -> String {
        format!("{} {} {}", self.program.infix(numbers, mode, true), "=".dimmed(), self.result)
    }

    /// Returns the program equation in infix style in discrete steps
    pub fn program_steps(&self, numbers: &[u32]) -> Vec<String> {
        self.program.steps(numbers, true)
    }

    /// Returns the program equation in reverse polish notation
    pub fn program_rpn(&self, numbers: &[u32]) -> String {
        self.program.rpn(numbers, true)
    }

}

impl<'a> Ord for Solution<'a> {

    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord = self.result.cmp(&other.result);

        if ord == Ordering::Equal {
            ord = self.program.len().cmp(&other.program.len())
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
        self.result == other.result && self.program.len() == other.program.len()
    }

}

// Support functions

fn generate_num_programs(programs: &mut Programs, nums: usize, num_cnt: usize, op_counts: &OpCounts, op_combs: &Vec<Vec<ProgOp>>, inc_commutative: bool) {
    let mut set = HashSet::new();

    for nums in (0..nums).permutations(num_cnt) {
        if num_cnt == 1 {
            let mut program = Program::new(num_cnt);

            // Push the number
            program.push(ProgOp::Number(nums[0] as u8));

            programs.push(program);

        } else {
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

                    // Commutative check
                    if inc_commutative || program.duplicate_filter(&mut set) {
                        programs.push(program);
                    }
                }
            }
            
        }
    }
}

type OpCounts = Vec<Vec<usize>>;

fn op_counts(nums: usize) -> OpCounts {
    let mut results = Vec::new();

    if nums > 1 {
        op_counts_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, nums - 1, 2);
    }

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

fn op_combs(nums: usize, operators: &Vec<ProgOp>) -> OpCombs {
    let mut results = Vec::new();

    if nums > 1 {
        op_combs_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, operators);
    }

    results
}

fn op_combs_rec(results: &mut OpCombs, current: Vec<ProgOp>, slot: usize, slots: usize, operators: &Vec<ProgOp>) {
    let mut add = |op: ProgOp| {
        let mut next = current.clone();
        next.push(op);

        if slot == slots - 1 {
            results.push(next)
        } else {
            op_combs_rec(results, next, slot + 1, slots, operators)
        }
    };

    for op in operators.iter() {
        add(*op);
    }
}
