use super::Fields;
use crate::utils::{parse_metas, Metas};
use proc_macro2::Span;
use std::{convert::TryFrom, ops::Deref};
use syn::{spanned::Spanned, Result};

#[derive(Debug, Clone)]
pub struct Variants(Vec<Variant>);

impl Variants {
    /// Return true if all variant fields are named or unnamed and false if
    /// otherwise
    pub fn is_complete(&self) -> bool {
        self.0.iter().all(|variant| !variant.fields.is_unit())
    }

    /// Return true if all variant fields are unit and false otherwise
    pub fn is_unit(&self) -> bool {
        self.0.iter().all(|variant| variant.fields.is_unit())
    }

    pub fn values(&self) -> Vec<syn::LitInt> {
        let mut values = Vec::with_capacity(self.0.len());
        let mut current = 0;
        for variant in self.0.iter() {
            let value = match variant.value {
                Some(syn::Expr::Lit(ref expr)) => match expr.lit {
                    syn::Lit::Int(ref lit) => lit.value(),
                    syn::Lit::Byte(ref lit) => lit.value() as u64,
                    _ => current,
                },
                _ => current,
            };
            let value = syn::LitInt::new(value, syn::IntSuffix::None, variant.value.span());
            current = value.value() + 1;
            values.push(value);
        }
        values
    }

    pub fn indexes(&self) -> syn::Result<Vec<syn::LitInt>> {
        let mut indexes = Vec::with_capacity(self.0.len());
        let mut current = 0;
        for variant in self.0.iter() {
            let value = match variant.index()? {
                Some(syn::Lit::Int(ref lit)) => lit.value(),
                Some(syn::Lit::Byte(ref lit)) => lit.value() as u64,
                _ => current,
            };
            let value = syn::LitInt::new(value, syn::IntSuffix::None, variant.value.span());
            current = value.value() + 1;
            indexes.push(value);
        }
        Ok(indexes)
    }
}

impl Deref for Variants {
    type Target = [Variant];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<syn::Variant>> for Variants {
    type Error = syn::Error;

    fn try_from(variants: Vec<syn::Variant>) -> Result<Self> {
        let variants = variants
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Variants(variants))
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub span: Span,
    pub metas: Metas,
    pub name: syn::Ident,
    pub fields: Fields,
    pub value: Option<syn::Expr>,
}

impl Variant {
    pub fn index(&self) -> Result<Option<syn::Lit>> {
        self.metas
            .find_meta_nested("dbus")
            .find_meta_value("index")
            .map(|value| value.cloned())
    }
}

impl TryFrom<syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(variant: syn::Variant) -> Result<Self> {
        let span = variant.span();
        let metas = parse_metas(variant.attrs)?;
        let fields = Fields::try_from(variant.fields)?;
        let value = variant.discriminant.map(|(_, value)| value);

        Ok(Variant {
            span,
            metas,
            name: variant.ident,
            fields,
            value,
        })
    }
}
