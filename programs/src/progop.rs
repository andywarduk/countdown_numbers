use std::fmt;
use colored::*;

/// ProgOp enum - RPN program items and operators
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum ProgOp {
    Number(u8),
    OpAdd,
    OpSub,
    OpMul,
    OpDiv
}

impl ProgOp {

    /// Returns the string representation of a program operator, optionally coloured
    pub fn colour(&self, colour: bool, numbers: &[u32]) -> String {
        let mut res = match self {
            ProgOp::Number(n) => numbers[*n as usize].to_string(),
            ProgOp::OpAdd => "+".to_string(),
            ProgOp::OpSub => "-".to_string(),
            ProgOp::OpMul => "×".to_string(),
            ProgOp::OpDiv => "/".to_string(),
        };

        if colour {
            res = match self {
                ProgOp::Number(_) => res.on_blue().to_string(),
                ProgOp::OpAdd | ProgOp::OpSub | ProgOp::OpMul | ProgOp::OpDiv => res.dimmed().to_string(),
            }
        }

        res
    }

}

impl fmt::Debug for ProgOp {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::OpAdd => write!(f, "+"),
            Self::OpSub => write!(f, "-"),
            Self::OpMul => write!(f, "×"),
            Self::OpDiv => write!(f, "/"),
        }
    }

}
