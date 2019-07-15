use crate::utils::attr::{parse_named_metas, Metas};
use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream, Result};

pub struct DeriveTypeDef {
    span: Span,
    metas: Metas,
    name: syn::Ident,
    generics: syn::Generics,
    data: syn::Data,
}

impl DeriveTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;
        let generics = self.gen_generics()?;
        let body = self.gen_body()?;

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

    fn gen_body(&self) -> Result<TokenStream> {
        match self.data {
            syn::Data::Struct(ref data) => self.gen_struct_body(data),
            syn::Data::Enum(ref data) => self.gen_enum_body(data),
            syn::Data::Union(_) => Err(syn::Error::new(self.span, "Union are not supported")),
        }
    }

    fn gen_struct_body(&self, data: &syn::DataStruct) -> Result<TokenStream> {
        let field_types = match data.fields {
            syn::Fields::Named(ref fields) => {
                fields.named.iter().map(|field| field.ty.clone()).collect()
            }
            syn::Fields::Unnamed(ref fields) => fields
                .unnamed
                .iter()
                .map(|field| field.ty.clone())
                .collect(),
            syn::Fields::Unit => Vec::new(),
        };

        let signature_format_str = format!("({})", "{}".repeat(field_types.len()));

        let body = quote::quote! {
            fn code() -> u8 { b'r' }
            fn signature() -> String {
                format!(#signature_format_str, #(<#field_types>::signature()),*)
            }
            fn alignment() -> u8 { 8 }
        };

        Ok(body)
    }

    fn gen_enum_body(&self, _data: &syn::DataEnum) -> Result<TokenStream> {
        let body = quote::quote! {
            fn code() -> u8 { b'v' }
            fn signature() -> String { "v".into() }
            fn alignment() -> u8 { 1 }
        };

        Ok(body)
    }
}

impl Parse for DeriveTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::spanned::Spanned;

        let derive_input = input.parse::<syn::DeriveInput>()?;
        let metas = parse_named_metas(&derive_input.attrs, "dbus");

        Ok(DeriveTypeDef {
            span: derive_input.span(),
            metas,
            name: derive_input.ident,
            generics: derive_input.generics,
            data: derive_input.data,
        })
    }
}

pub fn derive_type(data: DeriveTypeDef) -> Result<TokenStream> {
    data.impl_type()
}
