//! Expression parsing

use crate::ast::{BinaryOperator, Expr, Subscript, UnaryOperator};
use crate::error::DrelError;
use crate::Rule;
use pest::iterators::Pair;

use super::helpers::{location, span, text};

/// Parse an expression from a PEST pair
pub fn parse_expr(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let expr_span = span(&pair);
    let (line, col) = location(&pair);

    match pair.as_rule() {
        Rule::expression | Rule::or_expr => {
            let inner = pair.into_inner().next();
            match inner {
                Some(p) => parse_expr(p),
                None => Err(DrelError::invalid_structure("Empty expression", line, col)),
            }
        }
        Rule::and_expr
        | Rule::not_expr
        | Rule::comparison
        | Rule::add_expr
        | Rule::mul_expr
        | Rule::power_expr
        | Rule::unary_expr
        | Rule::postfix_expr => parse_binary_or_unary(pair),
        Rule::primary => parse_primary(pair),
        Rule::literal => {
            // Recurse into the literal's inner rule
            let inner = pair.into_inner().next();
            match inner {
                Some(p) => parse_expr(p),
                None => Err(DrelError::invalid_structure("Empty literal", line, col)),
            }
        }
        Rule::integer | Rule::decimal_integer => {
            let s = text(&pair);
            let value = s.parse::<i64>().map_err(|_| {
                DrelError::invalid_structure(format!("Invalid integer: {}", s), line, col)
            })?;
            Ok(Expr::integer(value, expr_span))
        }
        Rule::hex_integer => {
            let s = text(&pair)
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            let value = i64::from_str_radix(s, 16).map_err(|_| {
                DrelError::invalid_structure(format!("Invalid hex integer: {}", s), line, col)
            })?;
            Ok(Expr::integer(value, expr_span))
        }
        Rule::float => {
            let s = text(&pair);
            let value = s.parse::<f64>().map_err(|_| {
                DrelError::invalid_structure(format!("Invalid float: {}", s), line, col)
            })?;
            Ok(Expr::float(value, expr_span))
        }
        Rule::imaginary => {
            let s = text(&pair).trim_end_matches(['j', 'J']);
            let value = s.parse::<f64>().map_err(|_| {
                DrelError::invalid_structure(format!("Invalid imaginary: {}", s), line, col)
            })?;
            Ok(Expr::imaginary(value, expr_span))
        }
        Rule::string | Rule::single_quoted_string | Rule::triple_quoted_string => {
            let s = text(&pair);
            // Remove quotes
            let content = if s.starts_with("'''") || s.starts_with("\"\"\"") {
                &s[3..s.len() - 3]
            } else {
                &s[1..s.len() - 1]
            };
            Ok(Expr::string(content, expr_span))
        }
        Rule::null_literal => Ok(Expr::null(expr_span)),
        Rule::missing_literal => Ok(Expr::missing(expr_span)),
        Rule::identifier => Ok(Expr::identifier(text(&pair), expr_span)),
        Rule::data_name => parse_data_name(pair),
        Rule::list_display => parse_list(pair),
        Rule::table_display => parse_table(pair),
        Rule::category_ref => Ok(Expr::identifier(text(&pair), expr_span)),
        _ => Err(DrelError::unexpected(
            format!("{:?}", pair.as_rule()),
            "expression",
            line,
            col,
        )),
    }
}

fn parse_binary_or_unary(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let outer_span = span(&pair);
    let (line, col) = location(&pair);
    let rule = pair.as_rule();
    let mut inner = pair.into_inner().peekable();

    // Check for unary operator first
    if rule == Rule::unary_expr || rule == Rule::not_expr {
        if let Some(first) = inner.peek() {
            if matches!(first.as_rule(), Rule::unary_op | Rule::not_op) {
                let op_pair = inner.next().unwrap();
                let op = parse_unary_op(&op_pair)?;
                let operand = inner
                    .next()
                    .ok_or_else(|| DrelError::invalid_structure("Missing operand", line, col))?;
                return Ok(Expr::unary(op, parse_expr(operand)?, outer_span));
            }
        }
    }

    // Special handling for postfix expressions
    if rule == Rule::postfix_expr {
        return parse_postfix(inner.collect(), outer_span);
    }

    // Parse first operand
    let first = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Empty expression", line, col))?;
    let mut result = parse_expr(first)?;

    // Parse operator-operand pairs
    while let Some(op_pair) = inner.next() {
        if let Some(right_pair) = inner.next() {
            let op = parse_binary_op(&op_pair)?;
            let right = parse_expr(right_pair)?;
            result = Expr::binary(result, op, right);
        }
    }

    Ok(result)
}

fn parse_postfix(pairs: Vec<Pair<Rule>>, _outer_span: crate::ast::Span) -> Result<Expr, DrelError> {
    let mut iter = pairs.into_iter();

    // First should be the primary expression
    let primary = iter
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Empty postfix expression", 0, 0))?;
    let mut result = parse_expr(primary)?;

    // Process postfix operations
    for pair in iter {
        let postfix_span = span(&pair);
        let (line, col) = location(&pair);
        // Merge span from result start to this postfix end
        let merged_span = result.span.merge(postfix_span);

        match pair.as_rule() {
            Rule::subscription => {
                let subscripts = parse_subscription(pair)?;
                result = Expr::subscript(result, subscripts, merged_span);
            }
            Rule::attribute_ref => {
                // attribute_ref = { "." ~ identifier }
                let attr = pair
                    .into_inner()
                    .next()
                    .map(|p| text(&p).to_string())
                    .ok_or_else(|| {
                        DrelError::invalid_structure("Missing attribute name", line, col)
                    })?;
                result = Expr::attr(result, attr, merged_span);
            }
            Rule::call => {
                // call = { "(" ~ arg_list? ~ ")" }
                let args = parse_call_args(pair)?;
                result = Expr::call(result, args, merged_span);
            }
            _ => {
                return Err(DrelError::unexpected(
                    format!("{:?}", pair.as_rule()),
                    "postfix operation",
                    line,
                    col,
                ));
            }
        }
    }

    Ok(result)
}

