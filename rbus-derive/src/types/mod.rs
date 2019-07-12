use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
pub use basic::impl_basic_type;
pub use derive::derive_type;

mod basic;
mod derive;

pub struct TypeDef {
    ty: syn::Type,
    code: syn::LitChar,
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;

        Ok(TypeDef { ty, code })
    }
}

pub fn impl_type(data: TypeDef) -> TokenStream {
    let TypeDef { ty, code } = data;

    let signature = format!("{}", code.value());

    let tokens = quote::quote! {
        impl crate::types::DBusType for #ty {
            fn code() -> u8 { #code as u8 }
            fn signature() -> String { #signature.into() }
        }
    };

    tokens.into()
}
