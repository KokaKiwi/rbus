use crate::ext::GenericsExt;
use crate::utils::{DBusMetas, Metas};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;

pub struct ImplGeneratorOptions {
    pub default_rbus_module: String,
    pub impl_basic_type: bool,
}

impl Default for ImplGeneratorOptions {
    fn default() -> ImplGeneratorOptions {
        ImplGeneratorOptions {
            default_rbus_module: "rbus".into(),
            impl_basic_type: false,
        }
    }
}

pub struct ImplGenerator {
    pub span: Span,
    pub metas: Metas,
    pub dbus: Metas,
    pub generics: syn::Generics,
    pub ty: syn::Type,
    pub methods: HashMap<String, TokenStream>,
    pub options: ImplGeneratorOptions,
}

impl ImplGenerator {
    pub fn new(
        span: Span,
        metas: Metas,
        generics: Option<syn::Generics>,
        ty: syn::Type,
    ) -> ImplGenerator {
        let dbus = metas.find_meta_nested("dbus");
        let generics = generics.unwrap_or_else(syn::Generics::empty);

        ImplGenerator {
            span,
            metas,
            dbus,
            generics,
            ty,
            methods: HashMap::with_capacity(5),
            options: Default::default(),
        }
    }

    pub fn new_ident(
        span: Span,
        metas: Metas,
        generics: Option<syn::Generics>,
        ty: syn::Ident,
    ) -> ImplGenerator {
        let generics = generics.unwrap_or_else(syn::Generics::empty);
        let (_, type_generics, _) = generics.split_for_impl();

        let ty = syn::parse_quote!(#ty #type_generics);

        ImplGenerator::new(span, metas, Some(generics), ty)
    }

    pub fn rbus_module(&self) -> syn::Ident {
        self.dbus
            .find_rbus_module(&self.options.default_rbus_module)
    }

    pub fn add_method<T: Into<String>>(&mut self, name: T, method: TokenStream) {
        self.methods.insert(name.into(), method);
    }

    pub fn gen_impl(self) -> TokenStream {
        let rbus_module = self
            .dbus
            .find_rbus_module(&self.options.default_rbus_module);
        let (impl_generics, _, where_clause) = self.generics.split_for_impl();
        let ty = &self.ty;

        let methods = self.methods.into_iter().map(|entry| entry.1);

        let dbus_type_impl = quote::quote! {
            impl #impl_generics #rbus_module::types::DBusType for #ty #where_clause {
                #(#methods)*
            }
        };

        let basic_type_impl = if let Ok(true) = self.dbus.has_meta_word("basic") {
            quote::quote! {
                impl #impl_generics #rbus_module::types::DBusBasicType for #ty #where_clause {}
            }
        } else {
            quote::quote!()
        };

        quote::quote! {
            #dbus_type_impl
            #basic_type_impl
        }
    }

    pub fn gen_code_method<Body: ToTokens>(&self, body: Body) -> TokenStream {
        quote::quote! {
            fn code() -> u8 { #body }
        }
    }

    pub fn gen_signature_method<Body: ToTokens>(&self, body: Body) -> TokenStream {
        quote::quote! {
            fn signature() -> String { #body }
        }
    }

    pub fn gen_alignment_method<Body: ToTokens>(&self, body: Body) -> TokenStream {
        quote::quote! {
            fn alignment() -> u8 { #body }
        }
    }

    pub fn gen_encode_method<Body: ToTokens>(
        &self,
        marshaller: syn::Ident,
        body: Body,
    ) -> TokenStream {
        let rbus_module = self.rbus_module();

        quote::quote! {
            fn encode<Inner>(&self, #marshaller: &mut #rbus_module::marshal::Marshaller<Inner>)
                -> #rbus_module::Result<()>
            where
                Inner: AsRef<[u8]> + AsMut<[u8]> + std::io::Write
            {
                use std::io::Write;

                #body
            }
        }
    }

    pub fn gen_decode_method<Body: ToTokens>(
        &self,
        marshaller: syn::Ident,
        body: Body,
    ) -> TokenStream {
        let rbus_module = self.rbus_module();

        quote::quote! {
            fn decode<Inner>(#marshaller: &mut #rbus_module::marshal::Marshaller<Inner>)
                -> #rbus_module::Result<Self>
            where
                Inner: AsRef<[u8]> + std::io::Read
            {
                use std::io::Read;

                #body
            }
        }
    }
}
