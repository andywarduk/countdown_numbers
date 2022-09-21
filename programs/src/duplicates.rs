use std::collections::HashSet;

use crate::program::*;
use crate::progop::*;
use crate::infix::*;

pub fn duplicated(program: &Program, set: &mut HashSet<InfixGrpTypeElem>) -> bool {
    infix_group_type(program, &mut |grp| {
        let mut second_op = false;
        let mut in_terms = false;
        let mut last_num: u8 = 0;

        for (i, (op, e)) in grp.iter().enumerate() {
            if i > 0 {
                match *op {
                    ProgOp::OpAdd | ProgOp::OpMul => {
                        if second_op {
                            // Got first operator after the second
                            return false
                        }
                    }
                    ProgOp::OpSub | ProgOp::OpDiv => {
                        if !second_op {
                            second_op = true;
                            in_terms = false;
                            last_num = 0;
                        }
                    }
                    _ => panic!("Operator expected")
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
    }).and_then(|grp| {
        if set.contains(&grp) {
            Err(())
        } else {
            set.insert(grp);
            Ok(())
        }
    }).is_err()
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    fn test_int(rpn: &str, numbers: &[u32], exp_infix: &str, exp_ans: u32, exp_grps: usize, exp_dup: bool) {
        // Create program
        let program: Program = rpn.into();

        // Create element vector
        let elems: Vec<u32> = (0..numbers.len()).map(|i| i as u32).collect();

        // Get infix groups
        let mut groups = Vec::new();

        infix_group_type(&program, &mut |grp| {
            groups.push(format!("{}", InfixGrpTypeElem::Group(grp.clone()).colour(&elems, false)));
            true
        }).unwrap();
        
        // Get simplified infix strings
        let infix_elem = infix_simplify_type(&program).colour(&elems, false);
        let infix_nums = infix_simplify_type(&program).colour(numbers, false);

        // Is a duplicate?
        let mut set = HashSet::new();
        let duplicate = duplicated(&program, &mut set);

        println!("RPN: {}, infix (elems): {}, infix (nums): {}, dup : {}, groups: {}",
            rpn,
            infix_elem,
            infix_nums,
            duplicate,
            groups.iter().join(", ")
        );

        // Run the program
        let mut stack = Vec::new();

        let result = program.run(numbers, &mut stack).unwrap();

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

        // 0 - (1 + 2) == -3 != 0 - 1 + 2 == 1
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
        // RPN: 100 25 10 × 10 - × 75 50 + /
        // Equation: 100 × (25 × 10 - 10) / (75 + 50) = 192
        test_int("0 3 4 * 5 - * 1 2 + /", &[100, 75, 50, 25, 10, 10], "100 × ((25 × 10) - 10) / (75 + 50)", 192, 4, false); 
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

        for prog in &programs.programs {
            println!("Equation: {}", prog.infix_type(&numbers, true));            
        }

        assert_eq!(expected.len(), programs.len());

        for (exp, prog) in expected.iter().zip(programs.programs.iter()) {
            assert_eq!(*exp, prog.infix_type(&numbers, false))
        }
    }

}
