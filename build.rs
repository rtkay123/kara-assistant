const LIB_LINK_PATH: &str = "kara-lib";

fn main() {
    println!("cargo:rustc-link-search={LIB_LINK_PATH}");
}
