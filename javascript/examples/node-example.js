/**
 * CIF Parser - Node.js Example
 *
 * This example demonstrates using the CIF parser in Node.js to parse
 * CIF files and extract crystallographic data.
 *
 * Usage:
 *   node node-example.js
 */

const { parse, version, author } = require('../pkg-node/cif_parser.js');

// Example CIF content
const cifContent = `
data_protein_structure
_entry.id    1ABC
_cell.length_a   50.000
_cell.length_b   60.000
_cell.length_c   70.000
_cell.angle_alpha   90.0
_cell.angle_beta    90.0
_cell.angle_gamma   90.0

loop_
_atom_site.id
_atom_site.type_symbol
_atom_site.label_atom_id
_atom_site.Cartn_x
_atom_site.Cartn_y
_atom_site.Cartn_z
_atom_site.occupancy
1   C   CA   10.123  20.456  30.789  1.00
2   C   CB   11.234  21.567  31.890  1.00
3   N   N    12.345  22.678  32.901  1.00
4   O   O    13.456  23.789  34.012  1.00
5   S   SG   14.567  24.890  35.123  0.95

data_refinement
_refine.ls_d_res_high  2.5
_refine.ls_d_res_low   50.0
_refine.ls_R_factor    0.195
`;

console.log('='.repeat(60));
console.log('CIF Parser - Node.js Example');
console.log('='.repeat(60));
console.log(`Version: ${version()}`);
console.log(`Author: ${author()}`);
console.log('='.repeat(60));

try {
  // Parse CIF content
  console.log('\nParsing CIF content...');
  const doc = parse(cifContent);

  console.log(`\n✓ Successfully parsed ${doc.blockCount} data blocks`);
  console.log(`  Block names: ${doc.blockNames.join(', ')}`);

  // Process first block
  console.log(`\n${'='.repeat(60)}`);
  console.log('BLOCK 1: Protein Structure Data');
  console.log('='.repeat(60));

  const block = doc.get_block(0);
  if (block) {
    console.log(`Name: ${block.name}`);
    console.log(`Data items: ${block.itemKeys.length}`);
    console.log(`Loops: ${block.numLoops}`);

    // Display some data items
    console.log('\nSample Data Items:');
    console.log('-'.repeat(60));

    const sampleKeys = ['_entry.id', '_cell.length_a', '_cell.length_b', '_cell.length_c'];
    for (const key of sampleKeys) {
      const value = block.get_item(key);
      if (value) {
        if (value.is_numeric()) {
          console.log(`  ${key.padEnd(30)} = ${value.numeric_value}`);
        } else if (value.is_text()) {
          console.log(`  ${key.padEnd(30)} = "${value.text_value}"`);
        }
      }
    }

    // Display loop data
    if (block.numLoops > 0) {
      console.log('\nLoop Data (Atom Sites):');
      console.log('-'.repeat(60));

      const loop = block.get_loop(0);
      console.log(`Dimensions: ${loop.numRows} rows × ${loop.numColumns} columns`);
      console.log(`Columns: ${loop.tags.join(', ')}`);

      console.log('\nFirst 3 rows:');
      for (let i = 0; i < Math.min(3, loop.numRows); i++) {
        const row = loop.get_row_dict(i);
        console.log(`\nRow ${i + 1}:`);
        for (const [key, value] of Object.entries(row)) {
          let displayValue;
          if (value.is_numeric()) {
            displayValue = value.numeric_value;
          } else if (value.is_text()) {
            displayValue = `"${value.text_value}"`;
          } else {
            displayValue = '?';
          }
          console.log(`  ${key.padEnd(35)} = ${displayValue}`);
        }
      }

      // Extract specific column
      console.log('\nAll atom types (type_symbol column):');
      const typeSymbols = loop.get_column('_atom_site.type_symbol');
      if (typeSymbols) {
        const atoms = typeSymbols
          .map((v) => v.text_value)
          .filter(Boolean)
          .join(', ');
        console.log(`  ${atoms}`);
      }
    }
  }

  // Process second block
  console.log(`\n${'='.repeat(60)}`);
  console.log('BLOCK 2: Refinement Data');
  console.log('='.repeat(60));

  const refineBlock = doc.get_block_by_name('refinement');
  if (refineBlock) {
    console.log(`Name: ${refineBlock.name}`);
    console.log('\nRefinement Statistics:');
    console.log('-'.repeat(60));

    for (const key of refineBlock.itemKeys) {
      const value = refineBlock.get_item(key);
      if (value?.is_numeric()) {
        console.log(`  ${key.padEnd(30)} = ${value.numeric_value}`);
      }
    }
  }

  console.log(`\n${'='.repeat(60)}`);
  console.log('Parse completed successfully!');
  console.log('='.repeat(60));
} catch (error) {
  console.error('\n✗ Parse Error:');
  console.error(`  ${error}`);
  process.exit(1);
}
