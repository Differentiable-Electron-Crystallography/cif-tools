//! Statement parsing

use crate::ast::{AssignOp, Expr, ExprKind, Stmt, StmtKind};
use crate::error::DrelError;
use crate::Rule;
use pest::iterators::Pair;

use super::expr::parse_expr;
use super::helpers::{location, span, text};

/// Parse a statement from a PEST pair
pub fn parse_stmt(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);

    match pair.as_rule() {
        Rule::statement => {
            let inner = pair.into_inner().next();
            match inner {
                Some(p) => parse_stmt(p),
                None => Err(DrelError::invalid_structure("Empty statement", line, col)),
            }
        }
        Rule::if_stmt => parse_if(pair),
        Rule::for_stmt => parse_for(pair),
        Rule::loop_stmt => parse_loop(pair),
        Rule::do_stmt => parse_do(pair),
        Rule::repeat_stmt => parse_repeat(pair),
        Rule::with_stmt => parse_with(pair),
        Rule::function_def => parse_function(pair),
        Rule::break_stmt => Ok(Stmt::break_stmt(stmt_span)),
        Rule::next_stmt => Ok(Stmt::next_stmt(stmt_span)),
        Rule::assignment => parse_assignment(pair),
        Rule::expression_stmt => {
            let inner = pair.into_inner().next();
            match inner {
                Some(p) => {
                    let expr = parse_expr(p)?;
                    Ok(Stmt::expr_stmt(expr, stmt_span))
                }
                None => Err(DrelError::invalid_structure(
                    "Empty expression statement",
                    line,
                    col,
                )),
            }
        }
        // Handle expression directly as statement
        Rule::expression
        | Rule::or_expr
        | Rule::and_expr
        | Rule::comparison
        | Rule::add_expr
        | Rule::mul_expr => {
            let expr = parse_expr(pair)?;
            let expr_span = expr.span;
            Ok(Stmt::expr_stmt(expr, expr_span))
        }
        _ => Err(DrelError::unexpected(
            format!("{:?}", pair.as_rule()),
            "statement",
            line,
            col,
        )),
    }
}

fn parse_if(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    // Parse condition
    let condition = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing if condition", line, col))?;
    let condition = parse_expr(condition)?;

    // Parse then block
    let then_block = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing then block", line, col))?;
    let then_block = parse_block(then_block)?;

    // Parse elseif and else blocks
    let mut elseif_blocks = Vec::new();
    let mut else_block = None;

    while let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::expression => {
                // ElseIf condition
                let elseif_cond = parse_expr(next)?;
                let elseif_block = inner.next().ok_or_else(|| {
                    DrelError::invalid_structure("Missing elseif block", line, col)
                })?;
                elseif_blocks.push((elseif_cond, parse_block(elseif_block)?));
            }
            Rule::compound_stmt => {
                // Else block
                else_block = Some(parse_block(next)?);
            }
            _ => {}
        }
    }

    Ok(Stmt::new(
        StmtKind::If {
            condition,
            then_block,
            elseif_blocks,
            else_block,
        },
        stmt_span,
    ))
}

fn parse_for(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let var = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing for variable", line, col))?;
    let var = text(&var).to_string();

    let iterable = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing for iterable", line, col))?;
    let iterable = parse_expr(iterable)?;

    let body = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing for body", line, col))?;
    let body = parse_block(body)?;

    Ok(Stmt::new(
        StmtKind::For {
            var,
            iterable,
            body,
        },
        stmt_span,
    ))
}

fn parse_loop(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let var = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing loop variable", line, col))?;
    let var = text(&var).to_string();

    let category = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing loop category", line, col))?;
    let category = text(&category).to_string();

    let mut index_var = None;
    let mut condition = None;
    let mut body = Vec::new();

    for next in inner {
        match next.as_rule() {
            Rule::identifier => {
                // Index variable
                index_var = Some(text(&next).to_string());
            }
            Rule::expression => {
                // Where condition
                condition = Some(parse_expr(next)?);
            }
            Rule::compound_stmt => {
                body = parse_block(next)?;
            }
            _ => {}
        }
    }

    Ok(Stmt::new(
        StmtKind::Loop {
            var,
            category,
            index_var,
            condition,
            body,
        },
        stmt_span,
    ))
}

