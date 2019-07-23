use super::{Fields, ImplGenerator, ImplMethods};
use proc_macro2::TokenStream;
use std::convert::{TryFrom, TryInto};
use syn::parse::{Error, Result};

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    fields: Fields,
}

impl DeriveStruct {
    pub fn gen_methods(&self, gen: &ImplGenerator) -> Result<ImplMethods> {
        let field_types = self.field_types();
        let signature_format_str = format!("({})", "{}".repeat(field_types.len()));

        let encode_method = self.gen_encode_method(gen)?;
        let decode_method = self.gen_decode_method(gen)?;

        Ok(vec![
            ("code", gen.gen_code_method(quote::quote!(b'r'))),
            (
                "signature",
                gen.gen_signature_method(quote::quote! {
                    format!(#signature_format_str, #(<#field_types>::signature()),*)
                }),
            ),
            ("alignment", gen.gen_alignment_method(quote::quote!(8))),
            ("encode", encode_method),
            ("decode", decode_method),
        ])
    }

    fn field_names(&self) -> Vec<TokenStream> {
        self.fields
            .to_vec()
            .into_iter()
            .map(|field| field.name.clone())
            .collect()
    }

    fn field_types(&self) -> Vec<&syn::Type> {
        self.fields
            .to_vec()
            .into_iter()
            .map(|field| field.ty)
            .collect()
    }

    fn gen_encode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let field_names = self.field_names();
        let field_extras = self
            .fields
            .to_vec()
            .into_iter()
            .map(|field| {
                let dbus = field.metas.find_meta_nested("dbus");
                let mutate_marshaller = dbus.find_meta_value_str("mutate_marshaller").and_then(
                    |value| match value {
                        Some(value) => {
                            let name = &field.name;
                            let path: TokenStream = value.parse()?;

                            let tokens = quote::quote! {
                                #path(&self.#name, marshaller);
                            };

                            Ok(Some(tokens))
                        }
                        None => Ok(None),
                    },
                )?;

                Ok(quote::quote! {
                    #mutate_marshaller
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let body = quote::quote! {
            #(
                marshaller.write_padding(Self::alignment())?;
                self.#field_names.encode(marshaller)?;
                #(#field_extras)*
            )*
            Ok(())
        };

        Ok(gen.gen_encode_method(syn::parse_quote!(marshaller), body))
    }

    fn gen_decode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let field_extras = self
            .fields
            .to_vec()
            .into_iter()
            .map(|field| {
                let dbus = field.metas.find_meta_nested("dbus");
                let mutate_marshaller = dbus.find_meta_value_str("mutate_marshaller").and_then(
                    |value| match value {
                        Some(value) => {
                            let name = &field.name;
                            let path: TokenStream = value.parse()?;

                            let tokens = quote::quote! {
                                #path(&#name, marshaller);
                            };

                            Ok(Some(tokens))
                        }
                        None => Ok(None),
                    },
                )?;

                Ok(quote::quote! {
                    #mutate_marshaller
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let body = match self.fields {
            Fields::Named(ref fields) => {
                let (names, types) = Fields::split_named(&fields);
                let names = names.as_slice();

                quote::quote! {
                    #(
                        marshaller.read_padding(Self::alignment())?;
                        let #names = <#types>::decode(marshaller)?;
                        #field_extras
                    )*

                    Ok(Self {
                        #(#names,)*
                    })
                }
            }
            Fields::Unnamed(ref fields) => {
                let types = fields.iter().map(|field| &field.ty);

                quote::quote! {
                    Ok(Self(#({
                        marshaller.read_padding(Self::alignment())?;
                        <#types>::decode(marshaller)?
                    }),*))
                }
            }
            Fields::Unit => quote::quote! {
                Ok(Self)
            },
        };

        Ok(gen.gen_decode_method(syn::parse_quote!(marshaller), body))
    }
}

impl TryFrom<syn::DataStruct> for DeriveStruct {
    type Error = Error;

    fn try_from(data: syn::DataStruct) -> Result<Self> {
        Ok(DeriveStruct {
            fields: data.fields.try_into()?,
        })
    }
}
