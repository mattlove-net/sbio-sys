fn main() {
    let lib_path =
        ::std::env::var("SBIO_LIB_PATH").expect("Please provide the 'SBIO_LIB_PATH' env var");
    println!("cargo:rustc-link-lib=dylib=greio");
    println!("cargo:rustc-link-search={}", lib_path);
}
