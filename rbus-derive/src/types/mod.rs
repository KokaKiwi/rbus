use crate::utils::attr::Metas;
pub use basic::impl_basic_type;
pub use derive::derive_type;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

mod basic;
mod derive;

fn parse_dbus_metas(input: ParseStream) -> Result<Metas> {
    Metas::parse_named(input, "dbus")
}

pub struct TypeDef {
    ty: syn::Type,
    code: syn::LitChar,
    metas: Metas,
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let metas = input.call(parse_dbus_metas)?;

        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;

        Ok(TypeDef { ty, code, metas })
    }
}

pub fn impl_type(data: TypeDef) -> Result<TokenStream> {
    let TypeDef { ty, code, metas } = data;

    let signature = format!("{}", code.value());
    let alignment = match metas.find_meta_value("align")? {
        Some(syn::Lit::Int(lit)) => quote::quote!(#lit as u8),
        Some(syn::Lit::Str(lit)) if lit.value() == "size" => {
            quote::quote!(std::mem::size_of::<Self>() as u8)
        }
        Some(lit) => {
            return Err(syn::Error::new(
                lit.span(),
                "Bad align value, only integer or \"size\"",
            ))
        }
        None => quote::quote!(1),
    };

    let dbus_type_impl = quote::quote! {
        impl crate::types::DBusType for #ty {
            fn code() -> u8 { #code as u8 }
            fn signature() -> String { #signature.into() }
            fn alignment() -> u8 { #alignment }
        }
    };

    let basic_type_impl = if let Ok(true) = metas.find_meta_word("basic") {
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

    Ok(tokens.into())
}
