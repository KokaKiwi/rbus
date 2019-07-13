use cbindgen::{Builder, Language};
use std::env;

fn generate_headers(lang: Language, out_filename: &str) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    Builder::new()
        .with_crate(crate_dir)
        .with_language(lang)
        .with_include_guard("rbus_h")
        .with_include_version(true)
        .generate()
        .expect("Unable to generate bindings.")
        .write_to_file(out_filename);
}

fn main() {
    generate_headers(Language::C, "rbus.h");
    generate_headers(Language::Cxx, "rbus.hpp");
}
