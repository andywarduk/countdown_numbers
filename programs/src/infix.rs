use itertools::Itertools;

use crate::progop::*;

pub fn build_infix_tree(instructions: &[ProgOp]) -> Infix {
    let mut stack = Vec::with_capacity(instructions.len());

    for op in instructions {
        match op {
            ProgOp::Number(n) => stack.push(Infix::Number(*n)),
            _ => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(Infix::Term(Box::new(n2), *op, Box::new(n1)))
            }
        }
    }

    stack.pop().unwrap()
}

pub fn infix_simplify(infix: &Infix, mode: InfixSimplifyMode) -> InfixFmtElem {
    let mut top = match mode {
        InfixSimplifyMode::Full => infix_simplify_recurse_full(infix, 0),
        InfixSimplifyMode::Prec => infix_simplify_recurse_prec(infix, 0)
    };

    if top.len() == 1 {
        match top.pop().unwrap() {
            InfixFmtElem::Term(e) => InfixFmtElem::TopTerm(e),
            elem => elem
        }
    } else {
        InfixFmtElem::TopTerm(top)
    }
}

fn infix_simplify_recurse_full(infix: &Infix, parent_precedence: u32) -> Vec<InfixFmtElem> {
    match infix {
        Infix::Number(n) => vec![InfixFmtElem::Number(*n)],
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

            result.append(&mut infix_simplify_recurse_full(left, precedence + left_add));
            result.push(InfixFmtElem::Op(*op));
            result.append(&mut infix_simplify_recurse_full(right, precedence + right_add));

            if parent_precedence > precedence {
                result = vec![InfixFmtElem::Term(result)];
            }

            result
        }
    }
}

fn infix_simplify_recurse_prec(infix: &Infix, parent_precedence: u32) -> Vec<InfixFmtElem> {
    match infix {
        Infix::Number(n) => vec![InfixFmtElem::Number(*n)],
        Infix::Term(left, op, right) => {
            let mut result = Vec::with_capacity(10);
            
            let precedence = op.precedence() as u32 * 2;

            result.append(&mut infix_simplify_recurse_prec(left, precedence));
            result.push(InfixFmtElem::Op(*op));
            result.append(&mut infix_simplify_recurse_prec(right, precedence));

            if parent_precedence != precedence {
                result = vec![InfixFmtElem::Term(result)];
            }

            result
        }
    }
}

/// Infix operator tree
#[derive(Debug, PartialEq, Eq)]
pub enum Infix {
    Number(u8),
    Term(Box<Infix>, ProgOp, Box<Infix>)
}

/// Infix simplification mode
#[derive(PartialEq, Eq, Hash)]
pub enum InfixSimplifyMode {
    Full,
    Prec,
}

/// Formatted infix node
#[derive(Debug, PartialEq, Eq)]
pub enum InfixFmtElem {
    Number(u8),
    Op(ProgOp),
    Term(Vec<InfixFmtElem>),
    TopTerm(Vec<InfixFmtElem>)
}

impl InfixFmtElem {

    pub fn colour(&self, colour: bool, numbers: &[u32]) -> String {
        match self {
            InfixFmtElem::Number(n) => ProgOp::Number(*n).colour(colour, numbers),
            InfixFmtElem::Op(o) => o.colour(colour, numbers),
            InfixFmtElem::Term(t) => {
                format!("({})", t.iter().map(|e| e.colour(colour, numbers)).join(" "))
            }
            InfixFmtElem::TopTerm(t) => {
                t.iter().map(|e| e.colour(colour, numbers)).join(" ")
            }
        }
    }

}

// Tests

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    #[test]
    fn simplify_mode() {
        let program: Program = "0 1 * 2 * 3 +".into();

        let numbers = vec![1, 2, 3, 4];

        let infix_tree = program.infix_tree();
        println!("{:?}", infix_tree);

        let simple1 = infix_simplify(&infix_tree, InfixSimplifyMode::Full);
        println!("{:?}", simple1);
        assert_eq!(simple1.colour(false, &numbers), "1 × 2 × 3 + 4");

        let simple2 = infix_simplify(&infix_tree, InfixSimplifyMode::Prec);
        println!("{:?}", simple2);
        assert_eq!(simple2.colour(false, &numbers), "(1 × 2 × 3) + 4");
    }

    #[test]
    fn simplify_add() {
        let program: Program = "0 1 2 3 + + +".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 + 2 + 3 + 4");
    }

    #[test]
    fn simplify_mul() {
        let program: Program = "0 1 2 3 * * *".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 × 2 × 3 × 4");
    }

    #[test]
    fn simplify_sub_1() {
        let program: Program = "0 1 2 3 - - -".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 - (2 - (3 - 4))");
    }

    #[test]
    fn simplify_sub_2() {
        let program: Program = "0 1 - 2 - 3 -".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 - 2 - 3 - 4");
    }

    #[test]
    fn simplify_div_1() {
        let program: Program = "0 1 2 3 / / /".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 / (2 / (3 / 4))");
    }

    #[test]
    fn simplify_div_2() {
        let program: Program = "0 1 / 2 / 3 /".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 / 2 / 3 / 4");
    }

    #[test]
    fn simplify_3() {
        let program: Program = "0 1 2 + -".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 - (2 + 3)");
    }

    #[test]
    fn simplify_4() {
        let program: Program = "0 1 - 2 +".into();

        assert_eq!(program.infix(&[1, 2, 3, 4], false), "1 - 2 + 3");
    }

}