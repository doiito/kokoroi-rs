fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:warning=Build target: {}", target);
}
