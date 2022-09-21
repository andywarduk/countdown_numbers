use std::collections::HashSet;
use std::cmp::min;

use itertools::Itertools;

use crate::*;
use crate::duplicates::*;

pub fn generate_num_programs(programs: &mut Programs, nums: usize, num_cnt: usize, op_counts: &OpCounts, op_combs: &Vec<Vec<ProgOp>>, inc_commutative: bool) {
    let mut set = HashSet::with_capacity(1_000_000 * num_cnt);
    let mut stack = Vec::with_capacity(num_cnt);

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

                    // Commutative check
                    if inc_commutative || !duplicated(&program, &mut stack, &mut set) {
                        programs.push(program);
                    }
                }
            }
            
        }
    }
}

type OpCounts = Vec<Vec<usize>>;

pub fn op_counts(nums: usize) -> OpCounts {
    let mut results = Vec::new();

    if nums > 1 {
        op_counts_rec(&mut results, Vec::with_capacity(nums - 1), 0, nums - 1, nums - 1, 2);
    }

    results
}

fn op_counts_rec(results: &mut OpCounts, mut current: Vec<usize>, slot: usize, slots: usize, to_alloc: usize, stacked: usize) {
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
