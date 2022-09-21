use crate::progop::*;
use crate::program::*;

/// Operator type simplification equation element
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InfixGrpTypeElem {
    Number(u8),
    Term(Box<InfixGrpTypeElem>, ProgOp, Box<InfixGrpTypeElem>),
    Group(Vec<(ProgOp, InfixGrpTypeElem)>),
}

impl InfixGrpTypeElem {

    /// Formats an operator type simplification equation element with optional colour
    pub fn colour(&self, numbers: &[u32], colour: bool) -> String {
        self.colour_internal(numbers, colour, false)
    }

    fn colour_internal(&self, numbers: &[u32], colour: bool, bracket: bool) -> String {
        match self {
            InfixGrpTypeElem::Number(n) => ProgOp::Number(*n).colour(numbers, colour),
            InfixGrpTypeElem::Term(t1, op, t2) => {
                let inner = format!("{} {} {}",
                    t1.colour_internal(numbers, colour, true),
                    op.colour(numbers, colour),
                    t2.colour_internal(numbers, colour, true),
                );

                if bracket {
                    format!("({})", inner)
                } else {
                    inner
                }
            }
            InfixGrpTypeElem::Group(terms) => {
                let inner = terms.iter().enumerate().map(|(i, (op, elem))| {
                    let elem_str = elem.colour_internal(numbers, colour, true);

                    if i == 0 {
                        elem_str
                    } else {
                        format!(" {} {}", op.colour(numbers, colour), elem_str)
                    }
                }).collect();

                if bracket {
                    format!("({})", inner)
                } else {
                    inner
                }
            }
        }
    }

}

/// Returns the infix structure for the program
pub fn infix_group(program: &Program) -> InfixGrpTypeElem {
    infix_group_cb(program, &mut |_| true).unwrap()
}

/// Returns an operator type simplified equation tree for a program
pub fn infix_group_cb<F>(program: &Program, grp_cb: &mut F) -> Result<InfixGrpTypeElem, ()>
where F: FnMut(&Vec<(ProgOp, InfixGrpTypeElem)>) -> bool {
    let mut stack = Vec::new();

    infix_group_cb_stack(program, &mut stack, grp_cb)
}

