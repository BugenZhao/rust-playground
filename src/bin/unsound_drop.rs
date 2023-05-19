// opt-in to the unsoundness!
// #![feature(unsafe_destructor)]

pub mod mcbean {
    use std::cell::Cell;

    pub struct StarOffMachine {
        usable: bool,
        dollars: Cell<u64>,
    }

    impl Drop for StarOffMachine {
        fn drop(&mut self) {
            let contents = self.dollars.get();
            println!("Dropping a machine; sending {} dollars to Sylvester.",
                     contents);
            self.dollars.set(0);
            self.usable = false;
        }
    }

    impl StarOffMachine {
        pub fn new() -> StarOffMachine {
            StarOffMachine { usable: true, dollars: Cell::new(0) }
        }
        pub fn remove_star(&self, s: &mut Sneetch) {
            assert!(self.usable,
                    "No different than a read of a dangling pointer.");
            self.dollars.set(self.dollars.get() + 10);
            s.has_star = false;
        }
    }

    pub struct Sneetch<'a> {
        name: &'static str,
        has_star: bool,
        machine: Cell<Option<&'a StarOffMachine>>,
    }

    impl<'a> Sneetch<'a> {
        pub fn new(name: &'static str) -> Sneetch<'a> {
            Sneetch {
                name: name,
                has_star: true,
                machine: Cell::new(None)
            }
        }

        pub fn find_machine(&self, m: &'a StarOffMachine) {
            self.machine.set(Some(m));
        }
    }

    // #[unsafe_destructor]
    impl<'a> Drop for Sneetch<'a> {
        fn drop(&mut self) {
            if let Some(m) = self.machine.get() {
                println!("{} says ``before I die, I want to join my \
                          plain-bellied brethren.''", self.name);
                m.remove_star(self);
            }
        }
    }
}

fn unwary_client() {
    use mcbean::{Sneetch, StarOffMachine};
    let (s1, m, s2, s3); // (accommodate PR 21657)
    s1 = Sneetch::new("Sneetch One");
    m = StarOffMachine::new();
    s2 = Sneetch::new("Sneetch Two");
    s3 = Sneetch::new("Sneetch Zee");

    // s1.find_machine(&m); <- ERROR
    s2.find_machine(&m);
    s3.find_machine(&m);
}

fn main() {
    unwary_client();
}
