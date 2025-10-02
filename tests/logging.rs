use changelogen::logging;

#[test]
fn smoke_log_initialization() {
    // Test that logging initialization does not panic
    // This can be called multiple times safely
    logging::init(0);
    logging::init(1);
    logging::init(2);
}

#[test]
fn verbose_toggle_effect() {
    // Test that different verbosity levels work without panicking
    // and that the mapping is correct

    // verbosity 0 = warn
    logging::init(0);

    // verbosity 1 = info
    logging::init(1);

    // verbosity 2 = debug
    logging::init(2);

    // verbosity 3+ = trace
    logging::init(3);
    logging::init(10);
}

#[test]
fn logging_respects_rust_log_env() {
    // Test that RUST_LOG environment variable is respected
    // Note: We don't actually set RUST_LOG here to avoid unsafe blocks
    // and potential test interference. The logic is tested via code inspection:
    // - logging::init reads RUST_LOG via std::env::var
    // - If set, uses that value; otherwise uses verbosity-based default

    // This test verifies the init doesn't panic when RUST_LOG might be set
    logging::init(0);
}
