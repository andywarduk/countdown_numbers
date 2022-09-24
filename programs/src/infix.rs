#![warn(missing_docs)]

//! This module is responsible for converting an RPN program in to an infix expression.
//! For a given RPN program a tree of elements is returned describing the grouping of
//! operations.

use crate::progop::*;

/// Operator type simplification equation element
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InfixGrpTypeElem {
    /// A number
    Number(u8),
    /// An expression containing an expression followed by an operator and another expression.
    /// The two terms on either side should be bracketed if they are not numbers
    Term(Box<InfixGrpTypeElem>, ProgOp, Box<InfixGrpTypeElem>),
    /// An expression containing a string of operators and expressions.
    /// The operators are all either + and - or * and /.
    /// The operator on the first element describes which overall operator the group has
    Group(Vec<(ProgOp, InfixGrpTypeElem)>),
}

impl InfixGrpTypeElem {
    /// Formats an operator type simplification equation element with optional colour
    pub fn colour(&self, numbers: &[u32], colour: bool) -> String {
        self.colour_internal(numbers, colour, false)
    }

    fn colour_internal(&self, numbers: &[u32], colour: bool, brackets: bool) -> String {
        let mut no_brackets = false;

        let inner = match self {
            InfixGrpTypeElem::Number(n) => {
                no_brackets = true;

                ProgOp::new_number(*n).colour(numbers, colour)
            }
            InfixGrpTypeElem::Term(t1, op, t2) => {
                format!("{} {} {}",
                    t1.colour_internal(numbers, colour, true),
                    op.colour(numbers, colour),
                    t2.colour_internal(numbers, colour, true),
                )
            }
            InfixGrpTypeElem::Group(terms) => terms
                .iter()
                .enumerate()
                .map(|(i, (op, elem))| {
                    let elem_str = elem.colour_internal(numbers, colour, true);

                    if i == 0 {
                        elem_str
                    } else {
                        format!(" {} {}", op.colour(numbers, colour), elem_str)
                    }
                })
                .collect(),
        };

        if brackets && !no_brackets {
            format!("({})", inner)
        } else {
            inner
        }
    }
}

/// Returns the infix structure for the program
pub(crate) fn infix_group(instructions: &[ProgOp]) -> InfixGrpTypeElem {
    infix_group_cb(instructions, &mut |_| true).unwrap()
}

/// Returns an operator type simplified equation tree for a program
pub(crate) fn infix_group_cb<F>(instructions: &[ProgOp], grp_cb: &mut F) -> Option<InfixGrpTypeElem>
where
    F: FnMut(&Vec<(ProgOp, InfixGrpTypeElem)>) -> bool,
{
    let mut stack = Vec::new();

    infix_group_cb_stack(instructions, &mut stack, grp_cb)
}

/// Returns an operator type simplified equation tree for a program
pub(crate) fn infix_group_cb_stack<F>(
    instructions: &[ProgOp],
    stack: &mut Vec<InfixGrpTypeElem>,
    grp_cb: &mut F,
) -> Option<InfixGrpTypeElem>
where
    F: FnMut(&Vec<(ProgOp, InfixGrpTypeElem)>) -> bool,
{
    stack.clear();

    let inst_cnt = instructions.len();

    let build_grp = |other_op, t1, op, t2, inc_right, grp_cb: &mut F| -> Option<InfixGrpTypeElem> {
        let mut grp = Vec::with_capacity(inst_cnt);

        match t1 {
            InfixGrpTypeElem::Group(mut t1_terms)
                if t1_terms[0].0 == op || t1_terms[0].0 == other_op =>
            {
                // Group with compatible operator (ie + and - or * and /)
                t1_terms[0].0 = op;
                grp.append(&mut t1_terms);
            }
            InfixGrpTypeElem::Group(ref t1_terms) => {
                // Group with incompatible operators
                if !grp_cb(t1_terms) {
                    None?
                }

                grp.push((op, t1))
            }
            _ => grp.push((op, t1)),
        }

        if inc_right {
            match t2 {
                InfixGrpTypeElem::Group(mut t2_terms)
                    if t2_terms[0].0 == other_op || t2_terms[0].0 == op =>
                {
                    // Group with compatible operator (ie + and - or * and /)
                    t2_terms[0].0 = op;
                    grp.append(&mut t2_terms)
                }
                InfixGrpTypeElem::Group(ref t2_terms) => {
                    // Group with incompatible operators
                    if !grp_cb(t2_terms) {
                        None?
                    }

                    grp.push((op, t2))
                }
                _ => grp.push((op, t2)),
            }
        } else {
            match t2 {
                InfixGrpTypeElem::Group(ref t2_terms) => {
                    if !grp_cb(t2_terms) {
                        None?
                    }

                    grp.push((op, t2))
                }
                _ => grp.push((op, t2)),
            }
        }

        Some(InfixGrpTypeElem::Group(grp))
    };

    let build_term = |t1, op, t2, grp_cb: &mut F| -> Option<InfixGrpTypeElem> {
        if let InfixGrpTypeElem::Group(grp1) = &t1 {
            if !grp_cb(grp1) {
                None?
            }
        }

        if let InfixGrpTypeElem::Group(grp2) = &t2 {
            if !grp_cb(grp2) {
                None?
            }
        }

        Some(InfixGrpTypeElem::Term(Box::new(t1), op, Box::new(t2)))
    };

    let outer_term = process_instructions(
        instructions,
        stack,
        |n| Some(InfixGrpTypeElem::Number(n)),
        |t1, op, t2| match op & ProgOp::PROG_OP_MASK {
            ProgOp::PROG_OP_ADD => build_grp(ProgOp::PROG_OP_SUB, t1, op, t2, true, grp_cb),
            ProgOp::PROG_OP_MUL => build_grp(ProgOp::PROG_OP_DIV, t1, op, t2, true, grp_cb),
            ProgOp::PROG_OP_SUB => build_grp(ProgOp::PROG_OP_ADD, t1, op, t2, false, grp_cb),
            ProgOp::PROG_OP_DIV => build_grp(ProgOp::PROG_OP_MUL, t1, op, t2, false, grp_cb),
            _ => build_term(t1, op, t2, grp_cb),
        },
    )?;

    if let InfixGrpTypeElem::Group(grp) = &outer_term {
        if !grp_cb(grp) {
            None?
        }
    }

    Some(outer_term)
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::*;

    fn test_rpn_infix(rpn: &str, exp_infix: &str) {
        let programs: Programs = rpn.into();

        let num_count = programs
            .instructions(0)
            .iter()
            .filter(|i| i.is_number())
            .count();

        let numbers: Vec<u32> = (0..num_count).map(|i| i as u32).collect();

        test_program_infix(&programs, exp_infix, &numbers);
    }

    fn test_rpn_infix_and_result(rpn: &str, exp_infix: &str, numbers: &[u32], exp_ans: Result<u32, ProgErr>) {
        let programs: Programs = rpn.into();

        test_program_infix(&programs, exp_infix, numbers);

        let ans = programs.run(0, numbers);

        assert_eq!(exp_ans, ans);
    }

    fn test_program_infix(programs: &Programs, exp_infix: &str, numbers: &[u32]) {
        let infix = infix_group(programs.instructions(0));

        println!("RPN: {}, infix: {}", programs.rpn(0, numbers, false), infix.colour(numbers, false));

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
