use crate::utils::attr::Metas;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
pub use basic::impl_basic_type;
pub use derive::derive_type;

mod basic;
mod derive;

pub struct TypeDef {
    ty: syn::Type,
    code: syn::LitChar,
    metas: Metas,
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;

        let metas = if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            Some(content.parse_terminated::<_, syn::Token![,]>(syn::Meta::parse)?
                 .iter().map(Clone::clone).collect())
        } else {
            None
        }.into();

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

    let basic_type_impl = if let Ok(true) = data.metas.find_meta_word("basic") {
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
