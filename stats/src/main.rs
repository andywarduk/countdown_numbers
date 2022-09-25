mod calc;
mod results;
mod stats;

use std::error::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path;
use std::path::PathBuf;
use std::process;

use clap::Parser;

use results::*;
use stats::*;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Directory to process
    #[clap(value_parser)]
    dir: PathBuf,
}

fn main() {
    // Parse arguments
    let args = Args::parse();

    // Create results struct
    let mut results = Results::default();

    // Process the directory
    let res = process_dir(&mut results, &args.dir);

    if res != 0 {
        process::exit(res);
    }

    // Output the results
    results.output();
}

fn process_dir(results: &mut Results, dir: &PathBuf) -> i32 {
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
            eprintln!("Failed to scan {} ({})", dir.display(), e);

            2
        }
    }
}

struct FileDetails {
    path: path::PathBuf,
    cards: Vec<u8>,
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
        None?
    }

    // Get file stem
    let os_file_stem = path.file_stem()?;

    let file_stem = os_file_stem.to_str()?;

    // Check file stem
    let cards = file_stem
        .split('-')
        .map(|c| c.parse::<u8>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    // Check we have some numbers
    (!cards.is_empty()).then_some(())?;

    Some(FileDetails { path, cards })
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
            _ => return Err(format!("Invalid character '{}' found in {}", c, details.path.display()).into()),
        }
    }

    results.update(&details.cards, sols, &sol_reached);

    Ok(())
}
