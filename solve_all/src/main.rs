use std::collections::{HashSet, VecDeque};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;
use itertools::Itertools;

use cards::*;
use numformat::*;
use programs::programs::*;

// Structure to hold parsed command line arguments

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output equations in results files
    #[clap(short = 'e', long = "equations", action)]
    output_equations: bool,

    /// Number of threads to run
    #[clap(short = 't', long = "threads", default_value_t = 0, value_parser)]
    threads: u16,

    /// Output directory
    #[clap(short = 'o', long = "outdir", default_value = "", value_parser)]
    out_dir: String,

    /// Use special cards
    #[clap(short = 's', long = "special", action)]
    special_cards: bool,

    /// Include duplicated equations
    #[clap(short = 'd', long = "duplicated", action)]
    inc_duplicated: bool,

    /// Verbose output
    #[clap(short = 'v', long = "verbose", action)]
    verbose: bool,
}

fn main() {
    // Parse command line arguments
    let mut args = Args::parse();

    // Get card set
    let cards = if args.special_cards {
        get_special_cards()
    } else {
        get_default_cards()
    };

    // Make sure we have a valid output path
    if create_out_dir(&mut args, cards) {
        // Generate RPN equations
        if !args.verbose {
            print!("Generating programs...");
            io::stdout().flush().unwrap();
        } else {
            println!("Generating programs...");
        }

        let programs = Programs::new(6, args.inc_duplicated, args.verbose);

        if !args.verbose {
            println!(" {} programs generated", programs.len().num_format());
        }

        // Generate card combinations
        print!("Generating card combinations...");
        io::stdout().flush().unwrap();
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

            println!(" {} card combinations generated", card_combs.len());

            card_combs
        }));

        // Calculate number of threads to use
        let mut threads = args.threads as usize;
        if args.threads == 0 {
            threads = num_cpus::get();
        }

        // Start thread scope
        println!("Starting {} threads...", threads);
        thread::scope(|thread_scope| {
            let mut handles = vec![];

            // Start worker threads
            for thread_no in 0..threads {
                // Clone reference to card combinations
                let thread_card_combs = card_combs.clone();

                // Create references to passed in structs
                let args_ref = &args;
                let programs_ref = &programs;

                // Start a thread
                let handle = thread::Builder::new()
                    .name(format!("{}", thread_no + 1))
                    .spawn_scoped(thread_scope, move ||
                {
                    let thread = thread::current();
                    let thread_name = thread.name().unwrap();

                    if args_ref.verbose {
                        println!("Thread {:4<}: Started", thread_name);
                    }

                    loop {
                        // Get next card selection
                        let numbers_opt = thread_card_combs.lock().unwrap().pop_front();

                        match numbers_opt {
                            Some(numbers) => {
                                let (file_path, eqn_file_path) = file_paths(args_ref, &numbers);

                                if needs_calculating(args_ref, &file_path, &eqn_file_path) {
                                    // Run all equations for this card selection
                                    println!("Thread {:4<}: Calculating {:?}...", thread_name, numbers);

                                    solve(args_ref, programs_ref, &numbers, &file_path, &eqn_file_path);
                                }
                            }
                            None => break,
                        }
                    }

                    if args_ref.verbose {
                        println!("Thread {:4<}: Finished", thread_name);
                    }
                }).unwrap();

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
        let comm_str = if args.inc_duplicated { "C" } else { "NC" };
        args.out_dir = format!("solutions-{}-{}", comm_str, cards.iter().map(|c| c.to_string()).join("-"))
    };

    // Convert to Path
    let path = Path::new(&args.out_dir);

    // Get metadata for the path
    if let Ok(meta) = path.metadata() {
        // Check it's a directory
        if !meta.is_dir() {
            eprintln!("{} is not a directory", &args.out_dir);
            ok = false;
        }
    } else {
        // Try and create the directory
        if let Err(e) = fs::create_dir(path) {
            eprintln!("Error creating {} ({})", &args.out_dir, e);
            ok = false;
        }
    }

    ok
}

fn file_paths(args: &Args, numbers: &[u32]) -> (String, String) {
    let nums_str = numbers.iter().map(|n| format!("{}", n)).join("-");

    let file_name = format!("{}.txt", nums_str);
    let file_path = format!("{}/{}", &args.out_dir, &file_name);

    let eqn_file_name = format!("{}-eqn.txt", nums_str);
    let eqn_file_path = format!("{}/{}", &args.out_dir, &eqn_file_name);

    (file_path, eqn_file_path)
}

fn needs_calculating(args: &Args, file_path: &String, eqn_file_path: &String) -> bool {
    // Already calculated this set?
    if !Path::new(file_path).exists() { return true };

    if args.output_equations && !Path::new(eqn_file_path).exists() {
        return true;
    }

    false
}

fn solve(args: &Args, programs: &Programs, numbers: &Vec<u32>, file_path: &String, eqn_file_path: &String) {
    // Run all of the programs for this set of numbers
    let results = programs.run_all(numbers);

    // Count the number of solutions for each target number
    let mut sol_cnt: [u32; 900] = [0; 900];

    for solution in results.solutions.iter() {
        sol_cnt[solution.result as usize - 100] += 1;
    }

    // Create a solutions map string where '#' is > 0 and '.' = 0
    let sol_map: String = sol_cnt.iter().map(|x| {
        if *x > 0 { '#' } else { '.' }
    }).collect();

    // Create a string listing all of the target numbers with the number of solutions
    let sol_cnt_str = sol_cnt
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}={}", i + 100, c))
        .join(", ");

    // Count how many target numbers have > 0 solutions
    let covered = sol_cnt.iter().filter(|&&c| c > 0).count();

    // Create the output file
    let mut file = File::create(file_path).unwrap();

    // Write details to the output file
    writeln!(&mut file, "solution map: {}", sol_map).unwrap();
    writeln!(&mut file, "solution coverage: {}", covered).unwrap();
    writeln!(&mut file, "solution counts: {}", sol_cnt_str).unwrap();
    writeln!(&mut file, "results: {}", results.solutions.len()).unwrap();
    writeln!(&mut file, "zero intermediate: {}", results.zero).unwrap();
    writeln!(&mut file, "negative intermediate: {}", results.negative).unwrap();
    writeln!(&mut file, "div by zero: {}", results.div_zero).unwrap();
    writeln!(&mut file, "non-integer: {}", results.non_integer).unwrap();
    writeln!(&mut file, "multiply by 1: {}", results.mult_by_1).unwrap();
    writeln!(&mut file, "divide by 1: {}", results.div_by_1).unwrap();
    writeln!(&mut file, "< 100: {}", results.under_range).unwrap();
    writeln!(&mut file, "> 999: {}", results.above_range).unwrap();
    writeln!(&mut file, "duplicates included: {}", if args.inc_duplicated { "Yes" } else { "No" }).unwrap();

    if args.output_equations {
        // Write all equations to the equation output file
        let mut eqn_file = File::create(eqn_file_path).unwrap();

        for solution in results.solutions.iter().sorted() {
            writeln!(&mut eqn_file, "{}", programs.infix(solution.program, numbers, false)).unwrap();
        }
    }
}
