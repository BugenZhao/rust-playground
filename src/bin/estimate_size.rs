#[easy_ext::ext]
impl<T: ?Sized> T {
    fn my_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

fn main() {
    let v = vec![0u8; 10].into_boxed_slice();
    dbg!(v.my_size());
    dbg!((*v).my_size());
}
