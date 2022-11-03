#![feature(bound_as_ref)]
#![feature(exact_size_is_empty)]

use std::{
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
    sync::{Arc, RwLock},
    vec,
};

use itertools::Itertools;
use rand::Rng;

struct Iter<K, V> {
    inner: Arc<RwLock<BTreeMap<K, V>>>,

    range: (Bound<K>, Bound<K>),

    current: vec::IntoIter<(K, V)>,
}

impl<K, V> Iter<K, V> {
    fn new(inner: Arc<RwLock<BTreeMap<K, V>>>, range: (Bound<K>, Bound<K>)) -> Self {
        Self {
            inner,
            range,
            current: Vec::new().into_iter(),
        }
    }
}

impl<K, V> Iter<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    const BATCH_SIZE: usize = 256;

    fn refill(&mut self) {
        assert!(self.current.is_empty());

        let batch: Vec<(K, V)> = self
            .inner
            .read()
            .unwrap()
            .range((self.range.0.as_ref(), self.range.1.as_ref()))
            .take(Self::BATCH_SIZE)
            .map(|(k, v)| (K::clone(k), V::clone(v)))
            .collect_vec();

        if let Some((last_key, _)) = batch.last() {
            self.range.0 = Bound::Excluded(K::clone(last_key));
        }
        self.current = batch.into_iter();
    }
}

impl<K, V> Iterator for Iter<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.next() {
            Some(r) => return Some(r),
            None => {
                self.refill();
                self.current.next()
            }
        }
    }
}

fn main() {
    let key_range = 1..=10000;
    let map: BTreeMap<i32, String> = key_range.clone().map(|k| (k, k.to_string())).collect();
    let map = Arc::new(RwLock::new(map));

    let rand_bound = || {
        let key = rand::thread_rng().gen_range(key_range.clone());
        match rand::thread_rng().gen_range(1..=3) {
            1 => Bound::Included(key),
            2 => Bound::Excluded(key),
            _ => Bound::Unbounded,
        }
    };

    for _ in 0..10000 {
        let range = loop {
            let range = (rand_bound(), rand_bound());
            let (start, end) = (range.start_bound(), range.end_bound());
            match (start, end) {
                (Bound::Excluded(s), Bound::Excluded(e)) if s == e => {
                    continue;
                }
                (
                    Bound::Included(s) | Bound::Excluded(s),
                    Bound::Included(e) | Bound::Excluded(e),
                ) if s > e => {
                    continue;
                }
                _ => break range,
            }
        };
        let v1 = Iter::new(map.clone(), range.clone()).collect_vec();
        let v2 = map
            .read()
            .unwrap()
            .range(range)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect_vec();
        assert_eq!(v1, v2);
    }
}
