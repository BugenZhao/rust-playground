// Copyright 2024 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct Pausable<St>
        where St: Stream
    {
        #[pin]
        stream: St,
        paused: Arc<AtomicBool>,
    }
}

#[derive(Clone)]
pub struct Valve {
    paused: Arc<AtomicBool>,
}

impl Valve {
    /// Pause the stream controlled by the valve.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed)
    }

    /// Resume the stream controlled by the valve.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }
}

impl<St> Pausable<St>
where
    St: Stream,
{
    pub(crate) fn new(stream: St) -> (Self, Valve) {
        let paused = Arc::new(AtomicBool::new(false));
        (
            Pausable {
                stream,
                paused: paused.clone(),
            },
            Valve { paused },
        )
    }
}

impl<St> Stream for Pausable<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if this.paused.load(Ordering::Relaxed) {
            Poll::Pending
        } else {
            this.stream.poll_next(cx)
        }
    }
}

#[tokio::main]
async fn main() {
    let stream = futures::stream::repeat(()).then(|_| async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        42
    });

    let (pausable, valve) = Pausable::new(stream);

    tokio::spawn(async move {
        let mut pausable = std::pin::pin!(pausable);
        while let Some(n) = pausable.next().await {
            println!("got {}", n);
        }
    });

    tokio::time::sleep(Duration::from_secs(3)).await;
    valve.pause();
    println!("paused");
    tokio::time::sleep(Duration::from_secs(3)).await;
    valve.resume();
    println!("resumed");
    tokio::time::sleep(Duration::from_secs(3)).await;
}
