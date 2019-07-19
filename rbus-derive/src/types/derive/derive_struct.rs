use super::DeriveTypeDef;
use crate::utils::{split_tuples, DBusMetas};
use proc_macro2::TokenStream;
use syn::parse::Result;
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub struct DeriveStruct {
    fields: Fields,
}

impl DeriveStruct {
    pub fn gen_body(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let field_types = self.field_types();
        let signature_format_str = format!("({})", "{}".repeat(field_types.len()));

        let encode_method = self.gen_encode_method(ty)?;
        let decode_method = self.gen_decode_method(ty)?;

        let body = quote::quote! {
            fn code() -> u8 { b'r' }
            fn signature() -> String {
                format!(#signature_format_str, #(<#field_types>::signature()),*)
            }
            fn alignment() -> u8 { 8 }

            #encode_method
            #decode_method
        };

        Ok(body)
    }

    fn field_names(&self) -> Vec<TokenStream> {
        self.fields
            .to_vec()
            .into_iter()
            .map(|(name, _)| quote::quote!(#name))
            .collect()
    }

    fn field_types(&self) -> Vec<&syn::Type> {
        self.fields.to_vec().into_iter().map(|(_, ty)| ty).collect()
    }

    fn gen_encode_method(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_rbus_module("rbus")?;
        let field_names = self.field_names();

        let tokens = quote::quote! {
            fn encode<Inner>(&self, marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + std::io::Write
            {
                #(self.#field_names.encode(marshaller)?;)*
                Ok(())
            }
        };

        Ok(tokens)
    }

    fn gen_decode_method(&self, ty: &DeriveTypeDef) -> Result<TokenStream> {
        let rbus_module = ty.metas.find_rbus_module("rbus")?;

        let tokens = match self.fields {
            Fields::Named(ref fields) => {
                let (names, types) = split_tuples(&fields);

                quote::quote! {
                    fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
                    where
                        Inner: AsRef<[u8]> + std::io::Read
                    {
                        Ok(Self {
                            #(#names: #types::decode(marshaller)?,)*
                        })
                    }
                }
            }
            Fields::Unnamed(ref fields) => {
                let types = fields.iter().map(|(_, ty)| ty);

                quote::quote! {
                    fn decode<Inner>(marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
                    where
                        Inner: AsRef<[u8]> + std::io::Read
                    {
                        Ok(Self(#(#types::decode(marshaller)?),*))
                    }
                }
            }
            Fields::Unit => quote::quote! {
                fn decode<Inner>(_marshaller: &mut #rbus_module::marshal::Marshaller<Inner>) -> #rbus_module::Result<Self>
                where
                    Inner: AsRef<[u8]> + std::io::Read
                {
                    Ok(Self)
                }
            },
        };

        Ok(tokens)
    }
}

impl From<syn::DataStruct> for DeriveStruct {
    fn from(data: syn::DataStruct) -> Self {
        DeriveStruct {
            fields: data.fields.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Fields {
    Named(Vec<(syn::Ident, syn::Type)>),
    Unnamed(Vec<(syn::LitInt, syn::Type)>),
    Unit,
}

impl Fields {
    fn to_vec(&self) -> Vec<(TokenStream, &syn::Type)> {
        match self {
            Fields::Named(fields) => fields
                .iter()
                .map(|(name, ty)| (quote::quote!(#name), ty))
                .collect(),
            Fields::Unnamed(fields) => fields
                .iter()
                .map(|(pos, ty)| (quote::quote!(#pos), ty))
                .collect(),
            Fields::Unit => Vec::new(),
        }
    }
}

impl From<syn::Fields> for Fields {
    fn from(fields: syn::Fields) -> Fields {
        match fields {
            syn::Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| (field.ident.clone().unwrap(), field.ty.clone()))
                    .collect();

                Fields::Named(fields)
            }
            syn::Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(pos, field)| {
                        let pos = syn::LitInt::new(pos as u64, syn::IntSuffix::None, field.span());
                        (pos, field.ty.clone())
                    })
                    .collect();

                Fields::Unnamed(fields)
            }
            syn::Fields::Unit => Fields::Unit,
        }
    }
}