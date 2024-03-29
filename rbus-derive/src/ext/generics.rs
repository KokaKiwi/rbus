use syn::{punctuated::Punctuated, Generics};

pub trait GenericsExt {
    fn empty() -> Generics {
        Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        }
    }
}

impl GenericsExt for Generics {}
