use crate::utils::*;
pub use basic::impl_basic_type;
pub use derive::*;
pub use gen::ImplGenerator;
use method::Methods;
use proc_macro2::{Span, TokenStream};
pub use proxy::gen_proxy_methods;
use syn::parse::{Parse, ParseStream, Result};

mod basic;
mod derive;
mod gen;
mod method;
mod proxy;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    metas: Metas,
    generics: syn::Generics,
    ty: syn::Type,
    code: Option<syn::LitChar>,
    methods: Methods,
}

impl TypeDef {
    fn impl_type(self) -> Result<TokenStream> {
        let dbus = self.metas.find_meta_nested("dbus");
        let proxy = dbus.find_meta_nested("proxy");

        let code_body = self.gen_code_body();
        let signature_body = self.gen_signature_body();
        let alignment_body = self.gen_alignment_body();

        let methods = self.methods;

        let mut gen = ImplGenerator::new(Span::call_site(), self.metas, Some(self.generics), self.ty);
        gen.options.default_rbus_module = "crate".into();

        gen.add_method("code", gen.gen_code_method(code_body, &[]));
        gen.add_method("signature", gen.gen_signature_method(signature_body, &[]));
        gen.add_method("alignment", gen.gen_alignment_method(alignment_body, &[]));

        if !proxy.is_empty() {
            let methods = gen_proxy_methods(&gen, Span::call_site(), proxy)?;
            for (name, method) in methods.into_iter() {
                gen.add_method(name, method);
            }
        }

        for method in methods.into_iter() {
            let (name, method) = method.gen_dbus_method(&gen)?;
            gen.add_method(name, method);
        }

        gen.override_methods_from_metas()?;
        gen.gen_impl()
    }

    fn code(&self) -> char {
        self.code.as_ref().map(|code| code.value()).unwrap_or('\0')
    }

    fn gen_code_body(&self) -> TokenStream {
        let code = self.code();

        quote::quote!(#code as u8)
    }

    fn gen_signature_body(&self) -> TokenStream {
        let signature = self
            .metas
            .find_meta_nested("dbus")
            .find_meta_value_str("signature")
            .map(|value| value.value())
            .unwrap_or_else(|| format!("{}", self.code()));

        quote::quote!(#signature.into())
    }

    fn gen_alignment_body(&self) -> TokenStream {
        let alignment = match self.metas.find_meta_nested("dbus").find_meta_value_lit("align") {
            Some(syn::Lit::Int(lit)) => quote::quote!(#lit as u8),
            Some(syn::Lit::Str(lit)) if lit.value() == "size" => quote::quote!(std::mem::size_of::<Self>() as u8),
            _ => quote::quote!(1),
        };

        quote::quote!(#alignment)
    }
}

impl Parse for TypeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        use crate::ext::GenericsExt;

        let attrs = input.call(Attribute::parse_many)?;
        let metas = attrs.into_iter().collect();

        let mut generics = if input.peek(syn::Token![impl]) {
            input.parse::<syn::Token![impl]>()?;
            input.parse()?
        } else {
            syn::Generics::empty()
        };

        let ty = input.parse()?;
        let code = if input.peek(syn::Token![:]) {
            input.parse::<syn::Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        generics.where_clause = input.parse()?;
        let methods = input.call(method::parse_methods)?;

        Ok(TypeDef {
            metas,
            generics,
            ty,
            code,
            methods,
        })
    }
}

pub fn impl_type(data: TypeDef) -> Result<TokenStream> {
    data.impl_type()
}
