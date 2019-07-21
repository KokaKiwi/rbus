use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Fields {
    Named(Vec<NamedField>),
    Unnamed(Vec<UnnamedField>),
    Unit,
}

impl Fields {
    pub fn to_vec(&self) -> Vec<(TokenStream, &syn::Type)> {
        match self {
            Fields::Named(fields) => fields.iter().map(NamedField::quoted).collect(),
            Fields::Unnamed(fields) => fields.iter().map(UnnamedField::quoted).collect(),
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
            panic!("Unit variant have no signature!")
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
            panic!("Unit variant have no alignment");
        }
    }

    pub fn pat(&self) -> TokenStream {
        match self {
            Fields::Named(fields) => {
                let names = fields.iter().map(|field| &field.name);

                quote::quote! {
                    {#(ref #names),*}
                }
            }
            Fields::Unnamed(fields) => {
                let names = fields.iter().map(UnnamedField::pat_name);

                quote::quote! {
                    ( #(ref #names),*)
                }
            }
            Fields::Unit => quote::quote!(),
        }
    }

    pub fn pat_names(&self) -> Vec<syn::Ident> {
        match self {
            Fields::Named(fields) => fields.iter().map(|field| field.name.clone()).collect(),
            Fields::Unnamed(fields) => fields.iter().map(UnnamedField::pat_name).collect(),
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

impl From<syn::Fields> for Fields {
    fn from(fields: syn::Fields) -> Fields {
        match fields {
            syn::Fields::Named(fields) => {
                let fields = fields.named.iter().cloned().map(NamedField::from).collect();

                Fields::Named(fields)
            }
            syn::Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(UnnamedField::from)
                    .collect();

                Fields::Unnamed(fields)
            }
            syn::Fields::Unit => Fields::Unit,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedField {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl NamedField {
    fn from(field: syn::Field) -> NamedField {
        NamedField {
            name: field.ident.unwrap(),
            ty: field.ty,
        }
    }

    fn quoted(&self) -> (TokenStream, &syn::Type) {
        let name = &self.name;
        (quote::quote!(#name), &self.ty)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnnamedField {
    pub pos: syn::LitInt,
    pub ty: syn::Type,
}

impl UnnamedField {
    fn from((pos, field): (usize, syn::Field)) -> UnnamedField {
        let pos = syn::LitInt::new(pos as u64, syn::IntSuffix::None, field.span());
        UnnamedField { pos, ty: field.ty }
    }

    fn quoted(&self) -> (TokenStream, &syn::Type) {
        let pos = &self.pos;
        (quote::quote!(#pos), &self.ty)
    }

    fn pat_name(&self) -> syn::Ident {
        let name = format!("_field_{}", self.pos.value());
        syn::Ident::new(&name, self.pos.span())
    }
}
