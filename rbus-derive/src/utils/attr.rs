use std::iter::FromIterator;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub struct Metas(syn::AttributeArgs);

#[allow(dead_code)]
impl Metas {
    pub fn values(&self) -> Vec<&syn::Lit> {
        self.0
            .iter()
            .filter_map(|nested_meta| match nested_meta {
                syn::NestedMeta::Literal(lit) => Some(lit),
                _ => None,
            })
            .collect()
    }

    pub fn find_meta<'a>(&'a self, name: &str) -> Option<&'a syn::Meta> {
        self.0.iter().find_map(|nested_meta| match nested_meta {
            syn::NestedMeta::Meta(meta) if meta.name() == name => Some(meta),
            _ => None,
        })
    }

    pub fn find_metas<'a>(&'a self, name: &str) -> Vec<&'a syn::Meta> {
        self.0
            .iter()
            .filter_map(|nested_meta| match nested_meta {
                syn::NestedMeta::Meta(meta) if meta.name() == name => Some(meta),
                _ => None,
            })
            .collect()
    }

    pub fn has_meta_word(&self, name: &str) -> Result<bool> {
        match self.find_meta(name) {
            Some(syn::Meta::Word(_)) => Ok(true),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected ident: `{}`", name),
            )),
            None => Ok(false),
        }
    }

    pub fn find_meta_nested(&self, name: &str) -> Metas {
        self.find_metas(name)
            .iter()
            .filter_map(|meta| match meta {
                syn::Meta::List(list) => {
                    Some(list.nested.iter().map(Clone::clone).collect::<Vec<_>>())
                }
                _ => None,
            })
            .flatten()
            .collect()
    }

    pub fn find_meta_value(&self, name: &str) -> Result<Option<&syn::Lit>> {
        match self.find_meta(name) {
            Some(syn::Meta::NameValue(meta)) => Ok(Some(&meta.lit)),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected named value: `{}`", name),
            )),
            None => Ok(None),
        }
    }

    pub fn find_meta_value_str(&self, name: &str) -> Result<Option<&syn::LitStr>> {
        self.find_meta_value(name).and_then(|value| match value {
            Some(syn::Lit::Str(lit)) => Ok(Some(lit)),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected string value: `{}`", name),
            )),
            None => Ok(None),
        })
    }

    pub fn find_meta_value_int(&self, name: &str) -> Result<Option<&syn::LitInt>> {
        self.find_meta_value(name).and_then(|value| match value {
            Some(syn::Lit::Int(lit)) => Ok(Some(lit)),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected int value: `{}`", name),
            )),
            None => Ok(None),
        })
    }

    pub fn parse_named(input: ParseStream, name: &str) -> Result<Self> {
        input
            .parse()
            .map(|metas: Metas| metas.find_meta_nested(name))
    }
}

impl std::ops::Deref for Metas {
    type Target = [syn::NestedMeta];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<syn::NestedMeta> for Metas {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = syn::NestedMeta>,
    {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl From<syn::AttributeArgs> for Metas {
    fn from(value: Vec<syn::NestedMeta>) -> Metas {
        Metas(value)
    }
}

impl From<Option<syn::AttributeArgs>> for Metas {
    fn from(value: Option<Vec<syn::NestedMeta>>) -> Metas {
        Metas(value.unwrap_or_else(Vec::new))
    }
}

impl Parse for Metas {
    fn parse(input: ParseStream) -> Result<Self> {
        input
            .call(syn::Attribute::parse_outer)
            .and_then(parse_metas)
    }
}

pub fn parse_named_metas<T>(attrs: T, name: &str) -> Result<Metas>
where
    T: AsRef<[syn::Attribute]>,
{
    Ok(parse_metas(attrs)?.find_meta_nested(name))
}

pub fn parse_metas<T>(attrs: T) -> Result<Metas>
where
    T: AsRef<[syn::Attribute]>,
{
    attrs
        .as_ref()
        .iter()
        .map(syn::Attribute::parse_meta)
        .map(|meta| meta.map(syn::NestedMeta::Meta))
        .collect()
}
