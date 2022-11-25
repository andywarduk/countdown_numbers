#![warn(missing_docs)]

//! This module is responsible for holding and running a collection of RPN programs

mod duplicates;
mod generate;
mod infix;
mod progop;
mod solution;

use std::cmp::max;
use std::collections::{HashMap, HashSet};

use colored::Colorize;
use itertools::Itertools;

use duplicates::{duplicated, DupReason};
use generate::{calc_num_programs, generate_num_programs, op_combs, op_counts};
use infix::{infix_group, InfixGrpTypeElem};
use numformat::NumFormat;
use progop::ProgOp;

pub use solution::Solution;

/// Collection of RPN programs to run for a set of numbers
pub struct Programs {
    programs: Vec<ProgInstr>,
    instructions: Vec<ProgOp>,
    nums: u8,
}

impl Programs {
    /// Create a new Programs struct
    pub fn new(nums: u8, inc_duplicated: bool, verbose: bool) -> Self {
        let operators = vec![
            ProgOp::PROG_OP_ADD,
            ProgOp::PROG_OP_SUB,
            ProgOp::PROG_OP_MUL,
            ProgOp::PROG_OP_DIV,
        ];

        Self::new_with_operators(nums, inc_duplicated, operators, verbose)
    }

    /// Create a new Programs struct with a given set of operators
    pub fn new_with_operators(nums: u8, inc_duplicated: bool, operators: Vec<ProgOp>, verbose: bool) -> Self {
        // Calculate number permutations (=nums!)
        let num_perms: Vec<_> = (0..nums).permutations(nums as usize).collect();

        if verbose {
            println!("Card permutations: {}", num_perms.len().num_format());
        }

        // Calculate operator counts and combintions
        let mut op_map = HashMap::with_capacity(nums as usize);

        if verbose {
            println!("Operator placement counts and combinations for number of numbers:")
        }

        // Loop for the number of numbers in the RPN program
        for num_cnt in 1..=nums {
            // Generate operator count combinations for each operator slot
            let op_count = op_counts(num_cnt);

            // Generte operator combination
            let op_comb = op_combs(num_cnt, &operators);

            if verbose {
                println!(
                    "  {}: {:>6} {:>6}",
                    num_cnt,
                    op_count.len().num_format(),
                    op_comb.len().num_format()
                );
            }

            // Add to the hash map
            assert!(op_map.insert(num_cnt, (op_count, op_comb)).is_none());
        }

        // Create a vector to store the programs
        let prog_cnt_guess = calc_num_programs(nums, inc_duplicated, &num_perms, &op_map);
        let mut program_vec = Vec::with_capacity(prog_cnt_guess);

        // Create a vector to store program instructions
        let ins_cnt_guess = prog_cnt_guess * (nums as usize + (nums as usize - 1));
        let mut instruction_vec = Vec::with_capacity(ins_cnt_guess);

        // Vector to hold duplicate count
        let mut dups = Vec::with_capacity(nums as usize);

        // Loop for the number of numbers in the RPN program
        for num_cnt in 1..=nums {
            // Generate programs
            dups.push(generate_num_programs(
                &mut program_vec,
                &mut instruction_vec,
                num_cnt,
                &num_perms,
                &op_map,
                inc_duplicated,
            ));
        }

        if verbose {
            // Output some stats on the program generation
            if !inc_duplicated {
                println!("Duplicate programs filtered by number of numbers:");

                for (i, (term_dups, infix_dups)) in dups.iter().enumerate() {
                    println!(
                        "  {:>5}: terms {:>10}  infix {:>10}",
                        i + 1,
                        term_dups.num_format(),
                        infix_dups.num_format()
                    );
                }

                let (tterms, tinfix) = dups.iter().fold((0, 0), |(tt, ti), (t, i)| (tt + *t, ti + *i));

                println!(
                    "  Total: terms {:>10}  infix {:>10}",
                    tterms.num_format(),
                    tinfix.num_format()
                );
            }

            println!(
                "{} programs generated (guessed {})",
                program_vec.len().num_format(),
                prog_cnt_guess.num_format(),
            );

            println!(
                "{} total instructions (guessed {})",
                instruction_vec.len().num_format(),
                ins_cnt_guess.num_format(),
            );
        }

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
    pub fn run(&self, prog_elem: usize, numbers: &[u8]) -> Result<u32, ProgErr> {
        let instructions = self.instructions(prog_elem);
        let mut stack: Vec<u32> = Vec::with_capacity(self.nums as usize);

        Self::run_instructions(instructions, numbers, &mut stack)
    }

    /// Runs all of the programs in the programs collection with a given set of numbers and returns the results
    pub fn run_all(&self, numbers: &[u8]) -> Results {
        let mut stack: Vec<u32> = Vec::with_capacity(self.nums as usize);
        let mut results = Results::default();

        assert!(numbers.len() == self.nums as usize);

        for (i, program) in self.programs.iter().enumerate() {
            let instructions = self.instructions_for_program(program);

            match Self::run_instructions(instructions, numbers, &mut stack) {
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
                },
            }
        }

        results
    }

