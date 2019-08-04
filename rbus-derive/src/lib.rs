#![recursion_limit = "256"]
#![allow(clippy::cast_lossless)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_utils::impl_macro_input;

mod ext;
mod types;
mod utils;

#[doc(hidden)]
#[proc_macro]
pub fn impl_basic_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(?types::impl_basic_type, item)
}

#[proc_macro]
pub fn impl_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(?types::impl_type, item)
}

#[proc_macro_derive(DBusType, attributes(dbus))]
pub fn derive_dbus_type(item: TokenStream) -> TokenStream {
    impl_macro_input!(?types::derive_type, item)
}
