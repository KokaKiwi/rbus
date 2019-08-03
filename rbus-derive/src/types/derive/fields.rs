use crate::utils::*;
use proc_macro2::{Span, TokenStream};
use std::convert::TryFrom;
use syn::{spanned::Spanned, Error, Result};

#[derive(Debug, Clone)]
pub enum Fields {
    Named(Vec<NamedField>),
    Unnamed(Vec<UnnamedField>),
    Unit,
}

impl Fields {
    pub fn to_vec(&self) -> Vec<Field> {
        match self {
            Fields::Named(fields) => fields.iter().map(Field::from).collect(),
            Fields::Unnamed(fields) => fields.iter().map(Field::from).collect(),
            Fields::Unit => Vec::new(),
        }
    }

    pub fn split_named(fields: &[NamedField]) -> (Vec<&syn::Ident>, Vec<&syn::Type>) {
        let mut names = Vec::with_capacity(fields.len());
        let mut types = Vec::with_capacity(fields.len());
        for field in fields.iter() {
            names.push(&field.name);
            types.push(&field.ty);
        }
        (names, types)
    }

    pub fn is_named(&self) -> bool {
        match self {
            Fields::Named(_) => true,
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            Fields::Unit => true,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Fields::Named(fields) => fields.len(),
            Fields::Unnamed(fields) => fields.len(),
            Fields::Unit => 0,
        }
    }

    pub fn signature(&self) -> TokenStream {
        let types = self.types();
        if self.is_named() || self.len() > 1 {
            let signature_format_str = format!("({})", "{}".repeat(self.len()));
            quote::quote!(format!(#signature_format_str, #(<#types>::signature()),*))
        } else if self.len() == 1 {
            let ty = types[0];
            quote::quote!(<#ty>::signature())
        } else {
            quote::quote!("")
        }
    }

    pub fn alignment(&self) -> TokenStream {
        if self.is_named() || self.len() > 1 {
            let alignment = syn::LitInt::new(8, syn::IntSuffix::None, Span::call_site());
            quote::quote!(#alignment)
        } else if self.len() == 1 {
            let types = self.types();
            let ty = types[0];
            quote::quote!(<#ty>::alignment())
        } else {
            quote::quote!(0)
        }
    }

    pub fn pat(&self, named: bool) -> TokenStream {
        match self {
            Fields::Named(fields) => {
                let names: TokenStream = if named {
                    let names = fields.iter().map(|field| &field.name);
                    quote::quote!(#(ref #names),*)
                } else {
                    quote::quote!(..)
                };

                quote::quote! {
                    {#names}
                }
            }
            Fields::Unnamed(fields) => {
                let names: TokenStream = if named {
                    let names = fields.iter().map(UnnamedField::binding);
                    quote::quote!(#(ref #names),*)
                } else {
                    quote::quote!(..)
                };

                quote::quote! {
                    (#names)
                }
            }
            Fields::Unit => quote::quote!(),
        }
    }

    pub fn bindings(&self) -> TokenStream {
        match self {
            Fields::Named(fields) => {
                let names = fields.iter().map(|field| &field.name);

                quote::quote! {
                    {#(#names),*}
                }
            }
            Fields::Unnamed(fields) => {
                let names = fields.iter().map(UnnamedField::binding);

                quote::quote! {
                    (#(#names),*)
                }
            }
            Fields::Unit => quote::quote!(),
        }
    }

    pub fn pat_names(&self) -> Vec<syn::Ident> {
        match self {
            Fields::Named(fields) => fields.iter().map(|field| field.name.clone()).collect(),
            Fields::Unnamed(fields) => fields.iter().map(UnnamedField::binding).collect(),
            Fields::Unit => vec![],
        }
    }

    pub fn types(&self) -> Vec<&syn::Type> {
        match self {
            Fields::Named(fields) => fields.iter().map(|field| &field.ty).collect(),
            Fields::Unnamed(fields) => fields.iter().map(|field| &field.ty).collect(),
            Fields::Unit => vec![],
        }
    }
}

impl TryFrom<syn::Fields> for Fields {
    type Error = Error;

    fn try_from(fields: syn::Fields) -> Result<Fields> {
        match fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .cloned()
                .map(NamedField::from)
                .collect::<Result<Vec<_>>>()
                .map(Fields::Named),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .cloned()
                .enumerate()
                .map(UnnamedField::from)
                .collect::<Result<Vec<_>>>()
                .map(Fields::Unnamed),
            syn::Fields::Unit => Ok(Fields::Unit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field<'a> {
    pub metas: &'a Metas,
    pub span: Span,
    pub name: TokenStream,
    pub binding: syn::Ident,
    pub ty: &'a syn::Type,
}

impl<'a> Field<'a> {
    pub fn dbus(&self) -> Metas {
        self.metas.find_meta_nested("dbus")
    }
}

impl<'a> From<&'a NamedField> for Field<'a> {
    fn from(field: &NamedField) -> Field {
        let name = &field.name;

        Field {
            metas: &field.metas,
            span: field.span,
            name: quote::quote!(#name),
            binding: field.binding(),
            ty: &field.ty,
        }
    }
}

impl<'a> From<&'a UnnamedField> for Field<'a> {
    fn from(field: &UnnamedField) -> Field {
        let pos = &field.pos;

        Field {
            metas: &field.metas,
            span: field.span,
            name: quote::quote!(#pos),
            binding: field.binding(),
            ty: &field.ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedField {
    pub metas: Metas,
    pub span: Span,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl NamedField {
    fn from(field: syn::Field) -> Result<NamedField> {
        Ok(NamedField {
            metas: Metas::from_attributes(&field.attrs)?,
            span: field.span(),
            name: field.ident.unwrap(),
            ty: field.ty,
        })
    }

    fn binding(&self) -> syn::Ident {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct UnnamedField {
    pub metas: Metas,
    pub span: Span,
    pub pos: syn::LitInt,
    pub ty: syn::Type,
}

impl UnnamedField {
    fn from((pos, field): (usize, syn::Field)) -> Result<UnnamedField> {
        let pos = syn::LitInt::new(pos as u64, syn::IntSuffix::None, field.span());
        Ok(UnnamedField {
            metas: Metas::from_attributes(&field.attrs)?,
            span: field.span(),
            pos,
            ty: field.ty,
        })
    }

    fn binding(&self) -> syn::Ident {
        let name = format!("_field_{}", self.pos.value());
        syn::Ident::new(&name, self.pos.span())
    }
}
