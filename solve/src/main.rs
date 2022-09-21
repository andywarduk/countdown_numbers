use programs::*;
use infix::*;

use std::collections::HashSet;
use std::env;
use std::process;

use bitflags::bitflags;

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
                    let rpn = s.program_rpn(&args.cards);

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
                        println!("{}", s.program_rpn(&args.cards));
                    }

                    if args.output.contains(Output::INFIX) {
                        if num_outputs > 1 {
                            print!("Equation: ");
                        }
                        println!("{}", s.program_infix_type(&args.cards));
                    }

                    if args.output.contains(Output::STEPS) {
                        if num_outputs > 1 {
                            println!("Steps:");
                        }
                        for l in s.program_steps(&args.cards) {
                            if num_outputs > 1 {
                                print!("  ");
                            }
                            println!("{}", l);
                        }
                    }
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

bitflags! {
    #[derive(Default)]
    struct Output: u8 {
        const INFIX = 0b00000001;
        const RPN = 0b00000010;
        const STEPS = 0b00000100;
    }
}

struct Args {
    target: u32,
    cards: Vec<u32>,
    inc_commutative: bool,
    output: Output
}

fn parse_args() -> Result<Args, i32> {
    let mut arg_iter = env::args().skip(1);
    let mut inc_commutative = false;
    let mut output: Output = Default::default();

    let mut add_output = |o| {
        if output.contains(o) {
            eprintln!("Only one of -i, -r and -s can be given")
        }
        output.insert(o);
    };

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
                        "i" | "-infix" => {
                            add_output(Output::INFIX);
                        },
                        "r" | "-rpn" => {
                            add_output(Output::RPN);
                        },
                        "s" | "-steps" => {
                            add_output(Output::STEPS);
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

    if output.is_empty() {
        output = Output::all();
    }

    Ok(Args {
        target,
        cards,
        inc_commutative,
        output
    })
}

fn usage() {
    println!("Usage: solve <flags> <target> <card> [<card> ...]");
    println!("Where flags are:");
    println!("  -c | --commutative   Include commutative equations");
    println!("  -i | --infix         Output infix equations");
    println!("  -r | --rpn           Output reverse Polish notation");
    println!("  -s | --steps         Output steps");
}
