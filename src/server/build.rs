fn main() {
    println!("cargo:rerun-if-changed=storage/sqlite/migrations");
}
