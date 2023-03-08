use itertools::Itertools;
use syn::{parse_quote, DeriveInput};

pub struct A {
    /// Inner doc
    a: i32,
}

fn main() {
    let a: DeriveInput = parse_quote! {
        /// This is a doc comment.
        ///
        /// Multiple lines.
        pub struct A {
            /// Inner doc
            a: i32,
        }
    };

    for attr in a.attrs.iter().filter(|a| a.path.is_ident("doc")) {
        if let Some(meta) = attr.parse_meta().ok() {
            if let syn::Meta::NameValue(nv) = meta {
                if let syn::Lit::Str(lit_str) = nv.lit {
                    println!("{}", lit_str.value());
                }
            }
        }
    }

    let attrs = a
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("doc"))
        .collect_vec();

    let result = quote::quote! {
        #(#attrs)*
        pub type MyAlias = A;
    };

    println!("{}", result.to_string());
}

#[doc = r" This is a doc comment."]
#[doc = r""]
#[doc = r" Multiple lines."]
pub type MyAlias = A;
