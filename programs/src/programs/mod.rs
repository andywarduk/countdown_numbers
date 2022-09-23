//! This module is responsible for holding and running a collection of RPN programs

mod duplicates;
mod generate;

use std::cmp::max;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Index;

use colored::Colorize;
use itertools::Itertools;

use crate::infix::*;
use crate::progop::*;
use duplicates::*;
use generate::*;

/// Holds instruction element numbers for each program
pub struct ProgInstr {
    /// Start element of the instructions vector
    pub start: usize,
    /// End element of the instructions vector
    pub end: usize,
}

/// Collection of RPN program to run for a set of numbers
pub struct Programs {
    programs: Vec<ProgInstr>,
    instructions: Vec<ProgOp>,
    nums: u8,
}

impl Programs {
    /// Create a new Programs struct
    pub fn new(nums: u8, inc_duplicated: bool) -> Self {
        let operators = vec![ProgOp::OpAdd, ProgOp::OpSub, ProgOp::OpMul, ProgOp::OpDiv];

        Self::new_with_operators(nums, inc_duplicated, operators)
    }

    /// Create a new Programs struct with a given set of valid operators
    pub fn new_with_operators(nums: u8, inc_duplicated: bool, operators: Vec<ProgOp>) -> Self {
        // Create a vector to store the programs
        let prog_cnt = calc_num_programs(nums, inc_duplicated, &operators);
        let mut program_vec = Vec::with_capacity(prog_cnt);
        let mut instruction_vec = Vec::with_capacity(prog_cnt * (nums as usize + (nums as usize - 1)));

        for num_cnt in 1..=nums {
            // Generate operator counts
            let op_count = op_counts(num_cnt);

            // Generate operator combintions
            let op_comb = op_combs(num_cnt, &operators);

            // Generate programs
            generate_num_programs(
                &mut program_vec,
                &mut instruction_vec,
                nums,
                num_cnt,
                &op_count,
                &op_comb,
                inc_duplicated,
            );
        }

        program_vec.shrink_to_fit();
        instruction_vec.shrink_to_fit();

        Programs {
            programs: program_vec,
            instructions: instruction_vec,
            nums,
        }
    }

    /// Returns number of programs contained in the programs collection
    pub fn len(&self) -> usize {
        self.programs.len()
    }

    /// Returns true if the programs collection is empty
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    /// Runs one of the programs with a given set of numbers
    pub fn run(&self, prog_elem: usize, numbers: &[u32]) -> Result<u32, ProgErr> {
        let mut stack: Vec<u32> = Vec::with_capacity(self.nums as usize);

        run_instructions(self.instructions(prog_elem), numbers, &mut stack)
    }

