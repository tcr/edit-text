use std::sync::{
    Arc,
    Barrier,
};

// Sync barrier, optional sequential barrier.
pub struct Checkpoint {
    pub index: usize,
    all: Arc<Barrier>,               // joint
    sequential: bool,                // sequence active
    seq_barriers: Arc<Vec<Barrier>>, // sequential
}

impl Checkpoint {
    pub fn new_pair() -> (Checkpoint, Checkpoint) {
        let mut pair = Checkpoint::generate(2);
        assert_eq!(pair.len(), 2);
        (pair.remove(0), pair.remove(0))
    }

    pub fn generate(count: usize) -> Vec<Checkpoint> {
        let all = Arc::new(Barrier::new(count));
        let seq_barriers = Arc::new(
            (0..count - 1)
                .map(|index| Barrier::new(count - index))
                .collect::<Vec<_>>(),
        );
        (0..count)
            .map(|index| Checkpoint {
                index,
                all: all.clone(),
                sequential: false,
                seq_barriers: seq_barriers.clone(),
            })
            .collect()
    }

    pub fn sync(&mut self) {
        // Unblock other sequential clients.
        if self.sequential {
            self.sequential = false;
            // Only if we're not the last client...
            if self.seq_barriers.len() != self.index {
                // Unblock clients waiting on us.
                self.seq_barriers[self.index].wait();
            }
        }

        // Synchronize *all* clients.
        self.all.wait();
    }

    // Sequential until next .sync()
    pub fn sequential(&mut self) {
        if self.sequential {
            unreachable!();
        }
        self.sequential = true;

        // Wait on all locks lower than our current lock.
        for i in 0..self.index {
            self.seq_barriers[i].wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use rand::Rng;
    use std::sync::mpsc::channel;

    #[test]
    fn it_works() {
        const COUNT: usize = 32;

        let (tx, rx) = channel();

        Checkpoint::generate(COUNT)
            .into_iter()
            .enumerate()
            .map(|(i, mut checkpoint)| {
                let tx = tx.clone();
                ::std::thread::spawn(move || {
                    eprintln!("---> thread #{:?} starts", i);

                    checkpoint.sync();
                    eprintln!("---> thread #{:?} synced", i);

                    // Sleep for random interval
                    let interval = rand::thread_rng().gen_range(100, 5_000);
                    eprintln!("---> thread #{:?} sleeping for {}ms", i, interval);
                    ::std::thread::sleep(::std::time::Duration::from_millis(interval));

                    checkpoint.sequential();
                    tx.send(i).unwrap();
                    eprintln!("---> thread #{:?} sequential", i);
                    ::std::thread::sleep(::std::time::Duration::from_millis(25));
                    checkpoint.sync();
                    eprintln!("---> thread #{:?} synced", i);
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|handle| {
                handle.join().unwrap();
            });

        let results = (0..COUNT).map(|_| rx.recv().unwrap()).collect::<Vec<_>>();
        assert_eq!(results, (0..COUNT).collect::<Vec<_>>());
    }
}
