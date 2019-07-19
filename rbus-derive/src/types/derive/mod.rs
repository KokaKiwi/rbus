use crate::utils::{parse_named_metas, Metas};
use derive_enum::*;
use derive_struct::*;
pub use fields::*;
use proc_macro2::{Span, TokenStream};
use std::convert::{TryFrom, TryInto};
use syn::parse::{Parse, ParseStream, Result};
pub use variants::*;

mod derive_enum;
mod derive_struct;
mod fields;
mod variants;

#[derive(Debug, Clone)]
pub struct DeriveTypeDef {
    pub span: Span,
    #[allow(dead_code)]
    pub metas: Metas,
    pub name: syn::Ident,
    pub generics: syn::Generics,
    pub data: DeriveData,
}

#[derive(Debug, Clone)]
pub enum DeriveData {
    Enum(DeriveEnum),
    Struct(DeriveStruct),
}

impl DeriveData {
    fn gen_body(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        match self {
            DeriveData::Enum(ref data) => data.gen_body(ty),
            DeriveData::Struct(ref data) => data.gen_body(ty),
        }
    }
}

impl TryFrom<syn::Data> for DeriveData {
    type Error = syn::Error;

    fn try_from(data: syn::Data) -> Result<DeriveData> {
        let data = match data {
            syn::Data::Enum(data) => DeriveData::Enum(data.try_into()?),
            syn::Data::Struct(data) => DeriveData::Struct(data.into()),
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Data not supported for derive",
                ))
            }
        };

        Ok(data)
    }
}

impl DeriveTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;
        let generics = self.gen_generics()?;
        let body = self.data.gen_body(&self)?;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let tokens = quote::quote! {
            impl #impl_generics DBusType for #name #ty_generics #where_clause {
                #body
            }
        };

        Ok(tokens)
    }

    fn gen_generics(&self) -> Result<syn::Generics> {
        let mut generics = self.generics.clone();
        if !generics.params.is_empty() || generics.where_clause.is_some() {
            let mut where_clause = generics.make_where_clause().clone();
            for type_param in generics.type_params() {
                let name = &type_param.ident;
                let predicate = syn::parse_quote! { #name: DBusType };
                where_clause.predicates.push(predicate);
            }

            generics.where_clause = Some(where_clause);
        }

        Ok(generics)
    }
}

impl Parse for DeriveTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::spanned::Spanned;

        let derive_input = input.parse::<syn::DeriveInput>()?;
        let span = derive_input.span();
        let metas = parse_named_metas(&derive_input.attrs, "dbus")?;
        let data = DeriveData::try_from(derive_input.data)?;

        Ok(DeriveTypeDef {
            span,
            metas,
            name: derive_input.ident,
            generics: derive_input.generics,
            data,
        })
    }
}

pub fn derive_type(data: DeriveTypeDef) -> Result<TokenStream> {
    data.impl_type()
}
