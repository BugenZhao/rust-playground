#![feature(generators)]

use std::{marker::PhantomData, pin::Pin};

use futures::{pin_mut, stream::BoxStream, Stream, StreamExt};
use futures_async_stream::stream;

struct Stats {
    count: usize,
}

impl Drop for Stats {
    fn drop(&mut self) {
        println!("Dropped with count {}", self.count);
    }
}

struct StatsStream<S, T> {
    inner: S,
    stats: Stats,
    _phantom: PhantomData<T>,
}

impl<S, T> StatsStream<S, T> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            stats: Stats { count: 0 },
            _phantom: PhantomData,
        }
    }
}

impl<S, T> StatsStream<S, T>
where
    S: Stream<Item = T>,
{
    #[stream(item = T)]
    async fn into_stream(mut self) {
        let inner = self.inner;
        pin_mut!(inner);
        while let Some(t) = inner.next().await {
            self.stats.count += 1;
            yield t;
        }
    }
}

#[tokio::main]
async fn main() {
    let st = StatsStream::new(futures::stream::iter(0..10).boxed()).into_stream();
    pin_mut!(st);
    st.by_ref().take(3).count().await;
    st.count().await;
}
