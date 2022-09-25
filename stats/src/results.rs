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

        for i in 0..MAX_BIG {
            let files = self.big_stats[i].files;
            let avg = average(self.big_stats[i].tot_sols, self.big_stats[i].files);

            println!("{}, {}, {:.2}, {}", i, files, avg, percent(avg, 900));
        }

        for i in 0..MAX_BIG {
            println!();
            self.big_stats[i].output(&format!("{} Big Numbers", i));
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
