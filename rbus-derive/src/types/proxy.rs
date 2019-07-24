use super::ImplGenerator;
use crate::utils::Metas;
use proc_macro2::{Span, TokenStream};
use syn::Result;

pub fn gen_proxy_methods(gen: &ImplGenerator, span: Span, proxy: Metas) -> Result<Vec<(&'static str, TokenStream)>> {
    let proxy_ty = proxy
        .words()
        .next()
        .ok_or_else(|| syn::Error::new(span, "You must define a proxy type"))?;
    let getter: TokenStream = proxy
        .find_meta_value_parse::<syn::Ident>("get")?
        .map(|getter| quote::quote!(self.#getter()))
        .unwrap_or_else(|| quote::quote!(self));
    let setter: TokenStream = proxy
        .find_meta_value_parse::<syn::Ident>("set")?
        .map(|setter| quote::quote!(Self::#setter(value)))
        .unwrap_or_else(|| quote::quote!(value));

    Ok(vec![
        ("code", gen.gen_code_method(quote::quote!(<#proxy_ty>::code()), None)),
        (
            "signature",
            gen.gen_signature_method(quote::quote!(<#proxy_ty>::signature()), None),
        ),
        (
            "alignment",
            gen.gen_alignment_method(quote::quote!(<#proxy_ty>::alignment()), None),
        ),
        (
            "encode",
            gen.gen_encode_method(
                syn::parse_quote!(marshaller),
                quote::quote!(#getter.encode(marshaller)),
                None,
            ),
        ),
        (
            "decode",
            gen.gen_decode_method(
                syn::parse_quote!(marshaller),
                quote::quote! {
                    let value = <#proxy_ty>::decode(marshaller)?;
                    Ok(#setter)
                },
                None,
            ),
        ),
    ])
}
