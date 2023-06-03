//! How can I drop something without moving it?

use std::pin::Pin;

static mut DROPPED_ADDR: *const Array = std::ptr::null();

struct Array([i32; 5]);

impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            DROPPED_ADDR = self;
        }
        println!("array dropped at {:p}", &self.0 as *const _);
    }
}

fn _1_assign_ok() {
    let mut array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    let array_ref = &mut array;
    *array_ref = None;
    assert_eq!(ptr, unsafe { DROPPED_ADDR });
}

fn _2_option_insert_ok() {
    let mut array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    let _new = array.insert(Array([114514, 2, 3, 4, 5]));
    assert_eq!(ptr, unsafe { DROPPED_ADDR });
}

fn _3_pin_set_ok() {
    let mut array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    let mut pin_array = Pin::new(&mut array);
    pin_array.set(None);
    assert_eq!(ptr, unsafe { DROPPED_ADDR });
}

fn _4_mem_replace_bad() {
    let mut array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    let _ = std::mem::replace(&mut array, None);
    assert_ne!(ptr, unsafe { DROPPED_ADDR });
}

fn _5_option_take_bad() {
    let mut array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    array.take(); // same as `mem::replace`
    assert_ne!(ptr, unsafe { DROPPED_ADDR });
}

fn _6_explicit_drop_bad() {
    let array = Some(Array([114514, 2, 3, 4, 5]));
    let ptr = array.as_ref().unwrap() as *const _;
    println!("array: {:p}", ptr);

    drop(array);
    assert_ne!(ptr, unsafe { DROPPED_ADDR });
}

fn main() {
    _1_assign_ok();
    _2_option_insert_ok();
    _3_pin_set_ok();
    _4_mem_replace_bad();
    _5_option_take_bad();
}