/// Returns an operator type simplified equation tree for a program
pub fn infix_group_cb_stack<F>(program: &Program, stack: &mut Vec<InfixGrpTypeElem>, grp_cb: &mut F) -> Result<InfixGrpTypeElem, ()>
where F: FnMut(&Vec<(ProgOp, InfixGrpTypeElem)>) -> bool {
    stack.clear();

    let build_grp = |other_op, t1, op, t2, inc_right, grp_cb: &mut F| -> Result<InfixGrpTypeElem, ()> {
        let mut grp = Vec::with_capacity(8);

        match t1 {
            InfixGrpTypeElem::Group(mut t1_terms)
            if t1_terms[0].0 == op || t1_terms[0].0 == other_op => {
                t1_terms[0].0 = op;
                grp.append(&mut t1_terms);
            }
            InfixGrpTypeElem::Group(ref t1_terms) => {
                if !grp_cb(t1_terms) { Err(())? }
                grp.push((op, t1))
            }
            _ => grp.push((op, t1))
        }

        if inc_right {
            match t2 {
                InfixGrpTypeElem::Group(mut t2_terms)
                if t2_terms[0].0 == other_op || t2_terms[0].0 == op => {
                    t2_terms[0].0 = op;
                    grp.append(&mut t2_terms)
                }
                InfixGrpTypeElem::Group(ref t2_terms) => {
                    if !grp_cb(t2_terms) { Err(())? }
                    grp.push((op, t2))
                }
                _ => grp.push((op, t2))
            }
        } else {
            match t2 {
                InfixGrpTypeElem::Group(ref t2_terms) => {
                    if !grp_cb(t2_terms) { Err(())? }
                    grp.push((op, t2))
                }
                _ => {
                    grp.push((op, t2))
                }
            }
        }

        Ok(InfixGrpTypeElem::Group(grp))
    };

    let build_term = |t1, op, t2, grp_cb: &mut F| -> Result<InfixGrpTypeElem, ()> {
        if let InfixGrpTypeElem::Group(grp1) = &t1 {
            if !grp_cb(grp1) {
                Err(())?
            }
        }

        if let InfixGrpTypeElem::Group(grp2) = &t2 {
            if !grp_cb(grp2) {
                Err(())?
            }
        }

        Ok(InfixGrpTypeElem::Term(Box::new(t1), op, Box::new(t2)))
    };

    let outer_term = program.process(stack, |n| {
        Ok(InfixGrpTypeElem::Number(n))
    }, |t1, op, t2| {
        match op {
            ProgOp::OpAdd => {
                build_grp(ProgOp::OpSub, t1, op, t2, true, grp_cb)
            }
            ProgOp::OpMul => {
                build_grp(ProgOp::OpDiv, t1, op, t2, true, grp_cb)
            }
            ProgOp::OpSub => {
                build_grp(ProgOp::OpAdd, t1, op, t2, false, grp_cb)
            }
            ProgOp::OpDiv => {
                build_grp(ProgOp::OpMul, t1, op, t2, false, grp_cb)
            }
            _ => {
                build_term(t1, op, t2, grp_cb)
            }
        }
    })?;

    if let InfixGrpTypeElem::Group(grp) = &outer_term {
        if !grp_cb(grp) {
            Err(())?
        }
    }

    Ok(outer_term)
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rpn_infix(rpn: &str, exp_infix: &str) {
        let program: Program = rpn.into();

        let num_count = program.instructions().iter().filter(|i| match i {
            ProgOp::Number(_) => true,
            _ => false
        }).count();

        let numbers: Vec<u32> = (0..num_count).map(|i| i as u32).collect();

        test_program_infix(&program, exp_infix, &numbers);
    }

    fn test_rpn_infix_and_result(rpn: &str, exp_infix: &str, numbers: &[u32], exp_ans: Result<u32, ProgErr>) {
        let program: Program = rpn.into();

        test_program_infix(&program, exp_infix, numbers);

        let mut stack = Vec::new();

        let ans = program.run(numbers, &mut stack);

        assert_eq!(exp_ans, ans);
    }

    fn test_program_infix(program: &Program, exp_infix: &str, numbers: &[u32]) {
        let infix = infix_group(&program);

        println!("RPN: {}, infix: {}",
            program.rpn(numbers, false), infix.colour(numbers, false));

        assert_eq!(exp_infix, infix.colour(numbers, false));
    }

    #[test]
    fn simplify_tests() {
        test_rpn_infix("0 1 2 3 + + +", "0 + 1 + 2 + 3");
        test_rpn_infix("0 1 2 3 - - -", "0 - (1 - (2 - 3))");
        test_rpn_infix("0 1 2 3 * * *", "0 × 1 × 2 × 3");
        test_rpn_infix("0 1 2 3 / / /", "0 / (1 / (2 / 3))");

        test_rpn_infix("0 1 - 2 - 3 -", "0 - 1 - 2 - 3");
        test_rpn_infix("0 1 / 2 / 3 /", "0 / 1 / 2 / 3");

        test_rpn_infix("0 1 2 + -", "0 - (1 + 2)");
        test_rpn_infix("0 1 - 2 +", "0 - 1 + 2");
        test_rpn_infix("0 1 2 - +", "0 + 1 - 2");
        test_rpn_infix("0 1 + 2 -", "0 + 1 - 2");

        test_rpn_infix("0 1 2 3 + + -", "0 - (1 + 2 + 3)");
        test_rpn_infix("0 1 2 3 + - +", "0 + 1 - (2 + 3)");
        test_rpn_infix("0 1 2 3 + - -", "0 - (1 - (2 + 3))");
        test_rpn_infix("0 1 2 3 - + +", "0 + 1 + 2 - 3");
        test_rpn_infix("0 1 2 3 - + -", "0 - (1 + 2 - 3)");
        test_rpn_infix("0 1 2 3 - - +", "0 + 1 - (2 - 3)");

        test_rpn_infix("0 1 2 + 3 + +", "0 + 1 + 2 + 3");
        test_rpn_infix("0 1 2 + 3 + -", "0 - (1 + 2 + 3)");
        test_rpn_infix("0 1 2 + 3 - +", "0 + 1 + 2 - 3");
        test_rpn_infix("0 1 2 + 3 - -", "0 - (1 + 2 - 3)");
        test_rpn_infix("0 1 2 - 3 + +", "0 + 1 - 2 + 3");
        test_rpn_infix("0 1 2 - 3 + -", "0 - (1 - 2 + 3)");
        test_rpn_infix("0 1 2 - 3 - +", "0 + 1 - 2 - 3");
        test_rpn_infix("0 1 2 - 3 - -", "0 - (1 - 2 - 3)");

        test_rpn_infix("0 1 2 + + 3 +", "0 + 1 + 2 + 3");
        test_rpn_infix("0 1 2 + + 3 -", "0 + 1 + 2 - 3");
        test_rpn_infix("0 1 2 + - 3 +", "0 - (1 + 2) + 3");
        test_rpn_infix("0 1 2 + - 3 -", "0 - (1 + 2) - 3");
        test_rpn_infix("0 1 2 - + 3 +", "0 + 1 - 2 + 3");
        test_rpn_infix("0 1 2 - + 3 -", "0 + 1 - 2 - 3");
        test_rpn_infix("0 1 2 - - 3 +", "0 - (1 - 2) + 3");
        test_rpn_infix("0 1 2 - - 3 -", "0 - (1 - 2) - 3");

        test_rpn_infix("0 1 + 2 + 3 +", "0 + 1 + 2 + 3");
        test_rpn_infix("0 1 + 2 + 3 -", "0 + 1 + 2 - 3");
        test_rpn_infix("0 1 + 2 - 3 +", "0 + 1 - 2 + 3");
        test_rpn_infix("0 1 + 2 - 3 -", "0 + 1 - 2 - 3");
        test_rpn_infix("0 1 - 2 + 3 +", "0 - 1 + 2 + 3");
        test_rpn_infix("0 1 - 2 + 3 -", "0 - 1 + 2 - 3");
        test_rpn_infix("0 1 - 2 - 3 +", "0 - 1 - 2 + 3");
        test_rpn_infix("0 1 - 2 - 3 -", "0 - 1 - 2 - 3");
    }

    #[test]
    fn group_tests() {
        // 1 + (2 - ((0 + 3) / 4)) => 75 + (50 - ((100 + 25) / 5))
        test_rpn_infix_and_result("1 2 0 3 + 4 / - +",
            "75 + 50 - ((100 + 25) / 5)",
            &[100, 75, 50, 25, 5], Ok(100));
        // 0 * (((3 * 4) - 5) / (1 + 2)) => 100 * (((25 * 10) - 5) / (75 + 50)) = 196
        test_rpn_infix_and_result("0 3 4 * 5 - 1 2 + / *",
            "100 × ((25 × 10) - 5) / (75 + 50)",
            &[100, 75, 50, 25, 10, 5], Err(ProgErr::NonInteger));
        // 0 * ((3 * 4) - 5) / (1 + 2) => 100 * ((25 * 10) - 5) / (75 + 50) = 196
        test_rpn_infix_and_result("0 3 4 * 5 - * 1 2 + /",
            "100 × ((25 × 10) - 5) / (75 + 50)",
            &[100, 75, 50, 25, 10, 5], Ok(196));
    }

}