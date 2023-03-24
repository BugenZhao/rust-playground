#![feature(panic_update_hook)]

use std::panic::catch_unwind;

use futures::FutureExt;

#[tokio::main]
async fn main() {
    std::panic::update_hook(|d, info| {
        println!("before hook");

        d(info);

        println!("after hook");

        std::process::abort();
    });

    let e = tokio::task::spawn(async { panic!() })
        .catch_unwind()
        .await
        .unwrap()
        .unwrap_err()
        .into_panic();

    // let e = catch_unwind(|| panic!()).unwrap_err();

    println!("caught panic: {:?}", e);
}
