use mnemebrain::MnemeBrainError;

// ── MnemeBrainError::Http Display ──

#[test]
fn test_http_error_display() {
    let e = MnemeBrainError::Http {
        status: 404,
        message: "not found".into(),
    };
    assert_eq!(e.to_string(), "HTTP error (status 404): not found");
}

#[test]
fn test_http_error_display_500() {
    let e = MnemeBrainError::Http {
        status: 500,
        message: "internal server error".into(),
    };
    assert_eq!(
        e.to_string(),
        "HTTP error (status 500): internal server error"
    );
}

// ── MnemeBrainError::Other Display ──

#[test]
fn test_other_error_display() {
    let e = MnemeBrainError::Other("something went wrong".into());
    assert_eq!(e.to_string(), "something went wrong");
}

#[test]
fn test_other_error_empty_string() {
    let e = MnemeBrainError::Other("".into());
    assert_eq!(e.to_string(), "");
}

// ── MnemeBrainError::Json from serde_json::Error ──

#[test]
fn test_json_error_conversion() {
    // Force a serde_json parse error and confirm it converts to MnemeBrainError::Json.
    let serde_err: serde_json::Error =
        serde_json::from_str::<serde_json::Value>("{bad json").unwrap_err();
    let msg = serde_err.to_string();

    let mb_err: MnemeBrainError = serde_err.into();
    match mb_err {
        MnemeBrainError::Json(inner) => {
            assert_eq!(inner.to_string(), msg);
        }
        _ => panic!("expected MnemeBrainError::Json"),
    }
}

#[test]
fn test_json_error_display_prefix() {
    let serde_err: serde_json::Error =
        serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let mb_err: MnemeBrainError = serde_err.into();
    // The Display impl wraps with "JSON error: ..."
    assert!(mb_err.to_string().starts_with("JSON error: "));
}

// ── Debug representation ──

#[test]
fn test_error_debug_http() {
    let e = MnemeBrainError::Http {
        status: 403,
        message: "forbidden".into(),
    };
    let debug = format!("{:?}", e);
    assert!(debug.contains("Http"));
    assert!(debug.contains("403"));
    assert!(debug.contains("forbidden"));
}

#[test]
fn test_error_debug_other() {
    let e = MnemeBrainError::Other("oops".into());
    let debug = format!("{:?}", e);
    assert!(debug.contains("Other"));
    assert!(debug.contains("oops"));
}

// ── Result<T> type alias ──

#[test]
fn test_result_type_alias_ok() {
    let r: mnemebrain::Result<i32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[test]
fn test_result_type_alias_err() {
    let r: mnemebrain::Result<i32> = Err(MnemeBrainError::Other("fail".into()));
    assert!(r.is_err());
}
