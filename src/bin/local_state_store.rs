#![allow(dead_code)]
#![allow(unreachable_code)]
#![feature(trait_alias)]

use std::{collections::HashMap, sync::Arc};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

type ActorId = u64;
type ImmId = u64;
type Imm = ();

enum StateStoreControlMessage {
    PruneImm(Vec<ImmId>),
}

struct GlobalStateStore {
    control_txs: HashMap<ActorId, mpsc::UnboundedSender<StateStoreControlMessage>>,
}

struct LocalStateStore {
    /// Should always no contending.
    imms: Arc<std::sync::Mutex<Vec<Imm>>>,
}

struct LocalStateStoreManager {
    /// Should always no contending.
    imms: Arc<std::sync::Mutex<Vec<Imm>>>,
    control_rx: mpsc::UnboundedReceiver<StateStoreControlMessage>,
}

impl GlobalStateStore {
    fn register(&mut self, actor_id: ActorId) -> LocalStateStoreManager {
        let (tx, rx) = mpsc::unbounded_channel();
        self.control_txs.insert(actor_id, tx);
        LocalStateStoreManager {
            imms: Default::default(),
            control_rx: rx,
        }
    }
}

impl LocalStateStoreManager {
    fn state_store(&self) -> LocalStateStore {
        LocalStateStore {
            imms: self.imms.clone(),
        }
    }

    async fn run_control_loop(mut self) {
        while let Some(message) = self.control_rx.recv().await {
            match message {
                StateStoreControlMessage::PruneImm(_) => todo!(),
            }
        }
    }
}

type Node = ();
type Executor = ();
type Message = ();
trait Consumer = Stream<Item = Message> + Unpin;

struct Actor<C> {
    local_state_store_manager: LocalStateStoreManager,
    consumer: C,
}

impl<C: Consumer> Actor<C> {
    fn build_consumer(local_state_store_manager: &LocalStateStoreManager, nodes: Vec<Node>) -> C {
        fn build_node(_node: Node, _local_state_store: LocalStateStore) -> Executor {
            todo!()
        }

        for node in nodes {
            let _ = build_node(node, local_state_store_manager.state_store());
        }
        todo!()
    }

    fn new(id: ActorId, state_store: &mut GlobalStateStore) -> Self {
        let local_state_store_manager = state_store.register(id);
        let consumer = Self::build_consumer(&local_state_store_manager, Default::default());
        Actor {
            local_state_store_manager,
            consumer,
        }
    }

    async fn run(mut self) {
        let state_store_control_loop = self.local_state_store_manager.run_control_loop();
        let streaming_loop = async move {
            while let Some(_message) = self.consumer.next().await {
                todo!();
            }
        };

        tokio::select! {
            biased;
            _ = state_store_control_loop => unreachable!(),
            _ = streaming_loop => return
        }
    }
}

fn main() {}
