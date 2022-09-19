use itertools::Itertools;

use crate::progop::*;
use crate::program::*;

/// Infix operator tree
#[derive(Debug, PartialEq, Eq)]
pub enum Infix {
    Number(u8),
    Term(Box<Infix>, ProgOp, Box<Infix>)
}

pub fn program_infixtree(program: &Program) -> Infix {
    let mut stack: Vec<Infix> = Vec::new();

    program.process(&mut stack, |n| {
        Infix::Number(n)
    }, |n2, op, n1| {
        Infix::Term(Box::new(n2), op, Box::new(n1))
    })
}

pub enum InfixGrpElem {
    Number(u8),
    Op(ProgOp),
    Term(Vec<InfixGrpElem>),
}

impl InfixGrpElem {

    pub fn colour(&self, colour: bool, numbers: &[u32]) -> String {
        self.colour_internal(colour, numbers, false)
    }

    pub fn colour_internal(&self, colour: bool, numbers: &[u32], bracket: bool) -> String {
        match self {
            InfixGrpElem::Number(n) => ProgOp::Number(*n).colour(colour, numbers),
            InfixGrpElem::Op(o) => o.colour(colour, numbers),
            InfixGrpElem::Term(t) => {
                let inner = t.iter().map(|e| e.colour_internal(colour, numbers, true)).join(" ");
                
                if bracket {
                    format!("({})", inner)
                } else {
                    inner
                }
            }
        }
    }

}

pub fn infix_group<F>(infix: &Infix, cb: F) -> Result<Vec<InfixGrpElem>, ()>
where F: Fn(&Vec<InfixGrpElem>) -> bool {
    let grp = infix_group_recurse(infix, &cb, 0)?;

    if cb(&grp) { 
        Ok(grp)
    } else {
        Err(())
    }
}

fn infix_group_recurse<F>(infix: &Infix, cb: &F, parent_precedence: u32) -> Result<Vec<InfixGrpElem>, ()>
where F: Fn(&Vec<InfixGrpElem>) -> bool {
    match infix {
        Infix::Number(n) => Ok(vec![InfixGrpElem::Number(*n)]),
        Infix::Term(left, op, right) => {
            let mut result = Vec::with_capacity(10);
            
            let assoc = op.associativity();
            let precedence = op.precedence() as u32 * 2;

            let mut left_add = 0;
            let mut right_add = 0;

            if assoc == ProgOpAssoc::Right {
                left_add = 1;
            }
            if assoc == ProgOpAssoc::Left {
                right_add = 1;
            }

            result.append(&mut infix_group_recurse(left, cb, precedence + left_add)?);
            result.push(InfixGrpElem::Op(*op));
            result.append(&mut infix_group_recurse(right, cb, precedence + right_add)?);

            if parent_precedence > precedence {
                if !cb(&result) { return Err(()) };
                result = vec![InfixGrpElem::Term(result)];
            }

            Ok(result)
        }
    }
}

pub fn infix_simplify(infix: &Infix) -> InfixGrpElem {
    InfixGrpElem::Term(infix_group(infix, |_| true).unwrap())
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn simplify_test(rpn: &str, expected_infix: &str) {
        println!("RPN: {}, expected infix: {}", rpn, expected_infix);

        let program: Program = rpn.into();

        let num_count = program.instructions().iter().filter(|i| match i {
            ProgOp::Number(_) => true,
            _ => false
        }).count();

        let mut numbers = Vec::with_capacity(num_count);

        for i in 0..num_count {
            numbers.push(i as u32)
        }

        assert_eq!(program.infix(&numbers, false), expected_infix);
    }

    #[test]
    fn simplify_tests() {
        simplify_test("0 1 2 3 + + +", "0 + 1 + 2 + 3");
        simplify_test("0 1 2 3 - - -", "0 - (1 - (2 - 3))");
        simplify_test("0 1 2 3 * * *", "0 × 1 × 2 × 3");
        simplify_test("0 1 2 3 / / /", "0 / (1 / (2 / 3))");

        simplify_test("0 1 - 2 - 3 -", "0 - 1 - 2 - 3");
        simplify_test("0 1 / 2 / 3 /", "0 / 1 / 2 / 3");

        simplify_test("0 1 2 + -", "0 - (1 + 2)");
        simplify_test("0 1 - 2 +", "0 - 1 + 2");
        simplify_test("0 1 2 - +", "0 + 1 - 2");
        simplify_test("0 1 + 2 -", "0 + 1 - 2");

        simplify_test("0 1 2 3 + + -", "0 - (1 + 2 + 3)");
        simplify_test("0 1 2 3 + - +", "0 + 1 - (2 + 3)");
        simplify_test("0 1 2 3 + - -", "0 - (1 - (2 + 3))");
        simplify_test("0 1 2 3 - + +", "0 + 1 + 2 - 3");
        simplify_test("0 1 2 3 - + -", "0 - (1 + 2 - 3)");
        simplify_test("0 1 2 3 - - +", "0 + 1 - (2 - 3)");

        simplify_test("0 1 2 + 3 + +", "0 + 1 + 2 + 3");
        simplify_test("0 1 2 + 3 + -", "0 - (1 + 2 + 3)");
        simplify_test("0 1 2 + 3 - +", "0 + 1 + 2 - 3");
        simplify_test("0 1 2 + 3 - -", "0 - (1 + 2 - 3)");
        simplify_test("0 1 2 - 3 + +", "0 + 1 - 2 + 3");
        simplify_test("0 1 2 - 3 + -", "0 - (1 - 2 + 3)");
        simplify_test("0 1 2 - 3 - +", "0 + 1 - 2 - 3");
        simplify_test("0 1 2 - 3 - -", "0 - (1 - 2 - 3)");

        simplify_test("0 1 2 + + 3 +", "0 + 1 + 2 + 3");
        simplify_test("0 1 2 + + 3 -", "0 + 1 + 2 - 3");
        simplify_test("0 1 2 + - 3 +", "0 - (1 + 2) + 3");
        simplify_test("0 1 2 + - 3 -", "0 - (1 + 2) - 3");
        simplify_test("0 1 2 - + 3 +", "0 + 1 - 2 + 3");
        simplify_test("0 1 2 - + 3 -", "0 + 1 - 2 - 3");
        simplify_test("0 1 2 - - 3 +", "0 - (1 - 2) + 3");
        simplify_test("0 1 2 - - 3 -", "0 - (1 - 2) - 3");

        simplify_test("0 1 + 2 + 3 +", "0 + 1 + 2 + 3");
        simplify_test("0 1 + 2 + 3 -", "0 + 1 + 2 - 3");
        simplify_test("0 1 + 2 - 3 +", "0 + 1 - 2 + 3");
        simplify_test("0 1 + 2 - 3 -", "0 + 1 - 2 - 3");
        simplify_test("0 1 - 2 + 3 +", "0 - 1 + 2 + 3");
        simplify_test("0 1 - 2 + 3 -", "0 - 1 + 2 - 3");
        simplify_test("0 1 - 2 - 3 +", "0 - 1 - 2 + 3");
        simplify_test("0 1 - 2 - 3 -", "0 - 1 - 2 - 3");

    }

}