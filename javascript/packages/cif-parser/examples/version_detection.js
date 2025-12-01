#!/usr/bin/env node
/**
 * Example demonstrating CIF version detection and CIF 2.0 features.
 *
 * This shows how to check the version of a CIF document and work with
 * CIF 2.0-specific features like lists and tables.
 */

// For Node.js usage with the WASM build
// Note: Assumes the WASM package is built and available
// Run: just wasm-build

const cif = require('../javascript/pkg-node/cif_parser.js');

// CIF 1.1 example (no magic header)
const cif1Content = `
data_test
_chemical_name 'Example Compound'
_cell_length_a 10.0
`;

// CIF 2.0 example (with magic header)
const cif2Content = `#\\#CIF_2.0
data_test
_chemical_name 'Example Compound'
_coordinates [1.0 2.0 3.0]
_properties {x:1.0 y:2.0 z:3.0}
`;

function main() {
    console.log("=".repeat(60));
    console.log("CIF Version Detection Example");
    console.log("=".repeat(60));

    try {
        // Parse CIF 1.1 document
        const doc1 = cif.parse(cif1Content);
        console.log("\nCIF 1.1 Document:");
        console.log(`  Version: ${doc1.version.toString()}`);
        console.log(`  Is CIF 1.1: ${doc1.isCif1()}`);
        console.log(`  Is CIF 2.0: ${doc1.isCif2()}`);

        // Parse CIF 2.0 document
        const doc2 = cif.parse(cif2Content);
        console.log("\nCIF 2.0 Document:");
        console.log(`  Version: ${doc2.version.toString()}`);
        console.log(`  Is CIF 1.1: ${doc2.isCif1()}`);
        console.log(`  Is CIF 2.0: ${doc2.isCif2()}`);

        // Check version properties
        console.log("\nVersion Properties:");
        console.log(`  CIF 1.1 version.isCif1(): ${doc1.version.isCif1()}`);
        console.log(`  CIF 2.0 version.isCif2(): ${doc2.version.isCif2()}`);

        // Working with values
        const block2 = doc2.first_block();
        if (block2) {
            // Check for CIF 2.0 list values
            const coords = block2.get_item("_coordinates");
            if (coords) {
                console.log("\nCoordinates value:");
                console.log(`  Type: ${coords.value_type}`);
                console.log(`  Is list: ${coords.is_list()}`);
                if (coords.is_list()) {
                    console.log(`  List value: ${JSON.stringify(coords.list_value)}`);
                }
            }

            // Check for CIF 2.0 table values
            const props = block2.get_item("_properties");
            if (props) {
                console.log("\nProperties value:");
                console.log(`  Type: ${props.value_type}`);
                console.log(`  Is table: ${props.is_table()}`);
                if (props.is_table()) {
                    console.log(`  Table value: ${JSON.stringify(props.table_value)}`);
                }
            }
        }

        console.log("\n" + "=".repeat(60));
        console.log("Summary:");
        console.log("  - CIF version is auto-detected from magic header");
        console.log("  - CIF 2.0 adds List and Table value types");
        console.log("  - Use doc.version to check which CIF version");
        console.log("  - Use value.is_list() and value.is_table() to check types");
        console.log("=".repeat(60));

    } catch (error) {
        console.error("Error:", error);
        console.log("\nNote: Make sure to build the WASM package first:");
        console.log("  just wasm-build");
    }
}

if (require.main === module) {
    main();
}

module.exports = { main };
