use super::DeriveTypeDef;
use super::{Fields, Variant, Variants};
use crate::utils::{DBusMetas, Metas};
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use syn::parse::Result;
use syn::punctuated::Pair;

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    variants: Variants,
}

impl DeriveEnum {
    pub fn gen_body(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        if self.variants.is_complete() {
            Ok(self.gen_complete_body(ty))
        } else if self.variants.is_unit() {
            self.gen_unit_body(ty)
        } else if let Some(index) = ty
            .metas
            .find_meta_nested("dbus")
            .find_meta_nested("index")
            .option()
        {
            self.gen_index_body(ty, index)
        } else {
            Err(syn::Error::new(
                ty.span,
                "Can't derive mixed unit and named/unnamed fields",
            ))
        }
    }

    fn gen_complete_body(&self, ty: &DeriveTypeDef) -> TokenStream {
        let encode_method = self.gen_encode_method(ty);
        let decode_method = self.gen_decode_method(ty);

        let body = quote::quote! {
            fn code() -> u8 { b'v' }
            fn signature() -> String { "v".into() }
            fn alignment() -> u8 { 1 }

            #encode_method
            #decode_method
        };

        body
    }

    fn gen_unit_body(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_meta_nested("dbus").find_rbus_module("rbus");
        let repr: syn::Ident = ty
            .metas
            .find_meta_nested("repr")
            .words()
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| syn::Error::new(ty.span, "Unit-only enums must have a fixed repr"))?;

        let ty_names = vec![&ty.name; self.variants.len()];
        let variant_names = self.variants.iter().map(|variant| &variant.name);
        let values = self.variants.values();

        let tokens = quote::quote! {
            fn code() -> u8 { <#repr>::code() }
            fn signature() -> String { <#repr>::signature() }
            fn alignment() -> u8 { <#repr>::alignment() }

            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + AsMut<[u8]> + std::io::Write
            {
                (*self as #repr).encode(marshaller)
            }

            fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                let value = <#repr>::decode(marshaller)?;
                match value {
                    #(#values => Ok(#ty_names::#variant_names),)*
                    value => Err(#rbus_module::Error::InvalidVariant { value: value as u64, }),
                }
            }
        };

        Ok(tokens)
    }

    fn gen_index_body(&self, ty: &DeriveTypeDef, index: Metas) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_meta_nested("dbus").find_rbus_module("rbus");
        let index_ty =
            index.words().first().cloned().ok_or_else(|| {
                syn::Error::new(ty.span, "Unit-only enums must have a fixed repr")
            })?;
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

        let tokens = quote::quote! {
            fn code() -> u8 { b'r' }
            fn signature() -> String { format!("({}v)", <#index_ty>::signature()) }
            fn alignment() -> u8 { 8 }

            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + AsMut<[u8]> + std::io::Write
            {
                use #rbus_module::types::Signature;

                match self {
                    #variant_encodes
                }
            }

            fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                use #rbus_module::types::Signature;

                let value = marshaller.io().read_u8()?;

                let signature = Signature::decode(marshaller)?;
                let signature = signature.as_str();

                #variant_decodes
                Err(#rbus_module::Error::Custom {
                    message: "Bad variant value".into(),
                })
            }
        };

        Ok(tokens)
    }

    fn gen_encode_method(&self, ty: &DeriveTypeDef) -> TokenStream {
        let rbus_module = ty.metas.find_meta_nested("dbus").find_rbus_module("rbus");
        let variants = self.variants.iter().map(|variant| {
            let body = self.gen_encode_variant_body(variant);
            self.gen_encode_variant(ty, variant, body)
        });

        let tokens = quote::quote! {
            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + AsMut<[u8]> + std::io::Write
            {
                use #rbus_module::types::Signature;

                match self {
                    #(#variants)*
                }
            }
        };

        tokens
    }

    fn gen_encode_variant(
        &self,
        ty: &DeriveTypeDef,
        variant: &Variant,
        body: TokenStream,
    ) -> TokenStream {
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

    fn gen_decode_method(&self, ty: &DeriveTypeDef) -> TokenStream {
        let rbus_module = ty.metas.find_meta_nested("dbus").find_rbus_module("rbus");
        let variants = self.variants.iter().map(|variant| {
            let body = self.gen_decode_variant_body(ty, variant);
            self.gen_decode_variant(variant, body)
        });

        let tokens = quote::quote! {
            fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                use #rbus_module::types::Signature;

                let signature = Signature::decode(marshaller)?;
                let signature = signature.as_str();

                #(#variants)*
                Err(#rbus_module::Error::Custom {
                    message: "Bad variant value".into(),
                })
            }
        };

        tokens
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
        let variants = data
            .variants
            .into_pairs()
            .map(Pair::into_value)
            .collect::<Vec<_>>();
        let variants = Variants::try_from(variants)?;

        Ok(DeriveEnum { variants })
    }
}
