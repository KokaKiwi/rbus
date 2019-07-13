use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};

pub struct BasicTypeDef {
    attrs: Vec<syn::Attribute>,
    ty: syn::Type,
    code: syn::LitChar,
}

impl Parse for BasicTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let code = input.parse()?;

        Ok(BasicTypeDef { attrs, ty, code })
    }
}

pub fn impl_basic_type(data: BasicTypeDef) -> TokenStream {
    let BasicTypeDef { attrs, ty, code } = data;

    let tokens = quote::quote! {
        rbus_derive::impl_type! { #(#attrs)* #[basic] #ty: #code }
    };

    tokens.into()
}
