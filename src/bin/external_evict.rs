#![feature(coroutines)]
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use futures::{future::poll_fn, stream::BoxStream, StreamExt};
use futures_async_stream::stream;
use tokio::sync::Mutex;

struct Chunk {
    key: i32,
    value: String,
}

enum Message {
    Chunk(Chunk),
    Barrier,
}

struct CacheInner {
    inner: HashMap<i32, String>,
}

impl CacheInner {
    fn evict(&mut self) {}
}

type Cache = Arc<Mutex<CacheInner>>;

trait Executor: Send + 'static {
    fn do_execute(self: Box<Self>) -> BoxStream<'static, Message>;

    fn cache(&self) -> Cache;

    fn execute(self: Box<Self>) -> BoxStream<'static, Message> {
        let cache = self.cache();

        (#[stream]
        async move {
            let mut stream = self.do_execute();

            while let Some(message) = poll_fn(|cx| {
                if let Ok(mut cache) = cache.try_lock() {
                    cache.evict();
                }
                stream.poll_next_unpin(cx)
            })
            .await
            {
                yield message;
            }
        })
        .boxed()
    }
}

type BoxExecutor = Box<dyn Executor>;

struct State {
    inner: HashMap<i32, String>,
}

impl State {
    fn put(&mut self, key: i32, value: String) {
        self.inner.insert(key, value);
    }

    fn delete(&mut self, key: i32) {
        self.inner.remove(&key);
    }

    async fn get(&self, key: i32) -> Option<&String> {
        self.inner.get(&key)
    }

    async fn flush(&mut self) {}
}

struct HashStringAggExecutor {
    input: BoxExecutor,
    state: State,
    cache: Cache,
}

impl Executor for HashStringAggExecutor {
    fn do_execute(mut self: Box<Self>) -> BoxStream<'static, Message> {
        (#[stream]
        async move {
            let mut input = self.input.execute();

            while let Some(message) = input.next().await {
                match message {
                    Message::Chunk(chunk) => {
                        let value = {
                            let mut cache = self.cache.try_lock().unwrap();

                            let value = match cache.inner.entry(chunk.key) {
                                Entry::Occupied(o) => o.into_mut(),
                                Entry::Vacant(v) => {
                                    let value = self
                                        .state
                                        .get(chunk.key)
                                        .await
                                        .cloned()
                                        .unwrap_or_default();
                                    v.insert(value)
                                }
                            };

                            value.push_str(&chunk.value);
                            value.clone()
                        };

                        self.state.put(chunk.key, value.clone());

                        yield Message::Chunk(Chunk {
                            key: chunk.key,
                            value,
                        });
                    }

                    Message::Barrier => {
                        self.state.flush().await;

                        yield Message::Barrier;
                    }
                }
            }
        })
        .boxed()
    }

    fn cache(&self) -> Cache {
        self.cache.clone()
    }
}

fn main() {}
