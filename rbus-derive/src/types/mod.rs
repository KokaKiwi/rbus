use crate::utils::Metas;
pub use basic::impl_basic_type;
pub use derive::derive_type;
use method::Methods;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

mod basic;
mod derive;
mod method;

fn parse_dbus_metas(input: ParseStream) -> Result<Metas> {
    Metas::parse_named(input, "dbus")
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    metas: Metas,
    ty: syn::Type,
    code: syn::LitChar,
    methods: Methods,
}

impl TypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let ty = &self.ty;

        let methods: Vec<(String, _)> = vec![
            ("code".into(), self.gen_code_method()?),
            ("signature".into(), self.gen_signature_method()?),
            ("alignment".into(), self.gen_alignment_method()?),
        ];

        let impl_methods = self
            .methods
            .iter()
            .map(|method| method.gen_dbus_method(&self.metas))
            .collect::<Result<Vec<_>>>()?;

        let methods: HashMap<_, _> = methods.into_iter().chain(impl_methods).collect();
        let methods: Vec<_> = methods.values().collect();

        let dbus_type_impl = quote::quote! {
            impl crate::types::DBusType for #ty {
                #(#methods)*
            }
        };

        let basic_type_impl = if let Ok(true) = self.metas.has_meta_word("basic") {
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

        Ok(tokens)
    }

    fn gen_code_method(&self) -> Result<TokenStream> {
        let code = &self.code;

        Ok(quote::quote! {
            fn code() -> u8 { #code as u8 }
        })
    }

    fn gen_signature_method(&self) -> Result<TokenStream> {
        let signature = format!("{}", self.code.value());

        Ok(quote::quote! {
            fn signature() -> String { #signature.into() }
        })
    }

    fn gen_alignment_method(&self) -> Result<TokenStream> {
        let alignment = match self.metas.find_meta_value("align")? {
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

        Ok(quote::quote! {
            fn alignment() -> u8 { #alignment }
        })
    }
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let metas = input.call(parse_dbus_metas)?;

        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;
        let methods = input.call(method::parse_methods)?;

        Ok(TypeDef {
            metas,
            ty,
            code,
            methods,
        })
    }
}

pub fn impl_type(data: TypeDef) -> Result<TokenStream> {
    data.impl_type()
}
