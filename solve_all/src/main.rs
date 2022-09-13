use programs::{Programs};
use cards::{get_default_cards, get_special_cards};

use std::collections::VecDeque;
use std::path::Path;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Mutex, Arc};
use std::thread;
use itertools::Itertools;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output equations in results files
    #[clap(short='e', long="equations", action)]
    output_equations: bool,

    /// Number of threads to run
    #[clap(short='t', long="threads", default_value_t=0, value_parser)]
    threads: u16,

    /// Output directory
    #[clap(short='o', long="outdir", default_value="", value_parser)]
    out_dir: String,

    /// Use special cards
    #[clap(short='s', long="special", action)]
    special_cards: bool,
}

fn main() {
    // Parse command line arguments
    let mut args = Args::parse();

    // Get card set
    let cards;

    if args.special_cards {
        cards = get_special_cards();
    } else {
        cards = get_default_cards();
    }

    // Make sure we have a valid output path
    if create_out_dir(&mut args, &cards) {
        // Generate equations
        println!("Generating programs...");
        let programs = Programs::new(6);

        // Generate card combinations
        println!("Generating card combinations...");    
        let card_combs = Arc::new(Mutex::new({
            let mut card_combs: VecDeque<Vec<u32>> = VecDeque::new();
            let mut hash: HashSet<Vec<&u32>> = HashSet::new();

            for choice in cards.iter().combinations(6) {
                if !hash.contains(&choice) {
                    let numbers = choice.iter().map(|x| **x).collect();
                    hash.insert(choice);
                    card_combs.push_back(numbers);
                }
            }

            card_combs
        }));

        // Calculate number of threads to use
        let mut threads = args.threads as usize;
        if args.threads == 0 {
            threads = num_cpus::get();
        }

        // Start thread scope
        println!("Starting {} threads...", threads);
        thread::scope(|s| {
            let mut handles = vec![];

            // Start worker threads
            for _ in 0..threads {
                // Clone reference to card combinations
                let thread_card_combs = card_combs.clone();

                // Create references to passed in structs
                let args_ref = &args;
                let programs_ref = &programs;

                // Start a thread
                let handle = s.spawn(move || {
                    loop {
                        // Get next card selection
                        let numbers_opt = thread_card_combs.lock().unwrap().pop_front();

                        match numbers_opt {
                            Some(numbers) => {
                                // Run all equations for this card selection
                                solve(args_ref, programs_ref, &numbers);
                            }
                            None => break
                        }
                    }
                });

                // Add thread handle to the handles vector
                handles.push(handle);
            }

            // Wait for all threads to finish
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }
}

fn create_out_dir(args: &mut Args, cards: &[u32]) -> bool {
    let mut ok = true;

    if args.out_dir.is_empty() {
        // Create default directory name
        args.out_dir = format!("solutions-{}", cards.iter().map(|c| c.to_string()).join("-"))
    };

    let path = Path::new(&args.out_dir);

    if let Ok(meta) = path.metadata() {
        if !meta.is_dir() {
            eprintln!("{} is not a directory", &args.out_dir);
            ok = false;
        }
    } else if let Err(e) = fs::create_dir(path) {
        eprintln!("Error creating {} ({})", &args.out_dir, e);
        ok = false;
    }

    ok
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
    let full_name = format!("{}/{}", &args.out_dir, &file_name);
    let file_path = Path::new(&full_name);
    
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

        let sol_cnt_str = sol_cnt.iter().enumerate().map(|(i, c)| format!("{}={}", i + 100, c)).join(", ");

        let covered = sol_cnt.iter().filter(|&&c| c > 0).count();

        let mut file = File::create(file_path).unwrap();

        writeln!(&mut file, "solution map: {}", sol_map).unwrap();
        writeln!(&mut file, "solution coverage: {}", covered).unwrap();
        writeln!(&mut file, "solution counts: {}", sol_cnt_str).unwrap();
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