fn main() {
    // Make sure one feature at least is enabled
    #[cfg(all(not(feature = "json"), not(feature = "sqlite")))]
    panic!("One feature at least must be enabled");
}
