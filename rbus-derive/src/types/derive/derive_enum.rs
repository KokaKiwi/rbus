use super::{DeriveTypeDef, Fields, ImplGenerator, ImplMethods, Variant, Variants};
use crate::utils::*;
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use syn::{parse::Result, punctuated::Pair};

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    variants: Variants,
}

impl DeriveEnum {
    pub fn gen_methods(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> Result<ImplMethods> {
        if self.variants.is_complete() {
            Ok(self.gen_complete_methods(ty, gen))
        } else if self.variants.is_unit() {
            self.gen_unit_methods(ty, gen)
        } else if let Some(index) = ty
            .metas
            .find_meta_nested("dbus")
            .find_meta_nested("index")
            .into_option()
        {
            self.gen_index_methods(ty, gen, index)
        } else {
            Err(syn::Error::new(
                ty.span,
                "Can't derive mixed unit and named/unnamed fields",
            ))
        }
    }

    fn gen_complete_methods(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> ImplMethods {
        vec![
            ("code", gen.gen_code_method(quote::quote!(b'v'), &[])),
            ("signature", gen.gen_signature_method(quote::quote!("v".into()), &[])),
            ("alignment", gen.gen_alignment_method(quote::quote!(1), &[])),
            ("encode", self.gen_encode_method(ty, gen)),
            ("decode", self.gen_decode_method(ty, gen)),
        ]
    }

    fn gen_unit_methods(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> Result<ImplMethods> {
        let rbus_module = gen.rbus_module();
        let repr: syn::Ident = ty
            .metas
            .find_meta_nested("repr")
            .words()
            .next()
            .cloned()
            .ok_or_else(|| syn::Error::new(ty.span, "Unit-only enums must have a fixed repr"))?;

        let ty_names = vec![&ty.name; self.variants.len()];
        let variant_names = self.variants.iter().map(|variant| &variant.name);
        let values = self.variants.values();

        Ok(vec![
            ("code", gen.gen_code_method(quote::quote!(#repr::code()), &[])),
            (
                "signature",
                gen.gen_signature_method(quote::quote!(#repr::signature()), &[]),
            ),
            (
                "alignment",
                gen.gen_alignment_method(quote::quote!(#repr::alignment()), &[]),
            ),
            (
                "encode",
                gen.gen_encode_method(
                    syn::parse_quote!(marshaller),
                    quote::quote!((*self as #repr).encode(marshaller)),
                    &[],
                ),
            ),
            (
                "decode",
                gen.gen_decode_method(
                    syn::parse_quote!(marshaller),
                    quote::quote! {
                         let value = #repr::decode(marshaller)?;
                         match value {
                             #(#values => Ok(#ty_names::#variant_names),)*
                             value => Err(#rbus_module::Error::InvalidVariant { value: value as u64 }),
                         }
                    },
                    &[],
                ),
            ),
        ])
    }

    fn gen_index_methods(&self, ty: &DeriveTypeDef, gen: &ImplGenerator, index: Metas) -> Result<ImplMethods> {
        let rbus_module = gen.rbus_module();
        let index_ty = index
            .words()
            .next()
            .cloned()
            .ok_or_else(|| syn::Error::new(ty.span, "Unit-only enums must have a fixed repr"))?;
        let indexes = self.variants.indexes()?;
        let variant_encodes: TokenStream = self
            .variants
            .iter()
            .zip(&indexes)
            .map(|(variant, index)| {
                let encode_body = self.gen_encode_variant_body(variant);
                let pre = quote::quote! {
                    marshaller.io().write_u8(#index)?;
                };
                let body = vec![pre, encode_body].into_iter().collect::<TokenStream>();
                self.gen_encode_variant(ty, variant, body)
            })
            .collect();
        let variant_decodes: TokenStream = self
            .variants
            .iter()
            .zip(&indexes)
            .map(|(variant, index)| {
                let body = self.gen_decode_variant_body(ty, variant);
                let signature = variant.fields.signature();

                quote::quote! {
                    if signature == #signature && value == #index {
                        #body
                    }
                }
            })
            .collect();

        Ok(vec![
            ("code", gen.gen_code_method(quote::quote!(b'r'), &[])),
            (
                "signature",
                gen.gen_signature_method(quote::quote!(format!("({}v)", <#index_ty>::signature())), &[]),
            ),
            ("alignment", gen.gen_alignment_method(quote::quote!(8), &[])),
            (
                "encode",
                gen.gen_encode_method(
                    syn::parse_quote!(marshaller),
                    quote::quote! {
                        use #rbus_module::types::Signature;

                        match self {
                            #variant_encodes
                        }
                    },
                    &[],
                ),
            ),
            (
                "decode",
                gen.gen_decode_method(
                    syn::parse_quote!(marshaller),
                    quote::quote! {
                        use #rbus_module::types::Signature;

                        let value = marshaller.io().read_u8()?;

                        let signature = Signature::decode(marshaller)?;
                        let signature = signature.as_str();

                        #variant_decodes
                        Err(#rbus_module::Error::Custom {
                            message: "Bad variant value".into(),
                        })
                    },
                    &[],
                ),
            ),
        ])
    }

    fn gen_encode_method(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> TokenStream {
        let rbus_module = gen.rbus_module();
        let variants = self.variants.iter().map(|variant| {
            let body = self.gen_encode_variant_body(variant);
            self.gen_encode_variant(ty, variant, body)
        });

        let body = quote::quote! {
            use #rbus_module::types::Signature;

            match self {
                #(#variants)*
            }
        };

        gen.gen_encode_method(syn::parse_quote!(marshaller), body, &[])
    }

    fn gen_encode_variant(&self, ty: &DeriveTypeDef, variant: &Variant, body: TokenStream) -> TokenStream {
        let ty_name = &ty.name;
        let variant_name = &variant.name;
        let pat = variant.fields.pat(true);

        let tokens = quote::quote! {
            #ty_name::#variant_name #pat => {
                #body
                Ok(())
            }
        };

        tokens
    }

    fn gen_encode_variant_body(&self, variant: &Variant) -> TokenStream {
        let names = variant.fields.pat_names();
        let signature = variant.fields.signature();
        let alignment = vec![variant.fields.alignment(); names.len()];

        let tokens = quote::quote! {
            let signature = Signature::new(#signature)?;
            signature.encode(marshaller)?;

            #(
            marshaller.write_padding(#alignment)?;
            #names.encode(marshaller)?;
            )*
        };

        tokens
    }

    fn gen_decode_method(&self, ty: &DeriveTypeDef, gen: &ImplGenerator) -> TokenStream {
        let rbus_module = gen.rbus_module();
        let variants = self.variants.iter().map(|variant| {
            let body = self.gen_decode_variant_body(ty, variant);
            self.gen_decode_variant(variant, body)
        });

        let body = quote::quote! {
            use #rbus_module::types::Signature;

            let signature = Signature::decode(marshaller)?;
            let signature = signature.as_str();

            #(#variants)*
            Err(#rbus_module::Error::Custom {
                message: "Bad variant value".into(),
            })
        };

        gen.gen_decode_method(syn::parse_quote!(marshaller), body, &[])
    }

    fn gen_decode_variant(&self, variant: &Variant, body: TokenStream) -> TokenStream {
        let signature = variant.fields.signature();

        let tokens = quote::quote! {
            if signature == #signature {
                #body
            }
        };

        tokens
    }

    fn gen_decode_variant_body(&self, ty: &DeriveTypeDef, variant: &Variant) -> TokenStream {
        let ty_name = &ty.name;
        let variant_name = &variant.name;
        let alignment = vec![variant.fields.alignment(); variant.fields.len()];

        let construct = match variant.fields {
            Fields::Named(ref fields) => {
                let (names, types) = Fields::split_named(&fields);

                quote::quote! {
                    #ty_name::#variant_name {
                        #(#names: {
                            marshaller.read_padding(#alignment)?;
                            <#types>::decode(marshaller)?
                        },)*
                    }
                }
            }
            Fields::Unnamed(_) => {
                let types = variant.fields.types();

                quote::quote! {
                    #ty_name::#variant_name(#({
                        marshaller.read_padding(#alignment)?;
                        <#types>::decode(marshaller)?
                    }),*)
                }
            }
            Fields::Unit => quote::quote!(#ty_name::#variant_name),
        };

        quote::quote!(return Ok(#construct))
    }
}

impl TryFrom<syn::DataEnum> for DeriveEnum {
    type Error = syn::Error;

    fn try_from(data: syn::DataEnum) -> Result<Self> {
        let variants = data.variants.into_pairs().map(Pair::into_value).collect::<Vec<_>>();
        let variants = Variants::try_from(variants)?;

        Ok(DeriveEnum { variants })
    }
}
