use std::{num::NonZeroU32, time::Duration};

use governor::{Quota, RateLimiter};

fn main() {}

#[tokio::test]
async fn test_basic() {
    let rl = RateLimiter::direct(
        // max_burst: 1000
        // token renew every 1/1000 seconds
        // can also customize by `with_period` and `allow_burst`
        Quota::per_second(NonZeroU32::new(1000).unwrap()),
    );

    rl.until_n_ready(NonZeroU32::new(1000).unwrap())
        .await
        .unwrap();

    rl.check_n(NonZeroU32::new(1001).unwrap()).unwrap_err(); // insufficient capacity

    rl.check_n(NonZeroU32::new(1000).unwrap())
        .unwrap() // sufficient capacity
        .unwrap_err(); // would block

    tokio::time::sleep(Duration::from_millis(500)).await;

    rl.check_n(NonZeroU32::new(500).unwrap()).unwrap().unwrap(); // capacity available
}
