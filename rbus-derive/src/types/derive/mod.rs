use crate::utils::{parse_metas, DBusMetas, Metas};
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
        let proxy = ty.metas.find_meta_nested("dbus").find_meta_nested("proxy");
        if proxy.is_empty() {
            match self {
                DeriveData::Enum(ref data) => data.gen_body(ty),
                DeriveData::Struct(ref data) => data.gen_body(ty),
            }
        } else {
            self.gen_proxy_body(ty, proxy)
        }
    }

    fn gen_proxy_body(&self, ty: &DeriveTypeDef, proxy: Metas) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_meta_nested("dbus").find_rbus_module("rbus");
        let proxy_ty: syn::Type = proxy.find_meta_value_parse("ty")?;
        let getter: syn::Ident = proxy.find_meta_value_parse("get")?;
        let setter: syn::Ident = proxy.find_meta_value_parse("set")?;

        let tokens = quote::quote! {
            fn code() -> u8 { <#proxy_ty>::code() }
            fn signature() -> String { <#proxy_ty>::signature() }
            fn alignment() -> u8 { <#proxy_ty>::alignment() }

            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + std::io::Write
            {
                self.#getter().encode(marshaller)
            }
            fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                let value = <#proxy_ty>::decode(marshaller)?;
                Ok(Self::#setter(value))
            }
        };

        Ok(tokens)
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
