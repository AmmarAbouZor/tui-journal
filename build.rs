fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=backend/src/sqlite/migrations");

    // Make sure one feature at least is enabled
    #[cfg(all(not(feature = "json"), not(feature = "sqlite")))]
    panic!("One feature at least must be enabled");
}
