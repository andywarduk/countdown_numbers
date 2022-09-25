use std::cmp::Ordering;

/// Holds the result of running a program
#[derive(Eq)]
pub struct Solution {
    /// Program element providing the solution
    pub program: usize,
    /// Length of the program instructions
    length: usize,
    /// The result of running the program with the given numbers
    pub result: u32,
}

impl Solution {
    /// Creates a new solution
    pub fn new(program: usize, length: usize, result: u32) -> Self {
        Self { program, length, result }
    }
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by result first
        let mut ord = self.result.cmp(&other.result);

        if ord == Ordering::Equal {
            // Order by length next
            ord = self.length.cmp(&other.length);

            if ord == Ordering::Equal {
                // Order by element number lastly
                ord = self.program.cmp(&other.program)
            }
        }

        ord
    }
}

impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.program == other.program
    }
}
