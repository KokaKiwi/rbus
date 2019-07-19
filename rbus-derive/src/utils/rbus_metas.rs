use super::Metas;
use syn::parse::Result;

pub trait DBusMetas {
    fn metas(&self) -> &Metas;

    fn find_rbus_module(&self, default: &str) -> Result<syn::Path> {
        let path = self
            .metas()
            .find_meta_value_str("module")?
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(#default))
            .parse()?;

        Ok(path)
    }
}

impl DBusMetas for Metas {
    fn metas(&self) -> &Metas {
        self
    }
}
