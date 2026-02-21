//! Shared operator dispatch for secret custom value types.
//!
//! Provides generic helpers that wire `PartialEq` and `PartialOrd` implementations
//! to Nushell's `CustomValue::operation()` trait method, supporting `==`, `!=`,
//! `<`, `>`, `<=`, and `>=` comparisons between secrets of the same type.

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

/// Dispatches ordering comparisons for a secret custom value type.
///
/// Handles `Comparison::LessThan`, `GreaterThan`, `LessThanOrEqual`, and
/// `GreaterThanOrEqual` by downcasting the right-hand operand to `T` and
/// delegating to the type's `PartialOrd` implementation. Returns an error
/// when `partial_cmp` yields `None` (e.g., NaN floats) or when the
/// right-hand side is not a matching secret type.
#[allow(clippy::result_large_err)] // Matches CustomValue::operation() return type
pub(crate) fn secret_ordering_operation<T: PartialOrd + 'static>(
    lhs: &T,
    lhs_span: Span,
    operator: Operator,
    op: Span,
    right: &Value,
    type_name: &str,
) -> Result<Value, ShellError> {
    let cmp = match operator {
        Operator::Comparison(
            Comparison::LessThan
            | Comparison::GreaterThan
            | Comparison::LessThanOrEqual
            | Comparison::GreaterThanOrEqual,
        ) => operator,
        _ => {
            return Err(ShellError::GenericError {
                error: format!("Operator {operator:?} is not supported for {type_name}"),
                msg: String::new(),
                span: Some(op),
                help: None,
                inner: vec![],
            });
        }
    };

    let other = if let Value::Custom { val, .. } = right {
        val.as_any().downcast_ref::<T>()
    } else {
        None
    };

    let Some(other) = other else {
        return Err(ShellError::GenericError {
            error: format!("Ordering comparison requires two {type_name} values"),
            msg: "right-hand side is not the same secret type".into(),
            span: Some(op),
            help: None,
            inner: vec![],
        });
    };

    let ordering = lhs
        .partial_cmp(other)
        .ok_or_else(|| ShellError::GenericError {
            error: format!("Values are not orderable for {type_name}"),
            msg: "comparison is undefined (e.g., NaN)".into(),
            span: Some(op),
            help: None,
            inner: vec![],
        })?;

    let result = match cmp {
        Operator::Comparison(Comparison::LessThan) => ordering.is_lt(),
        Operator::Comparison(Comparison::GreaterThan) => ordering.is_gt(),
        Operator::Comparison(Comparison::LessThanOrEqual) => ordering.is_le(),
        Operator::Comparison(Comparison::GreaterThanOrEqual) => ordering.is_ge(),
        _ => unreachable!(),
    };

    Ok(Value::bool(result, lhs_span))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SecretFloat, SecretInt, SecretString};
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

    // ── secret_ordering_operation tests ────────────────────────────────────

    #[test]
    fn less_than_with_smaller_lhs_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(1);
        let b = SecretInt::new(2);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn less_than_with_larger_lhs_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(5);
        let b = SecretInt::new(3);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn less_than_with_equal_values_returns_false() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(7);
        let b = SecretInt::new(7);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(false, lhs_span));
    }

    #[test]
    fn greater_than_with_larger_lhs_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(10);
        let b = SecretInt::new(3);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::GreaterThan),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn less_than_or_equal_with_equal_values_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(42);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThanOrEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn greater_than_or_equal_with_equal_values_returns_true() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let b = SecretInt::new(42);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::GreaterThanOrEqual),
            op_span,
            &right,
            "secret_int",
        )
        .unwrap();

        assert_eq!(result, Value::bool(true, lhs_span));
    }

    #[test]
    fn ordering_with_different_type_returns_error() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let right = Value::int(42, lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_int",
        );

        assert!(result.is_err());
    }

    #[test]
    fn ordering_with_different_secret_type_returns_error() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let other = SecretString::new("hello".to_string());
        let right = Value::custom(Box::new(other), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_int",
        );

        assert!(result.is_err());
    }

    #[test]
    fn ordering_nan_float_returns_error() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretFloat::new(f64::NAN);
        let b = SecretFloat::new(1.0);
        let right = Value::custom(Box::new(b), lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Comparison(Comparison::LessThan),
            op_span,
            &right,
            "secret_float",
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not orderable"));
    }

    #[test]
    fn ordering_unsupported_operator_returns_error() {
        let (lhs_span, op_span) = test_spans();
        let a = SecretInt::new(42);
        let right = Value::int(1, lhs_span);

        let result = secret_ordering_operation(
            &a,
            lhs_span,
            Operator::Math(Math::Add),
            op_span,
            &right,
            "secret_int",
        );

        assert!(result.is_err());
    }
}
