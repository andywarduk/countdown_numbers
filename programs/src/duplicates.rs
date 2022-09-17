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
                                // c2 + grp
                                if assoc == assoc1 && prec == prec1 {
                                    terms1.push_front(ProgEntity::Card(c2));
                                    stack.push(ProgEntity::Group(assoc, prec, terms1))
                                } else {
                                    // c2 + grp with different assoc / prec
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
                                // grp2 with same assoc / prec + c1
                                if assoc == assoc2 && prec == prec2 {
                                    terms2.push_back(ProgEntity::Card(c1));
                                    stack.push(ProgEntity::Group(assoc, prec, terms2))
                                } else {
                                    // grp2 with different assoc / prec + c1
                                    if !grp_cb(&terms2) { return true };
                                    let mut vec = VecDeque::with_capacity(8);
                                    vec.push_back(ProgEntity::Group(assoc2, prec2, terms2));
                                    vec.push_back(ProgEntity::Card(c1));
                                    stack.push(ProgEntity::Group(assoc, prec, vec));
                                }
                            }
                            ProgEntity::Group(assoc1, prec1, mut terms1) => {
                                if assoc1 == assoc2 && prec1 == prec2 && assoc1 == assoc && prec1 == prec {
                                    // grp2 + grp1 with same assoc / prec
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
    use super::*;

    #[test]
    fn test1() {
        let ins = vec![
            ProgOp::Number(0),
            ProgOp::Number(1),
            ProgOp::OpAdd,
        ];

        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test2() {
        let ins = vec![
            ProgOp::Number(1),
            ProgOp::Number(0),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 1 + 0 == 0 + 1
        assert_eq!(duplicated(&ins), true);
    }

    #[test]
    fn test3() {
        let ins = vec![
            ProgOp::Number(0),
            ProgOp::Number(1),
            ProgOp::OpAdd,
            ProgOp::Number(2),
            ProgOp::OpAdd,
        ];

        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test4() {
        let ins = vec![
            ProgOp::Number(0),
            ProgOp::Number(2),
            ProgOp::OpAdd,
            ProgOp::Number(1),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 0 + 2 + 1 == 0 + 1 + 2
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test5() {
        let ins = vec![
            ProgOp::Number(1),
            ProgOp::Number(0),
            ProgOp::OpAdd,
            ProgOp::Number(2),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 1 + 0 + 2 == 0 + 1 + 2
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test6() {
        let ins = vec![
            ProgOp::Number(1),
            ProgOp::Number(2),
            ProgOp::OpAdd,
            ProgOp::Number(0),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 1 + 2 + 0 == 0 + 1 + 2
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test7() {
        let ins = vec![
            ProgOp::Number(2),
            ProgOp::Number(0),
            ProgOp::OpAdd,
            ProgOp::Number(1),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 2 + 0 + 1 == 0 + 1 + 2
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test8() {
        let ins = vec![
            ProgOp::Number(2),
            ProgOp::Number(1),
            ProgOp::OpAdd,
            ProgOp::Number(0),
            ProgOp::OpAdd,
        ];

        // Should be a duplicate because 2 + 1 + 0 == 0 + 1 + 2
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test9() {
        let ins = vec![
            ProgOp::Number(0),
            ProgOp::Number(1),
            ProgOp::OpSub,
        ];

        // Should not be a duplicate 0 - 1
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn test10() {
        let ins = vec![
            ProgOp::Number(1),
            ProgOp::Number(0),
            ProgOp::OpSub,
        ];

        // Should not be a duplicate because 1 - 0 != 0 - 1
        assert_eq!(duplicated(&ins), false);
    }

    #[test]
    fn testgrp1() {
        // (0 - 1) + 2 == 0 - 1 + 2 == 1
        let ins = vec![
            ProgOp::Number(2),
            ProgOp::Number(1),
            ProgOp::OpSub,
            ProgOp::Number(0),
            ProgOp::OpAdd,
        ];

        let mut groups = Vec::new();

        let callback = |terms: &VecDeque<ProgEntity>| -> bool {
            groups.push(format!("{:?}", terms));
            true
        };

        assert_eq!(duplicated_cb(&ins, callback), false);

        println!("{:?}", &groups);

        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn testgrp2() {
        // 0 - (1 + 2) == -3 != 0 - 1 + 2 == 1
        let ins = vec![
            ProgOp::Number(0),
            ProgOp::Number(1),
            ProgOp::OpSub,
            ProgOp::Number(2),
            ProgOp::OpAdd,
        ];

        assert_eq!(duplicated(&ins), true);
    }

}
