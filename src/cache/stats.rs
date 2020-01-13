use crate::cache::Stats;

impl Default for Stats {
    #[must_use]
    fn default() -> Self {
        Self { hits: 0, misses: 0 }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Stats {}

#[allow(unsafe_code)]
unsafe impl Sync for Stats {}

impl Stats {
    pub(crate) fn register_hit(&mut self) {
        self.hits += 1;
    }

    pub(crate) fn register_miss(&mut self) {
        self.misses += 1;
    }

    pub(crate) fn get_hits(&self) -> usize {
        self.hits
    }

    pub(crate) fn get_misses(&self) -> usize {
        self.misses
    }

    pub(crate) fn clear(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}
