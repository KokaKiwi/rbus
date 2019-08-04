use super::{MetaValue, Metas};
use syn::spanned::Spanned;

pub trait DBusMetas {
    fn metas(&self) -> &Metas;

    fn find_rbus_module<T: AsRef<str>>(&self, default: T) -> MetaValue {
        let default = default.as_ref();

        match self.metas().find_meta_value("module").cloned() {
            Some(meta) => match meta {
                MetaValue::Name(..) => meta,
                MetaValue::Lit(syn::Lit::Str(lit)) => lit.parse().unwrap(),
                _ => MetaValue::Name(syn::Ident::new(default, self.metas().span()).into()),
            },
            _ => MetaValue::Name(syn::Ident::new(default, self.metas().span()).into()),
        }
    }
}

impl DBusMetas for Metas {
    fn metas(&self) -> &Metas {
        self
    }
}
