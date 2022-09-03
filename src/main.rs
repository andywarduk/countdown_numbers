mod programs;

use programs::*;
use std::path::Path;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use itertools::Itertools;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output equations in results files
    #[clap(short='e', long="equations", action)]
    output_equations: bool
}

fn main() {
    let args = Args::parse();

    println!("Generating programs...");
    let programs = Programs::new(6);

    println!("Running...");
    let cards: [u32; 24] = [100, 75, 50, 25, 10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1];
    
    let mut hash: HashSet<Vec<&u32>> = HashSet::new();
    for choice in cards.iter().combinations(6) {
        if !hash.contains(&choice) {
            let numbers = choice.iter().map(|x| **x).collect();
            hash.insert(choice);
            solve(&args, &programs, &numbers);
        }
    }
}

fn solve(args: &Args, programs: &Programs, numbers: &Vec<u32>) {
    let mut nums_str = String::new();

    for (i, n) in numbers.iter().enumerate() {
        if i == 0 {
            nums_str = format!("{}", n);
        } else {
            nums_str = format!("{}-{}", nums_str, n);
        }
    }

    let file_name = format!("{}.txt", nums_str);

    let file_path = Path::new(&file_name);
    
    if !file_path.exists() {
        println!("Calculating {:?}...", numbers);

        let results = programs.run(numbers);

        let mut sol_cnt: [u32; 900] = [0; 900];
        for solution in results.solutions.iter() {
            sol_cnt[solution.result as usize - 100] += 1;
        }

        let sol_map: String = sol_cnt.iter().map(|x| {
            if *x > 0 { '#' } else { '.' }
        }).collect();

        let mut file = File::create(file_path).unwrap();

        writeln!(&mut file, "solution map: {}", sol_map).unwrap();
        writeln!(&mut file, "solution counts: {:?}", sol_cnt).unwrap();
        writeln!(&mut file, "results: {}", results.solutions.len()).unwrap();
        writeln!(&mut file, "negative: {}", results.negative).unwrap();
        writeln!(&mut file, "div by zero: {}", results.div_zero).unwrap();
        writeln!(&mut file, "non-integer: {}", results.non_integer).unwrap();
        writeln!(&mut file, "< 100: {}", results.under_range).unwrap();
        writeln!(&mut file, "> 999: {}", results.above_range).unwrap();

        if args.output_equations {
            let eqn_file_name = format!("{}-eqn.txt", nums_str);

            let eqn_file_path = Path::new(&eqn_file_name);

            let mut eqn_file = File::create(eqn_file_path).unwrap();

            for solution in results.solutions.iter().sorted() {
                writeln!(&mut eqn_file, "{}", solution.format(numbers)).unwrap();
            }
        }
    }
}