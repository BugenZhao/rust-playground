use std::{
    collections::{btree_map, BTreeMap},
    sync::Arc,
};

use bytes::Bytes;

// pub type MergedImmIter<'a> = btree_map::Iter<'a, Bytes, Bytes>;

struct MergedImm {
    payload: Arc<BTreeMap<Bytes, Bytes>>,
}

struct MergedImmIter {
    payload: Arc<BTreeMap<Bytes, Bytes>>,

    iter: btree_map::Iter<'static, Bytes, Bytes>,
}

impl MergedImmIter {
    fn new(imm: MergedImm) -> Self {
        let r = Arc::as_ptr(&imm.payload);
        let iter = unsafe { &*r }.iter();

        Self {
            payload: imm.payload,
            iter,
        }
    }

    fn next(&mut self) -> Option<(Bytes, Bytes)> {
        self.iter.next().map(|(k, v)| (k.clone(), v.clone()))
    }
}

fn assert_static<T>()
where
    T: 'static,
{
}

fn main() {
    // assert_static::<MyIterator>();
}
