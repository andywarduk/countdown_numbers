use crate::program::*;
use crate::progop::*;
use crate::infix::*;

pub fn duplicated(program: &Program) -> Result<Vec<InfixGrpElem>, ()> {
    let infix = program_infixtree(program);
    duplicated_infix(&infix)
}

fn duplicated_infix(infix: &Infix) -> Result<Vec<InfixGrpElem>, ()> {
    infix_group(infix, &mut |grp| {
        // Returns true if :
        // 1. Numbers for a group of operators are not in ascending order
        // 2. Terms must follow numbers
        // 3. Operators must appear in the order * / + -

        let mut cur_op_ord = 0;
        let mut cur_num = None;
        let mut in_terms = false;

        for e in grp {
            match e {
                InfixGrpElem::Number(n) => {
                    if let Some(cur) = cur_num {
                        if n < cur || in_terms {
                            return false;
                        }
                        cur_num = Some(n)
                    } else {
                        // First number
                        if in_terms {
                            return false;
                        }
                        cur_num = Some(n)
                    }
                }
                InfixGrpElem::Op(op) => {
                    let ord = op_order(op);

                    if ord != cur_op_ord {
                        // Operator changed
                        if ord < cur_op_ord {
                            return false;
                        }

                        if cur_op_ord != 0 {
                            cur_num = None;
                            in_terms = false;
                        }

                        cur_op_ord = ord;
                    }
                }
                InfixGrpElem::Term(_) => {
                    in_terms = true;
                }
            }
        }

        true
    })
}

