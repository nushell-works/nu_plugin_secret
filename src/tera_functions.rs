//! Tera template functions for redaction templates
//!
//! This module provides custom Tera template functions that can be used in
//! redaction templates for flexible secret formatting.
//!
//! Available functions:
//! - `replicate(s="*", n=5)`: Returns a string of repeated characters
//! - `reverse(s="text")`: Returns the input string reversed
//! - `take(n=5, s="text")`: Returns the first n characters of input string
//! - `strlen(s="text")`: Returns the length of the input string as a number
//! - `mask_partial(s="text", l=2, r=2)`: Masks the middle portion of a string, keeping specified characters from left and right (l and r default to 0, optional c="*" for custom masking character)
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

/// Register the mask_partial function with a Tera instance
/// Masks the middle portion of a string, keeping specified characters from left and right
pub fn register_mask_partial_function(tera: &mut tera::Tera) {
    tera.register_function("mask_partial", mask_partial_function);
}

/// Register all standard template functions (excluding secret_string)
pub fn register_all_standard_functions(tera: &mut tera::Tera) {
    register_replicate_function(tera);
    register_reverse_function(tera);
    register_take_function(tera);
    register_strlen_function(tera);
    register_mask_partial_function(tera);
}

/// Replicate function implementation
fn replicate_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    let s = args
        .get("s")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeraError::msg("replicate function requires 's' parameter"))?;

    let n = args
        .get("n")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| TeraError::msg("replicate function requires 'n' parameter"))?;

    if n < 0 {
        return Ok(TeraValue::String("".to_string()));
    }

    let result = s.repeat(n as usize);
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

