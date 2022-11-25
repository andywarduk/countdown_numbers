#![warn(missing_docs)]

//! This module is responsible for detecting if an RPN program would be duplicated by another RPN program
//! if the order of operations is changed. It does this by converting the RPN to bracketed infix and
//! for each bracket group applying the following rules:
//!  * The order of operators must go from + to - or * to /
//!  * The order of terms for commutative operators must be numbers in ascending order followed by sub-terms
//!
//! The infix expression is not sufficient to determine if a program is unique.
//! For example the RPN program 0 3 4 * 5 - 1 2 + / * produces the infix 100 × ((25 × 10) - 5) / (75 + 50)
//! when the numbers 100, 75, 50, 25, 10, 5 are applied.
//! The program 0 3 4 * 5 - 1 2 + / * produces identical infix and result but the program execution behaves
//! differently. The first program produces a NonInteger error because the ((25 × 10) - 5) / (75 + 50)
//! term is evaluated first (1.96).

use std::collections::HashSet;

use super::infix::{infix_group_cb_stack, InfixGrpTypeElem};
use super::progop::ProgOp;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DupReason {
    NotDup,
    TermOrder,
    Infix,
}

/// Returns true if the program would be duplicated by rearranging the terms of the equation
pub(crate) fn duplicated(
    instructions: &[ProgOp],
    stack: &mut Vec<InfixGrpTypeElem>,
    set: &mut HashSet<InfixGrpTypeElem>,
) -> DupReason {
    let mut grp_cb = |grp: &Vec<(ProgOp, InfixGrpTypeElem)>| -> bool {
        let mut second_op = false;
        let mut in_terms = false;
        let mut last_num: u8 = 0;

        for (i, (op, e)) in grp.iter().enumerate() {
            if i > 0 {
                match *op & ProgOp::PROG_OP_MASK {
                    ProgOp::PROG_OP_ADD | ProgOp::PROG_OP_MUL => {
                        if second_op {
                            // Got first operator after the second
                            return false;
                        }
                    }
                    ProgOp::PROG_OP_SUB | ProgOp::PROG_OP_DIV => {
                        if !second_op {
                            second_op = true;
                            in_terms = false;
                            last_num = 0;
                        }
                    }
                    _ => panic!("Operator expected"),
                }
            }

            match e {
                InfixGrpTypeElem::Number(n) => {
                    if in_terms || *n < last_num {
                        // Got a number after a term or number element is bigger
                        return false;
                    }
                    last_num = *n;
                }
                InfixGrpTypeElem::Group(_) | InfixGrpTypeElem::Term(_, _, _) => {
                    in_terms = true;
                }
            }
        }

        true
    };

    match infix_group_cb_stack(instructions, stack, &mut grp_cb) {
        Some(grp) => {
            if set.insert(grp) {
                DupReason::NotDup
            } else {
                DupReason::Infix
            }
        }
        None => DupReason::TermOrder,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::infix::{infix_group, infix_group_cb};
    use crate::programs::Programs;

    use itertools::Itertools;

    fn test_int(rpn: &str, numbers: &[u8], exp_infix: &str, exp_ans: u32, exp_grps: usize, exp_dup: DupReason) {
        // Create program
        let programs: Programs = rpn.into();

        // Create element vector
        let elems = (0..numbers.len()).map(|i| i as u8).collect::<Vec<_>>();

        // Get infix groups
        let mut groups = Vec::new();

        infix_group_cb(programs.instructions(0), &mut |grp| {
            groups.push(InfixGrpTypeElem::Group(grp.clone()).colour(&elems, false));
            true
        })
        .unwrap();

        // Get simplified infix strings
        let infix_elem = infix_group(programs.instructions(0)).colour(&elems, false);
        let infix_nums = infix_group(programs.instructions(0)).colour(numbers, false);

        // Is a duplicate?
        let mut stack = Vec::new();
        let mut set = HashSet::new();

        let duplicate = duplicated(programs.instructions(0), &mut stack, &mut set);

        // Print details
        println!(
            "RPN: {}, infix (elems): {}, infix (nums): {}, dup : {:?}, groups: {}",
            rpn,
            infix_elem,
            infix_nums,
            duplicate,
            groups.iter().join(", ")
        );

        // Run the program
        let result = programs.run(0, numbers).unwrap();

        // Check answer
        assert_eq!(exp_ans, result);

        // Check infix
        assert_eq!(exp_infix, infix_nums);

        // Check groups
        assert_eq!(exp_grps, groups.len());

        // Check if expected to to duplicated
        assert_eq!(exp_dup, duplicate);
    }

    #[test]
    fn test1() {
        test_int("0 1 +", &[10, 20], "10 + 20", 30, 1, DupReason::NotDup);
        test_int("1 0 +", &[10, 20], "20 + 10", 30, 1, DupReason::TermOrder);

        test_int("0 1 + 2 +", &[10, 20, 30], "10 + 20 + 30", 60, 1, DupReason::NotDup);
        test_int("0 2 + 1 +", &[10, 20, 30], "10 + 30 + 20", 60, 1, DupReason::TermOrder);
        test_int("1 0 + 2 +", &[10, 20, 30], "20 + 10 + 30", 60, 1, DupReason::TermOrder);
        test_int("1 2 + 0 +", &[10, 20, 30], "20 + 30 + 10", 60, 1, DupReason::TermOrder);
        test_int("2 0 + 1 +", &[10, 20, 30], "30 + 10 + 20", 60, 1, DupReason::TermOrder);
        test_int("2 1 + 0 +", &[10, 20, 30], "30 + 20 + 10", 60, 1, DupReason::TermOrder);

        test_int("0 1 -", &[20, 15], "20 - 15", 5, 1, DupReason::NotDup);
        test_int("1 0 -", &[30, 50], "50 - 30", 20, 1, DupReason::NotDup);

        test_int("0 1 - 2 -", &[50, 10, 20], "50 - 10 - 20", 20, 1, DupReason::NotDup);
        test_int("0 2 - 1 -", &[50, 10, 20], "50 - 20 - 10", 20, 1, DupReason::TermOrder);
        test_int("1 0 - 2 -", &[10, 50, 20], "50 - 10 - 20", 20, 1, DupReason::NotDup);
        test_int("1 2 - 0 -", &[10, 50, 20], "50 - 20 - 10", 20, 1, DupReason::TermOrder);
        test_int("2 0 - 1 -", &[10, 20, 50], "50 - 10 - 20", 20, 1, DupReason::NotDup);
        test_int("2 1 - 0 -", &[10, 20, 50], "50 - 20 - 10", 20, 1, DupReason::TermOrder);

        // (0 - 1) + 2 == 0 - 1 + 2 == 1
        test_int("2 1 - 0 +", &[5, 10, 30], "30 - 10 + 5", 25, 1, DupReason::TermOrder);

        // 0 - (1 + 2) == -3 != 0 - 1 + 2 == 1
        test_int("0 1 2 + -", &[100, 10, 30], "100 - (10 + 30)", 60, 2, DupReason::NotDup);

        // (0 + 1) + (2 + 3) == 0 + 1 + 2 + 3
        test_int(
            "0 1 + 2 3 + +",
            &[2, 3, 5, 7],
            "2 + 3 + 5 + 7",
            17,
            1,
            DupReason::NotDup,
        );

        // (0 - 1) + (2 + 3) == 0 - 1 + 2 + 3
        test_int(
            "0 1 - 2 3 + +",
            &[5, 2, 6, 7],
            "5 - 2 + 6 + 7",
            16,
            1,
            DupReason::TermOrder,
        );

        // (0 + 1) - (2 + 3) == 0 + 1 - (2 + 3)
        test_int(
            "0 1 + 2 3 + -",
            &[5, 11, 6, 7],
            "5 + 11 - (6 + 7)",
            3,
            2,
            DupReason::NotDup,
        );

        // (0 + 1) + (2 - 3) == 0 + 1 + 2 - 3
        test_int(
            "0 1 + 2 3 - +",
            &[5, 11, 9, 7],
            "5 + 11 + 9 - 7",
            18,
            1,
            DupReason::NotDup,
        );

        // (0 - 1) - (2 + 3)
        test_int(
            "0 1 - 2 3 + -",
            &[20, 5, 7, 3],
            "20 - 5 - (7 + 3)",
            5,
            2,
            DupReason::NotDup,
        );
    }

    #[test]
    fn test2() {
        // Rearrangements /*
        // ((0 x 1) / 2) + 3 - 4
        test_int(
            "0 1 * 2 / 3 + 4 -",
            &[20, 30, 10, 7, 5],
            "(20 × 30 / 10) + 7 - 5",
            62,
            2,
            DupReason::TermOrder,
        );
        // ((0 x 1) / 2) - 4 + 3
        test_int(
            "0 1 * 2 / 4 - 3 +",
            &[20, 30, 10, 7, 5],
            "(20 × 30 / 10) - 5 + 7",
            62,
            2,
            DupReason::TermOrder,
        );
        // 3 + ((0 x 1) / 2) - 4
        test_int(
            "3 0 1 * 2 / + 4 -",
            &[20, 30, 10, 7, 5],
            "7 + (20 × 30 / 10) - 5",
            62,
            2,
            DupReason::NotDup,
        );
        // 3 - 4 + ((0 x 1) / 2)
        test_int(
            "3 4 - 0 1 * 2 / +",
            &[20, 30, 10, 7, 5],
            "7 - 5 + (20 × 30 / 10)",
            62,
            2,
            DupReason::TermOrder,
        );
    }

    #[test]
    fn test3() {
        // RPN: 75 50 100 10 + 10 / - +
        // Equation: 75 + 50 - (100 + 10) / 10 = 114
        test_int(
            "1 2 0 3 + 4 / - +",
            &[100, 75, 50, 10, 10],
            "75 + 50 - ((100 + 10) / 10)",
            114,
            3,
            DupReason::NotDup,
        );
        // RPN: 100 25 10 × 10 - × 75 50 + /
        // Equation: 100 × (25 × 10 - 10) / (75 + 50) = 192
        test_int(
            "0 3 4 * 5 - * 1 2 + /",
            &[100, 75, 50, 25, 10, 10],
            "100 × ((25 × 10) - 10) / (75 + 50)",
            192,
            4,
            DupReason::NotDup,
        );
    }

    #[test]
    fn test4() {
        let programs = Programs::new_with_operators(4, false, vec![ProgOp::PROG_OP_ADD], false);

        let numbers = vec![0, 1, 2, 3];

        let expected = vec![
            // Single term
            "0",
            "1",
            "2",
            "3",
            // Double term
            "0 + 1",
            "0 + 2",
            "0 + 3",
            "1 + 2",
            "1 + 3",
            "2 + 3",
            // Triple term
            "0 + 1 + 2",
            "0 + 1 + 3",
            "0 + 2 + 3",
            "1 + 2 + 3",
            // Quad term
            "0 + 1 + 2 + 3",
        ];

        for i in 0..programs.len() {
            println!("Equation: {}", programs.infix(i, &numbers, true));
        }

        assert_eq!(expected.len(), programs.len());

        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(*exp, programs.infix(i, &numbers, false))
        }
    }
}
