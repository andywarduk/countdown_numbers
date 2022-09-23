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
use std::collections::HashSet;

use itertools::Itertools;

use crate::duplicates::*;
use crate::progop::*;
use crate::program::*;

/// Calculates the number of programs that will be generated for a given number of numbers.
/// When duplicates are filtered out an estimate is returned
pub fn calc_num_programs(nums: usize, inc_duplicated: bool, operators: &Vec<ProgOp>) -> usize {
    let mut total = 0;

    for num_cnt in 1..=nums {
        let perms = (0..nums).permutations(num_cnt).count();

        if num_cnt == 1 {
            total += perms;
        } else {
            let op_count = op_counts(num_cnt).len();
            let op_comb = op_combs(num_cnt, operators).len();

            total += perms * op_count * op_comb;
        }
    }

    if !inc_duplicated {
        // Guess about 1/7 of programs left behind after duplicate filtering
        total /= 7;
    }

    total
}

/// Generates RPN programs for the given total number of numbers, the number of numbers selected
/// and operator counts and combinations
pub fn generate_num_programs(
    programs: &mut Vec<Program>,
    nums: usize,
    num_cnt: usize,
    op_counts: &OpCounts,
    op_combs: &Vec<Vec<ProgOp>>,
    inc_duplicated: bool,
) {
    let mut stack = Vec::with_capacity(num_cnt);

    let mut set = if inc_duplicated {
        // Not used when duplicates are included
        HashSet::new()
    } else {
        HashSet::with_capacity(programs.capacity())
    };

    for nums in (0..nums).permutations(num_cnt) {
        if num_cnt == 1 {
            let mut program = Program::new(num_cnt);

            // Push the number
            program.push(ProgOp::Number(nums[0] as u8));

            programs.push(program);
        } else {
            for op_count in op_counts {
                for op_comb in op_combs {
                    let mut program = Program::new(num_cnt);
                    let mut op_index = 0;

                    // Push first number
                    program.push(ProgOp::Number(nums[0] as u8));

                    for i in 0..(num_cnt - 1) {
                        // Push number
                        program.push(ProgOp::Number(nums[i + 1] as u8));

                        // Push operators
                        for _ in 0..op_count[i] {
                            program.push(op_comb[op_index]);
                            op_index += 1;
                        }
                    }

                    // Duplicate check
                    if inc_duplicated || !duplicated(&program, &mut stack, &mut set) {
                        programs.push(program);
                    }
                }
            }
        }
    }
}

type OpCounts = Vec<Vec<usize>>;

/// Generates a vector of vectors containing the combinations of number of operators in each slot in the RPN program
pub fn op_counts(nums: usize) -> OpCounts {
    let mut results = Vec::new();

    if nums > 1 {
        op_counts_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, nums - 1, 2);
    }

    results
}

fn op_counts_rec(
    results: &mut OpCounts,
    mut current: Vec<usize>,
    slot: usize,
    slots: usize,
    to_alloc: usize,
    stacked: usize,
) {
    if slot == slots - 1 {
        // Allocate all to the last slot
        current.push(to_alloc);
        results.push(current);
    } else {
        // How many can we allocate to this slot?
        let max_stack = stacked - 1;

        for i in 0..=min(to_alloc - 1, max_stack) {
            let mut next = current.clone();
            next.push(i);
            let next_stacked = stacked + 1 - i;
            op_counts_rec(results, next, slot + 1, slots, to_alloc - i, next_stacked);
        }
    }
}

type OpCombs = Vec<Vec<ProgOp>>;

/// Generates a vector of vectors containing the combinations of operators to use in the RPN programs
pub fn op_combs(nums: usize, operators: &Vec<ProgOp>) -> OpCombs {
    let mut results = Vec::new();

    if nums > 1 {
        op_combs_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, operators);
    }

    results
}

fn op_combs_rec(results: &mut OpCombs, current: Vec<ProgOp>, slot: usize, slots: usize, operators: &Vec<ProgOp>) {
    let mut add = |op: ProgOp| {
        let mut next = current.clone();
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

        let expected: Vec<Vec<usize>> = vec![
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
        let combs = op_combs(3, &vec![ProgOp::OpAdd, ProgOp::OpSub]);

        let expected = vec![
            vec![ProgOp::OpAdd, ProgOp::OpAdd],
            vec![ProgOp::OpAdd, ProgOp::OpSub],
            vec![ProgOp::OpSub, ProgOp::OpAdd],
            vec![ProgOp::OpSub, ProgOp::OpSub],
        ];

        assert_eq!(expected, combs);
    }
}