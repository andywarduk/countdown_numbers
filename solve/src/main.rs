use std::collections::HashSet;
use std::process;

use bitflags::bitflags;
use clap::Parser;
use numformat::NumFormat;
use solver::{Programs, Solution};

fn main() {
    // Parse command line arguments
    let exit_code = match parse_args() {
        Ok(args) => {
            // Arguments ok
            if args.verbose {
                println!("Target {}, Cards {:?}", args.target, args.cards);
            }

            println!("Generating programs...");
            let programs = Programs::new(args.cards.len() as u8, true, args.verbose);

            println!("Running programs...");
            let mut solutions = programs.run_all_target(args.target, &args.cards);

            if args.verbose {
                println!("{} total solutions found", solutions.len().num_format());
            }

            if solutions.is_empty() {
                if !args.verbose {
                    println!("== No solutions ==");
                }
            } else {
                let mut rpn_set = HashSet::with_capacity(solutions.len());
                let mut stack = Vec::new();
                let mut set = HashSet::new();

                let mut duplicate = 0;
                let mut identical = 0;

                solutions.retain(|s| {
                    // Filter out duplicated solutions
                    if !args.duplicated && programs.duplicated(s.program, &mut stack, &mut set) {
                        duplicate += 1;
                        return false;
                    }

                    // Filter out identical equations (can happen when duplicate card is chosen)
                    let rpn = programs.rpn(s.program, &args.cards, false);

                    if rpn_set.insert(rpn) {
                        true
                    } else {
                        identical += 1;
                        false
                    }
                });

                if args.verbose {
                    println!(
                        "Filtered out {} duplicate and {} identical solutions",
                        duplicate, identical
                    );
                }

                println!(
                    "{} {} found",
                    solutions.len(),
                    if solutions.len() == 1 {
                        "solution"
                    } else {
                        "solutions"
                    }
                );

                // Sort solutions by shortest first
                solutions.sort();

                // Output solutions
                print_solutions(&args, &programs, &solutions);
            }

            0
        }
        Err(code) => {
            // Invalid arguments
            code
        }
    };

    process::exit(exit_code)
}

fn print_solutions(args: &Args, programs: &Programs, solutions: &[Solution]) {
    // Print all solutions
    let num_outputs = args.output.bits().count_ones();
    let headings = num_outputs > 1 || args.output.contains(Output::STEPS);

    for (i, s) in solutions.iter().enumerate() {
        if headings {
            println!("== Solution {} ==", i + 1);
        }

        if args.output.contains(Output::RPN) {
            if num_outputs > 1 {
                print!("RPN: ");
            }
            println!("{}", programs.rpn(s.program, &args.cards, true));
        }

        if args.output.contains(Output::INFIX) {
            if num_outputs > 1 {
                print!("Equation: ");
            }
            println!("{}", programs.infix(s.program, &args.cards, true));
        }

        if args.output.contains(Output::FULLINFIX) {
            if num_outputs > 1 {
                print!("Full equation: ");
            }
            println!("{}", programs.infix_full(s.program, &args.cards, true));
        }

        if args.output.contains(Output::STEPS) {
            if num_outputs > 1 {
                println!("Steps:");
            }
            for l in programs.steps(s.program, &args.cards, true) {
                if num_outputs > 1 {
                    print!("  ");
                }
                println!("{}", l);
            }
        }
    }
}

bitflags! {
    #[derive(Default)]
    struct Output: u8 {
        const INFIX = 0b00000001;
        const FULLINFIX = 0b00000010;
        const RPN = 0b00000100;
        const STEPS = 0b00001000;
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Include duplicated equations
    #[clap(short = 'd', long = "duplicates", action)]
    duplicated: bool,

    /// Output simplified infix equations
    #[clap(short = 'i', long = "infix", action)]
    infix: bool,

    /// Output full infix equations
    #[clap(short = 'f', long = "full-infix", action)]
    full_infix: bool,

    /// Output reverse Polish notation
    #[clap(short = 'r', long = "rpn", action)]
    rpn: bool,

    /// Output steps
    #[clap(short = 's', long = "steps", action)]
    steps: bool,

    /// Output bitmask
    #[clap(skip)]
    output: Output,

    /// Verbose output
    #[clap(short = 'v', long = "verbose", action)]
    verbose: bool,

    // Target
    target: u32,

    // Cards chosen
    cards: Vec<u8>,
}

fn parse_args() -> Result<Args, i32> {
    // Parse command line arguments
    let mut args = Args::parse();

    if args.cards.is_empty() {
        eprintln!("No cards specified");
        Err(1)?
    }

    if args.cards.len() > 6 {
        eprintln!("Maximum of 6 cards allowed");
        Err(1)?
    }

    // Convert arg booleans to bitmask
    if args.infix {
        args.output |= Output::INFIX
    };

    if args.full_infix {
        args.output |= Output::FULLINFIX
    };

    if args.rpn {
        args.output |= Output::RPN
    };

    if args.steps {
        args.output |= Output::STEPS
    };

    // Default to infix and steps
    if args.output.is_empty() {
        args.output = Output::INFIX | Output::STEPS;
    }

    Ok(args)
}
