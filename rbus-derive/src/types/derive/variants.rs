use super::Fields;
use crate::utils::{parse_named_metas, Metas};
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct Variants(Vec<Variant>);

impl Deref for Variants {
    type Target = [Variant];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<syn::Variant>> for Variants {
    type Error = syn::Error;

    fn try_from(variants: Vec<syn::Variant>) -> Result<Self, Self::Error> {
        let variants = variants
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Variants(variants))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub metas: Metas,
    pub name: syn::Ident,
    pub fields: Fields,
}

impl TryFrom<syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(variant: syn::Variant) -> Result<Self, Self::Error> {
        let metas = parse_named_metas(variant.attrs, "dbus")?;
        let fields = Fields::from(variant.fields);

        Ok(Variant {
            metas,
            name: variant.ident,
            fields,
        })
    }
}
