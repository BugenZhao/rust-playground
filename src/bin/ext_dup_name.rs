#[easy_ext::ext(MyExt1)]
impl u32 {
    fn foo(&self) {}
}

#[easy_ext::ext(MyExt2)]
impl u64 {
    fn foo(&self) {}
}

fn main() {
    0u32.foo();
    0u64.foo();
}
