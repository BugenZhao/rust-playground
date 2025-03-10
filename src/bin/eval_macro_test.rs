use eval_macro::eval;

// Isn't it simply codegen?
eval! {
    let types = ["Number", "Boolean", "String"];

    for ty in types {
        output! {
            #[doc = "{ty}"]
            /// {ty}
            struct {{ty}};
        }
    }
}

fn main() {}
