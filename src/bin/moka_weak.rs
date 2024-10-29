use std::sync::{Arc, Weak};

use moka::sync::Cache;

struct Bomb;

impl Drop for Bomb {
    fn drop(&mut self) {
        println!("Dropping the bomb!");
    }
}

#[derive(Clone, Debug)]
struct SharedBomb(Weak<Bomb>);

impl SharedBomb {
    fn exists(&self) -> bool {
        self.upgrade().is_some()
    }

    fn upgrade(&self) -> Option<Arc<Bomb>> {
        self.0.upgrade()
    }
}

fn main() {
    let cache = Cache::builder().build();

    let inserted = {
        let mut strong_bomb = None;

        cache
            .entry(42)
            .or_insert_with(|| {
                let bomb = Arc::new(Bomb);
                let result = SharedBomb(Arc::downgrade(&bomb));
                strong_bomb = Some(bomb);
                result
            })
            .into_value()
            .upgrade()
            .unwrap()
    };

    assert!(cache.get(&42).unwrap().exists());

    drop(inserted);

    assert!(!cache.get(&42).unwrap().exists());
}
