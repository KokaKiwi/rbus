use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

type DeriveSettings = Vec<syn::Meta>;

trait DeriveCommon {
    fn settings(&self) -> &Option<DeriveSettings>;

    fn find_setting_meta<'a>(&'a self, name: &str) -> Option<&'a syn::Meta> {
        match self.settings() {
            Some(metas) => metas.iter().find(|meta| meta.name() == name),
            None => None
        }
    }

    fn find_setting_word(&self, name: &str) -> Result<bool> {
        match self.find_setting_meta(name) {
            Some(syn::Meta::Word(_)) => Ok(true),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected ident: `{}`", name))),
            _ => Ok(false),
        }
    }

    fn find_setting_value<'a>(&'a self, name: &str) -> Result<Option<&'a syn::Lit>> {
        match self.find_setting_meta(name) {
            Some(syn::Meta::NameValue(meta)) => Ok(Some(&meta.lit)),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected named value: `{}`", name))),
            _ => Ok(None)
        }
    }

    fn find_setting_value_str(&self, name: &str) -> Result<Option<String>> {
        self.find_setting_value(name).and_then(|value| match value {
            Some(syn::Lit::Str(lit)) => Ok(Some(lit.value())),
            Some(meta) => Err(syn::Error::new(meta.span(), format!("Expected string value: `{}`", name))),
            _ => Ok(None),
        })
    }

    fn parse_settings(attrs: Vec<syn::Attribute>) -> Option<DeriveSettings> {
        attrs.into_iter()
            .find_map(|attr| attr.parse_meta().ok())
            .filter(|meta| meta.name() == "settings")
            .and_then(|meta| if let syn::Meta::List(metas) = meta { Some(metas) } else { None })
            .map(|metalist| metalist.nested.iter()
                 .filter_map(|item| if let syn::NestedMeta::Meta(meta) = item { Some(meta) } else { None })
                 .map(Clone::clone)
                 .collect())
    }
}

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
    settings: Option<Vec<syn::Meta>>,
    generics: syn::Generics,
    fields: syn::Fields,
}

impl DeriveCommon for StructTypeDef {
    fn settings(&self) -> &Option<DeriveSettings> {
        &self.settings
    }
}

impl StructTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;

        let rbus_module = self.find_setting_value_str("module")?.unwrap_or("rbus_common".into());
        let rbus_module = syn::Ident::new(rbus_module.as_str(), Span::call_site());
        let dbus_type_path: syn::TraitBound = syn::parse_quote!(#rbus_module::types::DBusType);

        // Add DBusType trait dep for generics if any
        let mut generics = self.generics;
        if !generics.params.is_empty() {
            let type_param_bound = syn::TypeParamBound::Trait(dbus_type_path.clone());
            generics.type_params_mut()
                .for_each(|type_param| type_param.bounds.push_value(type_param_bound.clone()));
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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
            impl #impl_generics #dbus_type_path for #name #ty_generics #where_clause {
                fn code() -> u8 { b'r' }
                fn signature() -> String { format!(#format_str, #(<#field_types>::signature()),*) }
            }
        };

        Ok(tokens.into())
    }
}

impl Parse for StructTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemStruct>()?;

        let settings = Self::parse_settings(item.attrs);

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
    settings: Option<Vec<syn::Meta>>,
    generics: syn::Generics,
    variants: syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
}

impl DeriveCommon for EnumTypeDef {
    fn settings(&self) -> &Option<DeriveSettings> {
        &self.settings
    }
}

impl EnumTypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let name = &self.name;

        let rbus_module = self.find_setting_value_str("module")?.unwrap_or("rbus_common".into());
        let rbus_module = syn::Ident::new(rbus_module.as_str(), Span::call_site());
        let dbus_type_path: syn::TraitBound = syn::parse_quote!(#rbus_module::types::DBusType);

        // Add DBusType trait dep for generics if any
        let mut generics = self.generics;
        if !generics.params.is_empty() {
            let type_param_bound = syn::TypeParamBound::Trait(dbus_type_path.clone());
            generics.type_params_mut()
                .for_each(|type_param| type_param.bounds.push_value(type_param_bound.clone()));
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let tokens = quote::quote! {
            impl #impl_generics #dbus_type_path for #name #ty_generics #where_clause {
                fn code() -> u8 { b'v' }
                fn signature() -> String { "v" }
            }
        };

        Ok(tokens.into())
    }
}

impl Parse for EnumTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<syn::ItemEnum>()?;

        let settings = Self::parse_settings(item.attrs);

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