fn parse_subscription(pair: Pair<Rule>) -> Result<Vec<Subscript>, DrelError> {
    let mut subscripts = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::subscript_list {
            for sub in inner.into_inner() {
                subscripts.push(parse_subscript_item(sub)?);
            }
        }
    }
    Ok(subscripts)
}

fn parse_subscript_item(pair: Pair<Rule>) -> Result<Subscript, DrelError> {
    let (line, col) = location(&pair);

    match pair.as_rule() {
        Rule::subscript => {
            let inner = pair.into_inner().next();
            match inner {
                Some(p) => parse_subscript_item(p),
                None => Err(DrelError::invalid_structure("Empty subscript", line, col)),
            }
        }
        Rule::slice => {
            let inner_iter = pair.into_inner();
            let mut start = None;
            let mut stop = None;
            let mut step = None;

            // Slice is tricky: we need to handle cases like "1:3", "::2", ":3:", etc.
            // The grammar may or may not include explicit colon tokens
            for (idx, p) in inner_iter.enumerate() {
                let expr = parse_expr(p)?;
                match idx {
                    0 => start = Some(expr),
                    1 => stop = Some(expr),
                    _ => step = Some(expr),
                }
            }

            // If only one expression was parsed, it might be the stop value
            // depending on whether there was a leading colon
            Ok(Subscript::slice(start, stop, step))
        }
        Rule::key_match => {
            let mut inner = pair.into_inner();
            let key = inner.next().map(|p| text(&p).to_string()).ok_or_else(|| {
                DrelError::invalid_structure("Missing key in key_match", line, col)
            })?;
            let value = inner.next().ok_or_else(|| {
                DrelError::invalid_structure("Missing value in key_match", line, col)
            })?;
            Ok(Subscript::key_match(key, parse_expr(value)?))
        }
        _ => {
            // Default to index - this is an expression
            Ok(Subscript::index(parse_expr(pair)?))
        }
    }
}

fn parse_call_args(pair: Pair<Rule>) -> Result<Vec<Expr>, DrelError> {
    let mut args = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::arg_list {
            for arg in inner.into_inner() {
                args.push(parse_expr(arg)?);
            }
        }
    }
    Ok(args)
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let first = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Empty primary", line, col))?;

    parse_expr(first)
}

fn parse_data_name(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let expr_span = span(&pair);
    let s = text(&pair);
    // Format: _category.object
    let s = s.trim_start_matches('_');
    if let Some((category, object)) = s.split_once('.') {
        Ok(Expr::data_name(category, object, expr_span))
    } else {
        let (line, col) = location(&pair);
        Err(DrelError::invalid_structure(
            format!("Invalid data name: {}", s),
            line,
            col,
        ))
    }
}

fn parse_list(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let list_span = span(&pair);
    let mut items = Vec::new();
    for inner in pair.into_inner() {
        items.push(parse_expr(inner)?);
    }
    Ok(Expr::list(items, list_span))
}

fn parse_table(pair: Pair<Rule>) -> Result<Expr, DrelError> {
    let table_span = span(&pair);
    let mut entries = Vec::new();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::table_entry {
            let mut inner = entry.into_inner();
            if let (Some(key_pair), Some(value_pair)) = (inner.next(), inner.next()) {
                let key = text(&key_pair);
                // Remove quotes from key
                let key = if key.starts_with('"') || key.starts_with('\'') {
                    &key[1..key.len() - 1]
                } else {
                    key
                };
                let value = parse_expr(value_pair)?;
                entries.push((key.to_string(), value));
            }
        }
    }
    Ok(Expr::table(entries, table_span))
}

fn parse_binary_op(pair: &Pair<Rule>) -> Result<BinaryOperator, DrelError> {
    let (line, col) = location(pair);
    let s = text(pair).to_lowercase();

    match s.as_str() {
        "+" => Ok(BinaryOperator::Add),
        "-" => Ok(BinaryOperator::Sub),
        "*" => Ok(BinaryOperator::Mul),
        "/" => Ok(BinaryOperator::Div),
        "**" => Ok(BinaryOperator::Power),
        "^" => Ok(BinaryOperator::Cross),
        "==" => Ok(BinaryOperator::Eq),
        "!=" => Ok(BinaryOperator::Ne),
        "<" => Ok(BinaryOperator::Lt),
        ">" => Ok(BinaryOperator::Gt),
        "<=" => Ok(BinaryOperator::Le),
        ">=" => Ok(BinaryOperator::Ge),
        "in" => Ok(BinaryOperator::In),
        "and" | "&&" => Ok(BinaryOperator::And),
        "or" | "||" => Ok(BinaryOperator::Or),
        _ => {
            // Check for "not in"
            if s.contains("not") && s.contains("in") {
                Ok(BinaryOperator::NotIn)
            } else {
                Err(DrelError::unexpected(s, "binary operator", line, col))
            }
        }
    }
}

fn parse_unary_op(pair: &Pair<Rule>) -> Result<UnaryOperator, DrelError> {
    let (line, col) = location(pair);
    let s = text(pair).to_lowercase();

    match s.as_str() {
        "+" => Ok(UnaryOperator::Pos),
        "-" => Ok(UnaryOperator::Neg),
        "not" | "!" => Ok(UnaryOperator::Not),
        _ => Err(DrelError::unexpected(s, "unary operator", line, col)),
    }
}
