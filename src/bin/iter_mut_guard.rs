use std::cell::{RefCell, RefMut};

fn main() {}

struct Foo<T> {
    v: Vec<T>,
    size: RefCell<usize>,
}

struct MutGuard<'a, T> {
    e: &'a mut T,
    size: RefMut<'a, usize>,
}

impl<T> Foo<T> {
    fn iter_mut(&mut self) -> impl Iterator<Item = MutGuard<'_, T>> + '_ {
        self.v.iter_mut().map(|r| MutGuard {
            e: r,
            size: self.size.borrow_mut(),
        })
    }
}
