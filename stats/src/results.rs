use crate::calc::{average, percent};
use crate::stats::*;

pub struct Results {
    pub stats: Stats,
    pub big_stats: Vec<Stats>,
}

impl Results {
    pub fn update(&mut self, cards: &[u8], sols: usize, sol_reached: &[bool]) {
        // Updte total stats
        self.stats.update(cards, sols, sol_reached);

        // Update big number stats
        let big_cnt = cards.iter().filter(|&c| *c > 10).count();

        if big_cnt < MAX_BIG {
            self.big_stats[big_cnt].update(cards, sols, sol_reached);
        }
    }

    pub fn output(&self) {
        self.stats.output("Overall");

        println!();
        println!("Big Number Average Achieved");

        for (i, stats) in self.big_stats.iter().enumerate() {
            let files = stats.files;
            let avg = average(stats.tot_sols, stats.files);

            println!("{}, {}, {:.2}, {}", i, files, avg, percent(avg, 900));
        }

        for (i, stats) in self.big_stats.iter().enumerate() {
            println!();
            stats.output(&format!("{i} Big Numbers"));
        }
    }
}

impl Default for Results {
    fn default() -> Self {
        Self {
            stats: Stats::default(),
            big_stats: vec![Stats::default(); MAX_BIG],
        }
    }
}
