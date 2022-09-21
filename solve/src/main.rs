use std::collections::HashSet;
use std::env;
use std::process;

use bitflags::bitflags;

use programs::duplicates::*;
use programs::*;

fn main() {
    // Parse command line arguments
    let exit_code = match parse_args() {
        Ok(args) => {
            // Arguments ok
            println!("Target {}, Cards {:?}", args.target, args.cards);

            println!("Generating programs...");
            let programs = Programs::new(args.cards.len(), true);

            println!("Running {} programs...", programs.len());
            let mut solutions = programs.run_target(args.target, &args.cards);

            if solutions.is_empty() {
                println!("== No solutions ==");
            } else {
                let mut rpn_set = HashSet::with_capacity(solutions.len());
                let mut stack = Vec::new();
                let mut set = HashSet::new();

                solutions = solutions
                    .into_iter()
                    .filter(|s| {
                        // Filter out duplicated solutions
                        if !args.inc_commutative && duplicated(s.program, &mut stack, &mut set) {
                            return false;
                        }
                        
                        // Filter out identical equations (can happen when duplicate card is chosen)
                        let rpn = s.program.rpn(&args.cards, false);

                        rpn_set.insert(rpn)
                    })
                    .collect();

                println!("{} {} found", solutions.len(), if solutions.len() == 1 { "solution" } else { "solutions" });

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
                        println!("{}", s.program.rpn(&args.cards, true));
                    }

                    if args.output.contains(Output::INFIX) {
                        if num_outputs > 1 {
                            print!("Equation: ");
                        }
                        println!("{}", s.program.infix(&args.cards, true));
                    }

                    if args.output.contains(Output::STEPS) {
                        if num_outputs > 1 {
                            println!("Steps:");
                        }
                        for l in s.program.steps(&args.cards, true) {
                            if num_outputs > 1 {
                                print!("  ");
                            }
                            println!("{}", l);
                        }
                    }
                }
            }

            0
        }
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
    output: Output,
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
                if let Some(switch) = arg.strip_prefix('-') {
                    // A command line switch
                    match switch {
                        "c" | "-commutative" => {
                            inc_commutative = true;
                        }
                        "i" | "-infix" => {
                            add_output(Output::INFIX);
                        }
                        "r" | "-rpn" => {
                            add_output(Output::RPN);
                        }
                        "s" | "-steps" => {
                            add_output(Output::STEPS);
                        }
                        _ => {
                            eprintln!("Unrecognised switch '{}'", arg);
                            Err(1)?;
                        }
                    }
                } else {
                    // Parse target number
                    match arg.parse::<u32>() {
                        Ok(num) => break num,
                        Err(std::num::ParseIntError { .. }) => {
                            eprintln!("Target must be a number (\"{}\")", arg);
                            Err(1)?;
                        }
                    }
                }
            }
            None => {
                eprintln!("No target value");
                Err(1)?;
            }
        }
    };

    // Get cards from arguments
    let cards = match arg_iter.map(|a| a.parse::<u32>()).collect::<Result<Vec<u32>, _>>() {
        Err(std::num::ParseIntError { .. }) => {
            eprintln!("Cards must be numeric");
            Err(1)?
        }
        Ok(v) => v,
    };

    if cards.is_empty() {
        eprintln!("No cards specified");
        Err(1)?
    }

    if output.is_empty() {
        output = Output::all();
    }

    Ok(Args {
        target,
        cards,
        inc_commutative,
        output,
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
