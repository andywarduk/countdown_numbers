use crate::progop::*;
use crate::infix::*;
use crate::duplicates::*;
use colored::*;
use itertools::Itertools;
use std::collections::VecDeque;
use std::convert;

/// Holds a single RPN program
#[derive(Eq, PartialEq)]
pub struct Program {
    instructions: Vec<ProgOp>,
}

impl Program {

    /// Creates a new program
    pub fn new(num_cnt: usize) -> Self {
        Program {
            instructions: Vec::with_capacity(num_cnt + (num_cnt - 1)),
        }
    }

    /// Adds an instruction to the program
    pub fn push(&mut self, op: ProgOp) {
        self.instructions.push(op);
    } 

    // Returns the numer of operators in the program
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Borrows instructions from the program
    pub fn instructions(&self) -> &Vec<ProgOp> {
        &self.instructions
    }

    /// Processes a program
    pub fn process<S, N, T>(&self, stack: &mut Vec<S>, mut num_cb: N, mut op_cb: T) -> S 
    where N: FnMut(u8) -> S,
          T: FnMut(S, ProgOp, S) -> S {
        stack.clear();

        for op in &self.instructions {
            match op {
                ProgOp::Number(n) => {
                    stack.push(num_cb(*n))
                }
                _ => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(op_cb(n2, *op, n1))
                }
            }
        }
    
        stack.pop().unwrap()
    }
    
    /// Runs the program with a given set of numbers and preallocated stack
    pub fn run(&self, numbers: &[u32], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
        // NB this does not use the process function for speed
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

                    let int = n2 - n1;

                    if int == 0 {
                        return Err(ProgErr::Zero)
                    }

                    stack.push(int);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 1 || n2 == 1 {
                        return Err(ProgErr::Mul1)
                    }

                    let int = n2 * n1;

                    if int == 0 {
                        return Err(ProgErr::Zero)
                    }

                    stack.push(int);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 0 {
                        return Err(ProgErr::DivZero)
                    }

                    if n1 == 1 {
                        return Err(ProgErr::Div1)
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

    /// Returns false if the program contains a calculation which would be covered by another program
    pub fn duplicate_filter(&self) -> bool {
        !duplicated(&self.instructions)
    }

    pub fn duplicated(&self) -> bool {
        duplicated(&self.instructions)
    }

    pub fn duplicated_cb<F>(&self, grp_cb: F) -> bool 
    where F: FnMut(&VecDeque<ProgEntity>) -> bool {
        duplicated_cb(&self.instructions, grp_cb)
    }

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, numbers: &[u32], colour: bool) -> Vec<String> {
        let mut steps = Vec::new();
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut str_stack: Vec<String> = Vec::with_capacity(numbers.len());

        self.process(&mut stack, |n| {
            numbers[n as usize]
        }, |n2, op, n1| {
            let ans = match op {
                ProgOp::OpAdd => n2 + n1,
                ProgOp::OpSub => n2 - n1,
                ProgOp::OpMul => n2 * n1,
                ProgOp::OpDiv => n2 / n1,
                _ => panic!("Non-operator not expected")
            };

            let n1_str = str_stack.pop().unwrap();
            let n2_str = str_stack.pop().unwrap();

            let ans_str = ans.to_string();

            let equals = if colour { "=".dimmed().to_string() } else { "=".to_string() };

            steps.push(format!("{} {} {} {} {}", n2_str, op.colour(colour, numbers), n1_str, equals, ans_str));

            str_stack.push(ans_str);

            ans
        });

        steps
    }

    /// Converts the RPN program to infix equation
    pub fn infix(&self, numbers: &[u32], colour: bool) -> String {
        self.infix_format().colour(colour, numbers)
    }

    /// Returns the infix tree for the program
    pub fn infix_tree(&self) -> Infix {
        program_infixtree(&self)
    }

    /// Returns the simplified infix tree for the program
    pub fn infix_format(&self) -> InfixGrpElem {
        infix_simplify(&self.infix_tree())
    }

    /// Converts the RPN program to a string for a given set of numbers
    pub fn rpn(&self, numbers: &[u32], colour: bool) -> String {
        self.instructions.iter().map(|i| i.colour(colour, numbers)).join(" ")
    }

}

impl convert::From<Vec<ProgOp>> for Program {

    fn from(instructions: Vec<ProgOp>) -> Self {
        Program {
            instructions,
        }
    }

}

impl convert::From<&str> for Program {

    fn from(rpn: &str) -> Self {
        let instructions = rpn.chars().filter_map(|c| {
            match c {
                '0'..='9' => Some(ProgOp::Number(c as u8 - '0' as u8)),
                'a'..='z' => Some(ProgOp::Number(c as u8 - 'a' as u8)),
                'A'..='Z' => Some(ProgOp::Number(c as u8 - 'A' as u8)),
                '+' => Some(ProgOp::OpAdd),
                '-' => Some(ProgOp::OpSub),
                '*' => Some(ProgOp::OpMul),
                '/' => Some(ProgOp::OpDiv),
                _ => None
            }
        }).collect();

        Program {
            instructions,
        }
    }

}

/// Errors generated by RPN program run
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
