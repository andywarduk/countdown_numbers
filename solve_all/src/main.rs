use std::collections::{HashSet, VecDeque};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;
use itertools::Itertools;

use cards::{get_default_cards, get_special_cards};
use numformat::NumFormat;
use solver::Programs;

// Structure to hold parsed command line arguments

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Output equations in results files
    #[clap(short = 'e', long = "equations", action)]
    output_equations: bool,

    /// Number of threads to run
    #[clap(short, long, default_value_t = num_cpus::get(), value_parser)]
    threads: usize,

    /// Output directory
    #[clap(short = 'o', long = "outdir", value_parser)]
    out_dir: Option<PathBuf>,

    /// Use special cards
    #[clap(short = 's', long = "special", action)]
    special_cards: bool,

    /// Card set in use
    #[clap(skip)]
    cards: &'static [u8],

    /// Include duplicated equations
    #[clap(short = 'd', long = "duplicated", action)]
    inc_duplicated: bool,

    /// Verbose output
    #[clap(short, long, action)]
    verbose: bool,
}

fn main() {
    // Parse command line arguments
    let args = parse_args();

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
        let mut card_combs: VecDeque<Vec<u8>> = VecDeque::new();
        let mut hash: HashSet<Vec<&u8>> = HashSet::new();

        for choice in args.cards.iter().combinations(6) {
            if !hash.contains(&choice) {
                let numbers = choice.iter().map(|x| **x).collect();
                hash.insert(choice);
                card_combs.push_back(numbers);
            }
        }

        println!(" {} card combinations generated", card_combs.len());

        card_combs
    }));

    // Run solver threads
    run_solve_threads(&args, card_combs, &programs);
}

fn parse_args() -> Args {
    let mut args = Args::parse();

    // Sanitise number of threads
    if args.threads == 0 {
        args.threads = 1;
    }

    // Get card set
    args.cards = if args.special_cards {
        get_special_cards()
    } else {
        get_default_cards()
    };

    // Make sure we have a valid output path
    if !create_out_dir(&mut args) {
        std::process::exit(1);
    }

    args
}

fn create_out_dir(args: &mut Args) -> bool {
    let mut ok = true;

    if args.out_dir.is_none() {
        // Create default directory name
        let comm_str = if args.inc_duplicated { "C" } else { "NC" };

        args.out_dir = Some(
            format!("solutions-{}-{}",
                comm_str,
                args.cards.iter().map(|c| c.to_string()).join("-")
            )
            .into()
        );
    };

    // Convert to Path
    let path = args.out_dir.as_ref().unwrap().as_path();

    // Get metadata for the path
    if let Ok(meta) = path.metadata() {
        // Check it's a directory
        if !meta.is_dir() {
            eprintln!("{} is not a directory", path.display());
            ok = false;
        }
    } else {
        // Try and create the directory
        if let Err(e) = fs::create_dir(path) {
            eprintln!("Error creating {} ({})", path.display(), e);
            ok = false;
        }
    }

    ok
}

fn file_paths(args: &Args, numbers: &[u8]) -> (PathBuf, PathBuf) {
    let nums_str = numbers.iter().map(|n| format!("{}", n)).join("-");

    let file_name = format!("{}.txt", nums_str);
    let mut file_path = args.out_dir.clone().unwrap();
    file_path.push(file_name);

    let eqn_file_name = format!("{}-eqn.txt", nums_str);
    let mut eqn_file_path = args.out_dir.clone().unwrap();
    eqn_file_path.push(eqn_file_name);

    (file_path, eqn_file_path)
}

fn needs_calculating(args: &Args, file_path: &PathBuf, eqn_file_path: &PathBuf) -> bool {
    // Already calculated this set?
    if !Path::new(file_path).exists() {
        return true;
    };

    if args.output_equations && !Path::new(eqn_file_path).exists() {
        return true;
    }

    false
}

fn run_solve_threads(args: &Args, card_combs: Arc<Mutex<VecDeque<Vec<u8>>>>, programs: &Programs) {
    println!("Starting {} threads...", args.threads);

    // Start thread scope
    thread::scope(|thread_scope| {
        let mut handles = vec![];

        // Start worker threads
        for thread_no in 0..args.threads {
            // Clone reference to card combinations
            let thread_card_combs = card_combs.clone();

            // Start a thread
            let handle = thread::Builder::new()
                .name(format!("{}", thread_no + 1))
                .spawn_scoped(thread_scope, move || {
                    let thread = thread::current();
                    let thread_name = thread.name().unwrap();

                    if args.verbose {
                        println!("Thread {:4<}: Started", thread_name);
                    }

                    // Get next card selection
                    while let Some(numbers) = thread_card_combs.lock().unwrap().pop_front() {
                        let (file_path, eqn_file_path) = file_paths(args, &numbers);

                        if needs_calculating(args, &file_path, &eqn_file_path) {
                            // Run all equations for this card selection
                            println!("Thread {:4<}: Calculating {:?}...", thread_name, numbers);

                            solve(args, programs, &numbers, &file_path, &eqn_file_path);
                        }
                    }

                    if args.verbose {
                        println!("Thread {:4<}: Finished", thread_name);
                    }
                })
                .unwrap();

            // Add thread handle to the handles vector
            handles.push(handle);
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

fn solve(args: &Args, programs: &Programs, numbers: &[u8], file_path: &PathBuf, eqn_file_path: &PathBuf) {
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
