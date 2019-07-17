use super::method::{parse_methods, Methods};
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

pub struct BasicTypeDef {
    attrs: Vec<syn::Attribute>,
    ty: syn::Type,
    code: syn::LitChar,
    methods: Methods,
}

impl BasicTypeDef {
    fn impl_type(&self) -> Result<TokenStream> {
        let BasicTypeDef {
            attrs,
            ty,
            code,
            methods,
        } = self;

        let encode_method = self.gen_encode_method()?;
        let decode_method = self.gen_decode_method()?;

        let tokens = quote::quote! {
            rbus_derive::impl_type! {
                #(#attrs)*
                #[dbus(basic)]
                #ty: #code {
                    #encode_method
                    #decode_method
                    #(#methods)*
                }
            }
        };

        Ok(tokens)
    }

    fn gen_encode_method(&self) -> Result<TokenStream> {
        let BasicTypeDef { ty, .. } = self;

        let ty_ident: syn::Ident = syn::parse_quote!(#ty);
        let write_method_name = format!("write_{}", ty_ident);
        let write_method = syn::Ident::new(&write_method_name, ty.span());

        let tokens = quote::quote! {
            encode(marshaller) {
                marshaller.io().#write_method(*self)?;
                Ok(())
            }
        };

        Ok(tokens)
    }

    fn gen_decode_method(&self) -> Result<TokenStream> {
        let BasicTypeDef { ty, .. } = self;

        let ty_ident: syn::Ident = syn::parse_quote!(#ty);
        let read_method_name = format!("read_{}", ty_ident);
        let read_method = syn::Ident::new(&read_method_name, ty.span());

        let tokens = quote::quote! {
            decode(marshaller) {
                marshaller.io().#read_method()
            }
        };

        Ok(tokens)
    }
}

impl Parse for BasicTypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let ty = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let code = input.parse()?;
        let methods = input.call(parse_methods)?;

        Ok(BasicTypeDef {
            attrs,
            ty,
            code,
            methods,
        })
    }
}

pub fn impl_basic_type(data: BasicTypeDef) -> Result<TokenStream> {
    data.impl_type()
}
