use crate::progop::*;
use crate::infix::*;
use crate::duplicates::*;
use colored::*;
use itertools::Itertools;
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

    /// Runs the program with a given set of numbers and preallocated stack
    pub fn run(&self, numbers: &[u32], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
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

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, numbers: &[u32], colour: bool) -> Vec<String> {
        let mut steps = Vec::new();
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut str_stack: Vec<String> = Vec::with_capacity(numbers.len());

        let mut add_step = |op: &ProgOp, ans: u32, stack: &mut Vec<u32>, str_stack: &mut Vec<String>| {
            let n1_str = str_stack.pop().unwrap();
            let n2_str = str_stack.pop().unwrap();
            let ans_str = ans.to_string();

            let equals = if colour { "=".dimmed().to_string() } else { "=".to_string() };
            steps.push(format!("{} {} {} {} {}", n2_str, op.colour(colour, numbers), n1_str, equals, ans_str));

            stack.push(ans);
            str_stack.push(ans_str);
        };

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => {
                    stack.push(numbers[*x as usize]);
                    str_stack.push(op.colour(colour, numbers));
                },
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let ans = n2 + n1;
                    add_step(op, ans, &mut stack, &mut str_stack);
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let ans = n2 - n1;
                    add_step(op, ans, &mut stack, &mut str_stack);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let ans = n2 * n1;
                    add_step(op, ans, &mut stack, &mut str_stack);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let ans = n2 / n1;
                    add_step(op, ans, &mut stack, &mut str_stack);
                },
            }
        }

        steps
    }

    /// Converts the RPN program to infix equation
    pub fn infix(&self, numbers: &[u32], colour: bool) -> String {
        self.infix_format(InfixSimplifyMode::Full).colour(colour, numbers)
    }

    /// Returns the infix tree for the program
    pub fn infix_tree(&self) -> Infix {
        build_infix_tree(&self.instructions)
    }

    /// Returns the simplified infix tree for the program
    pub fn infix_format(&self, mode: InfixSimplifyMode) -> InfixFmtElem {
        infix_simplify(&self.infix_tree(), mode)
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
