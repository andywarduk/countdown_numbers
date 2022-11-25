#![warn(missing_docs)]

//! This module is responsible for generating all possible RPN programs for a game.
//!
//! For a set of numbers 1, 2, 3, 4 there are a number of slots in the RPN program
//! where operators can be inserted:
//! 1 2 <slot 1> 3 <slot 2> 4 <slot 3>
//! The number of slots and operators is always the number of numbers - 1.
//! Each slot except the last may be empty. Each slot can only contain a maximum of the
//! number of stacked numbers preceding it - 1. The counts of operators in each slot in
//! this example would be:
//! [0, 0, 3], [0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 1, 1]

use std::cmp::min;
use std::collections::{HashMap, HashSet};

use super::duplicates::{duplicated, DupReason};
use super::progop::ProgOp;
use super::ProgInstr;

/// Calculates the number of programs that will be generated for a given number of numbers.
/// When duplicates are filtered out an estimate is returned
pub(crate) fn calc_num_programs(
    nums: u8,
    inc_duplicated: bool,
    num_perms: &Vec<Vec<u8>>,
    op_map: &HashMap<u8, (OpCounts, OpCombs)>,
) -> usize {
    let mut total = 0;

    for num_cnt in 1..=nums {
        let mult = if num_cnt == 1 {
            // No operators
            1
        } else {
            // Get operator counts and combinations
            let (op_count, op_comb) = op_map.get(&num_cnt).unwrap();

            op_count.len() * op_comb.len()
        };

        total += num_perms.len() * mult;
    }

    if !inc_duplicated {
        // Guess about 1/7 of programs left behind after duplicate filtering
        total /= 7;
    }

    total
}

/// Generates RPN programs for the given total number of numbers, the number of numbers selected
/// and operator counts and combinations
pub(crate) fn generate_num_programs(
    programs: &mut Vec<ProgInstr>,
    instructions: &mut Vec<ProgOp>,
    num_cnt: u8,
    num_perms: &Vec<Vec<u8>>,
    op_map: &HashMap<u8, (OpCounts, OpCombs)>,
    inc_duplicated: bool,
) -> (usize, usize) {
    let mut stack = Vec::with_capacity(num_cnt as usize);

    let mut set = if inc_duplicated {
        // Not used when duplicates are included
        HashSet::new()
    } else {
        HashSet::with_capacity(programs.capacity())
    };

    // Get operator counts and combinations
    let (op_count, op_comb) = op_map.get(&num_cnt).unwrap();

    // Instruction vector pointer
    let mut inst_start = instructions.len();

    // Number of duplicates encountered
    let mut term_dups = 0;
    let mut infix_dups = 0;

    let mut add_program = |instructions: &mut Vec<ProgOp>| {
        let new_start = instructions.len();
        let inst_end = new_start - 1;

        // Duplicate check
        let ok = if !inc_duplicated {
            let reason = duplicated(&instructions[inst_start..=inst_end], &mut stack, &mut set);

            match reason {
                DupReason::NotDup => true,
                DupReason::TermOrder => {
                    term_dups += 1;
                    false
                }
                DupReason::Infix => {
                    infix_dups += 1;
                    false
                }
            }
        } else {
            true
        };

        if ok {
            programs.push(ProgInstr {
                start: inst_start as u32,
                end: inst_end as u32,
            });

            inst_start = new_start;
        } else {
            instructions.truncate(inst_start);
        }
    };

    for nums in num_perms {
        if num_cnt == 1 {
            // Push the number
            instructions.push(ProgOp::new_number(nums[0]));

            // Add the program
            add_program(instructions);
        } else {
            for op_count in op_count {
                for op_comb in op_comb {
                    let mut op_index = 0;

                    // Push first number
                    instructions.push(ProgOp::new_number(nums[0]));

                    for i in 0..(num_cnt - 1) {
                        // Push number
                        instructions.push(ProgOp::new_number(nums[i as usize + 1]));

                        // Push operators
                        for _ in 0..op_count[i as usize] {
                            instructions.push(op_comb[op_index]);
                            op_index += 1;
                        }
                    }

                    add_program(instructions);
                }
            }
        }
    }

    (term_dups, infix_dups)
}

type OpCounts = Vec<Vec<u8>>;

/// Generates a vector of vectors containing the combinations of number of operators in each slot in the RPN program
pub(crate) fn op_counts(nums: u8) -> OpCounts {
    let factorial = |num: usize| -> usize {
        match num {
            0 => 1,
            _ => (1..=num).product(),
        }
    };

    let catalan_number =
        |n: usize| -> usize { factorial(2 * n) / (factorial(n + 1) * factorial(n)) };

    let size = catalan_number(nums as usize - 1);
    let mut results = Vec::with_capacity(size);

    if nums > 1 {
        op_counts_rec(
            &mut results,
            Vec::with_capacity((nums - 1) as usize),
            0,
            nums - 1,
            nums - 1,
            2,
        );
    }

    results
}

fn op_counts_rec(
    results: &mut OpCounts,
    mut current: Vec<u8>,
    slot: u8,
    slots: u8,
    to_alloc: u8,
    stacked: u8,
) {
    if slot == slots - 1 {
        // Allocate all to the last slot
        current.push(to_alloc);
        results.push(current);
    } else {
        // How many can we allocate to this slot?
        let max_stack = stacked - 1;

        for i in 0..=min(to_alloc - 1, max_stack) {
            let mut next = Vec::with_capacity(current.capacity());
            next.clone_from(&current);
            next.push(i);

            let next_stacked = stacked + 1 - i;
            op_counts_rec(results, next, slot + 1, slots, to_alloc - i, next_stacked);
        }
    }
}

type OpCombs = Vec<Vec<ProgOp>>;

/// Generates a vector of vectors containing the combinations of operators to use in the RPN programs
pub(crate) fn op_combs(nums: u8, operators: &Vec<ProgOp>) -> OpCombs {
    let result_len = if nums > 1 {
        operators.len().pow(nums as u32 - 1)
    } else {
        0
    };

    let mut results = Vec::with_capacity(result_len);

    if nums > 1 {
        op_combs_rec(
            &mut results,
            Vec::with_capacity((nums - 1) as usize),
            0,
            nums - 1,
            operators,
        );
    }

    results
}

fn op_combs_rec(
    results: &mut OpCombs,
    current: Vec<ProgOp>,
    slot: u8,
    slots: u8,
    operators: &Vec<ProgOp>,
) {
    let mut add = |op: ProgOp| {
        let mut next = Vec::with_capacity(current.capacity());
        next.clone_from(&current);

        next.push(op);

        if slot == slots - 1 {
            results.push(next)
        } else {
            op_combs_rec(results, next, slot + 1, slots, operators)
        }
    };

    for op in operators.iter() {
        add(*op);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_counts() {
        let counts = op_counts(4);

        let expected: Vec<Vec<u8>> = vec![
            vec![0, 0, 3],
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 1, 1],
        ];

        assert_eq!(expected, counts);
    }

    #[test]
    fn test_op_combs() {
        let combs = op_combs(3, &vec![ProgOp::PROG_OP_ADD, ProgOp::PROG_OP_SUB]);

        let expected = vec![
            vec![ProgOp::PROG_OP_ADD, ProgOp::PROG_OP_ADD],
            vec![ProgOp::PROG_OP_ADD, ProgOp::PROG_OP_SUB],
            vec![ProgOp::PROG_OP_SUB, ProgOp::PROG_OP_ADD],
            vec![ProgOp::PROG_OP_SUB, ProgOp::PROG_OP_SUB],
        ];

        assert_eq!(expected, combs);
    }
}
