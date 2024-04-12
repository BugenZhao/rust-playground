use std::cell::RefCell;

// TODO: determine good values or make them configurable
const RED_ZONE: usize = 128 * 1024; // 128KiB
const STACK_SIZE: usize = 16 * RED_ZONE; // 2MiB

/// The tracker for a recursive function.
pub struct Tracker {
    depth: RefCell<usize>,
}

impl Tracker {
    /// Create a new tracker.
    pub const fn new() -> Self {
        Self {
            depth: RefCell::new(0),
        }
    }

    /// Retrieve the current depth of the recursion. Starts from 1.
    pub fn depth(&self) -> usize {
        *self.depth.borrow()
    }

    /// Run a recursive function. Grow the stack if necessary.
    fn recurse<T>(&self, f: impl FnOnce() -> T) -> T {
        struct DepthGuard<'a> {
            depth: &'a RefCell<usize>,
        }

        impl<'a> DepthGuard<'a> {
            fn new(depth: &'a RefCell<usize>) -> Self {
                *depth.borrow_mut() += 1;
                Self { depth }
            }
        }

        impl<'a> Drop for DepthGuard<'a> {
            fn drop(&mut self) {
                *self.depth.borrow_mut() -= 1;
            }
        }

        let _guard = DepthGuard::new(&self.depth);
        stacker::maybe_grow(RED_ZONE, STACK_SIZE, f)
    }
}

/// The extension trait for a thread-local tracker to run a recursive function.
#[easy_ext::ext(Recurse)]
impl std::thread::LocalKey<Tracker> {
    /// Run the given recursive function. Grow the stack if necessary.
    ///
    /// # Fearless Recursion
    ///
    /// This enables fearless recursion in most cases as long as a single frame
    /// does not exceed the [`RED_ZONE`] size. That is, the caller can recurse
    /// as much as it wants without worrying about stack overflow.
    ///
    /// # Tracker
    ///
    /// The caller can retrieve the [`Tracker`] of the current recursion from
    /// the closure argument. This can be useful for checking the depth of the
    /// recursion, logging or throwing an error gracefully if it's too deep.
    ///
    /// Note that different trackers defined in different functions are
    /// independent of each other. If there's a cross-function recursion, the
    /// tracker retrieved from the closure argument only represents the current
    /// function's state.
    ///
    /// # Example
    ///
    /// Define the tracker with [`tracker!`] and call this method on it to run
    /// a recursive function.
    ///
    /// ```ignore
    /// #[inline(never)]
    /// fn sum(x: u64) -> u64 {
    ///     tracker!().recurse(|t| {
    ///         if t.depth() % 100000 == 0 {
    ///            eprintln!("too deep!");
    ///         }
    ///         if x == 0 {
    ///             return 0;
    ///         }
    ///         x + sum(x - 1)
    ///     })
    /// }
    /// ```
    fn recurse<T>(&'static self, f: impl FnOnce(&Tracker) -> T) -> T {
        self.with(|t| t.recurse(|| f(t)))
    }
}

/// Define the tracker for recursion and return it.
///
/// Call [`Recurse::recurse`] on it to run a recursive function. See
/// documentation there for usage.
macro_rules! tracker {
    () => {{
        std::thread_local! {
            static __TRACKER: Tracker = const { Tracker::new() };
        }
        __TRACKER
    }};
}

#[inline(never)]
fn sum(x: u64) -> u64 {
    tracker!().recurse(|t| {
        if t.depth() % 100000 == 0 {
            eprintln!("too deep!");
        }
        if x == 0 {
            return 0;
        }
        x + sum(x - 1)
    })
}

fn main() {
    dbg!(sum(1919810));
}