    /// Runs all of the programs in the programs collection with a given set of numbers and a target and returns the solutions
    pub fn run_all_target(&self, target: u32, numbers: &[u8]) -> Vec<Solution> {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut solutions = Vec::new();

        assert!(numbers.len() == self.nums as usize);

        for (i, program) in self.programs.iter().enumerate() {
            let instructions = self.instructions_for_program(program);

            if let Ok(ans) = Self::run_instructions(instructions, numbers, &mut stack) {
                if ans == target {
                    solutions.push(Solution::new(i, instructions.len(), ans));
                }
            }
        }

        solutions
    }

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, prog_elem: usize, numbers: &[u8], colour: bool) -> Vec<String> {
        let mut steps = Vec::new();
        let mut stack: Vec<(u32, String)> = Vec::with_capacity(numbers.len());

        self.process_program_instructions(
            prog_elem,
            &mut stack,
            |n| {
                Some((
                    numbers[n as usize] as u32,
                    ProgOp::new_number(n).colour(numbers, colour),
                ))
            },
            |(n2, s2), op, (n1, s1)| {
                let ans = match op & ProgOp::PROG_OP_MASK {
                    ProgOp::PROG_OP_ADD => n2 + n1,
                    ProgOp::PROG_OP_SUB => n2 - n1,
                    ProgOp::PROG_OP_MUL => n2 * n1,
                    ProgOp::PROG_OP_DIV => n2 / n1,
                    _ => panic!("Non-operator not expected"),
                };

                let ans_str = ans.num_format();

                let equals = if colour {
                    "=".dimmed().to_string()
                } else {
                    "=".to_string()
                };

                steps.push(format!(
                    "{} {} {} {} {}",
                    s2,
                    op.colour(numbers, colour),
                    s1,
                    equals,
                    ans_str
                ));

                Some((ans, ans_str))
            },
        )
        .unwrap();

        steps
    }

    /// Converts the RPN program to operator type grouped infix equation
    pub fn infix(&self, prog_elem: usize, numbers: &[u8], colour: bool) -> String {
        infix_group(self.instructions(prog_elem)).colour(numbers, colour)
    }

    /// Converts the RPN program to full infix equation
    pub fn infix_full(&self, prog_elem: usize, numbers: &[u8], colour: bool) -> String {
        let mut stack: Vec<String> = Vec::with_capacity(numbers.len());

        let infix = self
            .process_program_instructions(
                prog_elem,
                &mut stack,
                |n| Some(ProgOp::new_number(n).colour(numbers, colour)),
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
    pub fn rpn(&self, prog_elem: usize, numbers: &[u8], colour: bool) -> String {
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
        duplicated(self.instructions(prog_elem), stack, set) != DupReason::NotDup
    }

    // == Private functions ==

    /// Returns a slice of instructions for the program element
    #[inline]
    pub(crate) fn instructions(&self, prog_elem: usize) -> &[ProgOp] {
        self.instructions_for_program(&self.programs[prog_elem])
    }

    /// Returns a slice of instructions for the program instruction pointer struct
    #[inline]
    fn instructions_for_program(&self, program: &ProgInstr) -> &[ProgOp] {
        &self.instructions[program.start as usize..=program.end as usize]
    }

    /// Runs the program with a given set of numbers and preallocated stack
    #[inline]
    fn run_instructions(instructions: &[ProgOp], numbers: &[u8], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
        // NB this does not use the process function for speed
        stack.clear();

        for op in instructions {
            match *op & ProgOp::PROG_OP_MASK {
                ProgOp::PROG_OP_NUM => stack.push(numbers[op.bits() as usize] as u32),
                ProgOp::PROG_OP_ADD => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    stack.push(n2 + n1);
                }
                ProgOp::PROG_OP_SUB => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n2 < n1 {
                        Err(ProgErr::Negative)?
                    }

                    let int = n2 - n1;

                    if int == 0 {
                        Err(ProgErr::Zero)?
                    }

                    stack.push(int);
                }
                ProgOp::PROG_OP_MUL => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 1 || n2 == 1 {
                        Err(ProgErr::Mul1)?
                    }

                    let int = n2 * n1;

                    if int == 0 {
                        Err(ProgErr::Zero)?
                    }

                    stack.push(int);
                }
                ProgOp::PROG_OP_DIV => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 0 {
                        Err(ProgErr::DivZero)?
                    }

                    if n1 == 1 {
                        Err(ProgErr::Div1)?
                    }

                    if n2 % n1 != 0 {
                        Err(ProgErr::NonInteger)?
                    }

                    stack.push(n2 / n1);
                }
                _ => panic!("Unexpected operator type"),
            }
        }

        Ok(stack.pop().unwrap())
    }

    /// Processes a set of instructions for a program element calling callbacks for numbers and operations
    #[inline]
    fn process_program_instructions<S, N, T>(
        &self,
        prog_elem: usize,
        stack: &mut Vec<S>,
        num_cb: N,
        op_cb: T,
    ) -> Option<S>
    where
        N: FnMut(u8) -> Option<S>,
        T: FnMut(S, ProgOp, S) -> Option<S>,
    {
        Self::process_instructions(self.instructions(prog_elem), stack, num_cb, op_cb)
    }

    /// Processes a set of instructions calling callbacks for numbers and operations
    #[inline]
    pub(crate) fn process_instructions<S, N, T>(
        instructions: &[ProgOp],
        stack: &mut Vec<S>,
        mut num_cb: N,
        mut op_cb: T,
    ) -> Option<S>
    where
        N: FnMut(u8) -> Option<S>,
        T: FnMut(S, ProgOp, S) -> Option<S>,
    {
        stack.clear();

        for op in instructions {
            if op.is_number() {
                stack.push(num_cb(op.bits())?)
            } else {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(op_cb(n2, *op, n1)?)
            }
        }

        stack.pop()
    }
}

