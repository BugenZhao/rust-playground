use prost::Message;

#[derive(Clone, PartialEq, ::prost::Message)]
struct Inner {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
struct Outer {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<Inner>,

    #[prost(map = "uint64, message", tag = "2")]
    pub map: ::std::collections::HashMap<u64, Inner>,
}

fn main() {
    let o = Outer::decode([].as_slice()).unwrap();
    println!("{:?}", o); // Outer { inner: None, map: {} }
}