/// Mask_partial function implementation
/// Masks the middle portion of a string, keeping l characters from the left and r characters from the right
fn mask_partial_function(args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
    let l = args.get("l").and_then(|v| v.as_i64()).unwrap_or(0);

    let r = args.get("r").and_then(|v| v.as_i64()).unwrap_or(0);

    let c = args.get("c").and_then(|v| v.as_str()).unwrap_or("*");

    let s = args
        .get("s")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeraError::msg("mask_partial function requires 's' parameter"))?;

    if l < 0 || r < 0 {
        return Err(TeraError::msg(
            "mask_partial function requires non-negative l and r parameters",
        ));
    }

    let chars: Vec<char> = s.chars().collect();
    let total_len = chars.len();
    let l_usize = l as usize;
    let r_usize = r as usize;

    // If l + r >= total length, return the original string (nothing to mask)
    if l_usize + r_usize >= total_len {
        return Ok(TeraValue::String(s.to_string()));
    }

    // Calculate the number of characters to mask in the middle
    let middle_len = total_len - l_usize - r_usize;

    // Take l characters from the left
    let left_part: String = chars.iter().take(l_usize).collect();

    // Take r characters from the right
    let right_part: String = chars.iter().skip(total_len - r_usize).collect();

    // Create the masked middle portion
    let middle_part = c.repeat(middle_len);

    // Combine all parts
    let result = format!("{}{}{}", left_part, middle_part, right_part);
    Ok(TeraValue::String(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Tera;

    #[test]
    fn test_replicate_function_direct() {
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("*".to_string()));
        args.insert("n".to_string(), TeraValue::Number(5.into()));

        let result = replicate_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "*****");

        // Test with multi-character string
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("**".to_string()));
        args.insert("n".to_string(), TeraValue::Number(4.into()));

        let result = replicate_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "********");
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
    fn test_mask_partial_function_direct() {
        // Test basic functionality with default masking character
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("s".to_string(), TeraValue::String("abcdefgh".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "ab****gh");

        // Test basic functionality with explicit masking character
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert("s".to_string(), TeraValue::String("abcdefgh".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "ab****gh");

        // Test with different left and right values
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(1.into()));
        args.insert("r".to_string(), TeraValue::Number(3.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert(
            "s".to_string(),
            TeraValue::String("password123".to_string()),
        );

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "p*******123");

        // Test edge case: l + r >= string length
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(3.into()));
        args.insert("r".to_string(), TeraValue::Number(3.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert("s".to_string(), TeraValue::String("hello".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello"); // Original string returned

        // Test with zero left
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(0.into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert("s".to_string(), TeraValue::String("secret".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "****et");

        // Test with zero right
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number(0.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert("s".to_string(), TeraValue::String("secret".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "se****");

        // Test with Unicode characters
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(1.into()));
        args.insert("r".to_string(), TeraValue::Number(1.into()));
        args.insert("c".to_string(), TeraValue::String("*".to_string()));
        args.insert("s".to_string(), TeraValue::String("🚀🎉🌟🔥⭐".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "🚀***⭐");

        // Test with different masking character
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("c".to_string(), TeraValue::String("#".to_string()));
        args.insert("s".to_string(), TeraValue::String("password".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "pa####rd");

        // Test with multi-character mask
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(1.into()));
        args.insert("r".to_string(), TeraValue::Number(1.into()));
        args.insert("c".to_string(), TeraValue::String("-X-".to_string()));
        args.insert("s".to_string(), TeraValue::String("test123".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "t-X--X--X--X--X-3");
    }

    #[test]
    fn test_mask_partial_function_defaults() {
        // Test with only s parameter (l and r default to 0)
        let mut args = HashMap::new();
        args.insert("s".to_string(), TeraValue::String("test".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "****");

        // Test missing l parameter (defaults to 0)
        let mut args = HashMap::new();
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("s".to_string(), TeraValue::String("test".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "**st");

        // Test missing r parameter (defaults to 0)
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("s".to_string(), TeraValue::String("test".to_string()));

        let result = mask_partial_function(&args).unwrap();
        assert_eq!(result.as_str().unwrap(), "te**");
    }

    #[test]
    fn test_mask_partial_function_errors() {
        // Test missing s parameter
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));

        let result = mask_partial_function(&args);
        assert!(result.is_err());

        // Test negative l parameter
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number((-1).into()));
        args.insert("r".to_string(), TeraValue::Number(2.into()));
        args.insert("s".to_string(), TeraValue::String("test".to_string()));

        let result = mask_partial_function(&args);
        assert!(result.is_err());

        // Test negative r parameter
        let mut args = HashMap::new();
        args.insert("l".to_string(), TeraValue::Number(2.into()));
        args.insert("r".to_string(), TeraValue::Number((-1).into()));
        args.insert("s".to_string(), TeraValue::String("test".to_string()));

        let result = mask_partial_function(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_mask_partial_function_registration() {
        let mut tera = Tera::default();

        register_mask_partial_function(&mut tera);

        // Test with default masking character (no 'c' parameter)
        tera.add_raw_template("test1", "{{mask_partial(l=2, r=2, s='abcdefgh')}}")
            .unwrap();
        let context = tera::Context::new();
        let result = tera.render("test1", &context).unwrap();
        assert_eq!(result, "ab****gh");

        // Test with explicit masking character
        tera.add_raw_template("test2", "{{mask_partial(l=2, r=2, c='*', s='abcdefgh')}}")
            .unwrap();
        let result = tera.render("test2", &context).unwrap();
        assert_eq!(result, "ab****gh");

        // Test with different masking character
        tera.add_raw_template("test3", "{{mask_partial(l=1, r=1, c='#', s='secret')}}")
            .unwrap();
        let result = tera.render("test3", &context).unwrap();
        assert_eq!(result, "s####t");

        // Test with default l and r parameters (both default to 0)
        tera.add_raw_template("test4", "{{mask_partial(s='test')}}")
            .unwrap();
        let result = tera.render("test4", &context).unwrap();
        assert_eq!(result, "****");

        // Test with only l parameter specified (r defaults to 0)
        tera.add_raw_template("test5", "{{mask_partial(l=2, s='hello')}}")
            .unwrap();
        let result = tera.render("test5", &context).unwrap();
        assert_eq!(result, "he***");

        // Test with only r parameter specified (l defaults to 0)
        tera.add_raw_template("test6", "{{mask_partial(r=2, s='world')}}")
            .unwrap();
        let result = tera.render("test6", &context).unwrap();
        assert_eq!(result, "***ld");
    }
}
