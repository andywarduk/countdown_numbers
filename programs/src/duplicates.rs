use std::collections::VecDeque;
use std::sync::Mutex;
use lazy_static::*;
use crate::progop::*;

lazy_static! {
    static ref STACK: Mutex<Vec<ProgEntity>> = Mutex::new(Vec::new());
}

#[derive(Debug)]
pub enum ProgEntity {
    Card(u8),
    Group(ProgOpAssoc, u8, VecDeque<ProgEntity>),
}

pub fn duplicated(instructions: &[ProgOp]) -> bool {
    duplicated_cb(instructions, check_terms)
}

pub fn duplicated_cb<F>(instructions: &[ProgOp], mut grp_cb: F) -> bool 
where F: FnMut(&VecDeque<ProgEntity>) -> bool {
    let mut stack = STACK.lock().unwrap();

    for op in instructions {
        match op {
            ProgOp::Number(x) => stack.push(ProgEntity::Card(*x)),
            _ => {
                // An operator
                let assoc = op.associativity();
                let prec = op.precedence();

                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();

                match n2 {
                    ProgEntity::Card(c2) => {
                        match n1 {
                            ProgEntity::Card(c1) => {
                                // c2 + c1
                                let mut vec = VecDeque::with_capacity(8);
                                vec.push_back(ProgEntity::Card(c2));
                                vec.push_back(ProgEntity::Card(c1));
                                stack.push(ProgEntity::Group(assoc, prec, vec));
                            }
                            ProgEntity::Group(assoc1, prec1, mut terms1) => {
                                if prec == prec1 && (assoc == ProgOpAssoc::Both || assoc == ProgOpAssoc::Right) {
                                    // c2 + grp with same prec - add to front of group
                                    terms1.push_front(ProgEntity::Card(c2));
                                    stack.push(ProgEntity::Group(assoc, prec, terms1))
                                } else {
                                    // c2 + grp with different prec
                                    if !grp_cb(&terms1) { return true };
                                    let mut vec = VecDeque::with_capacity(8);
                                    vec.push_back(ProgEntity::Card(c2));
                                    vec.push_back(ProgEntity::Group(assoc1, prec1, terms1));
                                    stack.push(ProgEntity::Group(assoc, prec, vec));
                                }
                            }
                        }
                    }
                    ProgEntity::Group(assoc2, prec2, mut terms2) => {
                        match n1 {
                            ProgEntity::Card(c1) => {
                                if prec == prec2 && (assoc2 == ProgOpAssoc::Both || assoc2 == ProgOpAssoc::Left) {
                                    // grp2 with same prec and compatible assoc + c1 - append to group
                                    terms2.push_back(ProgEntity::Card(c1));
                                    stack.push(ProgEntity::Group(assoc, prec, terms2))
                                } else {
                                    // grp2 with different prec or incompatible assoc + c1
                                    if !grp_cb(&terms2) { return true };
                                    let mut vec = VecDeque::with_capacity(8);
                                    vec.push_back(ProgEntity::Group(assoc2, prec2, terms2));
                                    vec.push_back(ProgEntity::Card(c1));
                                    stack.push(ProgEntity::Group(assoc, prec, vec));
                                }
                            }
                            ProgEntity::Group(assoc1, prec1, mut terms1) => {
                                if prec1 == prec2 && prec1 == prec {
                                    // grp2 + grp1 with same prec
                                    let mut terms = VecDeque::with_capacity(8);
                                    terms.append(&mut terms2);
                                    terms.append(&mut terms1);
                                    stack.push(ProgEntity::Group(assoc, prec, terms));
                                } else {
                                    // grp2 + grp1 with different assoc / prec
                                    if !grp_cb(&terms1) { return true };
                                    if !grp_cb(&terms2) { return true };
                                    let mut terms = VecDeque::with_capacity(8);
                                    terms.push_front(ProgEntity::Group(assoc2, prec2, terms2));
                                    terms.push_front(ProgEntity::Group(assoc1, prec1, terms1));
                                    stack.push(ProgEntity::Group(assoc, prec, terms));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match stack.pop().unwrap() {
        ProgEntity::Group(_, _, terms) => {
            if !grp_cb(&terms) { return true };
        }
        _ => {}
    }

    false
}

fn check_terms(terms: &VecDeque<ProgEntity>) -> bool {
    // TODO
    println!("{:?}", terms);

    true
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    fn test_int(rpn: &str, numbers: &[u32], expected_infix: &str, expected_answer: u32, expected_groups: usize, expected_duplicate: bool) {
        let program: Program = rpn.into();

        println!("RPN: {}, infix: {}",
            rpn,
            program.infix(numbers, false),
        );

        let mut stack = Vec::new();

        let result = program.run(numbers, &mut stack).unwrap();

        assert_eq!(result, expected_answer);

        assert_eq!(expected_infix, program.infix(numbers, false));

        let mut groups = Vec::new();

        let callback = |terms: &VecDeque<ProgEntity>| -> bool {
            groups.push(format!("{:?}", terms));
            true
        };

        assert_eq!(duplicated_cb(program.instructions(), callback), false);

        assert_eq!(groups.len(), expected_groups);

        // TODO assert_eq!(program.duplicated(), expected_duplicate);
    }

    #[test]
    fn test1() {
        test_int("0 1 +", &[10, 20], "10 + 20", 30, 1, false);
    }

    #[test]
    fn test2() {
        test_int("1 0 +", &[10, 20], "20 + 10", 30, 1, true);
    }

    #[test]
    fn test3() {
        test_int("0 1 + 2 +", &[10, 20, 30], "10 + 20 + 30", 60, 1, false);
    }

    #[test]
    fn test4() {
        test_int("0 2 + 1 +", &[10, 20, 30], "10 + 30 + 20", 60, 1, true);
    }

    #[test]
    fn test5() {
        test_int("1 0 + 2 +", &[10, 20, 30], "20 + 10 + 30", 60, 1, true);
    }

    #[test]
    fn test6() {
        test_int("1 2 + 0 +", &[10, 20, 30], "20 + 30 + 10", 60, 1, true);
    }

    #[test]
    fn test7() {
        test_int("2 0 + 1 +", &[10, 20, 30], "30 + 10 + 20", 60, 1, true);
    }

    #[test]
    fn test8() {
        test_int("2 1 + 0 +", &[10, 20, 30], "30 + 20 + 10", 60, 1, true);
    }

    #[test]
    fn test9() {
        test_int("0 1 -", &[20, 15], "20 - 15", 5, 1, false);
    }

    #[test]
    fn test10() {
        test_int("1 0 -", &[30, 50], "50 - 30", 20, 1, false);
    }

    #[test]
    fn testgrp1() {
        // (0 - 1) + 2 == 0 - 1 + 2 == 1
        test_int("2 1 - 0 +", &[5, 10, 30], "30 - 10 + 5", 25, 1, true);
    }

    #[test]
    fn testgrp2() {
        // 0 - (1 + 2) == -3 == 0 - 1 + 2 == 1
        test_int("0 1 2 + -", &[100, 10, 30], "100 - (10 + 30)", 60, 2, false);
    }

    #[test]
    fn testgrp3() {
        // (0 + 1) + (2 + 3) == 0 + 1 + 2 + 3
        test_int("0 1 + 2 3 + +", &[2, 3, 5, 7], "2 + 3 + 5 + 7", 17, 1, false);
    }

    #[test]
    fn testgrp4() {
        // (0 - 1) + (2 + 3) == 0 - 1 + 2 + 3
        test_int("0 1 - 2 3 + +", &[5, 2, 6, 7], "5 - 2 + 6 + 7", 16, 1, false);
    }

    #[test]
    fn testgrp5() {
        // TODO
        // (0 + 1) - (2 + 3) == 0 + 1 - (2 + 3)
        test_int("0 1 + 2 3 + -", &[5, 11, 6, 7], "5 + 11 - (6 + 7)", 3, 2, false);
    }

    #[test]
    fn testgrp6() {
        // (0 + 1) + (2 - 3) == 0 + 1 + 2 - 3
        test_int("0 1 + 2 3 - +", &[5, 11, 9, 7], "5 + 11 + 9 - 7", 18, 1, false);
    }

    #[test]
    fn testgrp7() {
        // (0 - 1) - (2 + 3)
        test_int("0 1 - 2 3 + -", &[20, 5, 7, 3], "20 - 5 - (7 + 3)", 5, 2, false);
    }

}
