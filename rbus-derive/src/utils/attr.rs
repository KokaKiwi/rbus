use syn::parse::Result;
use syn::spanned::Spanned;

pub struct Metas(Option<Vec<syn::Meta>>);

impl Metas {
    pub fn find_meta_meta<'a>(&'a self, name: &str) -> Option<&'a syn::Meta> {
        match &self.0 {
            Some(metas) => metas.iter().find(|meta| meta.name() == name),
            None => None
        }
    }

    pub fn find_meta_word(&self, name: &str) -> Result<bool> {
        match self.find_meta_meta(name) {
            Some(syn::Meta::Word(_)) => Ok(true),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected ident: `{}`", name))),
            _ => Ok(false),
        }
    }

    pub fn find_meta_value<'a>(&'a self, name: &str) -> Result<Option<&'a syn::Lit>> {
        match self.find_meta_meta(name) {
            Some(syn::Meta::NameValue(meta)) => Ok(Some(&meta.lit)),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected named value: `{}`", name))),
            _ => Ok(None)
        }
    }

    pub fn find_meta_value_str(&self, name: &str) -> Result<Option<String>> {
        self.find_meta_value(name).and_then(|value| match value {
            Some(syn::Lit::Str(lit)) => Ok(Some(lit.value())),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected string value: `{}`", name))),
            _ => Ok(None),
        })
    }
}

impl From<Vec<syn::Meta>> for Metas {
    fn from(value: Vec<syn::Meta>) -> Metas {
        Metas(Some(value))
    }
}

impl From<Option<Vec<syn::Meta>>> for Metas {
    fn from(value: Option<Vec<syn::Meta>>) -> Metas {
        Metas(value)
    }
}

pub fn parse_named_metas(attrs: Vec<syn::Attribute>, name: &str) -> Metas {
    attrs.into_iter()
        .find_map(|attr| attr.parse_meta().ok())
        .filter(|meta| meta.name() == name)
        .and_then(|meta| if let syn::Meta::List(metas) = meta { Some(metas) } else { None })
        .map(|metalist| metalist.nested.iter()
             .filter_map(|item| if let syn::NestedMeta::Meta(meta) = item { Some(meta) } else { None })
             .map(Clone::clone)
             .collect()).into()
}

pub fn parse_metas(attrs: Vec<syn::Attribute>) -> Metas {
    attrs.into_iter()
        .find_map(|attr| attr.parse_meta().ok())
        .and_then(|meta| if let syn::Meta::List(metas) = meta { Some(metas) } else { None })
        .map(|metalist| metalist.nested.iter()
             .filter_map(|item| if let syn::NestedMeta::Meta(meta) = item { Some(meta) } else { None })
             .map(Clone::clone)
             .collect()).into()
}
