//! Tests using real dREL expressions from cif_core.dic
//!
//! These tests verify that the parser can handle actual dREL code
//! found in production CIF dictionaries.

use drel_parser::{extract_references, parse, parse_expr, ExprKind, StmtKind};

/// Test parsing cell.atomic_mass calculation (from cif_core.dic)
#[test]
fn test_cell_atomic_mass() {
    let source = r#"
        mass = 0.
        Loop t as atom_type {
            mass += t.number_in_cell * t.atomic_mass
        }
        _cell.atomic_mass = mass
    "#;

    let stmts = parse(source).expect("Failed to parse cell.atomic_mass");
    assert_eq!(stmts.len(), 3);

    // First statement: mass = 0.
    assert!(matches!(stmts[0].kind, StmtKind::Assignment { .. }));

    // Second statement: Loop
    assert!(matches!(stmts[1].kind, StmtKind::Loop { .. }));

    // Third statement: _cell.atomic_mass = mass
    assert!(matches!(stmts[2].kind, StmtKind::Assignment { .. }));

    // Check references
    let refs = extract_references(&stmts);
    assert!(refs.iter().any(|r| r.full_name() == "_cell.atomic_mass"));
    assert!(refs
        .iter()
        .any(|r| r.category == "atom_type" && r.is_category()));
}

/// Test parsing matrix construction (from cif_core.dic cell.convert_Uij_to_betaij)
#[test]
fn test_matrix_construction() {
    let source = r#"
        With c as cell
        _cell.convert_Uij_to_betaij = 1.4142 * Pi * Matrix([
            [ c.reciprocal_length_a, 0, 0 ],
            [ 0, c.reciprocal_length_b, 0 ],
            [ 0, 0, c.reciprocal_length_c ]
        ])
    "#;

    let stmts = parse(source).expect("Failed to parse matrix construction");
    // "With alias As category" without braces creates an alias that persists for rest of method
    // The assignment is a separate statement that uses the alias
    assert_eq!(stmts.len(), 2);

    // First statement: With alias
    assert!(matches!(stmts[0].kind, StmtKind::With { .. }));

    // Second statement: Assignment using Matrix()
    assert!(matches!(stmts[1].kind, StmtKind::Assignment { .. }));
}

/// Test With statement with explicit body (braces)
#[test]
fn test_with_explicit_body() {
    let source = r#"
        With c as cell {
            _cell.density = c.mass / c.volume
        }
    "#;

    let stmts = parse(source).expect("Failed to parse With with body");
    assert_eq!(stmts.len(), 1);

    match &stmts[0].kind {
        StmtKind::With { var, body, .. } => {
            assert_eq!(var, "c");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected With statement"),
    }
}

/// Test parsing If/ElseIf/Else statement (from cif_core.dic diffrn.flux_density)
#[test]
fn test_if_elseif_else() {
    let source = r#"
        If (_diffrn_radiation.probe == "neutron") {
            _units.code = "neutrons_per_millimetre_squared_per_second"
        }
        ElseIf (_diffrn_radiation.probe == "electron") {
            _units.code = "electrons_per_angstrom_squared_per_second"
        }
        Else {
            _units.code = "photons_per_millimetre_squared_per_second"
        }
    "#;

    let stmts = parse(source).expect("Failed to parse If/ElseIf/Else");
    assert_eq!(stmts.len(), 1);

    match &stmts[0].kind {
        StmtKind::If {
            elseif_blocks,
            else_block,
            ..
        } => {
            assert_eq!(elseif_blocks.len(), 1);
            assert!(else_block.is_some());
        }
        _ => panic!("Expected If statement"),
    }
}

/// Test parsing simple arithmetic expression
#[test]
fn test_simple_arithmetic() {
    let expr = parse_expr("a + b * c").expect("Failed to parse arithmetic");

    // Due to operator precedence, this should be: a + (b * c)
    match &expr.kind {
        ExprKind::BinaryOp { left, right, .. } => {
            assert!(matches!(left.kind, ExprKind::Identifier(_)));
            assert!(matches!(right.kind, ExprKind::BinaryOp { .. }));
        }
        _ => panic!("Expected BinaryOp"),
    }
}

/// Test parsing data name references
#[test]
fn test_data_name_parsing() {
    let expr = parse_expr("_cell.length_a * _cell.length_b").expect("Failed to parse data names");

    let refs = match &expr.kind {
        ExprKind::BinaryOp { left, right, .. } => {
            let mut refs = vec![];
            if let ExprKind::DataName { category, object } = &left.kind {
                refs.push((category.as_str(), object.as_str()));
            }
            if let ExprKind::DataName { category, object } = &right.kind {
                refs.push((category.as_str(), object.as_str()));
            }
            refs
        }
        _ => panic!("Expected BinaryOp"),
    };

    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&("cell", "length_a")));
    assert!(refs.contains(&("cell", "length_b")));
}

