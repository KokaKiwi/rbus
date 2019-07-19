use super::DeriveTypeDef;
use proc_macro2::TokenStream;
use syn::parse::Result;
use syn::punctuated::Pair;

#[derive(Debug, Clone)]
pub struct DeriveEnum {
    variants: Vec<syn::Variant>,
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

    fn gen_encode_method(&self, _ty: &DeriveTypeDef) -> Result<TokenStream> {
        let tokens = quote::quote! {};

        Ok(tokens)
    }

    fn gen_decode_method(&self, _ty: &DeriveTypeDef) -> Result<TokenStream> {
        let tokens = quote::quote! {};

        Ok(tokens)
    }
}

impl From<syn::DataEnum> for DeriveEnum {
    fn from(data: syn::DataEnum) -> Self {
        DeriveEnum {
            variants: data.variants.into_pairs().map(Pair::into_value).collect(),
        }
    }
}
