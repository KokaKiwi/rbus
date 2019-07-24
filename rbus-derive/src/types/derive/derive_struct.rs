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
        let signature_format_str = if gen.is_packed() {
            format!("{}", "{}".repeat(field_types.len()))
        } else {
            format!("({})", "{}".repeat(field_types.len()))
        };

        let code = if gen.is_packed() {
            quote::quote!(0)
        } else {
            quote::quote!(b'r')
        };
        let alignment = if gen.is_packed() {
            quote::quote!(1)
        } else {
            quote::quote!(8)
        };

        let encode_method = self.gen_encode_method(gen)?;
        let decode_method = self.gen_decode_method(gen)?;

        Ok(vec![
            ("code", gen.gen_code_method(code, None)),
            (
                "signature",
                gen.gen_signature_method(
                    quote::quote! {
                        format!(#signature_format_str, #(<#field_types>::signature()),*)
                    },
                    None,
                ),
            ),
            ("alignment", gen.gen_alignment_method(alignment, None)),
            ("encode", encode_method),
            ("decode", decode_method),
        ])
    }

    fn field_types(&self) -> Vec<&syn::Type> {
        self.fields.to_vec().into_iter().map(|field| field.ty).collect()
    }

    fn gen_encode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let fields = self
            .fields
            .to_vec()
            .into_iter()
            .map(|field| {
                let dbus = field.dbus();
                let mut tokens = TokenStream::new();

                if !gen.is_packed() {
                    tokens.extend(quote::quote! {
                        marshaller.write_padding(Self::alignment())?;
                    });
                }

                let name = &field.name;
                tokens.extend(quote::quote! {
                    self.#name.encode(marshaller)?;
                });

                if let Some(value) = dbus.find_meta_value_str("mutate_marshaller")? {
                    let path: TokenStream = value.parse()?;

                    tokens.extend(quote::quote! {
                        #path(&self.#name, marshaller);
                    });
                }

                Ok(tokens)
            })
            .collect::<Result<Vec<_>>>()?;

        let body = quote::quote! {
            #(#fields)*
            Ok(())
        };

        Ok(gen.gen_encode_method(syn::parse_quote!(marshaller), body, None))
    }

    fn gen_decode_method(&self, gen: &ImplGenerator) -> Result<TokenStream> {
        let fields = self
            .fields
            .to_vec()
            .into_iter()
            .map(|field| {
                let dbus = field.dbus();
                let mut tokens = TokenStream::new();

                if !gen.is_packed() {
                    tokens.extend(quote::quote! {
                        marshaller.read_padding(Self::alignment())?;
                    });
                }

                let binding = field.binding;
                let ty = field.ty;
                tokens.extend(quote::quote! {
                    let #binding = <#ty>::decode(marshaller)?;
                });

                if let Some(value) = dbus.find_meta_value_str("mutate_marshaller")? {
                    let path: TokenStream = value.parse()?;

                    tokens.extend(quote::quote! {
                        #path(&#binding, marshaller);
                    });
                }

                Ok(tokens)
            })
            .collect::<Result<Vec<_>>>()?;
        let bindings = self.fields.bindings();

        let body = quote::quote! {
            #(#fields)*
            Ok(Self #bindings)
        };

        Ok(gen.gen_decode_method(syn::parse_quote!(marshaller), body, None))
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
