use syn::parse::Result;
use syn::punctuated::Pair;
use syn::spanned::Spanned;

pub struct Metas(Option<Vec<syn::NestedMeta>>);

impl Metas {
    pub fn values(&self) -> Option<Vec<&syn::Lit>> {
        match &self.0 {
            Some(metas) => {
                let lits = metas
                    .iter()
                    .filter_map(|nested_meta| match nested_meta {
                        syn::NestedMeta::Literal(lit) => Some(lit),
                        _ => None,
                    })
                    .collect();
                Some(lits)
            }
            None => None,
        }
    }

    pub fn find_meta<'a>(&'a self, name: &str) -> Option<&'a syn::Meta> {
        match &self.0 {
            Some(metas) => metas.iter().find_map(|nested_meta| match nested_meta {
                syn::NestedMeta::Meta(meta) if meta.name() == name => Some(meta),
                _ => None,
            }),
            None => None,
        }
    }

    pub fn find_meta_word(&self, name: &str) -> Result<bool> {
        match self.find_meta(name) {
            Some(syn::Meta::Word(_)) => Ok(true),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected ident: `{}`", name),
            )),
            None => Ok(false),
        }
    }

    pub fn find_meta_value<'a>(&'a self, name: &str) -> Result<Option<&'a syn::Lit>> {
        match self.find_meta(name) {
            Some(syn::Meta::NameValue(meta)) => Ok(Some(&meta.lit)),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected named value: `{}`", name),
            )),
            None => Ok(None),
        }
    }

    pub fn find_meta_value_str(&self, name: &str) -> Result<Option<String>> {
        self.find_meta_value(name).and_then(|value| match value {
            Some(syn::Lit::Str(lit)) => Ok(Some(lit.value())),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected string value: `{}`", name),
            )),
            None => Ok(None),
        })
    }

    pub fn find_meta_value_int(&self, name: &str) -> Result<Option<u64>> {
        self.find_meta_value(name).and_then(|value| match value {
            Some(syn::Lit::Int(lit)) => Ok(Some(lit.value())),
            Some(meta) => Err(syn::Error::new(
                meta.span(),
                format!("Expected int value: `{}`", name),
            )),
            None => Ok(None),
        })
    }
}

impl From<Vec<syn::NestedMeta>> for Metas {
    fn from(value: Vec<syn::NestedMeta>) -> Metas {
        Metas(Some(value))
    }
}

impl From<Option<Vec<syn::NestedMeta>>> for Metas {
    fn from(value: Option<Vec<syn::NestedMeta>>) -> Metas {
        Metas(value)
    }
}

pub fn parse_named_metas(attrs: Vec<syn::Attribute>, name: &str) -> Metas {
    attrs
        .into_iter()
        .filter_map(|attr| attr.parse_meta().ok()) // TODO: Don't silently ignore errors.
        .find(|meta| meta.name() == name)
        .and_then(|meta| {
            if let syn::Meta::List(metas) = meta {
                Some(metas)
            } else {
                None
            }
        })
        .map(|metas| metas.nested.into_pairs().map(Pair::into_value).collect())
        .into()
}

pub fn parse_metas(attrs: Vec<syn::Attribute>) -> Metas {
    attrs
        .into_iter()
        .filter_map(|attr| attr.parse_meta().ok()) // TODO: Don't silently ignore errors.
        .map(syn::NestedMeta::Meta)
        .collect::<Vec<syn::NestedMeta>>()
        .into()
}
