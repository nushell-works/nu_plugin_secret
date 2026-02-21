//! Integration tests for comparison and ordering operators across secret types.

use nu_plugin_secret::{
    SecretBinary, SecretBool, SecretDate, SecretFloat, SecretInt, SecretList, SecretRecord,
    SecretString,
};
use nu_protocol::ast::{Comparison, Operator};
use nu_protocol::{CustomValue, Span, Value};

fn span() -> Span {
    Span::test_data()
}

fn eq_op() -> Operator {
    Operator::Comparison(Comparison::Equal)
}

fn ne_op() -> Operator {
    Operator::Comparison(Comparison::NotEqual)
}

fn lt_op() -> Operator {
    Operator::Comparison(Comparison::LessThan)
}

fn gt_op() -> Operator {
    Operator::Comparison(Comparison::GreaterThan)
}

fn le_op() -> Operator {
    Operator::Comparison(Comparison::LessThanOrEqual)
}

fn ge_op() -> Operator {
    Operator::Comparison(Comparison::GreaterThanOrEqual)
}

/// Calls `operation()` on `lhs` with the given operator and `rhs`, returning the boolean result.
fn compare(lhs: &dyn CustomValue, op: Operator, rhs: Value) -> bool {
    let result = lhs.operation(span(), op, span(), &rhs).unwrap();
    match result {
        Value::Bool { val, .. } => val,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

/// Calls `operation()` expecting an error result.
fn compare_err(lhs: &dyn CustomValue, op: Operator, rhs: Value) {
    let result = lhs.operation(span(), op, span(), &rhs);
    assert!(result.is_err(), "Expected error, got {:?}", result);
}

// ── SecretInt ───────────────────────────────────────────────────────────────

#[test]
fn int_equal_matching() {
    let a = SecretInt::new(42);
    let b = SecretInt::new(42);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_equal_non_matching() {
    let a = SecretInt::new(42);
    let b = SecretInt::new(99);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_equal() {
    let a = SecretInt::new(42);
    let b = SecretInt::new(99);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_equal_boundary_values() {
    let a = SecretInt::new(i64::MAX);
    let b = SecretInt::new(i64::MAX);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));

    let c = SecretInt::new(i64::MIN);
    let d = SecretInt::new(i64::MIN);
    assert!(compare(&c, eq_op(), Value::custom(Box::new(d), span())));
}

// ── SecretBool ──────────────────────────────────────────────────────────────

#[test]
fn bool_equal_matching() {
    let a = SecretBool::new(true);
    let b = SecretBool::new(true);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn bool_equal_non_matching() {
    let a = SecretBool::new(true);
    let b = SecretBool::new(false);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn bool_not_equal() {
    let a = SecretBool::new(true);
    let b = SecretBool::new(false);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

// ── SecretFloat ─────────────────────────────────────────────────────────────

#[test]
fn float_equal_matching() {
    let a = SecretFloat::new(std::f64::consts::PI);
    let b = SecretFloat::new(std::f64::consts::PI);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_equal_non_matching() {
    let a = SecretFloat::new(std::f64::consts::PI);
    let b = SecretFloat::new(std::f64::consts::E);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_not_equal() {
    let a = SecretFloat::new(1.0);
    let b = SecretFloat::new(2.0);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_nan_equals_nan() {
    let a = SecretFloat::new(f64::NAN);
    let b = SecretFloat::new(f64::NAN);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_positive_and_negative_zero_not_equal() {
    let a = SecretFloat::new(0.0);
    let b = SecretFloat::new(-0.0);
    // Bitwise comparison: +0.0 and -0.0 have different bit patterns
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

// ── SecretDate ──────────────────────────────────────────────────────────────

#[test]
fn date_equal_matching() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt);
    let b = SecretDate::new(dt);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_equal_non_matching() {
    let dt1 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let dt2 = chrono::DateTime::parse_from_rfc3339("2025-06-15T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt1);
    let b = SecretDate::new(dt2);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_not_equal() {
    let dt1 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let dt2 = chrono::DateTime::parse_from_rfc3339("2025-06-15T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt1);
    let b = SecretDate::new(dt2);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

// ── SecretBinary ────────────────────────────────────────────────────────────

#[test]
fn binary_equal_matching() {
    let a = SecretBinary::new(vec![1, 2, 3, 4]);
    let b = SecretBinary::new(vec![1, 2, 3, 4]);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn binary_equal_non_matching() {
    let a = SecretBinary::new(vec![1, 2, 3, 4]);
    let b = SecretBinary::new(vec![5, 6, 7, 8]);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn binary_not_equal() {
    let a = SecretBinary::new(vec![1, 2, 3]);
    let b = SecretBinary::new(vec![4, 5, 6]);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn binary_equal_empty() {
    let a = SecretBinary::new(vec![]);
    let b = SecretBinary::new(vec![]);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

// ── SecretList ──────────────────────────────────────────────────────────────

#[test]
fn list_equal_matching() {
    let items = vec![Value::int(1, span()), Value::string("two", span())];
    let a = SecretList::new(items.clone());
    let b = SecretList::new(items);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn list_equal_non_matching() {
    let a = SecretList::new(vec![Value::int(1, span())]);
    let b = SecretList::new(vec![Value::int(2, span())]);
    assert!(!compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn list_not_equal() {
    let a = SecretList::new(vec![Value::int(1, span())]);
    let b = SecretList::new(vec![Value::int(2, span())]);
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

// ── SecretRecord ────────────────────────────────────────────────────────────

#[test]
fn record_equal_still_works_after_refactor() {
    let rec = nu_protocol::record! {
        "key" => Value::string("value", span()),
    };
    let a = SecretRecord::new(rec.clone());
    let b = SecretRecord::new(rec);
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn record_not_equal_still_works_after_refactor() {
    let a = SecretRecord::new(nu_protocol::record! {
        "key" => Value::string("value1", span()),
    });
    let b = SecretRecord::new(nu_protocol::record! {
        "key" => Value::string("value2", span()),
    });
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

// ── SecretString ────────────────────────────────────────────────────────────

#[test]
fn string_equal_still_works_after_refactor() {
    let a = SecretString::new("secret".to_string());
    let b = SecretString::new("secret".to_string());
    assert!(compare(&a, eq_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn string_not_equal_still_works_after_refactor() {
    let a = SecretString::new("one".to_string());
    let b = SecretString::new("two".to_string());
    assert!(compare(&a, ne_op(), Value::custom(Box::new(b), span())));
}

// ── Cross-type comparison ───────────────────────────────────────────────────

#[test]
fn cross_type_equal_returns_false() {
    let int = SecretInt::new(42);
    let string = SecretString::new("42".to_string());
    assert!(!compare(
        &int,
        eq_op(),
        Value::custom(Box::new(string), span())
    ));
}

#[test]
fn cross_type_not_equal_returns_true() {
    let int = SecretInt::new(42);
    let string = SecretString::new("42".to_string());
    assert!(compare(
        &int,
        ne_op(),
        Value::custom(Box::new(string), span())
    ));
}

#[test]
fn comparison_with_plain_value_returns_false() {
    let secret = SecretInt::new(42);
    assert!(!compare(&secret, eq_op(), Value::int(42, span())));
}

// ── SecretInt ordering ─────────────────────────────────────────────────────

#[test]
fn int_less_than() {
    let a = SecretInt::new(1);
    let b = SecretInt::new(2);
    assert!(compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_less_than_when_greater() {
    let a = SecretInt::new(5);
    let b = SecretInt::new(3);
    assert!(!compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_less_than_when_equal() {
    let a = SecretInt::new(7);
    let b = SecretInt::new(7);
    assert!(!compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_greater_than() {
    let a = SecretInt::new(10);
    let b = SecretInt::new(3);
    assert!(compare(&a, gt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_greater_than_when_less() {
    let a = SecretInt::new(1);
    let b = SecretInt::new(9);
    assert!(!compare(&a, gt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_less_than_or_equal_when_less() {
    let a = SecretInt::new(1);
    let b = SecretInt::new(2);
    assert!(compare(&a, le_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_less_than_or_equal_when_equal() {
    let a = SecretInt::new(42);
    let b = SecretInt::new(42);
    assert!(compare(&a, le_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_less_than_or_equal_when_greater() {
    let a = SecretInt::new(10);
    let b = SecretInt::new(5);
    assert!(!compare(&a, le_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_greater_than_or_equal_when_greater() {
    let a = SecretInt::new(10);
    let b = SecretInt::new(3);
    assert!(compare(&a, ge_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_greater_than_or_equal_when_equal() {
    let a = SecretInt::new(42);
    let b = SecretInt::new(42);
    assert!(compare(&a, ge_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_not_greater_than_or_equal_when_less() {
    let a = SecretInt::new(1);
    let b = SecretInt::new(9);
    assert!(!compare(&a, ge_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn int_ordering_boundary_values() {
    let min = SecretInt::new(i64::MIN);
    let max = SecretInt::new(i64::MAX);
    assert!(compare(
        &min,
        lt_op(),
        Value::custom(Box::new(max.clone()), span())
    ));
    assert!(compare(&max, gt_op(), Value::custom(Box::new(min), span())));
}

#[test]
fn int_ordering_with_plain_value_returns_error() {
    let a = SecretInt::new(42);
    compare_err(&a, lt_op(), Value::int(42, span()));
}

#[test]
fn int_ordering_with_different_secret_type_returns_error() {
    let a = SecretInt::new(42);
    let b = SecretString::new("42".to_string());
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}

// ── SecretFloat ordering ───────────────────────────────────────────────────

#[test]
fn float_less_than() {
    let a = SecretFloat::new(1.0);
    let b = SecretFloat::new(2.0);
    assert!(compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_greater_than() {
    let a = SecretFloat::new(std::f64::consts::PI);
    let b = SecretFloat::new(std::f64::consts::E);
    assert!(compare(&a, gt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_less_than_or_equal_when_equal() {
    let a = SecretFloat::new(1.5);
    let b = SecretFloat::new(1.5);
    assert!(compare(&a, le_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_greater_than_or_equal_when_equal() {
    let a = SecretFloat::new(1.5);
    let b = SecretFloat::new(1.5);
    assert!(compare(&a, ge_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn float_infinity_ordering() {
    let neg_inf = SecretFloat::new(f64::NEG_INFINITY);
    let pos_inf = SecretFloat::new(f64::INFINITY);
    let normal = SecretFloat::new(0.0);

    assert!(compare(
        &neg_inf,
        lt_op(),
        Value::custom(Box::new(normal.clone()), span())
    ));
    assert!(compare(
        &pos_inf,
        gt_op(),
        Value::custom(Box::new(normal), span())
    ));
    assert!(compare(
        &neg_inf,
        lt_op(),
        Value::custom(Box::new(pos_inf), span())
    ));
}

#[test]
fn float_positive_and_negative_zero_ordering() {
    // +0.0 and -0.0 are equal in ordering even though they differ in equality (bitwise)
    let pos = SecretFloat::new(0.0);
    let neg = SecretFloat::new(-0.0);
    assert!(!compare(
        &pos,
        lt_op(),
        Value::custom(Box::new(neg.clone()), span())
    ));
    assert!(!compare(
        &pos,
        gt_op(),
        Value::custom(Box::new(neg), span())
    ));
}

#[test]
fn float_nan_ordering_returns_error() {
    let nan = SecretFloat::new(f64::NAN);
    let normal = SecretFloat::new(1.0);
    compare_err(&nan, lt_op(), Value::custom(Box::new(normal), span()));
}

#[test]
fn float_nan_ordering_rhs_returns_error() {
    let normal = SecretFloat::new(1.0);
    let nan = SecretFloat::new(f64::NAN);
    compare_err(&normal, gt_op(), Value::custom(Box::new(nan), span()));
}

#[test]
fn float_nan_ordering_both_nan_returns_error() {
    let a = SecretFloat::new(f64::NAN);
    let b = SecretFloat::new(f64::NAN);
    compare_err(&a, le_op(), Value::custom(Box::new(b), span()));
}

// ── SecretDate ordering ────────────────────────────────────────────────────

#[test]
fn date_less_than() {
    let dt1 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let dt2 = chrono::DateTime::parse_from_rfc3339("2025-06-15T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt1);
    let b = SecretDate::new(dt2);
    assert!(compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_greater_than() {
    let dt1 = chrono::DateTime::parse_from_rfc3339("2025-06-15T08:30:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let dt2 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt1);
    let b = SecretDate::new(dt2);
    assert!(compare(&a, gt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_less_than_or_equal_when_equal() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt);
    let b = SecretDate::new(dt);
    assert!(compare(&a, le_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_greater_than_or_equal_when_equal() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt);
    let b = SecretDate::new(dt);
    assert!(compare(&a, ge_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_not_less_than_when_equal() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let a = SecretDate::new(dt);
    let b = SecretDate::new(dt);
    assert!(!compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

#[test]
fn date_ordering_across_timezones() {
    // Same instant expressed in different timezones — ordering compares by UTC
    let dt1 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00+02:00").unwrap();
    let dt2 = chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00+00:00").unwrap();
    // dt1 is 10:00 UTC, dt2 is 12:00 UTC, so dt1 < dt2
    let a = SecretDate::new(dt1);
    let b = SecretDate::new(dt2);
    assert!(compare(&a, lt_op(), Value::custom(Box::new(b), span())));
}

// ── Non-orderable types still reject ordering ──────────────────────────────

#[test]
fn string_ordering_returns_error() {
    let a = SecretString::new("abc".to_string());
    let b = SecretString::new("def".to_string());
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}

#[test]
fn bool_ordering_returns_error() {
    let a = SecretBool::new(true);
    let b = SecretBool::new(false);
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}

#[test]
fn binary_ordering_returns_error() {
    let a = SecretBinary::new(vec![1, 2, 3]);
    let b = SecretBinary::new(vec![4, 5, 6]);
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}

#[test]
fn list_ordering_returns_error() {
    let a = SecretList::new(vec![Value::int(1, span())]);
    let b = SecretList::new(vec![Value::int(2, span())]);
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}

#[test]
fn record_ordering_returns_error() {
    let a = SecretRecord::new(nu_protocol::record! {
        "key" => Value::string("value", span()),
    });
    let b = SecretRecord::new(nu_protocol::record! {
        "key" => Value::string("value", span()),
    });
    compare_err(&a, lt_op(), Value::custom(Box::new(b), span()));
}
