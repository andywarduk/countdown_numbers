#![warn(missing_docs)]

//! This module contains operators for RPN programs and functions to process a stream of instructions

use bitflags::bitflags;
use colored::*;
use numformat::*;

bitflags! {
    /// Program operator type bitmask. Top 3 bits are operator type, low 5 bits used for numbers (0-31)
    pub struct ProgOp: u8 {
        /// A number. The number is in the low 4 bits (0-15)
        const PROG_OP_NUM = 0b00000000;
        /// Addition operator
        const PROG_OP_ADD = 0b00100000;
        /// Subtraction operator
        const PROG_OP_SUB = 0b01000000;
        /// Multiplication operator
        const PROG_OP_MUL = 0b01100000;
        /// Division operator
        const PROG_OP_DIV = 0b10000000;
        /// Operator type mask
        const PROG_OP_MASK = 0b11110000;
    }
}

impl ProgOp {
    /// Constructs a new number operator
    #[inline]
    pub fn new_number(n: u8) -> ProgOp {
        let result = ProgOp { bits: n };

        // Check we don't overflow 4 bits
        debug_assert!(result & ProgOp::PROG_OP_MASK == ProgOp::PROG_OP_NUM);

        result
    }

    /// Returns true if the operator is a number
    #[inline]
    pub fn is_number(&self) -> bool {
        *self & ProgOp::PROG_OP_MASK == ProgOp::PROG_OP_NUM
    }

    /// Returns the string representation of a program operator, optionally coloured
    pub fn colour(&self, numbers: &[u32], colour: bool) -> String {
        let mut res = match *self & ProgOp::PROG_OP_MASK {
            ProgOp::PROG_OP_NUM => numbers[self.bits as usize].num_format(),
            ProgOp::PROG_OP_ADD => "+".to_string(),
            ProgOp::PROG_OP_SUB => "-".to_string(),
            ProgOp::PROG_OP_MUL => "×".to_string(),
            ProgOp::PROG_OP_DIV => "/".to_string(),
            _ => panic!("Unexpected operator type"),
        };

        if colour {
            res = if self.is_number() {
                res.on_blue().to_string()
            } else {
                res.dimmed().to_string()
            }
        }

        res
    }
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
            stack.push(num_cb(op.bits)?)
        } else {
            let n1 = stack.pop().unwrap();
            let n2 = stack.pop().unwrap();
            stack.push(op_cb(n2, *op, n1)?)
        }
    }

    stack.pop()
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

/// Runs the program with a given set of numbers and preallocated stack
#[inline]
pub(crate) fn run_instructions(instructions: &[ProgOp], numbers: &[u32], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
    // NB this does not use the process function for speed
    stack.clear();

    for op in instructions {
        match *op & ProgOp::PROG_OP_MASK {
            ProgOp::PROG_OP_NUM => stack.push(numbers[op.bits as usize]),
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_size() {
        assert_eq!(1, mem::size_of::<ProgOp>());
    }
}
