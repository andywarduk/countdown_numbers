use programs::{Programs};

use std::env;
use std::process;

fn main() {
    let exit_code = match parse_args() {
        Ok(args) => {
            println!("Target {}, Cards {:?}", args.target, args.cards);

            println!("Generating programs...");
            let programs = Programs::new(args.cards.len());
    
            println!("Running {} programs...", programs.len());
            let mut solutions = programs.run_target(args.target, &args.cards);

            if solutions.is_empty() {
                println!("== No solutions ==");
            } else {
                solutions.sort();

                for (i, s) in solutions.iter().enumerate() {
                    println!("== Solution {} [{}] ==", i + 1, s.program_dump(&args.cards));
                    s.print_steps(&args.cards);
                }
            }

            0
        },
        Err(code) => {
            usage();
            code
        }
    };

    process::exit(exit_code)
}

struct Args {
    target: u32,
    cards: Vec<u32>
}

fn parse_args() -> Result<Args, i32> {
    let mut arg_iter = env::args();

    // Skip program name
    arg_iter.next();

    // Get target from arguments
    let target = match arg_iter.next() {
        Some(str) => {
            match str.parse::<u32>() {
                Ok(num) => num,
                Err(std::num::ParseIntError { .. }) => {
                    eprintln!("Target must be a number (\"{}\")", str);
                    return Err(1)
                }
            }    
        },
        None => {
            eprintln!("No target value");
            return Err(1)
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
        cards
    })
}

fn usage() {
    println!("Usage: solve <target> <card> [<card> ...]")
}
