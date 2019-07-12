use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};

pub struct BasicTypeDef {
    ty: syn::Type,
    code: syn::LitChar,
}

impl Parse for BasicTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let code = input.parse()?;

        Ok(BasicTypeDef { ty, code })
    }
}

pub fn impl_basic_type(data: BasicTypeDef) -> TokenStream {
    let BasicTypeDef { ty, code } = data;

    let tokens = quote::quote! {
        rbus_derive::impl_type! { #ty: #code [basic] }
    };

    tokens.into()
}
