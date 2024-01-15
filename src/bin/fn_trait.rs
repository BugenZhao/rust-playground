#![feature(unboxed_closures)]
#![feature(fn_traits)]

struct Call;

impl FnOnce<(i32,)> for Call {
    type Output = i32;

    extern "rust-call" fn call_once(self, args: (i32,)) -> Self::Output {
        Fn::call(&self, args)
    }
}

impl FnMut<(i32,)> for Call {
    extern "rust-call" fn call_mut(&mut self, args: (i32,)) -> Self::Output {
        Fn::call(self, args)
    }
}

impl Fn<(i32,)> for Call {
    extern "rust-call" fn call(&self, (n,): (i32,)) -> Self::Output {
        println!("hello, {n}!");
        n
    }
}

fn main() {
    let call = Call;
    let _ = call(5);
    let _ = call(42);
}
