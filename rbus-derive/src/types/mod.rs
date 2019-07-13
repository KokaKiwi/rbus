use crate::utils::attr::{parse_metas, Metas};
pub use basic::impl_basic_type;
pub use derive::derive_type;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};

mod basic;
mod derive;

pub struct TypeDef {
    ty: syn::Type,
    code: syn::LitChar,
    metas: Metas,
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let metas = parse_metas(attrs);

        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;

        Ok(TypeDef { ty, code, metas })
    }
}

pub fn impl_type(data: TypeDef) -> TokenStream {
    let TypeDef {
        ref ty, ref code, ..
    } = data;

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
