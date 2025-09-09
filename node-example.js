// Node.js example using the CIF parser WASM module
//
// IMPORTANT: Before running this example, build the Node.js WASM package:
//   wasm-pack build --target nodejs --out-dir pkg-node
//
// Then run: node node-example.js
//
const { JsCifDocument, test_wasm, get_version } = require('./pkg-node/cif_parser.js');

console.log('CIF Parser WASM Node.js Example');
console.log('==============================');
console.log('Version:', get_version());
console.log('Test:', test_wasm());
console.log();

// Example CIF content
const cifContent = `
data_example
_cell_length_a 10.000
_cell_length_b 20.000
_cell_length_c 30.000
_cell_angle_alpha 90.0
_cell_angle_beta 90.0
_cell_angle_gamma 90.0
_author_name 'John Doe'
_title 'Example Crystal Structure'

loop_
_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
C1 C 0.1234 0.5678 0.9012
N1 N 0.2345 0.6789 0.0123
O1 O 0.3456 0.7890 0.1234

save_frame1
_frame_item1 'Frame data 1'
_frame_item2 42.0
save_
`;

try {
    console.log('Parsing CIF content...');
    const doc = JsCifDocument.parse(cifContent);
    
    console.log(`✅ Successfully parsed ${doc.get_block_count()} blocks:`);
    console.log(`   Block names: ${doc.get_block_names().join(', ')}`);
    console.log();
    
    // Process the first block
    const block = doc.get_first_block();
    if (block) {
        console.log(`📦 Block "${block.name}":`);
        console.log(`   - Items: ${block.get_item_keys().length}`);
        console.log(`   - Loops: ${block.get_loop_count()}`);
        console.log(`   - Frames: ${block.get_frame_count()}`);
        console.log();
        
        // Show some data items
        console.log('📋 Data Items:');
        const itemKeys = block.get_item_keys().slice(0, 5); // Show first 5
        for (const key of itemKeys) {
            const value = block.get_item(key);
            if (value) {
                let valueStr = '';
                if (value.is_text()) {
                    valueStr = `"${value.text_value}"`;
                } else if (value.is_numeric()) {
                    valueStr = value.numeric_value;
                } else {
                    valueStr = `(${value.value_type})`;
                }
                console.log(`   ${key}: ${valueStr}`);
            }
        }
        
        if (block.get_item_keys().length > 5) {
            console.log(`   ... and ${block.get_item_keys().length - 5} more items`);
        }
        console.log();
        
        // Show loop data
        if (block.get_loop_count() > 0) {
            const loop = block.get_loop(0);
            console.log('🔄 First Loop:');
            console.log(`   Columns: ${loop.get_column_count()} (${loop.get_tags().join(', ')})`);
            console.log(`   Rows: ${loop.get_row_count()}`);
            console.log();
            
            console.log('   Sample data:');
            const maxRows = Math.min(3, loop.get_row_count());
            for (let i = 0; i < maxRows; i++) {
                const rowData = [];
                for (let j = 0; j < loop.get_column_count(); j++) {
                    const value = loop.get_value(i, j);
                    if (value) {
                        if (value.is_text()) {
                            rowData.push(`"${value.text_value}"`);
                        } else if (value.is_numeric()) {
                            rowData.push(value.numeric_value);
                        } else {
                            rowData.push(value.value_type);
                        }
                    }
                }
                console.log(`   Row ${i}: ${rowData.join(', ')}`);
            }
            
            if (loop.get_row_count() > maxRows) {
                console.log(`   ... and ${loop.get_row_count() - maxRows} more rows`);
            }
            console.log();
        }
        
        // Show save frame data
        if (block.get_frame_count() > 0) {
            const frame = block.get_frame(0);
            console.log(`💾 Save Frame "${frame.name}":`);
            console.log(`   - Items: ${frame.get_item_keys().length}`);
            console.log(`   - Loops: ${frame.get_loop_count()}`);
            
            const frameKeys = frame.get_item_keys();
            for (const key of frameKeys) {
                const value = frame.get_item(key);
                if (value) {
                    let valueStr = '';
                    if (value.is_text()) {
                        valueStr = `"${value.text_value}"`;
                    } else if (value.is_numeric()) {
                        valueStr = value.numeric_value;
                    }
                    console.log(`   ${key}: ${valueStr}`);
                }
            }
            console.log();
        }
        
        // Demonstrate finding specific data
        console.log('🔍 Specific Data Access:');
        const cellA = block.get_item('_cell_length_a');
        if (cellA && cellA.is_numeric()) {
            console.log(`   Unit cell a: ${cellA.numeric_value} Å`);
        }
        
        const title = block.get_item('_title');
        if (title && title.is_text()) {
            console.log(`   Structure title: ${title.text_value}`);
        }
        
        // Find atoms loop and extract specific atom
        const atomLoop = block.find_loop('_atom_site_label');
        if (atomLoop) {
            const firstAtom = atomLoop.get_value_by_tag(0, '_atom_site_label');
            const firstAtomType = atomLoop.get_value_by_tag(0, '_atom_site_type_symbol');
            if (firstAtom && firstAtomType) {
                console.log(`   First atom: ${firstAtom.text_value} (${firstAtomType.text_value})`);
            }
        }
    }
    
} catch (error) {
    console.error('❌ Parsing failed:', error.toString());
    
    console.log('\n🔧 Troubleshooting tips:');
    console.log('- Check that data block names start with "data_"');
    console.log('- Ensure loops have matching numbers of tags and values');
    console.log('- Verify text fields are properly delimited with semicolons');
    console.log('- Make sure quotes are properly closed');
}