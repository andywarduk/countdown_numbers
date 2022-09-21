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

    /// Returns the associativity of the operator
    pub fn associativity(&self) -> ProgOpAssoc {
        match self {
            ProgOp::Number(_) => panic!("No associativity for number"),
            ProgOp::OpAdd => ProgOpAssoc::Both,
            ProgOp::OpSub => ProgOpAssoc::Left,
            ProgOp::OpMul => ProgOpAssoc::Both,
            ProgOp::OpDiv => ProgOpAssoc::Left,
        }
    }

    /// Returns the precedence of the operator
    pub fn precedence(&self) -> u8 {
        match self {
            ProgOp::Number(_) => panic!("No precedence for number"),
            ProgOp::OpAdd => 2,
            ProgOp::OpSub => 2,
            ProgOp::OpMul => 3,
            ProgOp::OpDiv => 3,
        }
    }
    
    /// Returns the string representation of a program operator, optionally coloured
    pub fn colour(&self, numbers: &[u32], colour: bool) -> String {
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

// Operator associativity
#[derive(Debug, PartialEq, Clone)]
pub enum ProgOpAssoc {
    Left,
    Both
}
