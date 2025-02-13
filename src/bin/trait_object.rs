fn main() {}

trait A: 'static {
    fn static_method();
}

// fn test(a: &dyn A) {
//     a.static_method();
// }

// ----------------------------

trait A2 {
    fn b(&self) -> Box<dyn B>;
}

trait B {
    fn method(&self);
}

// pub trait BoxedExecutorBuilder {
//     #[must_use]
//     #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
//     fn new_boxed_executor<'life0, 'life1, 'async_trait>(
//         source: &'life0 ExecutorBuilder<'life1>,
//         inputs: Vec<BoxedExecutor>,
//     ) -> ::core::pin::Pin<
//         Box<
//             dyn ::core::future::Future<Output = Result<BoxedExecutor>>
//                 + ::core::marker::Send
//                 + 'async_trait,
//         >,
//     >
//     where
//         'life0: 'async_trait,
//         'life1: 'async_trait;
// }

impl<T: A> A2 for T {
    fn b(&self) -> Box<dyn B> {
        struct BImpl<T> {
            _t: std::marker::PhantomData<T>,
        }

        impl<T: A> B for BImpl<T> {
            fn method(&self) {
                T::static_method();
            }
        }

        Box::new(BImpl::<T> {
            _t: std::marker::PhantomData,
        })
    }
}

fn test(b: &dyn B) {
    b.method();
}