/// Test parsing function calls
#[test]
fn test_function_call() {
    let expr = parse_expr("Sqrt(x**2 + y**2)").expect("Failed to parse function call");

    match &expr.kind {
        ExprKind::FunctionCall { function, args } => {
            assert!(matches!(&function.kind, ExprKind::Identifier(name) if name == "Sqrt"));
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

/// Test parsing trigonometric functions (common in crystallography)
#[test]
fn test_trig_functions() {
    let source = "Cosd(c.reciprocal_angle_alpha)";
    let expr = parse_expr(source).expect("Failed to parse trig function");

    match &expr.kind {
        ExprKind::FunctionCall { function, .. } => {
            assert!(matches!(&function.kind, ExprKind::Identifier(name) if name == "Cosd"));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

/// Test parsing list literals
#[test]
fn test_list_literal() {
    let expr = parse_expr("[1, 2, 3]").expect("Failed to parse list");

    match &expr.kind {
        ExprKind::List(items) => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected List"),
    }
}

/// Test parsing table literals
#[test]
fn test_table_literal() {
    let expr = parse_expr(r#"{"key1": 1, "key2": 2}"#).expect("Failed to parse table");

    match &expr.kind {
        ExprKind::Table(entries) => {
            assert_eq!(entries.len(), 2);
        }
        _ => panic!("Expected Table"),
    }
}

/// Test parsing complex subscripting
#[test]
fn test_subscript() {
    let expr = parse_expr("matrix[0, 1]").expect("Failed to parse subscript");

    match &expr.kind {
        ExprKind::Subscription { subscripts, .. } => {
            assert_eq!(subscripts.len(), 2);
        }
        _ => panic!("Expected Subscription"),
    }
}

/// Test parsing attribute reference
#[test]
fn test_attribute_ref() {
    let expr = parse_expr("atom.fract_x").expect("Failed to parse attribute ref");

    match &expr.kind {
        ExprKind::AttributeRef { attribute, .. } => {
            assert_eq!(attribute, "fract_x");
        }
        _ => panic!("Expected AttributeRef"),
    }
}

/// Test parsing Do loop
#[test]
fn test_do_loop() {
    let source = r#"
        Do i = 1, 10, 2 {
            sum += i
        }
    "#;

    let stmts = parse(source).expect("Failed to parse Do loop");
    assert_eq!(stmts.len(), 1);

    match &stmts[0].kind {
        StmtKind::Do { var, step, .. } => {
            assert_eq!(var, "i");
            assert!(step.is_some());
        }
        _ => panic!("Expected Do statement"),
    }
}

/// Test parsing For loop
#[test]
fn test_for_loop() {
    let source = r#"
        For x in [1, 2, 3] {
            sum += x
        }
    "#;

    let stmts = parse(source).expect("Failed to parse For loop");
    assert_eq!(stmts.len(), 1);

    match &stmts[0].kind {
        StmtKind::For { var, .. } => {
            assert_eq!(var, "x");
        }
        _ => panic!("Expected For statement"),
    }
}

/// Test parsing Repeat loop
#[test]
fn test_repeat_loop() {
    let source = r#"
        Repeat {
            x = x + 1
            If (x > 10) {
                Break
            }
        }
    "#;

    let stmts = parse(source).expect("Failed to parse Repeat loop");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(stmts[0].kind, StmtKind::Repeat { .. }));
}

/// Test parsing comparison operators
#[test]
fn test_comparison_operators() {
    let tests = vec![
        ("a == b", "=="),
        ("a != b", "!="),
        ("a < b", "<"),
        ("a > b", ">"),
        ("a <= b", "<="),
        ("a >= b", ">="),
    ];

    for (source, _) in tests {
        let result = parse_expr(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);
    }
}

/// Test parsing logical operators
#[test]
fn test_logical_operators() {
    let expr = parse_expr("a and b or c").expect("Failed to parse logical ops");

    // Should parse as: (a and b) or c
    assert!(matches!(expr.kind, ExprKind::BinaryOp { .. }));
}

/// Test parsing unary operators
#[test]
fn test_unary_operators() {
    let expr = parse_expr("-x").expect("Failed to parse unary minus");
    assert!(matches!(expr.kind, ExprKind::UnaryOp { .. }));

    let expr = parse_expr("not flag").expect("Failed to parse not");
    assert!(matches!(expr.kind, ExprKind::UnaryOp { .. }));
}

/// Test parsing special literals
#[test]
fn test_special_literals() {
    assert!(matches!(parse_expr("Null").unwrap().kind, ExprKind::Null));
    assert!(matches!(
        parse_expr("Missing").unwrap().kind,
        ExprKind::Missing
    ));
}

/// Test parsing imaginary numbers
#[test]
fn test_imaginary() {
    let expr = parse_expr("3.14j").expect("Failed to parse imaginary");
    assert!(matches!(expr.kind, ExprKind::Imaginary { .. }));
}

/// Test extract_references correctly identifies categories
#[test]
fn test_extract_categories() {
    let source = r#"
        With t as atom_type
        Loop a as atom_site {
            If (a.type_symbol == t.symbol) {
                _atom_type.number_in_cell += a.occupancy * a.symmetry_multiplicity
            }
        }
    "#;

    let stmts = parse(source).expect("Failed to parse");
    let refs = extract_references(&stmts);

    // Should have atom_type and atom_site as category references
    assert!(refs.iter().any(|r| r.category == "atom_type"));
    assert!(refs.iter().any(|r| r.category == "atom_site"));

    // Should have data name reference
    assert!(refs
        .iter()
        .any(|r| r.full_name() == "_atom_type.number_in_cell"));
}
