//! # Query DSL Parser
//!
//! Parses a simple SQL-like query string into a structured [`Query`].
//!
//! Syntax:
//! ```text
//! SELECT * FROM stream1, stream2 WHERE field > 100 AND field2 = "value" SINCE 5m LIMIT 100
//! ```

use super::{CompareOp, Condition, Query};

/// Parse a raw query string into a [`Query`] struct.
pub fn parse(input: &str) -> Result<Query, String> {
    let input = input.trim();
    let upper = input.to_uppercase();

    let mut query = Query {
        from: Vec::new(),
        conditions: Vec::new(),
        since: None,
        until: None,
        limit: 100,
        offset: 0,
    };

    // Extract FROM clause
    if let Some(from_pos) = upper.find("FROM ") {
        let after_from = &input[from_pos + 5..];
        let end = find_keyword_pos(after_from);
        let from_str = after_from[..end].trim();
        query.from = from_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Extract WHERE clause
    if let Some(where_pos) = upper.find("WHERE ") {
        let after_where = &input[where_pos + 6..];
        let end = find_keyword_pos(after_where);
        let where_str = after_where[..end].trim();
        query.conditions = parse_conditions(where_str)?;
    }

    // Extract SINCE clause
    if let Some(since_pos) = upper.find("SINCE ") {
        let after_since = &input[since_pos + 6..];
        let end = find_keyword_pos(after_since);
        let since_str = after_since[..end].trim();
        query.since = Some(since_str.to_string());
    }

    // Extract UNTIL clause
    if let Some(until_pos) = upper.find("UNTIL ") {
        let after_until = &input[until_pos + 6..];
        let end = find_keyword_pos(after_until);
        let until_str = after_until[..end].trim();
        query.until = Some(until_str.to_string());
    }

    // Extract LIMIT clause
    if let Some(limit_pos) = upper.find("LIMIT ") {
        let after_limit = &input[limit_pos + 6..];
        let end = find_keyword_pos(after_limit);
        let limit_str = after_limit[..end].trim();
        if let Ok(n) = limit_str.parse::<usize>() {
            query.limit = n;
        }
    }

    // Extract OFFSET clause
    if let Some(offset_pos) = upper.find("OFFSET ") {
        let after_offset = &input[offset_pos + 7..];
        let end = find_keyword_pos(after_offset);
        let offset_str = after_offset[..end].trim();
        if let Ok(n) = offset_str.parse::<usize>() {
            query.offset = n;
        }
    }

    Ok(query)
}

fn find_keyword_pos(s: &str) -> usize {
    let upper = s.to_uppercase();
    let keywords = [
        "WHERE ", "FROM ", "SINCE ", "UNTIL ", "LIMIT ", "OFFSET ", "ORDER ",
    ];
    let mut min = s.len();
    for kw in &keywords {
        if let Some(pos) = upper.find(kw) {
            if pos < min {
                min = pos;
            }
        }
    }
    min
}

fn parse_conditions(s: &str) -> Result<Vec<Condition>, String> {
    let mut conditions = Vec::new();

    // Split on AND (case insensitive)
    let parts: Vec<&str> = split_and(s);

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Try operators in order of specificity
        let (field, op, value) = if let Some(pos) = part.find(">=") {
            (&part[..pos], CompareOp::Gte, &part[pos + 2..])
        } else if let Some(pos) = part.find("<=") {
            (&part[..pos], CompareOp::Lte, &part[pos + 2..])
        } else if let Some(pos) = part.find("!=") {
            (&part[..pos], CompareOp::Neq, &part[pos + 2..])
        } else if let Some(pos) = part.find('>') {
            (&part[..pos], CompareOp::Gt, &part[pos + 1..])
        } else if let Some(pos) = part.find('<') {
            (&part[..pos], CompareOp::Lt, &part[pos + 1..])
        } else if let Some(pos) = part.find('=') {
            (&part[..pos], CompareOp::Eq, &part[pos + 1..])
        } else if part.to_uppercase().contains(" CONTAINS ") {
            let idx = part.to_uppercase().find(" CONTAINS ").unwrap();
            (&part[..idx], CompareOp::Contains, &part[idx + 10..])
        } else if part.to_uppercase().contains(" STARTSWITH ") {
            let idx = part.to_uppercase().find(" STARTSWITH ").unwrap();
            (&part[..idx], CompareOp::StartsWith, &part[idx + 12..])
        } else {
            return Err(format!("Cannot parse condition: '{}'", part));
        };

        let field = field.trim().to_string();
        let value_str = value.trim().trim_matches('"').trim_matches('\'');
        let value = parse_value(value_str);

        conditions.push(Condition { field, op, value });
    }

    Ok(conditions)
}

fn split_and(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let upper = s.to_uppercase();
    let mut last = 0;
    let pattern = " AND ";
    let mut search_pos = 0;

    while let Some(pos) = upper[search_pos..].find(pattern) {
        let absolute_pos = search_pos + pos;
        parts.push(&s[last..absolute_pos]);
        last = absolute_pos + pattern.len();
        search_pos = last;
    }
    parts.push(&s[last..]);
    parts
}

fn parse_value(s: &str) -> serde_json::Value {
    if let Ok(n) = s.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(n) = s.parse::<f64>() {
        serde_json::json!(n)
    } else if s == "true" {
        serde_json::Value::Bool(true)
    } else if s == "false" {
        serde_json::Value::Bool(false)
    } else if s == "null" {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let q = parse("SELECT * FROM orders WHERE amount > 100 SINCE 5m LIMIT 50").unwrap();
        assert_eq!(q.from, vec!["orders"]);
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0].field, "amount");
        assert_eq!(q.conditions[0].op, CompareOp::Gt);
        assert_eq!(q.limit, 50);
        assert_eq!(q.since, Some("5m".to_string()));
    }

    #[test]
    fn test_multi_stream_query() {
        let q = parse("SELECT * FROM journal, kafka_orders").unwrap();
        assert_eq!(q.from, vec!["journal", "kafka_orders"]);
    }

    #[test]
    fn test_complex_where() {
        let q = parse("SELECT * FROM events WHERE status >= 500 AND method = \"POST\"").unwrap();
        assert_eq!(q.conditions.len(), 2);
        assert_eq!(q.conditions[0].op, CompareOp::Gte);
        assert_eq!(q.conditions[1].op, CompareOp::Eq);
    }

    #[test]
    fn test_startswith_operator() {
        let q = parse("SELECT * FROM events WHERE path STARTSWITH \"/api\"").unwrap();
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0].op, CompareOp::StartsWith);
    }
}
