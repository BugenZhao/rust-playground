use std::{any::TypeId, fmt::Display, mem::transmute};

trait MyToString {
    fn my_to_string(&self) -> String;
}

impl<T: Display> MyToString for T {
    fn my_to_string(&self) -> String {
        if let Ok(s) = {
            #[allow(unused_imports)]
            use castaway::internal::*;
            let value = self;
            let src_token = CastToken::of_val(&value);
            let dest_token = CastToken::<&String>::of();
            let result: ::core::result::Result<&String, _> =
                (&&&&&&&(src_token, dest_token)).try_cast(value); // 7"&"s are used for specialization on `try_cast` method.
                                                                  // See https://github.com/drmingdrmer/tips/blob/b824689041100db65d232f919c250469c8c72c2e/tips/rust/Rust%20%E5%88%A9%E7%94%A8%20autoref%20%E5%AE%9E%E7%8E%B0%20specialization.md
                                                                  // It's like comparing the `TypeId` of two types.
                                                                  // However, by dispatching `try_cast` and implement type id manually,
                                                                  // it gets rid of the `'static` bound on `TypeId::of`.
                                                                  // Also, it helps us to do the casting safely.
            result
        }
        // castaway::cast!(self, &String)
        {
            println!("specialized");
            s.to_owned()
        } else {
            println!("fallback");
            format!("{}", self)
        }
    }
}

fn main() {
    let a = "string".to_owned();
    let a = a.my_to_string();
    println!("{}", a);
}
