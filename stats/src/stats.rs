use crate::calc::{average, percent};

pub const MAX_BIG: usize = 5;
pub const TARGET_COUNT: usize = 900;

#[derive(Clone)]
pub struct Stats {
    pub files: usize,
    pub sol_count: Vec<usize>,
    pub min_sol_cnt: usize,
    pub min_sols: Option<Vec<Vec<u8>>>,
    pub max_sol_cnt: usize,
    pub max_sols: Option<Vec<Vec<u8>>>,
    pub tot_sols: usize,
    pub sol_25_bucket: Vec<usize>,
    pub sol_50_bucket: Vec<usize>,
    pub sol_100_bucket: Vec<usize>,
    pub tot_combs: usize,
    pub tot_combs_reached: usize,
}

impl Stats {
    pub fn update(&mut self, cards: &[u8], sols: usize, sol_reached: &[bool]) {
        for (i, reached) in sol_reached.iter().enumerate() {
            if *reached {
                self.sol_count[i] += 1;
            }
        }

        // Count this file
        self.files += 1;

        // Add solution count to the total number of solutions
        self.tot_sols += sols;

        if sols > 0 {
            // Add count to the count buckets
            self.sol_25_bucket[(sols - 1) / 25] += 1;
            self.sol_50_bucket[(sols - 1) / 50] += 1;
            self.sol_100_bucket[(sols - 1) / 100] += 1;
        }

        // Update minimum solutions list
        if self.min_sols.is_none() {
            self.min_sols = Some(Vec::new());
            self.min_sol_cnt = sols;
        }

        if sols < self.min_sol_cnt {
            let min_sols = self.min_sols.as_mut().unwrap();
            min_sols.clear();
            self.min_sol_cnt = sols;
        }

        if sols == self.min_sol_cnt {
            let min_sols = self.min_sols.as_mut().unwrap();
            min_sols.push(cards.to_vec());
        }

        // Update maximum solutions list
        if self.max_sols.is_none() {
            self.max_sols = Some(Vec::new());
            self.max_sol_cnt = sols;
        }

        if sols > self.max_sol_cnt {
            let max_sols = self.max_sols.as_mut().unwrap();
            max_sols.clear();
            self.max_sol_cnt = sols;
        }

        if sols == self.max_sol_cnt {
            let max_sols = self.max_sols.as_mut().unwrap();
            max_sols.push(cards.to_vec());
        }

        self.tot_combs += 900;
        self.tot_combs_reached += sols;
    }

    pub fn output(&self, desc: &str) {
        let mut min_sols = self.sol_count[0];
        let mut min_sol_elems = Vec::new();
        let mut max_sols = self.sol_count[0];
        let mut max_sol_elems = Vec::new();

        println!("===== {} =====", desc);
        println!("Target, Combinations");

        for (i, &n) in self.sol_count.iter().enumerate() {
            println!("{}, {}, {}", i + 100, n, percent(n, self.files));

            // Calculate the target(s) with the minimum number of solutions
            if n < min_sols {
                min_sols = n;
                min_sol_elems.clear();
            }

            if n == min_sols {
                min_sol_elems.push(i);
            }

            // Calculate the target(s) with the maximum number of solutions
            if n > max_sols {
                max_sols = n;
                max_sol_elems.clear();
            }

            if n == max_sols {
                max_sol_elems.push(i);
            }
        }

        // Output bucket statistics
        let bucket_output = |buckets: &[usize], size| {
            let mut cumul = 0;

            println!();
            println!("{} Targets Achieved (buckets of {})", desc, size);

            for (i, n) in buckets.iter().enumerate() {
                cumul += n;

                println!("{}-{}, {}, {}, {}, {}",
                    (i * size) + 1,
                    (i + 1) * size,
                    n,
                    percent(*n, self.files),
                    cumul,
                    percent(cumul, self.files)
                );
            }
        };

        bucket_output(&self.sol_25_bucket, 25);
        bucket_output(&self.sol_50_bucket, 50);
        bucket_output(&self.sol_100_bucket, 100);

        // General statistics section
        println!();
        println!("{} Statistics", desc);

        let elems = min_sol_elems
            .iter()
            .map(|n| (n + 100).to_string())
            .collect::<Vec<String>>()
            .join(", ");

        println!("Min Target Achieved, {}, {}, Targets, {}", min_sols, percent(min_sols, self.files), elems);

        let elems = max_sol_elems
            .iter()
            .map(|n| (n + 100).to_string())
            .collect::<Vec<String>>()
            .join(", ");

        println!("Max Target Achieved, {}, {}, Targets, {}", max_sols, percent(max_sols, self.files), elems);

        let avg_achieved = average(self.tot_sols, self.files);
        println!("Average Target Achieved, {:.2}, {}", avg_achieved, percent(avg_achieved, 900));

        // Minimum solutions
        let sols = self.min_sols.as_ref().unwrap();
        let count = sols.len();
        print!("Min Solutions, {}, {}, Count, {}", self.min_sol_cnt, percent(self.min_sol_cnt, 900), count);

        if count <= 5 {
            println!(", Cards, {:?}", sols);
        } else {
            println!();
        }

        // Maximum solutions
        let sols = self.max_sols.as_ref().unwrap();
        let count = sols.len();
        print!("Max Solutions, {}, {}, Count, {}", self.max_sol_cnt, percent(self.max_sol_cnt, 900), count);

        if count <= 5 {
            println!(", Cards, {:?}", sols);
        } else {
            println!();
        }

        println!("Card Combinations, {}", self.files);
        println!("Card/Target combinations, {}", self.tot_combs);
        println!("Card/Target combinations reached, {}, {}", self.tot_combs_reached, percent(self.tot_combs_reached, self.tot_combs));
    }
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
            sol_100_bucket: vec![0; TARGET_COUNT / 100],
            tot_combs: 0,
            tot_combs_reached: 0,
        }
    }
}
