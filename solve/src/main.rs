use std::collections::HashSet;
use std::env;
use std::process;

use bitflags::bitflags;

use programs::programs::*;

fn main() {
    // Parse command line arguments
    let exit_code = match parse_args() {
        Ok(args) => {
            // Arguments ok
            println!("Target {}, Cards {:?}", args.target, args.cards);

            println!("Generating programs...");
            let programs = Programs::new(args.cards.len() as u8, true);

            println!("Running {} programs...", programs.len());
            let mut solutions = programs.run_all_target(args.target, &args.cards);

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
                        if !args.inc_duplicated && programs.duplicated(s.program, &mut stack, &mut set) {
                            return false;
                        }

                        // Filter out identical equations (can happen when duplicate card is chosen)
                        let rpn = programs.rpn(s.program, &args.cards, false);

                        rpn_set.insert(rpn)
                    })
                    .collect();

                println!("{} {} found", solutions.len(), if solutions.len() == 1 { "solution" } else { "solutions" });

                // Sort solutions by shortest first
                solutions.sort();

                // Output solutions
                print_solutions(&args, &programs, &solutions);
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

struct Args {
    target: u32,
    cards: Vec<u32>,
    inc_duplicated: bool,
    output: Output,
}

fn parse_args() -> Result<Args, i32> {
    let mut arg_iter = env::args().skip(1);
    let mut inc_duplicated = false;
    let mut output: Output = Default::default();

    let mut add_output = |o| -> Result<(), i32> {
        if output.contains(o) {
            eprintln!("Only one of -i, -f, -r and -s can be given");
            Err(1)?;
        }

        output.insert(o);

        Ok(())
    };

    // Get target value and any flags from arguments
    let target = loop {
        match arg_iter.next() {
            Some(arg) => {
                if let Some(switch) = arg.strip_prefix('-') {
                    // A command line switch
                    match switch {
                        "d" | "-duplicated" => {
                            inc_duplicated = true;
                        }
                        "i" | "-infix" => {
                            add_output(Output::INFIX)?;
                        }
                        "f" | "-full-infix" => {
                            add_output(Output::FULLINFIX)?;
                        }
                        "r" | "-rpn" => {
                            add_output(Output::RPN)?;
                        }
                        "s" | "-steps" => {
                            add_output(Output::STEPS)?;
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
        output = Output::INFIX | Output::STEPS;
    }

    Ok(Args {
        target,
        cards,
        inc_duplicated,
        output,
    })
}

fn usage() {
    println!("Usage: solve <flags> <target> <card> [<card> ...]");
    println!("Where flags are:");
    println!("  -d | --duplicated   Include duplicated equations");
    println!("  -i | --infix        Output simplified infix equations");
    println!("  -f | --full-infix   Output full infix equations");
    println!("  -r | --rpn          Output reverse Polish notation");
    println!("  -s | --steps        Output steps");
}
