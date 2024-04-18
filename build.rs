fn main() {
    let lib_path =
        ::std::env::var("GREIO_LIB_PATH").expect("Please provide the 'GREIO_LIB_PATH' env var");
    println!("cargo:rustc-link-lib=static=greio");
    println!("cargo:rustc-link-search={}", lib_path);
}
