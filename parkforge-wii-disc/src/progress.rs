#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    completed: u64,
    total: u64,
}

impl Progress {
    pub(crate) fn new(completed: u64, total: u64) -> Self {
        debug_assert!(completed <= total);
        Self { completed, total }
    }

    #[must_use]
    pub fn completed(self) -> u64 {
        self.completed
    }

    #[must_use]
    pub fn total(self) -> u64 {
        self.total
    }

    #[must_use]
    pub fn fraction(self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.completed as f64 / self.total as f64
        }
    }

    #[must_use]
    pub fn percent(self) -> f64 {
        self.fraction() * 100.0
    }

    #[must_use]
    pub fn is_complete(self) -> bool {
        self.completed == self.total
    }
}