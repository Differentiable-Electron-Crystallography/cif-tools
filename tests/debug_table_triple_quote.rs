use cif_parser::{CIFParser, Rule};
use pest::Parser;

#[test]
fn debug_table_triple_quote() {
    // Try to parse just the table
    let input = r#"{"""long key""":1.0}"#;

    match CIFParser::parse(Rule::table, input) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("Matched rule: {:?}", pair.as_rule());
            println!("Matched text: {:?}", pair.as_str());
            for inner in pair.into_inner() {
                println!(
                    "  Inner rule: {:?}, text: {:?}",
                    inner.as_rule(),
                    inner.as_str()
                );
                for inner2 in inner.into_inner() {
                    println!(
                        "    Inner2 rule: {:?}, text: {:?}",
                        inner2.as_rule(),
                        inner2.as_str()
                    );
                }
            }
        }
        Err(e) => {
            println!("Failed to match as table: {:?}", e);
        }
    }

    // Try parsing the triple quoted string alone
    let tq_input = r#""""long key""""#;
    match CIFParser::parse(Rule::triple_quoted_string, tq_input) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("\nTriple quoted string matched: {:?}", pair.as_str());
        }
        Err(e) => {
            println!("\nFailed to match triple quoted string: {:?}", e);
        }
    }

    // Try parsing a table entry
    let entry_input = r#""""long key""":1.0"#;
    match CIFParser::parse(Rule::table_entry, entry_input) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("\nTable entry matched: {:?}", pair.as_str());
            for inner in pair.into_inner() {
                println!(
                    "  Inner rule: {:?}, text: {:?}",
                    inner.as_rule(),
                    inner.as_str()
                );
            }
        }
        Err(e) => {
            println!("\nFailed to match table entry: {:?}", e);
        }
    }
}
