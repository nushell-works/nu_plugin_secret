//! Shared operator dispatch for secret custom value types.
//!
//! Provides a generic helper that wires `PartialEq` implementations to Nushell's
//! `CustomValue::operation()` trait method, supporting `==` and `!=` comparisons
//! between secrets of the same type.

use nu_protocol::{
    ast::{Comparison, Operator},
    ShellError, Span, Value,
};

/// Dispatches equality and inequality comparisons for a secret custom value type.
///
/// Handles `Comparison::Equal` and `Comparison::NotEqual` by downcasting the
/// right-hand operand to `T` and delegating to the type's `PartialEq`
/// implementation. Returns `false` for `==` (or `true` for `!=`) when the
/// right-hand side is a different type. Returns an error for unsupported operators.
#[allow(clippy::result_large_err)] // Matches CustomValue::operation() return type
pub(crate) fn secret_comparison_operation<T: PartialEq + 'static>(
    lhs: &T,
    lhs_span: Span,
    operator: Operator,
    op: Span,
    right: &Value,
    type_name: &str,
) -> Result<Value, ShellError> {
    match operator {
        Operator::Comparison(Comparison::Equal) => {
            if let Value::Custom { val, .. } = right {
                if let Some(other) = val.as_any().downcast_ref::<T>() {
                    Ok(Value::bool(lhs == other, lhs_span))
                } else {
                    Ok(Value::bool(false, lhs_span))
                }
            } else {
                Ok(Value::bool(false, lhs_span))
            }
        }
        Operator::Comparison(Comparison::NotEqual) => {
            if let Value::Custom { val, .. } = right {
                if let Some(other) = val.as_any().downcast_ref::<T>() {
                    Ok(Value::bool(lhs != other, lhs_span))
                } else {
                    Ok(Value::bool(true, lhs_span))
                }
            } else {
                Ok(Value::bool(true, lhs_span))
            }
        }
        _ => Err(ShellError::GenericError {
            error: format!("Operator {operator:?} is not supported for {type_name}"),
            msg: String::new(),
            span: Some(op),
            help: None,
            inner: vec![],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SecretInt, SecretString};
    use nu_protocol::ast::Math;

    fn test_spans() -> (Span, Span) {
        (Span::test_data(), Span::test_data())
    }

    #[test]
    fn equal_with_matching_values_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(42);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn equal_with_non_matching_values_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(99);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn not_equal_with_matching_values_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(42);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::NotEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn not_equal_with_non_matching_values_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(99);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::NotEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn equal_with_different_custom_type_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let other = SecretString::new("not an int".to_string());
        let right = Value::custom(Box::new(other), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn equal_with_non_custom_value_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let right = Value::int(42, lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::Equal),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn not_equal_with_different_custom_type_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let other = SecretString::new("not an int".to_string());
        let right = Value::custom(Box::new(other), lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::NotEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn not_equal_with_non_custom_value_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let right = Value::int(42, lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::NotEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn unsupported_operator_returns_error() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let right = Value::int(1, lhs_span);

        let result = secret_comparison_operation(
            &a,
            lhs_span,
            Operator::Math(Math::Add),
            op_span,
            &right,
            "secret_int",
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("secret_int"));
    }
}
