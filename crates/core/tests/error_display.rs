use novalyn_core::error::NovalynError;

/// Test that NovalynError::Config displays the correct error message.
#[test]
fn test_config_error_display() {
    let err = NovalynError::Config("Invalid configuration".to_string());
    assert!(err.to_string().contains("Invalid configuration"));
}

/// Test that NovalynError::Git displays the correct error message.
#[test]
fn test_git_error_display() {
    let err = NovalynError::Git("Repository not found".to_string());
    assert!(err.to_string().contains("Repository not found"));
}

/// Test that NovalynError::Io displays the correct error message.
#[test]
fn test_io_error_display() {
    let err = NovalynError::Io("File not found".to_string());
    assert!(err.to_string().contains("File not found"));
}

#[test]
fn test_semantic_error_display() {
    let err = NovalynError::Semantic("Invalid commit format".to_string());
    assert!(err.to_string().contains("Invalid commit format"));
}

#[test]
fn test_error_exit_codes() {
    let config_err = NovalynError::Config("test".to_string());
    assert_eq!(config_err.exit_code(), 2);

    let git_err = NovalynError::Git("test".to_string());
    assert_eq!(git_err.exit_code(), 4);

    let io_err = NovalynError::Io("test".to_string());
    assert_eq!(io_err.exit_code(), 5);

    let semantic_err = NovalynError::Semantic("test".to_string());
    assert_eq!(semantic_err.exit_code(), 6);
}

#[test]
fn test_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let novalyn_error: NovalynError = io_error.into();
    assert!(matches!(novalyn_error, NovalynError::Io(_)));
}

/// Test conversion from anyhow::Error to NovalynError.
#[test]
fn test_error_from_anyhow() {
    let anyhow_error = anyhow::anyhow!("some error");
    let novalyn_error: NovalynError = anyhow_error.into();
    assert!(matches!(novalyn_error, NovalynError::Semantic(_)));
}
