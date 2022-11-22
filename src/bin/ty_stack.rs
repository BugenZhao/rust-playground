use std::cell::RefCell;
use std::time::Duration;

tokio::task_local! {
    static TRACE: TraceContext;
}

#[derive(Debug, Clone)]
pub struct TraceContext {
    // SAFETY: We never export &stack or &mut stack, and no await point in all public methods.
    stack: RefCell<Vec<&'static str>>,
}

pub struct TraceGuard;

impl TraceContext {
    #[must_use]
    pub fn enter(&self, context: &'static str) -> TraceGuard {
        self.stack.borrow_mut().push(context);
        TraceGuard
    }

    pub fn collect(&self) -> Vec<&'static str> {
        self.stack.borrow().clone()
    }
}

impl Drop for TraceGuard {
    fn drop(&mut self) {
        TRACE.with(|context| {
            context.stack.borrow_mut().pop();
        });
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    TRACE
        .scope(
            TraceContext {
                stack: RefCell::new(vec![]),
            },
            async move {
                // The monitor loop, we can check runtime flag here and report the stack.
                let monitor = async move {
                    println!("Start monitor!");
                    let mut i = 8;
                    while i > 0 {
                        i -= 1;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        println!("What are you doing?");
                        let stack = TRACE.with(|context| context.collect());
                        println!("{:?}", stack);
                    }
                };
                let task = outer();
                tokio::join!(monitor, task);
            },
        )
        .await;
}

async fn outer() {
    let _guard = TRACE.with(|context| context.enter("outer"));

    tokio::spawn(inner()).await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;
}

async fn inner() {
    let _guard = TRACE.with(|context| context.enter("inner"));

    let mut i = 5;
    while i > 0 {
        i -= 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("I'm touching fish!");
    }
}
