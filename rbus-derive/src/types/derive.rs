use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};

pub enum DeriveTypeDef {
    Struct(StructTypeDef),
    Enum(EnumTypeDef),
}

impl DeriveTypeDef {
    pub fn impl_type(self) -> TokenStream {
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
    fields: syn::Fields,
}

impl StructTypeDef {
    fn impl_type(self) -> TokenStream {
        let name = self.name;

        let field_types = match self.fields {
            syn::Fields::Named(fields) => {
                fields.named.iter().map(|field| field.ty.clone()).collect()
            }
            syn::Fields::Unnamed(fields) => {
                fields.unnamed.iter().map(|field| field.ty.clone()).collect()
            }
            syn::Fields::Unit => Vec::new(),
        };

        let format_str = format!("({})", "{}".repeat(field_types.len()));

        let tokens = quote::quote! {
            impl crate::types::DBusType for #name {
                fn code() -> u8 { b'r' }
                fn signature() -> String { format!(#format_str, #(<#field_types>::signature()),*) }
            }
        };

        tokens.into()
    }
}

impl Parse for StructTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemStruct>()?;

        Ok(StructTypeDef {
            name: item.ident,
            fields: item.fields,
        })
    }
}

pub struct EnumTypeDef {
    name: syn::Ident,
    variants: syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
}

impl EnumTypeDef {
    fn impl_type(self) -> TokenStream {
        let tokens = quote::quote! {

        };

        tokens.into()
    }
}

impl Parse for EnumTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemEnum>()?;

        Ok(EnumTypeDef {
            name: item.ident,
            variants: item.variants,
        })
    }
}

pub fn derive_type(data: DeriveTypeDef) -> TokenStream {
    data.impl_type()
}
