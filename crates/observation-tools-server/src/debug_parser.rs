//! Parser for Rust Debug output format
//!
//! Converts Rust's `{:#?}` Debug output into JSON for rendering in the web UI.

use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, take_while1};
use nom::character::complete::{char, multispace0, none_of, one_of};
use nom::combinator::{opt, recognize, value};
use nom::multi::separated_list0;
use nom::sequence::{pair, tuple};
use nom::IResult;
use serde_json::{Map, Value};

/// Parse Rust Debug output into a JSON value
///
/// Returns the parsed JSON value, or a JSON string containing the raw input
/// if parsing fails.
pub fn parse_debug_to_json(input: &str) -> Value {
  match debug_value(input.trim()) {
    Ok(("", value)) => value,
    Ok((remaining, _)) if remaining.trim().is_empty() => {
      // Trailing whitespace is ok
      match debug_value(input.trim()) {
        Ok((_, value)) => value,
        Err(_) => Value::String(input.to_string()),
      }
    }
    _ => Value::String(input.to_string()),
  }
}

// Parse an identifier (struct/field/enum names)
fn identifier(input: &str) -> IResult<&str, &str> {
  take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

// Parse a string literal
fn string_literal(input: &str) -> IResult<&str, Value> {
  let (input, _) = char('"')(input)?;
  let (input, content) = opt(escaped(none_of("\\\""), '\\', one_of("\\\"nrt0'")))(input)?;
  let (input, _) = char('"')(input)?;

  let s = content.unwrap_or("");
  // Unescape the string
  let unescaped = s
    .replace("\\n", "\n")
    .replace("\\r", "\r")
    .replace("\\t", "\t")
    .replace("\\\\", "\\")
    .replace("\\\"", "\"")
    .replace("\\'", "'");

  Ok((input, Value::String(unescaped)))
}

// Parse a number (integer or float, possibly negative)
fn number(input: &str) -> IResult<&str, Value> {
  let (input, num_str) = recognize(tuple((
    opt(char('-')),
    take_while1(|c: char| c.is_ascii_digit()),
    opt(pair(char('.'), take_while1(|c: char| c.is_ascii_digit()))),
  )))(input)?;

  // Try to parse as i64 first, then f64
  if let Ok(n) = num_str.parse::<i64>() {
    Ok((input, Value::Number(n.into())))
  } else if let Ok(n) = num_str.parse::<f64>() {
    Ok((
      input,
      serde_json::Number::from_f64(n)
        .map(Value::Number)
        .unwrap_or(Value::String(num_str.to_string())),
    ))
  } else {
    Ok((input, Value::String(num_str.to_string())))
  }
}

// Parse a boolean
fn boolean(input: &str) -> IResult<&str, Value> {
  alt((
    value(Value::Bool(true), tag("true")),
    value(Value::Bool(false), tag("false")),
  ))(input)
}

// Parse an array: [item, item, ...]
fn array(input: &str) -> IResult<&str, Value> {
  let (input, _) = char('[')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, items) = separated_list0(
    tuple((multispace0, char(','), multispace0)),
    debug_value,
  )(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = opt(char(','))(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char(']')(input)?;

  Ok((input, Value::Array(items)))
}

// Parse a tuple: (item, item, ...)
fn tuple_value(input: &str) -> IResult<&str, Value> {
  let (input, _) = char('(')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, items) = separated_list0(
    tuple((multispace0, char(','), multispace0)),
    debug_value,
  )(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = opt(char(','))(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char(')')(input)?;

  let mut map = Map::new();
  map.insert("_tuple".to_string(), Value::Array(items));
  Ok((input, Value::Object(map)))
}

// Parse a struct field: name: value
fn struct_field(input: &str) -> IResult<&str, (String, Value)> {
  let (input, _) = multispace0(input)?;
  let (input, name) = identifier(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char(':')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, value) = debug_value(input)?;
  Ok((input, (name.to_string(), value)))
}

// Parse a named struct: Name { field: value, ... }
fn named_struct(input: &str) -> IResult<&str, Value> {
  let (input, name) = identifier(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char('{')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, fields) = separated_list0(
    tuple((multispace0, char(','), multispace0)),
    struct_field,
  )(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = opt(char(','))(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char('}')(input)?;

  let mut map = Map::new();
  map.insert("_type".to_string(), Value::String(name.to_string()));
  for (field_name, field_value) in fields {
    map.insert(field_name, field_value);
  }

  Ok((input, Value::Object(map)))
}

// Parse a tuple struct: Name(value, value, ...)
fn tuple_struct(input: &str) -> IResult<&str, Value> {
  let (input, name) = identifier(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char('(')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, values) = separated_list0(
    tuple((multispace0, char(','), multispace0)),
    debug_value,
  )(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = opt(char(','))(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char(')')(input)?;

  let mut map = Map::new();
  map.insert("_type".to_string(), Value::String(name.to_string()));
  map.insert("_values".to_string(), Value::Array(values));

  Ok((input, Value::Object(map)))
}

// Parse a unit struct/enum variant: Name (just an identifier)
fn unit_variant(input: &str) -> IResult<&str, Value> {
  let (input, name) = identifier(input)?;

  // Make sure it's not followed by { or (
  let (input, next) = opt(alt((char('{'), char('('))))(input)?;
  if next.is_some() {
    return Err(nom::Err::Error(nom::error::Error::new(
      input,
      nom::error::ErrorKind::Tag,
    )));
  }

  let mut map = Map::new();
  map.insert("_type".to_string(), Value::String(name.to_string()));

  Ok((input, Value::Object(map)))
}

// Parse a map-like structure: { key: value, ... }
fn map_value(input: &str) -> IResult<&str, Value> {
  let (input, _) = char('{')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, entries) = separated_list0(
    tuple((multispace0, char(','), multispace0)),
    map_entry,
  )(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = opt(char(','))(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char('}')(input)?;

  let mut map = Map::new();
  for (key, value) in entries {
    map.insert(key, value);
  }

  Ok((input, Value::Object(map)))
}

// Parse a map entry: key: value (key can be a value too, but we convert to string)
fn map_entry(input: &str) -> IResult<&str, (String, Value)> {
  let (input, _) = multispace0(input)?;
  let (input, key) = debug_value(input)?;
  let (input, _) = multispace0(input)?;
  let (input, _) = char(':')(input)?;
  let (input, _) = multispace0(input)?;
  let (input, value) = debug_value(input)?;

  // Convert key to string for JSON compatibility
  let key_str = match key {
    Value::String(s) => s,
    Value::Number(n) => n.to_string(),
    Value::Bool(b) => b.to_string(),
    _ => serde_json::to_string(&key).unwrap_or_else(|_| "unknown".to_string()),
  };

  Ok((input, (key_str, value)))
}

// Parse any debug value
fn debug_value(input: &str) -> IResult<&str, Value> {
  let (input, _) = multispace0(input)?;

  alt((
    // Try these in order of specificity
    string_literal,
    boolean,
    number,
    array,
    tuple_value,
    map_value,
    // Named struct must come before tuple_struct and unit_variant
    named_struct,
    tuple_struct,
    unit_variant,
  ))(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_parse_unit() {
    let result = parse_debug_to_json("None");
    assert_eq!(result, json!({"_type": "None"}));
  }

  #[test]
  fn test_parse_simple_struct() {
    let input = r#"Point {
    x: 10,
    y: 20,
}"#;
    let result = parse_debug_to_json(input);
    assert_eq!(result, json!({"_type": "Point", "x": 10, "y": 20}));
  }

  #[test]
  fn test_parse_nested_struct() {
    let input = r#"Outer {
    inner: Inner {
        value: 42,
    },
}"#;
    let result = parse_debug_to_json(input);
    assert_eq!(
      result,
      json!({
        "_type": "Outer",
        "inner": {
          "_type": "Inner",
          "value": 42
        }
      })
    );
  }

  #[test]
  fn test_parse_tuple_struct() {
    let input = "Some(\n    42,\n)";
    let result = parse_debug_to_json(input);
    assert_eq!(result, json!({"_type": "Some", "_values": [42]}));
  }

  #[test]
  fn test_parse_array() {
    let input = "[\n    1,\n    2,\n    3,\n]";
    let result = parse_debug_to_json(input);
    assert_eq!(result, json!([1, 2, 3]));
  }

  #[test]
  fn test_parse_string() {
    let result = parse_debug_to_json(r#""hello world""#);
    assert_eq!(result, json!("hello world"));
  }

  #[test]
  fn test_parse_string_with_escapes() {
    let result = parse_debug_to_json(r#""hello\nworld""#);
    assert_eq!(result, json!("hello\nworld"));
  }

  #[test]
  fn test_parse_boolean() {
    assert_eq!(parse_debug_to_json("true"), json!(true));
    assert_eq!(parse_debug_to_json("false"), json!(false));
  }

  #[test]
  fn test_parse_number() {
    assert_eq!(parse_debug_to_json("42"), json!(42));
    assert_eq!(parse_debug_to_json("-17"), json!(-17));
    assert_eq!(parse_debug_to_json("3.14"), json!(3.14));
  }

  #[test]
  fn test_parse_tuple() {
    let result = parse_debug_to_json("(\n    1,\n    \"two\",\n)");
    assert_eq!(result, json!({"_tuple": [1, "two"]}));
  }

  #[test]
  fn test_parse_complex_struct() {
    let input = r#"User {
    name: "Alice",
    age: 30,
    active: true,
    scores: [
        100,
        95,
        87,
    ],
}"#;
    let result = parse_debug_to_json(input);
    assert_eq!(
      result,
      json!({
        "_type": "User",
        "name": "Alice",
        "age": 30,
        "active": true,
        "scores": [100, 95, 87]
      })
    );
  }

  #[test]
  fn test_parse_option_none() {
    let result = parse_debug_to_json("None");
    assert_eq!(result, json!({"_type": "None"}));
  }

  #[test]
  fn test_parse_option_some() {
    let result = parse_debug_to_json("Some(\n    \"value\",\n)");
    assert_eq!(result, json!({"_type": "Some", "_values": ["value"]}));
  }

  #[test]
  fn test_fallback_on_invalid() {
    let input = "this is { invalid } debug";
    let result = parse_debug_to_json(input);
    // Should fall back to returning the raw string
    assert_eq!(result, json!("this is { invalid } debug"));
  }

  #[test]
  fn test_empty_struct() {
    let result = parse_debug_to_json("Empty {}");
    assert_eq!(result, json!({"_type": "Empty"}));
  }

  #[test]
  fn test_empty_array() {
    let result = parse_debug_to_json("[]");
    assert_eq!(result, json!([]));
  }
}
