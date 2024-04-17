use std::fs::File;

fn main() {
    let file = File::create(std::env::temp_dir().join("my_test_singleton_proc")).unwrap();
    let mut lock = fd_lock::RwLock::new(file);

    println!("acquiring lock...");
    let _guard = lock.write().unwrap();
    println!("lock acquired");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
