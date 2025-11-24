use cif_parser::{CIFParser, Rule};
use pest::Parser;

#[test]
fn test_pest_empty_list_matching() {
    // Test if the pest grammar can match an empty list
    let input = "[]";

    // Try to parse as a list
    match CIFParser::parse(Rule::list, input) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("Matched rule: {:?}", pair.as_rule());
            println!("Matched text: {:?}", pair.as_str());
            println!("Inner pairs: {:?}", pair.into_inner().collect::<Vec<_>>());
        }
        Err(e) => {
            println!("Failed to match as list: {:?}", e);
        }
    }

    // Try to parse as data_value
    match CIFParser::parse(Rule::data_value, input) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            println!("\nMatched as data_value");
            println!("Rule: {:?}", pair.as_rule());
            println!("Text: {:?}", pair.as_str());

            for inner in pair.into_inner() {
                println!(
                    "  Inner rule: {:?}, text: {:?}",
                    inner.as_rule(),
                    inner.as_str()
                );
            }
        }
        Err(e) => {
            println!("\nFailed to match as data_value: {:?}", e);
        }
    }
}
