// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(generators)]
#![feature(map_try_insert)]
#![feature(lint_reasons)]
// FIXME: This is a false-positive clippy test, remove this while bumping toolchain.
// https://github.com/tokio-rs/tokio/issues/4836
// https://github.com/rust-lang/rust-clippy/issues/8493
#![expect(clippy::declare_interior_mutable_const)]

use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::hash::Hash;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::Poll;
use std::time::{Duration, Instant};

use futures::future::Fuse;
use futures::{Future, FutureExt};
use indextree::{Arena, NodeId};
use itertools::Itertools;
use pin_project::{pin_project, pinned_drop};
use tokio::sync::watch;

pub type SpanValue = Cow<'static, str>;

/// The report of a stack trace.
#[derive(Debug, Clone)]
pub struct StackTraceReport {
    pub report: String,
    pub capture_time: Instant,
}

impl Default for StackTraceReport {
    fn default() -> Self {
        Self {
            report: "<not reported>".to_string(),
            capture_time: Instant::now(),
        }
    }
}

impl std::fmt::Display for StackTraceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[captured {:?} ago]\n{}",
            self.capture_time.elapsed(),
            self.report
        )
    }
}

/// Node in the span tree.
#[derive(Debug)]
struct SpanNode {
    span: SpanValue,
    // TODO: may use a more efficient timing mechanism
    start_time: Instant,
}

impl SpanNode {
    /// Create a new node with the given value.
    fn new(span: SpanValue) -> Self {
        Self {
            span,
            start_time: Instant::now(),
        }
    }
}

type ContextId = u64;

#[derive(Debug)]
struct TraceContext {
    id: ContextId,
    arena: Arena<SpanNode>,
    root: NodeId,
    current: NodeId,
}

impl std::fmt::Display for TraceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_node(
            f: &mut std::fmt::Formatter<'_>,
            arena: &Arena<SpanNode>,
            node: NodeId,
            depth: usize,
        ) -> std::fmt::Result {
            f.write_str(&" ".repeat(depth * 2))?;

            let inner = arena[node].get();
            f.write_str(inner.span.as_ref())?;

            let elapsed = inner.start_time.elapsed();
            f.write_fmt(format_args!(
                " [{}{:?}]",
                if depth > 0 && elapsed.as_secs() >= 1 {
                    "!!! "
                } else {
                    ""
                },
                elapsed
            ))?;

            f.write_char('\n')?;
            for child in node
                .children(arena)
                .sorted_by(|&a, &b| arena[a].get().span.cmp(&arena[b].get().span))
            {
                fmt_node(f, arena, child, depth + 1)?;
            }

            Ok(())
        }

        fmt_node(f, &self.arena, self.root, 0)
    }
}

impl TraceContext {
    /// Create a new stack trace context with the given root span.
    fn new(root_span: SpanValue) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = ID.fetch_add(1, Ordering::SeqCst);

        let mut arena = Arena::new();
        let root = arena.new_node(SpanNode::new(root_span));

        Self {
            id,
            arena,
            root,
            current: root,
        }
    }

    /// Get the count of active span nodes in this context.
    #[cfg_attr(not(test), expect(dead_code))]
    fn active_node_count(&self) -> usize {
        self.arena.iter().filter(|n| !n.is_removed()).count()
    }

    /// Get the report of the current state of the stack trace.
    fn to_report(&self) -> StackTraceReport {
        let report = format!("{}", self);
        StackTraceReport {
            report,
            capture_time: Instant::now(),
        }
    }

    /// Push a new span as a child of current span. Returns the new current span.
    fn push(&mut self, span: SpanValue) -> NodeId {
        let child = self.arena.new_node(SpanNode::new(span));
        self.current.append(child, &mut self.arena);
        self.current = child;
        child
    }

    /// Step in the current span to the given child.
    ///
    /// If the child is not actually a child of the current span, it means we are using a new future
    /// to poll it, so we need to detach it from the previous parent, and attach it to the current
    /// span.
    fn step_in(&mut self, child: NodeId) {
        if !self.current.children(&self.arena).contains(&child) {
            // Actually we can always call this even if `child` is already a child of `current`.
            self.current.append(child, &mut self.arena);
        }
        self.current = child;
    }

    /// Pop the current span to the parent.
    fn pop(&mut self) {
        assert!(self.current.children(&self.arena).next().is_none());
        let parent = self.arena[self.current]
            .parent()
            .expect("the root node should not be popped");
        self.current.remove(&mut self.arena);
        self.current = parent;
    }

    /// Step out the current span to the parent.
    fn step_out(&mut self) {
        let parent = self.arena[self.current]
            .parent()
            .expect("the root node should not be stepped out");
        self.current = parent;
    }

    /// Remove the current span and detach the children, used for future aborting. The children might be polled again later, and will be added as the children of a new span.
    fn remove_and_detach(&mut self, node: NodeId) {
        node.detach(&mut self.arena);
        // Removing detached `node` makes children detached.
        node.remove(&mut self.arena);
    }
}

