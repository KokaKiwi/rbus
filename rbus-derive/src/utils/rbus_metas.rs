use super::Metas;
use syn::parse::Result;

pub trait DBusMetas {
    fn metas(&self) -> &Metas;

    fn find_rbus_module(&self) -> Result<syn::Path> {
        let path = self
            .metas()
            .find_meta_value_str("module")?
            .map(|lit| lit.parse())
            .transpose()?
            .unwrap_or_else(|| syn::parse_quote!(rbus));

        Ok(path)
    }
}

impl DBusMetas for Metas {
    fn metas(&self) -> &Metas {
        self
    }
}