fn parse_do(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let var = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing do variable", line, col))?;
    let var = text(&var).to_string();

    let start = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing do start", line, col))?;
    let start = parse_expr(start)?;

    let end = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing do end", line, col))?;
    let end = parse_expr(end)?;

    let mut step = None;
    let mut body = Vec::new();

    for next in inner {
        match next.as_rule() {
            Rule::expression => {
                step = Some(parse_expr(next)?);
            }
            Rule::compound_stmt => {
                body = parse_block(next)?;
            }
            _ => {}
        }
    }

    Ok(Stmt::new(
        StmtKind::Do {
            var,
            start,
            end,
            step,
            body,
        },
        stmt_span,
    ))
}

fn parse_repeat(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let body = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing repeat body", line, col))?;
    let body = parse_block(body)?;

    Ok(Stmt::repeat_stmt(body, stmt_span))
}

fn parse_with(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let var = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing with variable", line, col))?;
    let var = text(&var).to_string();

    let second = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing with value/category", line, col))?;

    // Check if this is "With var As category" or "With var = expr"
    // In the "As" form, the second element is category_ref
    // In the "=" form, the second element is expression
    if second.as_rule() == Rule::category_ref {
        // "With alias As category [body]" form
        let category_span = span(&second);
        let category = text(&second).to_string();
        let value = Expr::new(ExprKind::Identifier(category), category_span);

        // Body is optional for this form
        let body = if let Some(body_pair) = inner.next() {
            parse_block(body_pair)?
        } else {
            Vec::new() // No explicit body - alias persists for rest of method
        };

        Ok(Stmt::new(StmtKind::With { var, value, body }, stmt_span))
    } else {
        // "With var = expr { body }" form
        let value = parse_expr(second)?;

        let body = inner
            .next()
            .ok_or_else(|| DrelError::invalid_structure("Missing with body", line, col))?;
        let body = parse_block(body)?;

        Ok(Stmt::new(StmtKind::With { var, value, body }, stmt_span))
    }
}

fn parse_function(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing function name", line, col))?;
    let name = text(&name).to_string();

    let mut params = Vec::new();
    let mut body = Vec::new();

    for next in inner {
        match next.as_rule() {
            Rule::param_list => {
                for param in next.into_inner() {
                    params.push(text(&param).to_string());
                }
            }
            Rule::identifier => {
                params.push(text(&next).to_string());
            }
            Rule::compound_stmt => {
                body = parse_block(next)?;
            }
            _ => {}
        }
    }

    Ok(Stmt::new(
        StmtKind::FunctionDef { name, params, body },
        stmt_span,
    ))
}

fn parse_assignment(pair: Pair<Rule>) -> Result<Stmt, DrelError> {
    let stmt_span = span(&pair);
    let (line, col) = location(&pair);
    let mut inner = pair.into_inner();

    let target = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing assignment target", line, col))?;
    let target = parse_expr(target)?;

    let op = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing assignment operator", line, col))?;
    let op = parse_assign_op(&op)?;

    let value = inner
        .next()
        .ok_or_else(|| DrelError::invalid_structure("Missing assignment value", line, col))?;
    let value = parse_expr(value)?;

    Ok(Stmt::new(
        StmtKind::Assignment { target, op, value },
        stmt_span,
    ))
}

fn parse_assign_op(pair: &Pair<Rule>) -> Result<AssignOp, DrelError> {
    let (line, col) = location(pair);
    let s = text(pair);

    match s {
        "=" => Ok(AssignOp::Assign),
        "+=" => Ok(AssignOp::AddAssign),
        "-=" => Ok(AssignOp::SubAssign),
        "*=" => Ok(AssignOp::MulAssign),
        "++=" => Ok(AssignOp::AppendAssign),
        "--=" => Ok(AssignOp::PrependAssign),
        _ => Err(DrelError::unexpected(s, "assignment operator", line, col)),
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<Stmt>, DrelError> {
    let mut statements = Vec::new();
    for inner in pair.into_inner() {
        statements.push(parse_stmt(inner)?);
    }
    Ok(statements)
}
