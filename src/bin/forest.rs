use std::time::Duration;

use futures::{future::join_all, stream::BoxStream, FutureExt, StreamExt};
use tracing::{info, instrument, span, Subscriber};
use tracing_forest::ForestLayer;
use tracing_futures::Instrument;
use tracing_subscriber::{
    filter::{FilterFn, Targets},
    fmt,
    prelude::*,
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer, Registry,
};

#[instrument()]
async fn work_2() {
    tokio::time::sleep(Duration::from_millis(1000))
        .instrument(span!(tracing::Level::INFO, "sleep in work_2"))
        .await;
}

#[instrument()]
async fn work_3() {
    tokio::time::sleep(Duration::from_millis(2000))
        .instrument(span!(tracing::Level::INFO, "sleep in work_3"))
        .await;
}

#[instrument()]
async fn work_1() {
    join_all([work_2().boxed(), work_3().boxed()])
        .instrument(span!(tracing::Level::INFO, "join_all"))
        .await;

    futures::stream::once(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        ()
    })
    .boxed()
    .instrument(span!(tracing::Level::INFO, "stream next"))
    .next()
    .await;
}

struct MyLayer;

impl<S> Layer<S> for MyLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        info!("on_enter: {:?} {}", id, span.name())
    }

    fn on_exit(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        info!("on_exit: {:?} {}", id, span.name())
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        info!("on_close: {:?} {}", id, span.name())
    }
}

#[tokio::main]
async fn main() {
    Registry::default()
        .with(ForestLayer::default().with_filter(FilterFn::new(|m| m.is_span())))
        .with(MyLayer)
        .with(tracing_subscriber::fmt::layer())
        .with(Targets::new().with_target("forest", tracing::Level::INFO))
        .init();

    work_1().await;
}
