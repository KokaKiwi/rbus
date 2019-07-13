use crate::utils::attr::{parse_named_metas, Metas};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};

pub enum DeriveTypeDef {
    Struct(StructTypeDef),
    Enum(EnumTypeDef),
}

impl DeriveTypeDef {
    pub fn impl_type(self) -> Result<TokenStream> {
        match self {
            DeriveTypeDef::Struct(def) => def.impl_type(),
            DeriveTypeDef::Enum(def) => def.impl_type(),
        }
    }
}

impl Parse for DeriveTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(def) = input.parse() {
            Ok(DeriveTypeDef::Struct(def))
        } else if let Ok(def) = input.parse() {
            Ok(DeriveTypeDef::Enum(def))
        } else {
            Err(input.error("DBusType can only be derive for struct or enum"))
        }
    }
}

pub struct StructTypeDef {
    name: syn::Ident,
    settings: Metas,
    generics: syn::Generics,
    fields: syn::Fields,
}

impl StructTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;

        let rbus_module = self
            .settings
            .find_meta_value_str("module")?
            .unwrap_or_else(|| "rbus_common".into());
        let rbus_module = syn::Ident::new(rbus_module.as_str(), Span::call_site());
        let dbus_type_path: syn::TraitBound = syn::parse_quote!(#rbus_module::types::DBusType);

        // Add DBusType trait dep for generics if any
        let mut generics = self.generics;
        if !generics.params.is_empty() {
            let type_param_bound = syn::TypeParamBound::Trait(dbus_type_path.clone());
            generics
                .type_params_mut()
                .for_each(|type_param| type_param.bounds.push_value(type_param_bound.clone()));
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let field_types = match self.fields {
            syn::Fields::Named(fields) => {
                fields.named.iter().map(|field| field.ty.clone()).collect()
            }
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(|field| field.ty.clone())
                .collect(),
            syn::Fields::Unit => Vec::new(),
        };

        let format_str = format!("({})", "{}".repeat(field_types.len()));

        let tokens = quote::quote! {
            impl #impl_generics #dbus_type_path for #name #ty_generics #where_clause {
                fn code() -> u8 { b'r' }
                fn signature() -> String { format!(#format_str, #(<#field_types>::signature()),*) }
                fn alignment() -> u8 { 8 }
            }
        };

        Ok(tokens.into())
    }
}

impl Parse for StructTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemStruct>()?;

        let settings = parse_named_metas(item.attrs, "dbus");

        Ok(StructTypeDef {
            name: item.ident,
            settings,
            generics: item.generics,
            fields: item.fields,
        })
    }
}

pub struct EnumTypeDef {
    name: syn::Ident,
    settings: Metas,
    generics: syn::Generics,
    #[allow(dead_code)]
    variants: syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
}

impl EnumTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;

        let rbus_module = self
            .settings
            .find_meta_value_str("module")?
            .unwrap_or_else(|| "rbus_common".into());
        let rbus_module = syn::Ident::new(rbus_module.as_str(), Span::call_site());
        let dbus_type_path: syn::TraitBound = syn::parse_quote!(#rbus_module::types::DBusType);

        // Add DBusType trait dep for generics if any
        let mut generics = self.generics;
        if !generics.params.is_empty() {
            let type_param_bound = syn::TypeParamBound::Trait(dbus_type_path.clone());
            generics
                .type_params_mut()
                .for_each(|type_param| type_param.bounds.push_value(type_param_bound.clone()));
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let tokens = quote::quote! {
            impl #impl_generics #dbus_type_path for #name #ty_generics #where_clause {
                fn code() -> u8 { b'v' }
                fn signature() -> String { "v" }
                fn alignment() -> u8 { 1 }
            }
        };

        Ok(tokens.into())
    }
}

impl Parse for EnumTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemEnum>()?;

        let settings = parse_named_metas(item.attrs, "dbus");

        Ok(EnumTypeDef {
            name: item.ident,
            settings,
            generics: item.generics,
            variants: item.variants,
        })
    }
}

pub fn derive_type(data: DeriveTypeDef) -> Result<TokenStream> {
    data.impl_type()
}