tokio::task_local! {
    static TRACE_CONTEXT: RefCell<TraceContext>
}

fn with_context<F, R>(f: F) -> R
where
    F: FnOnce(RefMut<TraceContext>) -> R,
{
    TRACE_CONTEXT.with(|trace_context| {
        let trace_context = trace_context.borrow_mut();
        f(trace_context)
    })
}

/// State for stack traced future.
enum StackTracedState {
    Initial(SpanValue),
    Polled {
        /// The node associated with this future.
        this_node: NodeId,
        // The id of the context where this future is first polled.
        this_context: ContextId,
    },
    Ready,
}

/// The future for [`StackTrace::stack_trace`].
#[pin_project(PinnedDrop)]
pub struct StackTraced<F: Future> {
    #[pin]
    inner: F,

    /// The state of this traced future.
    state: StackTracedState,
}

impl<F: Future> StackTraced<F> {
    fn new(inner: F, span: impl Into<SpanValue>) -> Self {
        Self {
            inner,
            state: StackTracedState::Initial(span.into()),
        }
    }
}

impl<F: Future> Future for StackTraced<F> {
    type Output = F::Output;

    // TODO: may disable based on features
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let current_context = TRACE_CONTEXT.try_with(|c| c.borrow().id);

        // For assertion.
        let old_current = with_context(|c| c.current.clone());

        let this_node = match this.state {
            StackTracedState::Initial(span) => {
                match current_context {
                    // First polled
                    Ok(current_context) => {
                        // First polled, push a new span to the context.
                        let node = with_context(|mut c| c.push(std::mem::take(span)));
                        *this.state = StackTracedState::Polled {
                            this_node: node,
                            this_context: current_context,
                        };
                        node
                    }
                    // Not in a context
                    Err(_) => return this.inner.poll(cx),
                }
            }
            StackTracedState::Polled {
                this_node,
                this_context,
            } => {
                match current_context {
                    // Context correct
                    Ok(current_context) if current_context == *this_context => {
                        // Polled before, just step in.
                        with_context(|mut c| c.step_in(*this_node));
                        *this_node
                    }
                    // Context changed
                    Ok(_) => {
                        tracing::warn!("stack traced future is polled in a different context as it was first polled, won't be traced now");
                        return this.inner.poll(cx);
                    }
                    // Out of context
                    Err(_) => {
                        tracing::warn!("stack traced future is not polled in a traced context, while it was when first polled, won't be traced now");
                        return this.inner.poll(cx);
                    }
                }
            }
            StackTracedState::Ready => unreachable!("the traced future should always be fused"),
        };

        // The current node must be the this_node.
        assert_eq!(this_node, with_context(|c| c.current.clone()));

        let r = match this.inner.poll(cx) {
            // The future is ready, clean-up this span by popping from the context.
            Poll::Ready(output) => {
                assert_eq!(this_node, with_context(|c| c.current.clone()));
                with_context(|mut c| c.pop());
                *this.state = StackTracedState::Ready;
                Poll::Ready(output)
            }
            // Still pending, just step out.
            Poll::Pending => {
                with_context(|mut c| c.step_out());
                Poll::Pending
            }
        };

        // The current node must be the same as we started with.
        assert_eq!(old_current, with_context(|c| c.current.clone()));

        r
    }
}

#[pinned_drop]
impl<F: Future> PinnedDrop for StackTraced<F> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        let current_context = TRACE_CONTEXT.try_with(|c| c.borrow().id);

        match this.state {
            StackTracedState::Polled {
                this_node,
                this_context,
            } => match current_context {
                // Context correct
                Ok(current_context) if current_context == *this_context => {
                    with_context(|mut c| c.remove_and_detach(*this_node));
                }
                // Context changed
                Ok(_) => {
                    tracing::warn!("stack traced future is dropped in a different context as it was first polled, cannot clean up!");
                }
                // Out of context
                Err(_) => {
                    tracing::warn!("stack traced future is not in a traced context, while it was when first polled, cannot clean up!");
                }
            },
            StackTracedState::Initial(_) | StackTracedState::Ready => {}
        }
    }
}

impl<T> StackTrace for T where T: Future {}

pub trait StackTrace: Future + Sized {
    /// Wrap this future, so that we're able to check the stack trace and find where and why this
    /// future is pending, with [`StackTraceReport`] and [`StackTraceManager`].
    fn stack_trace(self, span: impl Into<SpanValue>) -> Fuse<StackTraced<Self>> {
        StackTraced::new(self, span).fuse()
    }
}

pub type TraceSender = watch::Sender<StackTraceReport>;
pub type TraceReceiver = watch::Receiver<StackTraceReport>;

/// Manages the stack traces of multiple tasks.
#[derive(Default, Debug)]
pub struct StackTraceManager<K> {
    rxs: HashMap<K, TraceReceiver>,
}

