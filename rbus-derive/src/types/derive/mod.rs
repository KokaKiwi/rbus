use super::{gen_proxy_methods, ImplGenerator};
use crate::utils::{parse_metas, Metas};
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

type ImplMethod = (&'static str, TokenStream);
type ImplMethods = Vec<ImplMethod>;

#[derive(Debug, Clone)]
pub enum DeriveData {
    Enum(DeriveEnum),
    Struct(DeriveStruct),
}

impl DeriveData {
    fn gen_methods(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> Result<ImplMethods> {
        let dbus = ty.metas.find_meta_nested("dbus");
        let proxy = dbus.find_meta_nested("proxy");
        if proxy.is_empty() {
            match self {
                DeriveData::Enum(ref data) => data.gen_methods(ty, gen),
                DeriveData::Struct(ref data) => data.gen_methods(gen),
            }
        } else {
            gen_proxy_methods(gen, ty.span, proxy)
        }
    }
}

impl TryFrom<syn::Data> for DeriveData {
    type Error = syn::Error;

    fn try_from(data: syn::Data) -> Result<DeriveData> {
        let data = match data {
            syn::Data::Enum(data) => DeriveData::Enum(data.try_into()?),
            syn::Data::Struct(data) => DeriveData::Struct(data.try_into()?),
            _ => return Err(syn::Error::new(Span::call_site(), "Data not supported for derive")),
        };

        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct DeriveTypeDef {
    pub span: Span,
    pub metas: Metas,
    pub name: syn::Ident,
    pub generics: syn::Generics,
    pub data: DeriveData,
}

impl DeriveTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let generics = self.gen_generics()?;

        let mut gen = ImplGenerator::new_ident(self.span, self.metas.clone(), Some(generics), self.name.clone());

        let methods = self.data.gen_methods(&self, &gen)?;
        for (name, method) in methods.into_iter() {
            gen.add_method(name, method);
        }

        gen.gen_impl()
    }

    fn gen_generics(&self) -> Result<syn::Generics> {
        let mut generics = self.generics.clone();
        if !generics.params.is_empty() || generics.where_clause.is_some() {
            let where_clause = generics.make_where_clause();
            for type_param in self.generics.type_params() {
                let name = &type_param.ident;
                let predicate = syn::parse_quote! { #name: DBusType };
                where_clause.predicates.push(predicate);
            }
        }

        Ok(generics)
    }
}

impl Parse for DeriveTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::spanned::Spanned;

        let derive_input = input.parse::<syn::DeriveInput>()?;
        let span = derive_input.span();
        let metas = parse_metas(&derive_input.attrs)?;
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
