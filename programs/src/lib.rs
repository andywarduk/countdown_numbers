#![warn(missing_docs)]

//! This module 

pub mod duplicates;
pub mod generate;
pub mod infix;
pub mod progop;
pub mod program;

use std::cmp::Ordering;

use generate::*;
use progop::*;
use program::*;

/// RPN programs

/// Collection of RPN program to run for a set of numbers
pub struct Programs {
    programs: Vec<Program>,
    nums: usize,
}

impl Programs {
    /// Create a new Programs struct
    pub fn new(nums: usize, inc_duplicated: bool) -> Self {
        let operators = vec![ProgOp::OpAdd, ProgOp::OpSub, ProgOp::OpMul, ProgOp::OpDiv];

        Self::new_with_operators(nums, inc_duplicated, operators)
    }

    /// Create a new Programs struct with a given set of valid operators
    pub fn new_with_operators(nums: usize, inc_duplicated: bool, operators: Vec<ProgOp>) -> Self {
        let mut programs = Programs { programs: Vec::new(), nums };

        for num_cnt in 1..=nums {
            // Generate operator counts
            let op_count = op_counts(num_cnt);

            // Generate operator combintions
            let op_comb = op_combs(num_cnt, &operators);

            // Generate programs
            generate_num_programs(&mut programs, nums, num_cnt, &op_count, &op_comb, inc_duplicated);
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
                Err(e) => match e {
                    ProgErr::Zero => results.zero += 1,
                    ProgErr::Negative => results.negative += 1,
                    ProgErr::DivZero => results.div_zero += 1,
                    ProgErr::NonInteger => results.non_integer += 1,
                    ProgErr::Mul1 => results.mult_by_1 += 1,
                    ProgErr::Div1 => results.div_by_1 += 1,
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
    /// Valid solution collection
    pub solutions: Vec<Solution<'a>>, 
    /// Number of programs with answer below valid range
    pub under_range: usize,           
    /// Number of programs with answer above valid range
    pub above_range: usize,           
    /// Number of programs with zero intermediate result
    pub zero: usize,                  
    /// Number of programs with negative intermediate result
    pub negative: usize,              
    /// Number of programs encountering division by zero
    pub div_zero: usize,              
    /// Number of programs with non-integer intermediate result
    pub non_integer: usize,           
    /// Number of programs containing a multipy by 1
    pub mult_by_1: usize,             
    /// Number of programs containing a divide by 1
    pub div_by_1: usize,              
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
    /// Pointer to the program providing the solution
    pub program: &'a Program,
    /// The result of running the program with the given numbers
    pub result: u32,
}

impl<'a> Solution<'a> {
    /// Creates a new Solution struct
    fn new(program: &'a Program, result: u32) -> Self {
        Solution { program, result }
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
