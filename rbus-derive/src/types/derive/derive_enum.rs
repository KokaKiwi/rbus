use super::DeriveTypeDef;
use super::{Fields, Variant, Variants};
use crate::utils::DBusMetas;
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
        let encode_method = self.gen_encode_method(ty)?;
        let decode_method = self.gen_decode_method(ty)?;

        let body = quote::quote! {
            fn code() -> u8 { b'v' }
            fn signature() -> String { "v".into() }
            fn alignment() -> u8 { 1 }

            #encode_method
            #decode_method
        };

        Ok(body)
    }

    fn gen_encode_method(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_rbus_module("rbus")?;
        let variants = self
            .variants
            .iter()
            .map(|variant| self.gen_encode_variant(ty, variant))
            .collect::<Result<Vec<_>>>()?;

        let tokens = quote::quote! {
            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + std::io::Write
            {
                use #rbus_module::types::Signature;

                match self {
                    #(#variants)*
                }
                Ok(())
            }
        };

        Ok(tokens)
    }

    fn gen_encode_variant(&self, ty: &DeriveTypeDef, variant: &Variant) -> Result<TokenStream> {
        let ty_name = &ty.name;
        let variant_name = &variant.name;
        let pat = variant.fields.pat();
        let names = variant.fields.pat_names();
        let signature = variant.fields.signature();
        let alignment = vec![variant.fields.alignment(); names.len()];

        let tokens = quote::quote! {
            #ty_name::#variant_name #pat => {
                let signature = Signature::new(#signature)?;
                signature.encode(marshaller)?;

                #(
                marshaller.write_padding(#alignment)?;
                #names.encode(marshaller)?;
                )*
            }
        };

        Ok(tokens)
    }

    fn gen_decode_method(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_rbus_module("rbus")?;
        let variants = self
            .variants
            .iter()
            .map(|variant| self.gen_decode_variant(ty, variant))
            .collect::<Result<Vec<_>>>()?;

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

        Ok(tokens)
    }

    fn gen_decode_variant(&self, ty: &DeriveTypeDef, variant: &Variant) -> Result<TokenStream> {
        let ty_name = &ty.name;
        let variant_name = &variant.name;
        let signature = variant.fields.signature();
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

        let tokens = quote::quote! {
            if signature == #signature {
                return Ok(#construct);
            }
        };

        Ok(tokens)
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
