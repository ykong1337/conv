fn main() {
    let current = std::env::current_dir().unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        current.join("whisper").display()
    );
    println!("cargo:rustc-link-lib=static=whisper");
    println!("cargo:rustc-link-lib=static=ggml");
}
