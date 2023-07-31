#![warn(missing_docs)]

//! This module contains operators for RPN programs and functions to process a stream of instructions

use bitflags::bitflags;
use colored::Colorize;
use numformat::NumFormat;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// Program operator type bitmask. Top 3 bits are operator type, low 5 bits used for numbers (0-31)
    pub struct ProgOp: u8 {
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
        let result = ProgOp::from_bits_retain(n);

        // Check we don't overflow 5 bits
        debug_assert!((result & ProgOp::PROG_OP_MASK).is_empty());

        result
    }

    /// Returns true if the operator is a number
    #[inline]
    pub fn is_number(&self) -> bool {
        (*self & ProgOp::PROG_OP_MASK).is_empty()
    }

    /// Returns the string representation of a program operator, optionally coloured
    pub fn colour(&self, numbers: &[u8], colour: bool) -> String {
        let mut res = match *self & ProgOp::PROG_OP_MASK {
            ProgOp::PROG_OP_ADD => "+".to_string(),
            ProgOp::PROG_OP_SUB => "-".to_string(),
            ProgOp::PROG_OP_MUL => "×".to_string(),
            ProgOp::PROG_OP_DIV => "/".to_string(),
            _ => numbers[self.bits() as usize].num_format(),
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

// Tests

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(1, mem::size_of::<ProgOp>());
    }
}
