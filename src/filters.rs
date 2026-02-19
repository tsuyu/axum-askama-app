use std::fmt;
use std::sync::OnceLock;

static BASE_PATH: OnceLock<String> = OnceLock::new();

/// Call this once at startup with the value of APP_BASE_PATH (e.g. "/boilerplate").
/// Trailing slash is already stripped by the caller.
pub fn init(base_path: String) {
    BASE_PATH.set(base_path).ok();
}

/// Askama custom filter: prepend APP_BASE_PATH to any internal URL path.
///
/// Template usage:
///   href="{{ "/login"|url }}"
///   action="{{ form_action|url }}"
pub fn url(path: impl fmt::Display) -> askama::Result<String> {
    let base = BASE_PATH.get().map(|s| s.as_str()).unwrap_or("");
    Ok(format!("{}{}", base, path))
}

/// For use in controllers: prepend APP_BASE_PATH to a path string.
///
/// Controller usage:
///   Redirect::to(&filters::path("/login"))
pub fn path(p: &str) -> String {
    let base = BASE_PATH.get().map(|s| s.as_str()).unwrap_or("");
    format!("{}{}", base, p)
}