    /// Runs all of the programs in the programs collection with a given set of numbers and returns the results
    pub fn run_all(&self, numbers: &Vec<u32>) -> Results {
        let mut stack: Vec<u32> = Vec::with_capacity(self.nums as usize);
        let mut results = Results::new();

        assert!(numbers.len() == self.nums as usize);

        for (i, program) in self.programs.iter().enumerate() {
            let instructions = self.instructions_for_program(program);

            match run_instructions(instructions, numbers, &mut stack) {
                Ok(ans) => {
                    if ans < 100 {
                        results.under_range += 1;
                    } else if ans > 999 {
                        results.above_range += 1;
                    } else {
                        results.solutions.push(Solution::new(i, instructions.len(), ans));
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
    pub fn run_all_target(&self, target: u32, numbers: &Vec<u32>) -> Vec<Solution> {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut solutions = Vec::new();

        assert!(numbers.len() == self.nums as usize);

        for (i, program) in self.programs.iter().enumerate() {
            let instructions = self.instructions_for_program(program);

            if let Ok(ans) = run_instructions(instructions, numbers, &mut stack) {
                if ans == target {
                    solutions.push(Solution::new(i, instructions.len(), ans));
                }
            }
        }

        solutions
    }

    /// Returns a slice of instructions for the program element
    pub fn instructions(&self, prog_elem: usize) -> &[ProgOp] {
        self.instructions_for_program(&self.programs[prog_elem])
    }

    #[inline]
    fn instructions_for_program(&self, program: &ProgInstr) -> &[ProgOp] {
        &self.instructions[program.start..=program.end]
    }

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, prog_elem: usize, numbers: &[u32], colour: bool) -> Vec<String> {
        let mut steps = Vec::new();
        let mut stack: Vec<(u32, String)> = Vec::with_capacity(numbers.len());

        process_instructions(
            self.instructions(prog_elem),
            &mut stack,
            |n| Some((numbers[n as usize], ProgOp::Number(n).colour(numbers, colour))),
            |(n2, s2), op, (n1, s1)| {
                let ans = match op {
                    ProgOp::OpAdd => n2 + n1,
                    ProgOp::OpSub => n2 - n1,
                    ProgOp::OpMul => n2 * n1,
                    ProgOp::OpDiv => n2 / n1,
                    _ => panic!("Non-operator not expected"),
                };

                let ans_str = ans.to_string();

                let equals = if colour {
                    "=".dimmed().to_string()
                } else {
                    "=".to_string()
                };

                steps.push(format!("{} {} {} {} {}", s2, op.colour(numbers, colour), s1, equals, ans_str));

                Some((ans, ans_str))
            },
        )
        .unwrap();

        steps
    }

    /// Converts the RPN program to operator type grouped infix equation
    pub fn infix(&self, prog_elem: usize, numbers: &[u32], colour: bool) -> String {
        infix_group(self.instructions(prog_elem)).colour(numbers, colour)
    }

    /// Converts the RPN program to full infix equation
    pub fn infix_full(&self, prog_elem: usize, numbers: &[u32], colour: bool) -> String {
        let mut stack: Vec<String> = Vec::with_capacity(numbers.len());

        let infix = process_instructions(
            self.instructions(prog_elem),
            &mut stack,
            |n| Some(ProgOp::Number(n as u8).colour(numbers, colour)),
            |s2, op, s1| Some(format!("({} {} {})", s2, op.colour(numbers, colour), s1)),
        )
        .unwrap();

        if infix.starts_with('(') {
            // Strip outer brackets
            infix[1..infix.len() - 1].to_string()
        } else {
            infix
        }
    }

    /// Converts the RPN program to a string for a given set of numbers
    pub fn rpn(&self, prog_elem: usize, numbers: &[u32], colour: bool) -> String {
        self.instructions(prog_elem)
            .iter()
            .map(|i| i.colour(numbers, colour))
            .join(" ")
    }

    /// Returns true if the program would be duplicated by rearranging the terms of the equation
    pub fn duplicated(
        &self,
        prog_elem: usize,
        stack: &mut Vec<InfixGrpTypeElem>,
        set: &mut HashSet<InfixGrpTypeElem>,
    ) -> bool {
        duplicated(self.instructions(prog_elem), stack, set)
    }
}

impl From<&str> for Programs {
    fn from(rpn: &str) -> Self {
        let instructions: Vec<ProgOp> = rpn
            .chars()
            .filter_map(|c| match c {
                '0'..='9' => Some(ProgOp::Number(c as u8 - b'0')),
                'a'..='z' => Some(ProgOp::Number(c as u8 - b'a')),
                'A'..='Z' => Some(ProgOp::Number(c as u8 - b'A')),
                '+' => Some(ProgOp::OpAdd),
                '-' => Some(ProgOp::OpSub),
                '*' => Some(ProgOp::OpMul),
                '/' => Some(ProgOp::OpDiv),
                _ => None,
            })
            .collect();

        let programs = vec![ProgInstr {
            start: 0,
            end: instructions.len() - 1,
        }];

        let nums = instructions.iter().fold(0, |max_n, i| match i {
            ProgOp::Number(n) => max(max_n, *n),
            _ => max_n,
        });

        Programs {
            programs,
            instructions,
            nums,
        }
    }
}

impl Index<usize> for Programs {
    type Output = ProgInstr;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.programs[idx]
    }
}

/// Holds the results of running all programs with a set of numbers
#[derive(Default)]
pub struct Results {
    /// Valid solution collection
    pub solutions: Vec<Solution>,
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

impl Results {
    /// Create new Result
    fn new() -> Self {
        Results::default()
    }
}

/// Holds the result of running a program
#[derive(Eq)]
pub struct Solution {
    /// Pointer to the program providing the solution
    pub program: usize,
    /// Length of the program
    length: usize,
    /// The result of running the program with the given numbers
    pub result: u32,
}

impl Solution {
    /// Creates a new Solution struct
    fn new(program: usize, length: usize, result: u32) -> Self {
        Solution {
            program,
            length,
            result,
        }
    }
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord = self.result.cmp(&other.result);

        if ord == Ordering::Equal {
            ord = self.length.cmp(&other.length);

            if ord == Ordering::Equal {
                ord = self.program.cmp(&other.program)
            }
        }

        ord
    }
}

impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.program == other.program
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prog_add() {
        let programs: Programs = "0 1 +".into();

        assert_eq!(Ok(7), programs.run(0, &[3, 4]));
    }

    #[test]
    fn prog_sub() {
        let programs: Programs = "0 1 -".into();

        assert_eq!(Ok(4), programs.run(0, &[7, 3]));
        assert_eq!(Err(ProgErr::Zero), programs.run(0, &[3, 3]));
        assert_eq!(Err(ProgErr::Negative), programs.run(0, &[3, 4]));
    }

    #[test]
    fn prog_mul() {
        let programs: Programs = "0 1 *".into();

        assert_eq!(Ok(21), programs.run(0, &[7, 3]));
        assert_eq!(Err(ProgErr::Mul1), programs.run(0, &[7, 1]));
        assert_eq!(Err(ProgErr::Mul1), programs.run(0, &[1, 3]));
        assert_eq!(Err(ProgErr::Zero), programs.run(0, &[7, 0]));
        assert_eq!(Err(ProgErr::Zero), programs.run(0, &[0, 3]));
        assert_eq!(Err(ProgErr::Zero), programs.run(0, &[0, 0]));
    }

    #[test]
    fn prog_div() {
        let programs: Programs = "0 1 /".into();

        assert_eq!(Ok(4), programs.run(0, &[12, 3]));
        assert_eq!(Err(ProgErr::NonInteger), programs.run(0, &[13, 3]));
        assert_eq!(Err(ProgErr::DivZero), programs.run(0, &[3, 0]));
        assert_eq!(Err(ProgErr::Div1), programs.run(0, &[3, 1]));
    }
}