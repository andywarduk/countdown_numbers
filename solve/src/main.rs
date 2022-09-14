use programs::{Programs};

use std::collections::HashSet;
use std::env;
use std::process;

fn main() {
    // Parse command line arguments
    let exit_code = match parse_args() {
        Ok(args) => {
            // Arguments ok
            println!("Target {}, Cards {:?}", args.target, args.cards);

            println!("Generating programs...");
            let programs = Programs::new(args.cards.len(), args.inc_commutative);
    
            println!("Running {} programs...", programs.len());
            let mut solutions = programs.run_target(args.target, &args.cards);

            if solutions.is_empty() {
                println!("== No solutions ==");
            } else {
                // Filter out identical equations (can happen when duplicate card is chosen)
                let mut eqn_set = HashSet::with_capacity(solutions.len());

                solutions = solutions.into_iter().filter(|s| {
                    let rpn = s.program_dump(&args.cards);

                    if eqn_set.contains(&rpn) {
                        false
                    } else {
                        eqn_set.insert(rpn);
                        true
                    }
                }).collect();

                println!("{} solutions found", solutions.len());

                // Sort solutions by shortest first
                solutions.sort();

                // Print all solutions
                for (i, s) in solutions.iter().enumerate() {
                    println!("== Solution {} [{}] ==", i + 1, s.program_dump(&args.cards));
                    s.print_steps(&args.cards);
                }
            }

            0
        },
        Err(code) => {
            // Invalid arguments
            usage();
            code
        }
    };

    process::exit(exit_code)
}

struct Args {
    target: u32,
    cards: Vec<u32>,
    inc_commutative: bool,
}

fn parse_args() -> Result<Args, i32> {
    let mut arg_iter = env::args().skip(1);
    let mut inc_commutative = false;

    // Get target value and any flags from arguments
    let target = loop {
        match arg_iter.next() {
            Some(arg) => {
                if arg.starts_with('-') {
                    // A command line switch
                    match &arg[1..] {
                        "c" | "-commutative" => {
                            inc_commutative = true;
                        },
                        _ => {
                            eprintln!("Unrecognised switch '{}'", arg);
                            return Err(1)
                        }
                    }
                } else {
                    // Parse target number
                    match arg.parse::<u32>() {
                        Ok(num) => break num,
                        Err(std::num::ParseIntError { .. }) => {
                            eprintln!("Target must be a number (\"{}\")", arg);
                            return Err(1);
                        }
                    }
                }
            }
            None => {
                eprintln!("No target value");
                return Err(1)
            }
        }
    };

    // Get cards from arguments
    let cards = match arg_iter.map(|a| a.parse::<u32>()).collect::<Result<Vec<u32>, _>>() {
        Err(std::num::ParseIntError { .. }) => {
            eprintln!("Cards must be numeric");
            return Err(1)
        },
        Ok(v) => v
    };

    if cards.is_empty() {
        eprintln!("No cards specified");
        return Err(1)
    }

    Ok(Args {
        target,
        cards,
        inc_commutative,
    })
}

fn usage() {
    println!("Usage: solve [-c|--commutative] <target> <card> [<card> ...]");
    println!("Where:");
    println!("  -c | --commutative   Include commutative equations")
}
