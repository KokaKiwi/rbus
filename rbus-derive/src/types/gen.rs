use crate::{ext::GenericsExt, utils::*};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{spanned::Spanned, Error, Result};

const DBUS_TYPE_METHOD_NAMES: &[&str] = &["code", "signature", "alignment", "encode", "decode"];

pub struct ImplGeneratorOptions {
    pub default_rbus_module: String,
}

impl Default for ImplGeneratorOptions {
    fn default() -> ImplGeneratorOptions {
        ImplGeneratorOptions {
            default_rbus_module: "rbus".into(),
        }
    }
}

pub struct ImplGenerator {
    pub span: Span,
    pub metas: Metas,
    pub dbus: Metas,
    pub generics: syn::Generics,
    pub ty: syn::Type,
    pub methods: HashMap<String, TokenStream>,
    pub options: ImplGeneratorOptions,
}

impl ImplGenerator {
    pub fn new(span: Span, metas: Metas, generics: Option<syn::Generics>, ty: syn::Type) -> ImplGenerator {
        let dbus = metas.find_meta_nested("dbus");
        let generics = generics.unwrap_or_else(syn::Generics::empty);

        ImplGenerator {
            span,
            metas,
            dbus,
            generics,
            ty,
            methods: HashMap::with_capacity(5),
            options: Default::default(),
        }
    }

    pub fn new_ident(span: Span, metas: Metas, generics: Option<syn::Generics>, ty: syn::Ident) -> ImplGenerator {
        let generics = generics.unwrap_or_else(syn::Generics::empty);
        let (_, type_generics, _) = generics.split_for_impl();

        let ty = syn::parse_quote!(#ty #type_generics);

        ImplGenerator::new(span, metas, Some(generics), ty)
    }

    pub fn rbus_module(&self) -> MetaValue {
        self.dbus.find_rbus_module(&self.options.default_rbus_module)
    }

    pub fn is_packed(&self) -> bool {
        self.dbus.has_word("packed")
    }

    pub fn is_basic(&self) -> bool {
        self.dbus.has_word("basic")
    }

    pub fn add_method<T: Into<String>>(&mut self, name: T, method: TokenStream) {
        let name = name.into();
        let accept_names = DBUS_TYPE_METHOD_NAMES;

        if accept_names.contains(&name.as_str()) {
            self.methods.insert(name, method);
        }
    }

    pub fn override_methods_from_metas(&mut self) -> Result<()> {
        if let Some(meta) = self.dbus.find_meta_value_lit("code") {
            let body = match meta {
                syn::Lit::Byte(lit) => quote::quote!(#lit as u8),
                syn::Lit::Int(lit) => quote::quote!(#lit as u8),
                syn::Lit::Char(lit) => quote::quote!(#lit as u8),
                lit => return Err(Error::new(lit.span(), "Bad code value, expected byte or integer")),
            };

            self.add_method("code", self.gen_code_method(body, &[]));
        }

        if let Some(signature) = self.dbus.find_meta_value_str("signature") {
            let body = quote::quote!(#signature.into());

            self.add_method("signature", self.gen_signature_method(body, &[]));
        }

        if let Some(align) = self.dbus.find_meta_value_lit("align") {
            let body = match align {
                syn::Lit::Int(lit) => quote::quote!(#lit as u8),
                syn::Lit::Str(lit) if lit.value() == "size" => quote::quote!(std::mem::size_of::<Self>() as u8),
                lit => return Err(Error::new(lit.span(), "Bad align value, only integer or \"size\"")),
            };

            self.add_method("alignment", self.gen_alignment_method(body, &[]));
        }

        Ok(())
    }

    pub fn gen_impl(self) -> Result<TokenStream> {
        let rbus_module = self.rbus_module();
        let (impl_generics, _, where_clause) = self.generics.split_for_impl();
        let ty = &self.ty;

        let methods = DBUS_TYPE_METHOD_NAMES
            .iter()
            .map(|&name| {
                self.methods
                    .get(name)
                    .ok_or_else(|| Error::new(self.span, format!("Missing method: {}", name)))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut tokens = quote::quote! {
            impl #impl_generics #rbus_module::types::DBusType for #ty #where_clause {
                #(#methods)*
            }
        };

        if self.is_basic() && !self.is_packed() {
            tokens.extend(quote::quote! {
                impl #impl_generics #rbus_module::types::DBusBasicType for #ty #where_clause {}
            });
        }

        Ok(tokens)
    }

    pub fn gen_code_method<Body: ToTokens>(&self, body: Body, attrs: &[Attribute]) -> TokenStream {
        quote::quote! {
            #(#attrs)*
            fn code() -> u8 { #body }
        }
    }

    pub fn gen_signature_method<Body: ToTokens>(&self, body: Body, attrs: &[Attribute]) -> TokenStream {
        quote::quote! {
            #(#attrs)*
            fn signature() -> String { #body }
        }
    }

    pub fn gen_alignment_method<Body: ToTokens>(&self, body: Body, attrs: &[Attribute]) -> TokenStream {
        quote::quote! {
            #(#attrs)*
            fn alignment() -> u8 { #body }
        }
    }

    pub fn gen_encode_method<Body: ToTokens>(
        &self,
        marshaller: syn::Ident,
        body: Body,
        attrs: &[Attribute],
    ) -> TokenStream {
        let rbus_module = self.rbus_module();

        quote::quote! {
            #(#attrs)*
            fn encode<Inner>(&self, #marshaller: &mut #rbus_module::marshal::Marshaller<Inner>)
                -> #rbus_module::Result<()>
            where
                Inner: std::io::Write
            {
                #body
            }
        }
    }

    pub fn gen_decode_method<Body: ToTokens>(
        &self,
        marshaller: syn::Ident,
        body: Body,
        attrs: &[Attribute],
    ) -> TokenStream {
        let rbus_module = self.rbus_module();

        quote::quote! {
            #(#attrs)*
            fn decode<Inner>(#marshaller: &mut #rbus_module::marshal::Marshaller<Inner>)
                -> #rbus_module::Result<Self>
            where
                Inner: std::io::Read
            {
                #body
            }
        }
    }
}
