use super::Metas;
use proc_macro2::Span;
use syn::ext::IdentExt;

pub trait DBusMetas {
    fn metas(&self) -> &Metas;

    fn find_rbus_module<T: AsRef<str>>(&self, default: T) -> syn::Ident {
        self.metas()
            .find_meta_value_parse_with("module", syn::Ident::parse_any)
            .unwrap_or_else(|_| syn::Ident::new(default.as_ref(), Span::call_site()))
    }
}

impl DBusMetas for Metas {
    fn metas(&self) -> &Metas {
        self
    }
}
