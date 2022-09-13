use itertools::Itertools;
use std::cmp::{min, Ordering};
use std::fmt;
use colored::*;

// Programs struct - collection of RPN program to run for a set of numbers

pub struct Programs {
    programs: Vec<Program>,
    nums: usize
}

impl Programs {

    // Create a new Programs struct
    pub fn new(nums: usize) -> Self {
        let mut programs = Programs {
            programs: Vec::new(),
            nums
        };

        for num_cnt in 1..=nums {
            // Generate operator counts
            let op_count = op_counts(num_cnt);
    
            // Generate operator combintions
            let op_comb = op_combs(num_cnt);
    
            // Generate programs
            generate_num_programs(&mut programs, nums, num_cnt, &op_count, &op_comb);
        }
    
        programs
    }

    // Returns number of programs contained in the programs collection
    pub fn len(&self) -> usize {
        self.programs.len()
    }

    // Returns true if the programs collection is empty
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    // Adds a program to the programs collection
    fn push(&mut self, program: Program) {
        self.programs.push(program)
    }

    // Runs all of the programs in the programs collection with a given set of numbers and returns the results
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
                        ProgErr::Negative => results.negative += 1,
                        ProgErr::DivZero => results.div_zero += 1,
                        ProgErr::NonInteger => results.non_integer += 1,
                    }
                }
            }
        }

        results
    }

    // Runs all of the programs in the programs collection with a given set of numbers and a target and returns the solutions
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

}

// ProgOp enum - RPN program items and operators

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ProgOp {
    Number(u8),
    OpAdd,
    OpSub,
    OpMul,
    OpDiv
}

// ProgErr enum - errors generated by RPN program run

#[derive(Debug, Eq, PartialEq)]
enum ProgErr {
    Negative,  // Program generated a negative intermediate result
    DivZero,   // Program encountered a division by zero
    NonInteger // Program encountered a non-integer intermediate result
}

// ProgFmt enum - used to convert RPN program to infix style

enum ProgFmt {
    Expr(String), // A composite experssion
    Num(u32)      // A single number
}

impl fmt::Display for ProgFmt {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgFmt::Expr(s) => write!(f, "({})", s),
            ProgFmt::Num(n) => write!(f, "{}", n)
        }
    }

}

// Program struct - hold a single RPN program

#[derive(Eq, PartialEq)]
struct Program {
    instructions: Vec<ProgOp>
}

impl Program {

    // Creates a new program
    fn new(num_cnt: usize) -> Self {
        Program {
            instructions: Vec::with_capacity(num_cnt + (num_cnt - 1))
        }
    }

    // Adds an instruction to the program
    fn push(&mut self, op: ProgOp) {
        self.instructions.push(op);
    } 

    // Runs the program with a given set of numbers and preallocated stack
    fn run(&self, numbers: &[u32], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
        stack.clear();

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

    // Prints the formatted steps of a program for a given set of numbers
    fn print_steps(&self, numbers: &[u32]) {
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut str_stack: Vec<String> = Vec::with_capacity(numbers.len());

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => {
                    stack.push(numbers[*x as usize]);
                    str_stack.push(format!("{}", numbers[*x as usize].to_string().on_blue()));
                },
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 + n1;
                    let ans_str = ans.to_string();
                    println!("{} {} {} {} {}", n2_str, "+".dimmed(), n1_str, "=".dimmed(), ans_str);
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 - n1;
                    let ans_str = ans.to_string();
                    println!("{} {} {} {} {}", n2_str, "-".dimmed(), n1_str, "=".dimmed(), ans_str);
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 * n1;
                    let ans_str = ans.to_string();
                    println!("{} {} {} {} {}", n2_str, "×".dimmed(), n1_str, "=".dimmed(), ans_str);
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 / n1;
                    let ans_str = ans.to_string();
                    println!("{} {} {} {} {}", n2_str, "/".dimmed(), n1_str, "=".dimmed(), ans_str);
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
            }
        }
    }

    // Converts the RPN program to infix equation
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

    // Converts the RPN program to a string for a given set of numbers
    fn dump(&self, numbers: &[u32]) -> String {
        self.instructions.iter().map(|&i| {
            match i {
                ProgOp::Number(n) => numbers[n as usize].to_string(),
                ProgOp::OpAdd => "+".to_string(),
                ProgOp::OpSub => "-".to_string(),
                ProgOp::OpMul => "×".to_string(),
                ProgOp::OpDiv => "/".to_string(),
            }
        }).join(" ")
    }

}

impl fmt::Debug for Program {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prog_str = self.instructions.iter().map(|&i| {
            match i {
                ProgOp::Number(n) => format!("n[{}]", n.to_string()),
                ProgOp::OpAdd => "+".to_string(),
                ProgOp::OpSub => "-".to_string(),
                ProgOp::OpMul => "*".to_string(),
                ProgOp::OpDiv => "/".to_string(),
            }
        }).join(" ");

        write!(f, "{}", prog_str)
    }

}

// Results struct - Holds the results of running all programs with a set of numbers

#[derive(Default)]
pub struct Results<'a> {
    pub solutions: Vec<Solution<'a>>, // Valid solution collection
    pub under_range: usize,           // Number of programs with answer below valid range
    pub above_range: usize,           // Number of programs with answer above valid range
    pub negative: usize,              // Number of programs with negative intermediate result
    pub div_zero: usize,              // Number of programs encountering division by zero
    pub non_integer: usize,           // Number of programs with non-integer intermediate result
}

impl<'a> Results<'a> {

    // Create new Result
    fn new() -> Self {
        Results::default()
    }

}

// Solutions struct - holds the result of running a program

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

    pub fn print_steps(&self, numbers: &[u32]) {
        self.program.print_steps(numbers)
    }

    pub fn program_dump(&self, numbers: &[u32]) -> String {
        self.program.dump(numbers)
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

// Support functions

fn generate_num_programs(programs: &mut Programs, nums: usize, num_cnt: usize, op_counts: &OpCounts, op_combs: &Vec<Vec<ProgOp>>) {
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

                    programs.push(program);
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

fn op_combs(nums: usize) -> OpCombs {
    let mut results = Vec::new();

    if nums > 1 {
        op_combs_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1);
    }

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
    }

}
