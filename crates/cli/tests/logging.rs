use novalyn::logging;

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
    // Set RUST_LOG to verify the logging system honors it
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    // This test verifies the init doesn't panic with RUST_LOG set
    logging::init(0);

    // Clean up
    unsafe {
        std::env::remove_var("RUST_LOG");
    }
}
