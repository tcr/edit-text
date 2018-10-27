use std::sync::{
    Arc,
    Barrier,
};

// Sync barrier, optional sequential barrier.
// TODO this whole thing should be refactored:
// Checkpoint::create(5) -> Vec<Checkpoint; 5> then map to create your threads
#[derive(Clone)]
pub struct Checkpoint(Arc<Barrier>, (Arc<Barrier>, bool));

impl Checkpoint {
    pub fn new_pair() -> (Checkpoint, Checkpoint) {
        let both_barrier = Arc::new(Barrier::new(2));
        let seq_barrier = Arc::new(Barrier::new(2));
        (
            Checkpoint(both_barrier.clone(), (seq_barrier.clone(), false)),
            Checkpoint(both_barrier.clone(), (seq_barrier.clone(), true)),
        )
    }

    pub fn sync(&self) {
        if !(self.1).1 {
            (self.1).0.wait();
        }
        // Then synchronize both clients.
        self.0.wait();
    }

    // Sequential until next .sync()
    pub fn sequential(&self) {
        if (self.1).1 {
            (self.1).0.wait();
        }
    }
}
