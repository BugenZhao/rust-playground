mod inner {
    use std::ops::{Deref, DerefMut};

    use qcell::{QCell, QCellOwner};

    pub struct MyVec<T> {
        data: Vec<T>,

        mut_counter_owner: QCellOwner,
        mut_counter: QCell<usize>,
    }

    impl<T> MyVec<T> {
        pub fn new(data: Vec<T>) -> Self {
            let mut_counter_owner = QCellOwner::new();
            let mut_counter = mut_counter_owner.cell(0);

            Self {
                data,
                mut_counter_owner,
                mut_counter,
            }
        }

        pub fn get_mut(&mut self, index: usize) -> Option<MutGuard<'_, T>> {
            let item = self.data.get_mut(index)?;
            let mut_counter = self.mut_counter.get_mut();

            Some(MutGuard {
                data: item,
                mut_counter,
            })
        }

        pub fn iter_mut(
            &mut self,
        ) -> (
            &mut QCellOwner,
            impl Iterator<Item = SharedGuard<'_, T>> + '_,
        ) {
            let mut_counter = &self.mut_counter;
            let mut_counter_owner = &mut self.mut_counter_owner;

            let iter = self.data.iter_mut().map(move |item| SharedGuard {
                data: item,
                mut_counter,
            });

            (mut_counter_owner, iter)
        }

        pub fn inner(&self) -> &Vec<T> {
            &self.data
        }

        pub fn count(&self) -> usize {
            *self.mut_counter_owner.ro(&self.mut_counter)
        }
    }

    pub struct SharedGuard<'a, T> {
        data: &'a mut T,
        mut_counter: &'a QCell<usize>,
    }

    impl<'a, T> SharedGuard<'a, T> {
        pub fn into_mut(self, owner: &'a mut QCellOwner) -> MutGuard<'a, T> {
            let mut_counter = owner.rw(self.mut_counter);

            MutGuard {
                data: self.data,
                mut_counter,
            }
        }
    }

    pub struct MutGuard<'a, T> {
        data: &'a mut T,
        mut_counter: &'a mut usize,
    }

    impl<'a, T> Deref for MutGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.data
        }
    }

    impl<'a, T> DerefMut for MutGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.data
        }
    }

    impl<'a, T> Drop for MutGuard<'a, T> {
        fn drop(&mut self) {
            *self.mut_counter += 1;
        }
    }
}

use inner::*;

fn main() {
    let mut my_vec = MyVec::new(vec![1, 2, 3, 4, 5]);

    let (owner, iter) = my_vec.iter_mut();

    for item in iter {
        let mut item = item.into_mut(owner);
        *item *= 10;
    }

    println!("{:?}", my_vec.inner());
    println!("Mutations: {}", my_vec.count());
}