impl From<&str> for Programs {
    fn from(rpn: &str) -> Self {
        // Convert string to instructions vector
        let instructions: Vec<ProgOp> = rpn
            .chars()
            .filter_map(|c| match c {
                '0'..='9' => Some(ProgOp::new_number(c as u8 - b'0')),
                'a'..='z' => Some(ProgOp::new_number(c as u8 - b'a')),
                'A'..='Z' => Some(ProgOp::new_number(c as u8 - b'A')),
                '+' => Some(ProgOp::PROG_OP_ADD),
                '-' => Some(ProgOp::PROG_OP_SUB),
                '*' => Some(ProgOp::PROG_OP_MUL),
                '/' => Some(ProgOp::PROG_OP_DIV),
                _ => None,
            })
            .collect();

        // Add instruction pointers
        let programs = vec![ProgInstr {
            start: 0,
            end: (instructions.len() - 1) as u32,
        }];

        // Work out the maximum number present in the program
        let nums = instructions
            .iter()
            .fold(0, |max_n, i| if i.is_number() { max(max_n, i.bits()) } else { max_n });

        Programs {
            programs,
            instructions,
            nums,
        }
    }
}

/// Instruction element numbers for each program.
/// Pointers are 32 bits to keep the size down
pub(crate) struct ProgInstr {
    /// Start element of the instructions vector
    pub start: u32,
    /// End element of the instructions vector
    pub end: u32,
}

/// Errors generated by program run
#[derive(Debug, Eq, PartialEq)]
pub enum ProgErr {
    /// Program generated a zero intermediate result
    Zero,
    /// Program generated a negative intermediate result
    Negative,
    /// Program encountered a division by zero
    DivZero,
    /// Program encountered a non-integer intermediate result
    NonInteger,
    /// Program encountered multiply by 1 (noop)
    Mul1,
    /// Program encountered divide by 1 (noop)
    Div1,
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_size() {
        // Make sure ProgInstr is packed correctly (2 * 32 bit uints)
        assert_eq!(32 * 2, mem::size_of::<ProgInstr>() * 8);
    }

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
