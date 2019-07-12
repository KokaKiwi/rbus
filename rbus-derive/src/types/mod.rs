use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
pub use basic::impl_basic_type;
pub use derive::derive_type;

mod basic;
mod derive;

pub struct TypeDef {
    ty: syn::Type,
    code: syn::LitChar,
    metas: Option<syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>>,
}

impl TypeDef {
    fn find_meta(&self, name: &str) -> Option<&syn::Meta> {
        match self.metas {
            Some(ref metas) => metas.iter().find(|meta| meta.name() == name),
            None => None
        }
    }

    fn has_meta_word(&self, name: &str) -> bool {
        match self.find_meta(name) {
            Some(syn::Meta::Word(_)) => true,
            _ => false,
        }
    }

    fn get_meta_list(&self, name: &str) -> Option<Vec<&syn::NestedMeta>> {
        match self.find_meta(name) {
            Some(syn::Meta::List(list)) => Some(list.nested.iter().collect()),
            _ => None,
        }
    }

    fn get_meta_value(&self, name: &str) -> Option<&syn::Lit> {
        match self.find_meta(name) {
            Some(syn::Meta::NameValue(meta)) => Some(&meta.lit),
            _ => None,
        }
    }
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;

        let metas = if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            Some(content.parse_terminated(syn::Meta::parse)?)
        } else {
            None
        };

    Ok(TypeDef { ty, code, metas })
}
}

pub fn impl_type(data: TypeDef) -> TokenStream {
    let TypeDef { ref ty, ref code, .. } = data;

    let signature = format!("{}", code.value());

    let dbus_type_impl = quote::quote! {
        impl crate::types::DBusType for #ty {
            fn code() -> u8 { #code as u8 }
            fn signature() -> String { #signature.into() }
        }
    };

    let basic_type_impl = if data.has_meta_word("basic") {
        quote::quote! {
            impl crate::types::DBusBasicType for #ty {}
        }
    } else {
        quote::quote!()
    };

    let tokens = quote::quote! {
        #dbus_type_impl
        #basic_type_impl
    };

    tokens.into()
}
