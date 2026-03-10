use std::env;

fn main() {
    let lib_dir = env::var("DEP_DUCKLINGFFI_LIB_DIR")
        .expect("DEP_DUCKLINGFFI_LIB_DIR not set — is rust_duckling_host a dependency?");

    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
}