#[inline]
fn op_order(op: &ProgOp) -> usize {
    match *op {
        ProgOp::OpMul => 1,
        ProgOp::OpDiv => 2,
        ProgOp::OpAdd => 3,
        ProgOp::OpSub => 4,
        _ => panic!("Not expected")
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    fn test_int(rpn: &str, numbers: &[u32], exp_infix: &str, exp_ans: u32, exp_grps: usize, exp_dup: bool) {
        // Create program
        let program: Program = rpn.into();

        // Get infix tree
        let infix_tree = program_infixtree(&program);

        // Get infix groups
        let elems: Vec<u32> = (0..numbers.len()).map(|i| i as u32).collect();

        let mut groups = Vec::new();

        assert!(infix_group(&infix_tree, &mut |grp| {
            groups.push(format!("{}", grp.iter().map(|e| e.colour(false, &elems)).join(" ")));
            true
        }).is_ok());

        // Get simplified infix string
        let infix = infix_simplify(&infix_tree).colour(false, numbers);

        // Is a duplicate?
        let duplicate = duplicated(&program).is_err();

        println!("RPN: {}, infix: {}, dup : {}, groups: {}",
            rpn,
            infix,
            duplicate,
            groups.iter().join(", ")
        );

        // Run the program
        let mut stack = Vec::new();

        let result = program.run(numbers, &mut stack).unwrap();

        // Check answer
        assert_eq!(result, exp_ans);

        // Check infix
        assert_eq!(exp_infix, infix);

        // Check groups
        assert_eq!(groups.len(), exp_grps);

        // Check if expected to to duplicated
        assert_eq!(duplicate, exp_dup);
    }

    #[test]
    fn test1() {
        test_int("0 1 +", &[10, 20], "10 + 20", 30, 1, false);
        test_int("1 0 +", &[10, 20], "20 + 10", 30, 1, true);

        test_int("0 1 + 2 +", &[10, 20, 30], "10 + 20 + 30", 60, 1, false);
        test_int("0 2 + 1 +", &[10, 20, 30], "10 + 30 + 20", 60, 1, true);
        test_int("1 0 + 2 +", &[10, 20, 30], "20 + 10 + 30", 60, 1, true);
        test_int("1 2 + 0 +", &[10, 20, 30], "20 + 30 + 10", 60, 1, true);
        test_int("2 0 + 1 +", &[10, 20, 30], "30 + 10 + 20", 60, 1, true);
        test_int("2 1 + 0 +", &[10, 20, 30], "30 + 20 + 10", 60, 1, true);

        test_int("0 1 -", &[20, 15], "20 - 15", 5, 1, false);
        test_int("1 0 -", &[30, 50], "50 - 30", 20, 1, true);

        test_int("0 1 - 2 -", &[50, 10, 20], "50 - 10 - 20", 20, 1, false);
        test_int("0 2 - 1 -", &[50, 10, 20], "50 - 20 - 10", 20, 1, true);
        test_int("1 0 - 2 -", &[10, 50, 20], "50 - 10 - 20", 20, 1, true);
        test_int("1 2 - 0 -", &[10, 50, 20], "50 - 20 - 10", 20, 1, true);
        test_int("2 0 - 1 -", &[10, 20, 50], "50 - 10 - 20", 20, 1, true);
        test_int("2 1 - 0 -", &[10, 20, 50], "50 - 20 - 10", 20, 1, true);

        // (0 - 1) + 2 == 0 - 1 + 2 == 1
        test_int("2 1 - 0 +", &[5, 10, 30], "30 - 10 + 5", 25, 1, true);

        // 0 - (1 + 2) == -3 == 0 - 1 + 2 == 1
        test_int("0 1 2 + -", &[100, 10, 30], "100 - (10 + 30)", 60, 2, false);

        // (0 + 1) + (2 + 3) == 0 + 1 + 2 + 3
        test_int("0 1 + 2 3 + +", &[2, 3, 5, 7], "2 + 3 + 5 + 7", 17, 1, false);

        // (0 - 1) + (2 + 3) == 0 - 1 + 2 + 3
        test_int("0 1 - 2 3 + +", &[5, 2, 6, 7], "5 - 2 + 6 + 7", 16, 1, true);

        // (0 + 1) - (2 + 3) == 0 + 1 - (2 + 3)
        test_int("0 1 + 2 3 + -", &[5, 11, 6, 7], "5 + 11 - (6 + 7)", 3, 2, false);

        // (0 + 1) + (2 - 3) == 0 + 1 + 2 - 3
        test_int("0 1 + 2 3 - +", &[5, 11, 9, 7], "5 + 11 + 9 - 7", 18, 1, false);

        // (0 - 1) - (2 + 3)
        test_int("0 1 - 2 3 + -", &[20, 5, 7, 3], "20 - 5 - (7 + 3)", 5, 2, false);

        // (0 - 1) - (2 + 3)
        test_int("0 1 * 2 / 3 + 4 -", &[20, 30, 10, 7, 5], "20 × 30 / 10 + 7 - 5", 62, 1, false);
    }

    #[test]
    fn test2() {
        let programs = Programs::new_with_operators(4, false, vec![ProgOp::OpAdd]);

        let numbers = vec![0, 1, 2, 3];

        for p in &programs.programs {
            println!("RPN: {}  Equation: {}", p.rpn(&numbers, true), p.infix(&numbers, true));
        }

        assert_eq!(15, programs.len());

        assert_eq!("0", programs.programs[0].infix(&numbers, false));
        assert_eq!("1", programs.programs[1].infix(&numbers, false));
        assert_eq!("2", programs.programs[2].infix(&numbers, false));
        assert_eq!("3", programs.programs[3].infix(&numbers, false));

        assert_eq!("0 + 1", programs.programs[4].infix(&numbers, false));
        assert_eq!("0 + 2", programs.programs[5].infix(&numbers, false));
        assert_eq!("0 + 3", programs.programs[6].infix(&numbers, false));
        assert_eq!("1 + 2", programs.programs[7].infix(&numbers, false));
        assert_eq!("1 + 3", programs.programs[8].infix(&numbers, false));
        assert_eq!("2 + 3", programs.programs[9].infix(&numbers, false));

        assert_eq!("0 + 1 + 2", programs.programs[10].infix(&numbers, false));
        assert_eq!("0 + 1 + 3", programs.programs[11].infix(&numbers, false));
        assert_eq!("0 + 2 + 3", programs.programs[12].infix(&numbers, false));
        assert_eq!("1 + 2 + 3", programs.programs[13].infix(&numbers, false));

        assert_eq!("0 + 1 + 2 + 3", programs.programs[14].infix(&numbers, false));
    }

}
