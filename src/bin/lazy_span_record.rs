#![feature(test)]

extern crate test;

use std::marker::PhantomData;

use tokio::task_local;
use tracing::{
    field::{self, ValueSet},
    span::{self, Record},
    Event, Subscriber,
};
use tracing_subscriber::{
    fmt::{format, FormatFields},
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer,
};

task_local! {
    pub static MY_ID: u32;
}

pub struct LazyLayer<S: 'static, L> {
    inner: L,
    _subscriber: PhantomData<&'static S>,
}

impl<S, L> LazyLayer<S, L> {
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            _subscriber: PhantomData,
        }
    }
}

struct Recorded;

impl<S, L> Layer<S> for LazyLayer<S, L>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    L: Layer<S>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.lookup_current() {
            let id = span.id();
            let meta = span.metadata();

            if let Some(field) = meta.fields().field("my_id") {
                let recorded = span.extensions().get::<Recorded>().is_some();

                if !recorded {
                    let values = &[(&field, Some(&233 as &dyn field::Value))];
                    let value_set = meta.fields().value_set(values);
                    let record = Record::new(&value_set);

                    self.inner.on_record(&id, &record, ctx.clone());

                    span.extensions_mut().insert(Recorded);
                }
            }
        }

        self.inner.on_event(event, ctx);
    }

    fn on_register_dispatch(&self, collector: &tracing::Dispatch) {
        self.inner.on_register_dispatch(collector);
    }

    fn on_layer(&mut self, subscriber: &mut S) {
        self.inner.on_layer(subscriber);
    }

    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        self.inner.register_callsite(metadata)
    }

    fn enabled(&self, metadata: &tracing::Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.inner.enabled(metadata, ctx)
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        self.inner.on_new_span(attrs, id, ctx);
    }

    fn on_record(&self, _span: &span::Id, _values: &span::Record<'_>, _ctx: Context<'_, S>) {
        self.inner.on_record(_span, _values, _ctx);
    }

    fn on_follows_from(&self, _span: &span::Id, _follows: &span::Id, _ctx: Context<'_, S>) {
        self.inner.on_follows_from(_span, _follows, _ctx);
    }

    fn event_enabled(&self, _event: &Event<'_>, _ctx: Context<'_, S>) -> bool {
        self.inner.event_enabled(_event, _ctx)
    }

    fn on_enter(&self, _id: &span::Id, _ctx: Context<'_, S>) {
        self.inner.on_enter(_id, _ctx);
    }

    fn on_exit(&self, _id: &span::Id, _ctx: Context<'_, S>) {
        self.inner.on_exit(_id, _ctx);
    }

    fn on_close(&self, _id: span::Id, _ctx: Context<'_, S>) {
        self.inner.on_close(_id, _ctx);
    }

    fn on_id_change(&self, _old: &span::Id, _new: &span::Id, _ctx: Context<'_, S>) {
        self.inner.on_id_change(_old, _new, _ctx);
    }
}

fn it<const EAGER: bool>() {
    for i in 0..100_000 {
        let span = if EAGER {
            tracing::info_span!("my_span", my_id = 42)
        } else {
            tracing::info_span!("my_span", my_id = tracing::field::Empty)
        };
        let _guard = span.enter();

        if i % 1000 == 0 {
            tracing::info!("hello");
        }
    }
}

fn no_span() {
    for i in 0..100_000 {
        if i % 1000 == 0 {
            tracing::info!(my_id = 42, "hello");
        }
    }
}

// benchmark `it`
#[bench]
fn bench_lazy(b: &mut test::Bencher) {
    let null_file = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .unwrap();
    let layer = LazyLayer::new(tracing_subscriber::fmt::layer().with_writer(null_file));
    tracing_subscriber::registry().with(layer).init();

    b.iter(|| it::<false>());
}

#[bench]
fn bench_eager(b: &mut test::Bencher) {
    let null_file = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .unwrap();
    let layer = tracing_subscriber::fmt::layer().with_writer(null_file);
    tracing_subscriber::registry().with(layer).init();

    b.iter(|| it::<true>());
}

fn main() {
    let layer = LazyLayer::new(tracing_subscriber::fmt::layer());
    // let layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(layer).init();

    {
        let span = tracing::info_span!("my_span");
        let _guard = span.enter();

        tracing::info!("hello");
        tracing::info!("hello");
        tracing::info!("hello");
    }

    {
        let span = tracing::info_span!("my_span", my_id = tracing::field::Empty);
        let _guard = span.enter();

        tracing::info!("hello");
        tracing::info!("hello");
        tracing::info!("hello");
    }
}
