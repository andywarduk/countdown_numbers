pub mod op_tree;
pub mod progop;
pub mod program;

use itertools::Itertools;
use std::cmp::{min, Ordering};
use colored::*;

use progop::*;
use program::*;

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
    pub fn program_equation(&self, numbers: &[u32]) -> String {
        format!("{} {} {}", self.program.equation(numbers, true), "=".dimmed(), self.result)
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
                    if inc_commutative || program.commutative_filter() {
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prog_add() {
        let mut stack: Vec<u32> = Vec::new();
        let mut program = Program::new(2);

        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpAdd);

        assert_eq!(Ok(7), program.run(&[3, 4], &mut stack));
    }

    #[test]
    fn prog_sub() {
        let mut stack: Vec<u32> = Vec::new();
        let mut program = Program::new(2);

        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpSub);

        assert_eq!(Ok(4), program.run(&[7, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[3, 3], &mut stack));
        assert_eq!(Err(ProgErr::Negative), program.run(&[3, 4], &mut stack));
    }

    #[test]
    fn prog_mul() {
        let mut stack: Vec<u32> = Vec::new();
        let mut program = Program::new(2);

        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpMul);

        assert_eq!(Ok(21), program.run(&[7, 3], &mut stack));
        assert_eq!(Err(ProgErr::Mul1), program.run(&[7, 1], &mut stack));
        assert_eq!(Err(ProgErr::Mul1), program.run(&[1, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[7, 0], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[0, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[0, 0], &mut stack));
    }

    #[test]
    fn prog_div() {
        let mut stack: Vec<u32> = Vec::new();
        let mut program = Program::new(2);

        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpDiv);

        assert_eq!(Ok(4), program.run(&[12, 3], &mut stack));
        assert_eq!(Err(ProgErr::NonInteger), program.run(&[13, 3], &mut stack));
        assert_eq!(Err(ProgErr::DivZero), program.run(&[3, 0], &mut stack));
        assert_eq!(Err(ProgErr::Div1), program.run(&[3, 1], &mut stack));
    }

    #[test]
    fn eqn_test() {
        let mut program1 = Program::new(3);
        let mut program2 = Program::new(3);

        // 25 5 100 + -
        program1.push(ProgOp::Number(0));
        program1.push(ProgOp::Number(1));
        program1.push(ProgOp::Number(2));
        program1.push(ProgOp::OpAdd);
        program1.push(ProgOp::OpSub);

        // 25 5 - 100 +
        program2.push(ProgOp::Number(0));
        program2.push(ProgOp::Number(1));
        program2.push(ProgOp::OpSub);
        program2.push(ProgOp::Number(2));
        program2.push(ProgOp::OpAdd);

        let numbers = [25, 5, 4];

        println!("{:?}", program1.op_tree());
        println!("1: rpn: {} eqn: {} steps: {}",
            program1.rpn(&numbers, true),
            program1.equation(&numbers, true),
            program1.steps(&numbers, true).iter().join(", ")
        );

        println!("2: rpn: {} eqn: {} steps: {}",
            program2.rpn(&numbers, true),
            program2.equation(&numbers, true),
            program2.steps(&numbers, true).iter().join(", ")
        );

        assert!(1==0);
    }

    #[test]
    fn commutative_filter_test_mul() {
        let programs = Programs::new_with_operators(4, false, vec![ProgOp::OpMul]);

        let numbers = vec![1, 2, 3, 4];

        for p in &programs.programs {
            println!("RPN: {}  Equation: {}", p.rpn(&numbers, true), p.equation(&numbers, true));
        }

        assert_eq!(15, programs.len());

        assert_eq!("1", programs.programs[0].equation(&numbers, false));
        assert_eq!("2", programs.programs[1].equation(&numbers, false));
        assert_eq!("3", programs.programs[2].equation(&numbers, false));
        assert_eq!("4", programs.programs[3].equation(&numbers, false));

        assert_eq!("2 × 1", programs.programs[4].equation(&numbers, false));
        assert_eq!("3 × 1", programs.programs[5].equation(&numbers, false));
        assert_eq!("3 × 2", programs.programs[6].equation(&numbers, false));
        assert_eq!("4 × 1", programs.programs[7].equation(&numbers, false));
        assert_eq!("4 × 2", programs.programs[8].equation(&numbers, false));
        assert_eq!("4 × 3", programs.programs[9].equation(&numbers, false));

        assert_eq!("3 × 2 × 1", programs.programs[10].equation(&numbers, false));
        assert_eq!("4 × 2 × 1", programs.programs[11].equation(&numbers, false));
        assert_eq!("4 × 3 × 1", programs.programs[12].equation(&numbers, false));
        assert_eq!("4 × 3 × 2", programs.programs[13].equation(&numbers, false));

        assert_eq!("4 × 3 × 2 × 1", programs.programs[14].equation(&numbers, false));
    }

    #[test]
    fn commutative_filter_test_add() {
        let programs = Programs::new_with_operators(4, false, vec![ProgOp::OpAdd]);

        let numbers = vec![1, 2, 3, 4];

        for p in &programs.programs {
            println!("RPN: {}  Equation: {}", p.rpn(&numbers, true), p.equation(&numbers, true));
        }

        assert_eq!(15, programs.len());

        assert_eq!("1", programs.programs[0].equation(&numbers, false));
        assert_eq!("2", programs.programs[1].equation(&numbers, false));
        assert_eq!("3", programs.programs[2].equation(&numbers, false));
        assert_eq!("4", programs.programs[3].equation(&numbers, false));

        assert_eq!("2 + 1", programs.programs[4].equation(&numbers, false));
        assert_eq!("3 + 1", programs.programs[5].equation(&numbers, false));
        assert_eq!("3 + 2", programs.programs[6].equation(&numbers, false));
        assert_eq!("4 + 1", programs.programs[7].equation(&numbers, false));
        assert_eq!("4 + 2", programs.programs[8].equation(&numbers, false));
        assert_eq!("4 + 3", programs.programs[9].equation(&numbers, false));

        assert_eq!("3 + 2 + 1", programs.programs[10].equation(&numbers, false));
        assert_eq!("4 + 2 + 1", programs.programs[11].equation(&numbers, false));
        assert_eq!("4 + 3 + 1", programs.programs[12].equation(&numbers, false));
        assert_eq!("4 + 3 + 2", programs.programs[13].equation(&numbers, false));

        assert_eq!("4 + 3 + 2 + 1", programs.programs[14].equation(&numbers, false));
    }

}
