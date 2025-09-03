//! Tera template functions for redaction templates
//!
//! This module provides custom Tera template functions that can be used in
//! redaction templates for flexible secret formatting.
//!
//! Available functions:
//! - `replicate(character="*", length=5)`: Returns a string of repeated characters
//! - `reverse(s="text")`: Returns the input string reversed
//! - `take(n=5, s="text")`: Returns the first n characters of input string
//! - `strlen(s="text")`: Returns the length of the input string as a number
//! - `secret_string()`: Returns the actual secret value (WARNING: exposes sensitive data!)

use std::collections::HashMap;
use tera::{Error as TeraError, Result as TeraResult, Value as TeraValue};

/// Register the replicate function with a Tera instance
/// Returns a string of the given character repeated length times
pub fn register_replicate_function(tera: &mut tera::Tera) {
    tera.register_function("replicate", replicate_function);
}

/// Register the reverse function with a Tera instance
/// Returns the input string reversed
pub fn register_reverse_function(tera: &mut tera::Tera) {
    tera.register_function("reverse", reverse_function);
}

/// Register the take function with a Tera instance
/// Returns the first n characters of the input string
pub fn register_take_function(tera: &mut tera::Tera) {
    tera.register_function("take", take_function);
}

/// Register the strlen function with a Tera instance
/// Returns the length of the input string as a number
pub fn register_strlen_function(tera: &mut tera::Tera) {
    tera.register_function("strlen", strlen_function);
}

/// Register the secret_string function with a Tera instance
/// Returns the actual secret value as a string (WARNING: exposes sensitive data!)
pub fn register_secret_string_function(tera: &mut tera::Tera, secret_value: String) {
    tera.register_function(
        "secret_string",
        move |_args: &HashMap<String, TeraValue>| -> TeraResult<TeraValue> {
            Ok(TeraValue::String(secret_value.clone()))
        },
    );
}

/// Register the secret_string function with empty value (for None cases)
/// Returns empty string when called
pub fn register_secret_string_function_empty(tera: &mut tera::Tera) {
    tera.register_function(
        "secret_string",
        |_args: &HashMap<String, TeraValue>| -> TeraResult<TeraValue> {
            Ok(TeraValue::String("".to_string()))
        },
    );
}

/// Register all standard template functions (excluding secret_string)
pub fn register_all_standard_functions(tera: &mut tera::Tera) {
    register_replicate_function(tera);
    register_reverse_function(tera);
    register_take_function(tera);
    register_strlen_function(tera);
}

/// Replicate function implementation
fn replicate_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    let character = args
        .get("character")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeraError::msg("replicate function requires 'character' parameter"))?;

    let length = args
        .get("length")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| TeraError::msg("replicate function requires 'length' parameter"))?;

    if length < 0 {
        return Ok(TeraValue::String("".to_string()));
    }

    let mask_char = character.chars().next().unwrap_or('*');
    let result = mask_char.to_string().repeat(length as usize);
    Ok(TeraValue::String(result))
}

/// Reverse function implementation
fn reverse_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    // Support both positional and named arguments
    let input = if let Some(value) = args.get("0") {
        // First positional argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("reverse function argument must be a string"))?
    } else if let Some(value) = args.get("s") {
        // Named argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("reverse function 's' parameter must be a string"))?
    } else {
        return Err(TeraError::msg(
            "reverse function requires a string argument",
        ));
    };

    let reversed: String = input.chars().rev().collect();
    Ok(TeraValue::String(reversed))
}

/// Take function implementation
fn take_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    // Support both positional and named arguments
    let n = if let Some(value) = args.get("0") {
        // First positional argument
        value
            .as_i64()
            .ok_or_else(|| TeraError::msg("take function first argument must be a number"))?
    } else if let Some(value) = args.get("n") {
        // Named argument
        value
            .as_i64()
            .ok_or_else(|| TeraError::msg("take function 'n' parameter must be a number"))?
    } else {
        return Err(TeraError::msg(
            "take function requires first argument to be a number",
        ));
    };

    let s = if let Some(value) = args.get("1") {
        // Second positional argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("take function second argument must be a string"))?
    } else if let Some(value) = args.get("s") {
        // Named argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("take function 's' parameter must be a string"))?
    } else {
        return Err(TeraError::msg(
            "take function requires second argument to be a string",
        ));
    };

    if n < 0 {
        return Ok(TeraValue::String("".to_string()));
    }

    let chars: Vec<char> = s.chars().collect();
    let taken: String = chars.into_iter().take(n as usize).collect();
    Ok(TeraValue::String(taken))
}

/// Strlen function implementation
fn strlen_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    // Support both positional and named arguments
    let input = if let Some(value) = args.get("0") {
        // First positional argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("strlen function argument must be a string"))?
    } else if let Some(value) = args.get("s") {
        // Named argument
        value
            .as_str()
            .ok_or_else(|| TeraError::msg("strlen function 's' parameter must be a string"))?
    } else {
        return Err(TeraError::msg("strlen function requires a string argument"));
    };

    let length = input.chars().count();
    Ok(TeraValue::from(length))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Tera;

    #[test]
    fn test_replicate_function_direct() {
        let mut args = HashMap::new();
        args.insert("character".to_string(), TeraValue::String("*".to_string()));
        args.insert("length".to_string(), TeraValue::Number(5.into()));

        let result = replicate_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "*****");
    }

    #[test]
    fn test_reverse_function_direct() {
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("hello".to_string()));

        let result = reverse_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "olleh");
    }

    #[test]
    fn test_take_function_direct() {
        let mut args = HashMap::new();
        args.insert("n".to_string(), TeraValue::Number(3.into()));
        args.insert("s".to_string(), TeraValue::String("hello".to_string()));

        let result = take_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hel");
    }

    #[test]
    fn test_strlen_function_direct() {
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("hello".to_string()));

        let result = strlen_function(&args).unwrap();
        assert_eq!(result.as_i64().unwrap(), 5);

        // Test with Unicode
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("🚀🎉🌟".to_string()));

        let result = strlen_function(&args).unwrap();
        assert_eq!(result.as_i64().unwrap(), 3);
    }

    #[test]
    fn test_function_registration() {
        let mut tera = Tera::default();

        register_all_standard_functions(&mut tera);

        // Test that functions are registered by trying to use them
        tera.add_raw_template("test", "{{strlen(s='test')}}")
            .unwrap();
        let context = tera::Context::new();
        let result = tera.render("test", &context).unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_secret_string_function_registration() {
        let mut tera = Tera::default();

        register_secret_string_function(&mut tera, "secret_value".to_string());

        tera.add_raw_template("test", "{{secret_string()}}")
            .unwrap();
        let context = tera::Context::new();
        let result = tera.render("test", &context).unwrap();
        assert_eq!(result, "secret_value");
    }
}
