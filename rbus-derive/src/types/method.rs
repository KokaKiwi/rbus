use super::ImplGenerator;
use crate::utils::Metas;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    Error,
};

pub type Methods = Vec<Method>;

#[derive(Debug, Clone, PartialEq)]
pub struct Body(Vec<syn::Stmt>);

impl Parse for Body {
    fn parse(input: ParseStream) -> Result<Body> {
        let content;
        let input = if input.peek(syn::token::Brace) {
            syn::braced!(content in input);
            &content
        } else {
            input
        };
        let stmts = input.call(syn::Block::parse_within)?;
        Ok(Body(stmts))
    }
}

impl quote::ToTokens for Body {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;

        tokens.append_all(&self.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub metas: Metas,
    pub name: syn::Ident,
    pub args: Punctuated<syn::Ident, syn::Token![,]>,
    pub body: Body,
}

impl Method {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn gen_dbus_method(&self, gen: &ImplGenerator) -> Result<(String, TokenStream)> {
        let Method { metas, body, .. } = self;

        let name = self.name();
        let method = match name.as_str() {
            "code" => gen.gen_code_method(body, Some(metas)),
            "signature" => gen.gen_signature_method(body, Some(metas)),
            "alignment" => gen.gen_alignment_method(body, Some(metas)),
            "encode" => self.gen_encode_method(gen)?,
            "decode" => self.gen_decode_method(gen)?,
            _ => return Err(Error::new(self.name.span(), "Invalid DBusType method name")),
        };

        Ok((name, method))
    }

    fn gen_encode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let Method { metas, args, body, .. } = self;
        let marshaller = args
            .first()
            .map(|pair| pair.into_value())
            .cloned()
            .ok_or_else(|| Error::new(args.span(), "Not enough arguments"))?;

        Ok(gen.gen_encode_method(marshaller, body, Some(metas)))
    }

    fn gen_decode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let Method { metas, args, body, .. } = self;
        let marshaller = args
            .first()
            .map(|pair| pair.into_value())
            .cloned()
            .ok_or_else(|| Error::new(args.span(), "Not enough arguments"))?;

        Ok(gen.gen_decode_method(marshaller, body, Some(metas)))
    }
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Method> {
        let metas = input.parse()?;
        let name = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(syn::Ident::parse)?;
        let body = input.parse()?;

        Ok(Method {
            metas,
            name,
            args,
            body,
        })
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Method {
            metas,
            name,
            args,
            body,
        } = self;

        tokens.extend(quote::quote! {
            #metas
            #name(#args) {
                #body
            }
        });
    }
}

pub fn parse_methods(input: ParseStream) -> Result<Methods> {
    let methods = if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);

        let mut methods = Vec::new();
        while let Ok(method) = content.parse() {
            methods.push(method);
        }
        methods
    } else {
        Vec::new()
    };

    Ok(methods)
}
