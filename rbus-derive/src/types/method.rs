use crate::utils::{DBusMetas, Metas};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Error;

pub type Methods = Vec<Method>;

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: syn::Ident,
    pub args: Punctuated<syn::Ident, syn::Token![,]>,
    pub body: syn::Block,
}

impl Method {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn gen_dbus_method(&self, metas: &Metas) -> Result<(String, TokenStream)> {
        let name = self.name();
        let method = match name.as_str() {
            "code" => self.gen_code_method(),
            "signature" => self.gen_signature_method(),
            "alignment" => self.gen_alignment_method(),
            "encode" => self.gen_encode_method(metas)?,
            "decode" => self.gen_decode_method(metas)?,
            _ => return Err(Error::new(self.name.span(), "Invalid DBusType method name")),
        };

        Ok((name, method))
    }

    fn gen_code_method(&self) -> TokenStream {
        let Method { body, .. } = self;

        quote::quote! {
            fn code() -> u8 #body
        }
    }

    fn gen_signature_method(&self) -> TokenStream {
        let Method { body, .. } = self;

        quote::quote! {
            fn signature() -> String #body
        }
    }

    fn gen_alignment_method(&self) -> TokenStream {
        let Method { body, .. } = self;

        quote::quote! {
            fn alignment() -> u8 #body
        }
    }

    fn gen_encode_method(&self, metas: &Metas) -> Result<TokenStream> {
        let Method { args, body, .. } = self;
        let marshaller = args
            .first()
            .map(|pair| pair.into_value())
            .ok_or_else(|| Error::new(args.span(), "Not enough arguments"))?;

        let rbus_module = metas.find_rbus_module("crate");

        let tokens = quote::quote! {
            fn encode<Inner>(&self, #marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + std::io::Write
            {
                use std::io::Write;

                #body
            }
        };

        Ok(tokens)
    }

    fn gen_decode_method(&self, metas: &Metas) -> Result<TokenStream> {
        let Method { args, body, .. } = self;
        let marshaller = args
            .first()
            .map(|pair| pair.into_value())
            .ok_or_else(|| Error::new(args.span(), "Not enough arguments"))?;

        let rbus_module = metas.find_rbus_module("crate");

        let tokens = quote::quote! {
            fn decode<Inner>(#marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                use std::io::Read;
                #body
            }
        };

        Ok(tokens)
    }
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Method> {
        let content;
        let name = input.parse()?;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(syn::Ident::parse)?;
        let body = input.parse()?;

        Ok(Method { name, args, body })
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Method { name, args, body } = self;

        tokens.extend(quote::quote! {
            #name(#args) #body
        });
    }
}

pub fn parse_methods(input: ParseStream) -> Result<Methods> {
    let methods = if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);

        let mut methods = Vec::new();
        while content.peek(syn::Ident) {
            let method = content.parse()?;
            methods.push(method);
        }
        methods
    } else {
        Vec::new()
    };

    Ok(methods)
}
