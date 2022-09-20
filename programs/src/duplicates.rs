use crate::program::*;
use crate::progop::*;
use crate::infix::*;

pub fn duplicated(program: &Program) -> Result<Vec<InfixGrpElem>, ()> {
    let infix = program_infixtree(program);
    duplicated_infix(&infix)
}

fn duplicated_infix(infix: &Infix) -> Result<Vec<InfixGrpElem>, ()> {
    infix_group(infix, InfixGrpMode::Type, &mut |grp| {
        // Returns true if :
        // 1. Numbers for a group of operators are not in ascending order
        // 2. Terms must follow numbers for the same operator
        // 3. Operators must appear in the order * then / or + then -

        let mut first_op = true;
        let mut cur_num = -1;

        for e in grp {
            match e {
                InfixGrpElem::Number(n) => {
                    let cmp = *n as i32;

                    if cmp < cur_num {
                        return false;
                    }
                    cur_num = cmp;
                }
                InfixGrpElem::Op(op) => {
                    // Got an operator
                    let is_first_op = *op == ProgOp::OpAdd || *op == ProgOp::OpMul;

                    if first_op != is_first_op {
                        // Operator changed
                        if is_first_op {
                            return false;
                        }

                        cur_num = -1;
                        first_op = false;
                    }
                }
                InfixGrpElem::Term(_) => {
                    // Got a term
                    cur_num = i32::MAX;
                }
            }
        }

        true
    })
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

        assert!(infix_group(&infix_tree, InfixGrpMode::Type, &mut |grp| {
            groups.push(format!("{}", grp.iter().map(|e| e.colour(false, &elems)).join(" ")));
            true
        }).is_ok());

        // Get simplified infix string
        let infix = infix_simplify(&infix_tree, InfixGrpMode::Type).colour(false, numbers);

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
        assert_eq!(exp_ans, result);

        // Check infix
        assert_eq!(exp_infix, infix);

        // Check groups
        assert_eq!(exp_grps, groups.len());

        // Check if expected to to duplicated
        assert_eq!(exp_dup, duplicate);
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
        test_int("1 0 -", &[30, 50], "50 - 30", 20, 1, false);

        test_int("0 1 - 2 -", &[50, 10, 20], "50 - 10 - 20", 20, 1, false);
        test_int("0 2 - 1 -", &[50, 10, 20], "50 - 20 - 10", 20, 1, true);
        test_int("1 0 - 2 -", &[10, 50, 20], "50 - 10 - 20", 20, 1, false);
        test_int("1 2 - 0 -", &[10, 50, 20], "50 - 20 - 10", 20, 1, true);
        test_int("2 0 - 1 -", &[10, 20, 50], "50 - 10 - 20", 20, 1, false);
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
    }

    #[test]
    fn test2() {
        // Rearrangements /*
        // ((0 x 1) / 2) + 3 - 4
        test_int("0 1 * 2 / 3 + 4 -", &[20, 30, 10, 7, 5], "(20 × 30 / 10) + 7 - 5", 62, 2, true);
        // ((0 x 1) / 2) - 4 + 3
        test_int("0 1 * 2 / 4 - 3 +", &[20, 30, 10, 7, 5], "(20 × 30 / 10) - 5 + 7", 62, 2, true);
        // 3 + ((0 x 1) / 2) - 4
        test_int("3 0 1 * 2 / + 4 -", &[20, 30, 10, 7, 5], "7 + (20 × 30 / 10) - 5", 62, 2, false);
        // 3 - 4 + ((0 x 1) / 2)
        test_int("3 4 - 0 1 * 2 / +", &[20, 30, 10, 7, 5], "7 - 5 + (20 × 30 / 10)", 62, 2, true);
    }

    #[test]
    fn test3() {
        // RPN: 75 50 100 10 + 10 / - +
        // Equation: 75 + 50 - (100 + 10) / 10 = 114
        test_int("1 2 0 3 + 4 / - +", &[100, 75, 50, 10, 10],"75 + 50 - ((100 + 10) / 10)", 114, 3, false);
    }

    #[test]
    fn test4() {
        let programs = Programs::new_with_operators(4, false, vec![ProgOp::OpAdd]);

        let numbers = vec![0, 1, 2, 3];

        let expected = vec![
            "0",
            "1",
            "2",
            "3",

            "0 + 1",
            "0 + 2",
            "0 + 3",
            "1 + 2",
            "1 + 3",
            "2 + 3",

            "0 + 1 + 2",
            "0 + 1 + 3",
            "0 + 2 + 3",
            "1 + 2 + 3",

            "0 + 1 + 2 + 3"
        ];

        assert_eq!(expected.len(), programs.len());

        for (exp, prog) in expected.iter().zip(programs.programs.iter()) {
            println!("RPN: {}  Equation: {}", prog.rpn(&numbers, true), prog.infix(&numbers, InfixGrpMode::Full, true));
            assert_eq!(*exp, prog.infix(&numbers, InfixGrpMode::Full, false))
        }
    }

}
