fn main() {
    println!("cargo:rerun-if-changed=lib/jlinker/bin/libjlinker.a");
    println!("cargo:rustc-link-search=./lib/jlinker/bin/");
    println!("cargo:rustc-link-search=./lib/keystone/build/llvm/lib");
    println!("cargo:rustc-link-lib=jlinker");
    println!("cargo:rustc-link-lib=keystone");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=bfd");
}