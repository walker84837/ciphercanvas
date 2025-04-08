use log::error;
use std::borrow::Cow;
use toml::Value;

/// Get a string value from the config
/// Returns the default if the value is not found
/// # Example:
/// ```rust
/// use ciphercanvas::get_config_str;
/// let config = toml::from_str(r#"
/// [server]
/// host = "127.0.0.1"
/// "#).unwrap();
/// let host = get_config_str(&config, "server", "host", "127.0.0.1");
/// assert_eq!(host, "127.0.0.1");
/// ```
pub fn get_config_str<'a, D>(
    config: &'a Value,
    key: &str,
    second_key: &str,
    default: D,
) -> Cow<'a, str>
where
    D: Into<Cow<'a, str>>,
{
    match config.get(key).and_then(|k| k.get(second_key)) {
        Some(v) => v.as_str()
            .map(Cow::from)
            .unwrap_or_else(|| {
                panic!(
                    "Configuration error: Expected a string value for '{}.{}', but found a different type.",
                    key, second_key
                )
            }),
        None => {
            let default_cow = default.into();
            error!(
                "Configuration error: Unable to find the string value for '{}.{}'. Using default: '{}'.",
                key, second_key, default_cow
            );
            default_cow
        }
    }
}

/// Get an integer value from the config
/// Returns the default if the value is not found
/// # Example:
/// ```rust
/// use ciphercanvas::get_config_int;
/// let config = toml::from_str(r#"
/// [server]
/// port = 8080
/// "#).unwrap();
/// let port = get_config_int(&config, "server", "port", 8080);
/// assert_eq!(port, 8080);
/// ```
pub fn get_config_int(
    config: &Value,
    key: impl AsRef<str>,
    second_key: impl AsRef<str>,
    default: impl Into<i64>,
) -> i64 {
    let key_ref = key.as_ref();
    let second_key_ref = second_key.as_ref();
    let default_val = default.into();

    config
        .get(key_ref)
        .and_then(|k| k.get(second_key_ref))
        .and_then(|v| v.as_integer())
        .unwrap_or_else(|| {
            error!(
                "Configuration error: Unable to find the integer value for '{}.{}'. Using default: '{}'.",
                key_ref, second_key_ref, default_val
            );
            default_val
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_str_found() {
        let config = toml::from_str(
            r#"
            [server]
            host = "127.0.0.1"
        "#,
        )
        .unwrap();
        let host = get_config_str(&config, "server", "host", "localhost");
        assert_eq!(host, "127.0.0.1");
    }

    #[test]
    fn test_get_config_str_not_found() {
        let config = toml::from_str(
            r#"
            [server]
            port = 8080
        "#,
        )
        .unwrap();
        let host = get_config_str(&config, "server", "host", "localhost");
        assert_eq!(host, "localhost");
    }

    #[test]
    fn test_get_config_str_default() {
        let config = toml::from_str(
            r#"
            [server]
            host = "127.0.0.1"
        "#,
        )
        .unwrap();
        let host = get_config_str(&config, "server", "host", "localhost");
        assert_eq!(host, "127.0.0.1");
    }

    #[test]
    fn test_get_config_int_found() {
        let config = toml::from_str(
            r#"
            [server]
            port = 8080
        "#,
        )
        .unwrap();
        let port = get_config_int(&config, "server", "port", 8081);
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_get_config_int_not_found() {
        let config = toml::from_str(
            r#"
            [server]
            host = "127.0.0.1"
        "#,
        )
        .unwrap();
        let port = get_config_int(&config, "server", "port", 8081);
        assert_eq!(port, 8081);
    }

    #[test]
    fn test_get_config_int_default() {
        let config = toml::from_str(
            r#"
            [server]
            port = 8080
        "#,
        )
        .unwrap();
        let port = get_config_int(&config, "server", "port", 8081);
        assert_eq!(port, 8080);
    }

    #[test]
    #[should_panic]
    fn test_get_config_str_invalid_type() {
        let config = toml::from_str(
            r#"
            [server]
            host = 8080
        "#,
        )
        .unwrap();
        get_config_str(&config, "server", "host", "localhost");
    }

    #[test]
    #[should_panic]
    fn test_get_config_invalid_config() {
        let _: toml::Value = toml::from_str(
            r#"
            i use neovim btw
        "#,
        )
        .unwrap();
    }
}
