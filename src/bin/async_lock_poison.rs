use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;

enum State {
    Invalid,
    Value(i32),
}

impl Default for State {
    fn default() -> Self {
        Self::Value(0)
    }
}

#[derive(Clone, Default)]
struct SharedState(Arc<Mutex<State>>);

impl SharedState {
    async fn operate(&self) {
        let mut state = self.0.lock().await;

        let State::Value(v) = std::mem::replace(&mut *state, State::Invalid) else {
            panic!("bad state")
        };

        tokio::time::sleep(Duration::from_secs(5)).await;

        *state = State::Value(v + 1);
    }
}

async fn f(state: SharedState) {
    state.operate().await;
}

#[tokio::main]
async fn main() {
    let state = SharedState::default();
    let h1 = tokio::spawn(f(state.clone()));
    let h2 = tokio::spawn(f(state.clone()));

    tokio::time::sleep(Duration::from_secs(2)).await;
    h1.abort();
    h2.await.unwrap();
}
