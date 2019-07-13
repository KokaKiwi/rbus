extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod types;

macro_rules! impl_macro_input {
    ($fun:path, $($arg:ident: $ty:ty),*) => {
        {
            $fun($(parse_macro_input!($arg as $ty)),*)
        }
    };
    ($fun:path, $($arg:ident),*) => ( impl_macro_input!($fun, $($arg: _),*) );
}

#[proc_macro]
pub fn impl_basic_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(types::impl_basic_type, item)
}

#[proc_macro]
pub fn impl_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(types::impl_type, item)
}

#[proc_macro_derive(DBusType, attributes(settings))]
pub fn derive_dbus_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(types::derive_type, item)
}
