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
            // Output the results
            output_results(&results);

            0

        } else {
            res

        }

    });
}

const MAX_BIG: usize = 5;
const TARGET_COUNT: usize = 900;

#[derive(Clone)]
struct Stats {
    files: u32,
    sol_count: Vec<u32>,
    min_sol_cnt: usize,
    min_sols: Option<Vec<Vec<u32>>>,
    max_sol_cnt: usize,
    max_sols: Option<Vec<Vec<u32>>>,
    tot_sols: u64,
    sol_25_bucket: Vec<u32>,
    sol_50_bucket: Vec<u32>,
    sol_100_bucket: Vec<u32>,
}

impl Default for Stats {

    fn default() -> Self {
        Self {
            files: 0,
            sol_count: vec![0; TARGET_COUNT],
            min_sol_cnt: 0,
            min_sols: None,
            max_sol_cnt: 0,
            max_sols: None,
            tot_sols: 0,
            sol_25_bucket: vec![0; TARGET_COUNT / 25],
            sol_50_bucket: vec![0; TARGET_COUNT / 50],
            sol_100_bucket: vec![0; TARGET_COUNT / 100]
        }
    }

}

struct Results {
    stats: Stats,
    big_stats: Vec<Stats>,
}

impl Default for Results {

    fn default() -> Self {
        Self {
            stats: Stats::default(),
            big_stats: vec![Stats::default(); MAX_BIG]
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

    // Process the solution map file
    let mut sols: usize = 0;
    let mut sol_reached: [bool; TARGET_COUNT] = [false; TARGET_COUNT];

    for (i, c) in line[14..].chars().enumerate() {
        match c {
            '#' => {
                sol_reached[i] = true;
                sols += 1;
            }
            '.' | '\n' => (),
            _ => return Err(format!("Invalid character '{}' found in {}", c, details.path.display()).into())
        }
    }

    update_stats(&mut results.stats, &details, sols, &sol_reached);

    // Update big number stats
    let big_cnt = details.cards.iter().filter(|&c| *c > 10).count();

    if big_cnt < MAX_BIG {
        update_stats(&mut results.big_stats[big_cnt], &details, sols, &sol_reached);
    }

    Ok(())
}

fn update_stats(stats: &mut Stats, details: &FileDetails, sols: usize, sol_reached: &[bool]) {
    for (i, reached) in sol_reached.iter().enumerate() {
        if *reached {
            stats.sol_count[i] += 1;
        }
    }

    // Count this file
    stats.files += 1;

    // Add solution count to the total number of solutions
    stats.tot_sols += sols as u64;

    if sols > 0 {
        // Add count to the count buckets
        stats.sol_25_bucket[(sols - 1) / 25] += 1;
        stats.sol_50_bucket[(sols - 1) / 50] += 1;
        stats.sol_100_bucket[(sols - 1) / 100] += 1;
    }

    // Update minimum solutions list
    if stats.min_sols.is_none() {
        stats.min_sols = Some(Vec::new());
        stats.min_sol_cnt = sols;
    }

    if sols < stats.min_sol_cnt {
        let min_sols = stats.min_sols.as_mut().unwrap();
        min_sols.clear();
        stats.min_sol_cnt = sols;
    }

    if sols == stats.min_sol_cnt {
        let min_sols = stats.min_sols.as_mut().unwrap();
        min_sols.push(details.cards.clone());
    }

    // Update maximum solutions list
    if stats.max_sols.is_none() {
        stats.max_sols = Some(Vec::new());
        stats.max_sol_cnt = sols;
    }

    if sols > stats.max_sol_cnt {
        let max_sols = stats.max_sols.as_mut().unwrap();
        max_sols.clear();
        stats.max_sol_cnt = sols;
    }

    if sols == stats.max_sol_cnt {
        let max_sols = stats.max_sols.as_mut().unwrap();
        max_sols.push(details.cards.clone());
    }
}

fn output_results(results: &Results) {
    output_stats(&results.stats, "Overall");

    println!();
    println!("Big Number Average Achieved");
    for i in 0..MAX_BIG {
        println!("{}, {}, {:.2}", i, results.big_stats[i].files, results.big_stats[i].tot_sols as f64 / results.big_stats[i].files as f64)
    }

    for i in 0..MAX_BIG {
        println!();
        output_stats(&results.big_stats[i], &format!("{} Big Numbers", i));
    }
}

fn output_stats(stats: &Stats, desc: &str) {
    let mut min_sols = stats.sol_count[0];
    let mut min_sol_elems = Vec::new();
    let mut max_sols = stats.sol_count[0];
    let mut max_sol_elems= Vec::new();

    println!("===== {} =====", desc);
    println!("Target, Combinations");

    for (i, &n) in stats.sol_count.iter().enumerate() {
        println!("{}, {}, {}", i + 100, n, percent(n, stats.files));

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

    let mut cumul;

    println!();
    println!("{} Targets Achieved (buckets of 25)", desc);
    cumul = 0;
    for (i, n) in stats.sol_25_bucket.iter().enumerate() {
        cumul += n;
        println!("{}-{}, {}, {}, {}, {}", (i * 25) + 1, (i + 1) * 25, n, percent(*n, stats.files), cumul, percent(cumul, stats.files))
    }

    println!();
    println!("{} Targets Achieved (buckets of 50)", desc);
    cumul = 0;
    for (i, n) in stats.sol_50_bucket.iter().enumerate() {
        cumul += n;
        println!("{}-{}, {}, {}, {}, {}", (i * 50) + 1, (i + 1) * 50, n, percent(*n, stats.files), cumul, percent(cumul, stats.files))
    }

    println!();
    println!("{} Targets Achieved (buckets of 100)", desc);
    cumul = 0;
    for (i, n) in stats.sol_100_bucket.iter().enumerate() {
        cumul += n;
        println!("{}-{}, {}, {}, {}, {}", (i * 100) + 1, (i + 1) * 100, n, percent(*n, stats.files), cumul, percent(cumul, stats.files))
    }

    println!();
    println!("{} Statistics", desc);
    println!("Min Target Achieved, {}, {}, Targets, {}", min_sols, percent(min_sols, stats.files),
        min_sol_elems.iter().map(|n| (n + 100).to_string()).collect::<Vec<String>>().join(", "));
    println!("Max Target Achieved, {}, {}, Targets, {}", max_sols, percent(max_sols, stats.files),
        max_sol_elems.iter().map(|n| (n + 100).to_string()).collect::<Vec<String>>().join(", "));
    println!("Average Target Achieved, {:.2}", stats.tot_sols as f64 / stats.files as f64);
    println!("Min Solutions, {}, Count, {}, Cards, {:?}", stats.min_sol_cnt,
        stats.min_sols.as_ref().unwrap().len(), stats.min_sols.as_ref().unwrap());
    println!("Max Solutions, {}, Count, {}", stats.max_sol_cnt, stats.max_sols.as_ref().unwrap().len());
    println!("Card Combinations, {}", stats.files);

}

fn percent(n: u32, tot: u32) -> String {
    format!("{:.2}%", ((n as f64 / tot as f64) * 100f64))
}