impl<K> StackTraceManager<K>
where
    K: Hash + Eq + std::fmt::Debug,
{
    /// Register with given key. Returns a sender that should be provided to [`stack_traced`].
    pub fn register(&mut self, key: K) -> TraceSender {
        let (tx, rx) = watch::channel(Default::default());
        self.rxs.try_insert(key, rx).unwrap();
        tx
    }

    /// Get all trace reports registered in this manager.
    ///
    /// Note that the reports might not be updated if the traced task is doing some computation
    /// heavy work and never yields, one may see the captured time to check this.
    pub fn get_all(&mut self) -> impl Iterator<Item = (&K, watch::Ref<StackTraceReport>)> {
        self.rxs.retain(|_, rx| rx.has_changed().is_ok());
        self.rxs.iter_mut().map(|(k, v)| (k, v.borrow_and_update()))
    }
}

/// Provide a stack tracing context with the `root_span` for the given future `f`. A reporter will
/// be started in the current task and update the captured stack trace report through the given
/// `trace_sender` every `interval` time.
pub async fn stack_traced<F: Future>(
    f: F,
    root_span: impl Into<SpanValue>,
    trace_sender: TraceSender,
    interval: Duration,
) -> F::Output {
    TRACE_CONTEXT
        .scope(
            RefCell::new(TraceContext::new(root_span.into())),
            async move {
                let reporter = async move {
                    let mut interval = tokio::time::interval(interval);
                    loop {
                        interval.tick().await;
                        let new_trace = TRACE_CONTEXT.with(|c| c.borrow().to_report());
                        match trace_sender.send(new_trace) {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("Trace report error: failed to send trace: {}", e);
                                futures::future::pending().await
                            }
                        }
                    }
                };

                tokio::select! {
                    output = f => output,
                    _ = reporter => unreachable!()
                }
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use futures::future::{join_all, select_all};
    use futures::StreamExt;
    use futures_async_stream::stream;
    use tokio::sync::watch;

    use super::*;

    async fn sleep(time: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(time)).await;
        println!("slept {time}ms");
    }

    async fn sleep_nested() {
        join_all([
            sleep(1500).stack_trace("sleep nested 1500"),
            sleep(2500).stack_trace("sleep nested 2500"),
        ])
        .await;
    }

    async fn multi_sleep() {
        sleep(400).await;

        sleep(800).stack_trace("sleep another in multi slepp").await;
    }

    #[stream(item = ())]
    async fn stream1() {
        loop {
            sleep(150).await;
            yield;
        }
    }

    #[stream(item = ())]
    async fn stream2() {
        sleep(200).await;
        yield;
        join_all([
            sleep(400).stack_trace("sleep nested 400"),
            sleep(600).stack_trace("sleep nested 600"),
        ])
        .stack_trace("sleep nested another in stream 2")
        .await;
        yield;
    }

    async fn hello() {
        async move {
            // Join
            join_all([
                sleep(1000).boxed().stack_trace(format!("sleep {}", 1000)),
                sleep(2000).boxed().stack_trace("sleep 2000"),
                sleep_nested().boxed().stack_trace("sleep nested"),
                multi_sleep().boxed().stack_trace("multi sleep"),
            ])
            .await;

            // Join another
            join_all([
                sleep(1200).stack_trace("sleep 1200"),
                sleep(2200).stack_trace("sleep 2200"),
            ])
            .await;

            // Cancel
            select_all([
                sleep(666).boxed().stack_trace("sleep 666"),
                sleep_nested()
                    .boxed()
                    .stack_trace("sleep nested (should be cancelled)"),
            ])
            .await;

            // Check whether cleaned up
            sleep(233).stack_trace("sleep 233").await;

            // Check stream next drop
            {
                let mut stream1 = stream1().fuse().boxed();
                let mut stream2 = stream2().fuse().boxed();
                let mut count = 0;

                'outer: loop {
                    tokio::select! {
                        _ = stream1.next().stack_trace(format!("stream1 next {count}")) => {},
                        r = stream2.next().stack_trace(format!("stream2 next {count}")) => {
                            if r.is_none() { break 'outer }
                        },
                    }
                    count += 1;
                }
            }

            // Check whether cleaned up
            sleep(233).stack_trace("sleep 233").await;

            // TODO: add tests on sending the future to another task or context.
        }
        .stack_trace("hello")
        .await;

        // Aborted futures have been cleaned up. There should only be a single active node of root.
        assert_eq!(with_context(|c| c.active_node_count()), 1);
    }

    #[tokio::test]
    async fn test_stack_trace_display() {
        let (watch_tx, mut watch_rx) = watch::channel(Default::default());

        let collector = tokio::spawn(async move {
            while watch_rx.changed().await.is_ok() {
                println!("{}", &*watch_rx.borrow());
            }
        });

        stack_traced(hello(), "actor 233", watch_tx, Duration::from_millis(50)).await;

        collector.await.unwrap();
    }
}

fn main() {}
