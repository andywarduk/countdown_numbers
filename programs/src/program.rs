use crate::progop::*;
use crate::infix::*;
use colored::*;
use itertools::Itertools;

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

    /// Returns the number of operators in the program
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    // Returns true is the program contains no instructions
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Borrows instructions from the program
    pub fn instructions(&self) -> &Vec<ProgOp> {
        &self.instructions
    }

    /// Processes a program
    pub fn process<S, N, T>(&self, stack: &mut Vec<S>, mut num_cb: N, mut op_cb: T) -> Option<S> 
    where N: FnMut(u8) -> Option<S>,
          T: FnMut(S, ProgOp, S) -> Option<S> {
        stack.clear();

        for op in &self.instructions {
            match op {
                ProgOp::Number(n) => {
                    stack.push(num_cb(*n)?)
                }
                _ => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    stack.push(op_cb(n2, *op, n1)?)
                }
            }
        }
    
        stack.pop()
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

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, numbers: &[u32], colour: bool) -> Vec<String> {
        let mut steps = Vec::new();
        let mut stack: Vec<(u32, String)> = Vec::with_capacity(numbers.len());

        self.process(&mut stack, |n| {
            Some((numbers[n as usize], ProgOp::Number(n).colour(numbers, colour)))
        }, |(n2, s2), op, (n1, s1)| {
            let ans = match op {
                ProgOp::OpAdd => n2 + n1,
                ProgOp::OpSub => n2 - n1,
                ProgOp::OpMul => n2 * n1,
                ProgOp::OpDiv => n2 / n1,
                _ => panic!("Non-operator not expected")
            };

            let ans_str = ans.to_string();

            let equals = if colour { "=".dimmed().to_string() } else { "=".to_string() };

            steps.push(format!("{} {} {} {} {}", s2, op.colour(numbers, colour), s1, equals, ans_str));

            Some((ans, ans_str))
        }).unwrap();

        steps
    }

    /// Converts the RPN program to operator type grouped infix equation
    pub fn infix(&self, numbers: &[u32], colour: bool) -> String {
        infix_group(self).colour(numbers, colour)
    }
    
    /// Converts the RPN program to a string for a given set of numbers
    pub fn rpn(&self, numbers: &[u32], colour: bool) -> String {
        self.instructions.iter().map(|i| i.colour(numbers, colour)).join(" ")
    }

}

impl From<Vec<ProgOp>> for Program {

    fn from(instructions: Vec<ProgOp>) -> Self {
        Program {
            instructions,
        }
    }

}

impl From<&str> for Program {

    fn from(rpn: &str) -> Self {
        let instructions = rpn.chars().filter_map(|c| {
            match c {
                '0'..='9' => Some(ProgOp::Number(c as u8 - b'0')),
                'a'..='z' => Some(ProgOp::Number(c as u8 - b'a')),
                'A'..='Z' => Some(ProgOp::Number(c as u8 - b'A')),
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prog_add() {
        let program: Program = "0 1 +".into();
        let mut stack: Vec<u32> = Vec::new();

        assert_eq!(Ok(7), program.run(&[3, 4], &mut stack));
    }

    #[test]
    fn prog_sub() {
        let program: Program = "0 1 -".into();
        let mut stack: Vec<u32> = Vec::new();

        assert_eq!(Ok(4), program.run(&[7, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[3, 3], &mut stack));
        assert_eq!(Err(ProgErr::Negative), program.run(&[3, 4], &mut stack));
    }

    #[test]
    fn prog_mul() {
        let program: Program = "0 1 *".into();
        let mut stack: Vec<u32> = Vec::new();

        assert_eq!(Ok(21), program.run(&[7, 3], &mut stack));
        assert_eq!(Err(ProgErr::Mul1), program.run(&[7, 1], &mut stack));
        assert_eq!(Err(ProgErr::Mul1), program.run(&[1, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[7, 0], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[0, 3], &mut stack));
        assert_eq!(Err(ProgErr::Zero), program.run(&[0, 0], &mut stack));
    }

    #[test]
    fn prog_div() {
        let program: Program = "0 1 /".into();
        let mut stack: Vec<u32> = Vec::new();

        assert_eq!(Ok(4), program.run(&[12, 3], &mut stack));
        assert_eq!(Err(ProgErr::NonInteger), program.run(&[13, 3], &mut stack));
        assert_eq!(Err(ProgErr::DivZero), program.run(&[3, 0], &mut stack));
        assert_eq!(Err(ProgErr::Div1), program.run(&[3, 1], &mut stack));
    }

}
