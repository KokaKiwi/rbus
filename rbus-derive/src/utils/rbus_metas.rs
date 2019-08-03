use super::{MetaValue, Metas};
use syn::spanned::Spanned;

pub trait DBusMetas {
    fn metas(&self) -> &Metas;

    fn find_rbus_module<T: AsRef<str>>(&self, default: T) -> MetaValue {
        match self.metas().find_meta_value("module").cloned() {
            Some(meta) => match meta {
                MetaValue::Word(..) => meta,
                MetaValue::Path(..) => meta,
                MetaValue::Lit(syn::Lit::Str(lit)) => lit.parse().unwrap(),
                _ => MetaValue::Word(syn::Ident::new("rbus", self.metas().span())),
            },
            _ => MetaValue::Word(syn::Ident::new(default.as_ref(), self.metas().span())),
        }
    }
}

impl DBusMetas for Metas {
    fn metas(&self) -> &Metas {
        self
    }
}
