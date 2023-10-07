use std::ffi::{CStr, CString};

fn main() {
    let a = CString::new("Hello").unwrap();

    let b: &'static CStr = unsafe { &*(a.as_c_str() as *const _) };

    println!("{:?}", b);
}
