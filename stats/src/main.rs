use std::error::Error;
use std::env;
use std::process;
use std::fs;
use std::path;
use std::io;
use std::io::{BufRead};

fn main() {
    let mut arg_iter = env::args();

    process::exit(if arg_iter.len() != 2 {
        // Incorrect number of args
        eprintln!("Usage: stats <dir>");

        1

    } else {
        let mut results = Results::default();

        // Skip program name
        arg_iter.next();

        // Get directory to process
        let dir = arg_iter.next().unwrap();

        // Process the directory
        let res = process_dir(&mut results, &dir);

        if res == 0 {
            output_results(&results);

            0

        } else {
            res

        }

    });
}

struct Results {
    files: u32,
    sol_count: [u32; 900],
    min_sol_cnt: usize,
    min_sols: Option<Vec<Vec<u32>>>,
    max_sol_cnt: usize,
    max_sols: Option<Vec<Vec<u32>>>,
    tot_sols: u64,
    sol_25_bucket: [u32; 900 / 25],
    sol_50_bucket: [u32; 900 / 50],
    sol_100_bucket: [u32; 900 / 100],
}

impl Default for Results {

    fn default() -> Self {
        Self {
            files: 0,
            sol_count: [0; 900],
            min_sol_cnt: 0,
            min_sols: None,
            max_sol_cnt: 0,
            max_sols: None,
            tot_sols: 0,
            sol_25_bucket: [0; 900 / 25],
            sol_50_bucket: [0; 900 / 50],
            sol_100_bucket: [0; 900 / 100],
        }
    }

}

fn process_dir(results: &mut Results, dir: &str) -> i32 {
    match fs::read_dir(dir) {
        Ok(files) => {
            for f in files.flatten() {
                if let Some(details) = result_file_details(f) {
                    if let Err(e) = process_file(results, &details) {
                        eprintln!("Failed to process {} ({})", details.path.display(), e);
                    }
                }
            }

            0
        }
        Err(e) => {
            eprintln!("Failed to scan {} ({})", dir, e);

            2
        }
    }
}

struct FileDetails {
    path: path::PathBuf,
    cards: Vec<u32>
}

fn result_file_details(f: fs::DirEntry) -> Option<FileDetails> {
    // Check it's a file
    let ftype = f.file_type().ok()?;

    ftype.is_file().then_some(())?;

    // Get path
    let path = f.path();

    // Get extension
    let ext = path.extension()?;

    // Check extension
    if ext != "txt" {
        return None
    }

    // Get file stem
    let os_file_stem = path.file_stem()?;

    let file_stem = os_file_stem.to_str()?;

    // Check file stem
    let cards = file_stem.split('-').map(|c| c.parse::<u32>()).collect::<Result<Vec<u32>, _>>().ok()?;

    // Check we have some numbers
    (!cards.is_empty()).then_some(())?;

    Some(FileDetails {
        path,
        cards
    })
}

fn process_file(results: &mut Results, details: &FileDetails) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(&details.path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut line: String = String::new();

    // Read the solution maps line
    buf_reader.read_line(&mut line)?;

    // Check it
    if !line.starts_with("solution map: ") || line.len() <= 14 {
        return Err(format!("No solution map found in {}", details.path.display()).into());
    }

    // Process it
    let mut sols: usize = 0;

    for (i, c) in line[14..].chars().enumerate() {
        match c {
            '#' => {
                results.sol_count[i] += 1;
                sols += 1;
            }
            '.' | '\n' => (),
            _ => return Err(format!("Invalid character '{}' found in {}", c, details.path.display()).into())
        }
    }

    results.files += 1;

    results.tot_sols += sols as u64;
    if sols > 0 {
        results.sol_25_bucket[(sols - 1) / 25] += 1;
        results.sol_50_bucket[(sols - 1) / 50] += 1;
        results.sol_100_bucket[(sols - 1) / 100] += 1;
    }

    // Update minimum solutions list
    if results.min_sols.is_none() {
        results.min_sols = Some(Vec::new());
        results.min_sol_cnt = sols;
    }

    if sols < results.min_sol_cnt {
        let min_sols = results.min_sols.as_mut().unwrap();
        min_sols.clear();
        results.min_sol_cnt = sols;
    }

    if sols == results.min_sol_cnt {
        let min_sols = results.min_sols.as_mut().unwrap();
        min_sols.push(details.cards.clone());
    }

    // Update maximum solutions list
    if results.max_sols.is_none() {
        results.max_sols = Some(Vec::new());
        results.max_sol_cnt = sols;
    }

    if sols > results.max_sol_cnt {
        let max_sols = results.max_sols.as_mut().unwrap();
        max_sols.clear();
        results.max_sol_cnt = sols;
    }

    if sols == results.max_sol_cnt {
        let max_sols = results.max_sols.as_mut().unwrap();
        max_sols.push(details.cards.clone());
    }
    
    Ok(())
}

fn output_results(results: &Results) {
    let mut min_sols = results.sol_count[0];
    let mut min_sol_elems = Vec::new();
    let mut max_sols = results.sol_count[0];
    let mut max_sol_elems= Vec::new();

    println!("Target, Combinations");

    for (i, &n) in results.sol_count.iter().enumerate() {
        println!("{}, {}", i + 100, n);

        if n < min_sols {
            min_sols = n;
            min_sol_elems.clear();
        }

        if n == min_sols {
            min_sol_elems.push(i);
        }

        if n > max_sols {
            max_sols = n;
            max_sol_elems.clear();
        }

        if n == max_sols {
            max_sol_elems.push(i);
        }
    }

    println!();
    println!("Targets Achieved (buckets of 25)");
    for (i, n) in results.sol_25_bucket.iter().enumerate() {
        println!("{}, {}, {}", (i * 25) + 1, (i + 1) * 25, n)
    }

    println!();
    println!("Targets Achieved (buckets of 50)");
    for (i, n) in results.sol_50_bucket.iter().enumerate() {
        println!("{}, {}, {}", (i * 50) + 1, (i + 1) * 50, n)
    }

    println!();
    println!("Targets Achieved (buckets of 100)");
    for (i, n) in results.sol_100_bucket.iter().enumerate() {
        println!("{}, {}, {}", (i * 100) + 1, (i + 1) * 100, n)
    }

    println!();
    println!("Overall Statistics");
    println!("Min Target Achieved, {}, Targets, {}", min_sols, min_sol_elems.iter().map(|n| (n + 100).to_string()).collect::<Vec<String>>().join(", "));
    println!("Max Target Achieved, {}, Targets, {}", max_sols, max_sol_elems.iter().map(|n| (n + 100).to_string()).collect::<Vec<String>>().join(", "));
    println!("Average Target Achieved, {:.2}", results.tot_sols as f64 / results.files as f64);
    println!("Min Solutions, {}, Count, {}, Cards, {:?}", results.min_sol_cnt, results.min_sols.as_ref().unwrap().len(), results.min_sols.as_ref().unwrap());
    println!("Max Solutions, {}, Count, {}", results.max_sol_cnt, results.max_sols.as_ref().unwrap().len());
    println!("Card Combinations, {}", results.files);
}
