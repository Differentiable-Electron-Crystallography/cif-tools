/**
 * CIF Parser - TypeScript Example
 *
 * This example demonstrates using the CIF parser with TypeScript,
 * showing full type safety and IDE autocomplete support.
 *
 * Compile with: tsc typescript-example.ts
 * Run with: node typescript-example.js
 */

import init, {
  parse,
  version,
  type JsCifDocument,
  type JsCifBlock,
  type JsCifLoop,
  type JsCifValue,
} from '../pkg/cif_parser.js';

interface AtomSite {
  id: number;
  element: string;
  x: number;
  y: number;
  z: number;
  occupancy: number;
}

interface UnitCell {
  a: number;
  b: number;
  c: number;
  alpha: number;
  beta: number;
  gamma: number;
}

async function main() {
  // Initialize WASM module
  await init();

  console.log('CIF Parser - TypeScript Example');
  console.log(`Version: ${version()}`);
  console.log('='.repeat(60));

  const cifContent = `
data_crystal_structure
_cell_length_a 5.64
_cell_length_b 5.64
_cell_length_c 5.64
_cell_angle_alpha 90.0
_cell_angle_beta 90.0
_cell_angle_gamma 90.0

loop_
_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
_atom_site_occupancy
Na1  Na  0.0    0.0    0.0    1.0
Cl1  Cl  0.5    0.5    0.5    1.0
`;

  try {
    // Parse with type safety
    const doc: JsCifDocument = parse(cifContent);

    console.log(`\nParsed ${doc.blockCount} blocks`);

    // Get first block with proper typing
    const block: JsCifBlock | undefined = doc.first_block();
    if (!block) {
      throw new Error('No blocks found in document');
    }

    console.log(`\nProcessing block: ${block.name}`);

    // Extract unit cell with type checking
    const unitCell = extractUnitCell(block);
    console.log('\nUnit Cell Parameters:');
    console.log(`  a = ${unitCell.a} Å`);
    console.log(`  b = ${unitCell.b} Å`);
    console.log(`  c = ${unitCell.c} Å`);
    console.log(`  α = ${unitCell.alpha}°`);
    console.log(`  β = ${unitCell.beta}°`);
    console.log(`  γ = ${unitCell.gamma}°`);

    // Extract atom sites with type safety
    const atomSites = extractAtomSites(block);
    console.log(`\nFound ${atomSites.length} atom sites:`);
    atomSites.forEach((atom, i) => {
      console.log(
        `  ${i + 1}. ${atom.element} at (${atom.x.toFixed(3)}, ${atom.y.toFixed(3)}, ${atom.z.toFixed(3)})`
      );
    });
  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

/**
 * Extract unit cell parameters from a CIF block
 */
function extractUnitCell(block: JsCifBlock): UnitCell {
  const getNumeric = (key: string): number => {
    const value: JsCifValue | undefined = block.get_item(key);
    if (!value || !value.is_numeric()) {
      throw new Error(`Missing or non-numeric value for ${key}`);
    }
    const numValue = value.numeric_value;
    if (numValue === undefined) {
      throw new Error(`Undefined numeric value for ${key}`);
    }
    return numValue;
  };

  return {
    a: getNumeric('_cell_length_a'),
    b: getNumeric('_cell_length_b'),
    c: getNumeric('_cell_length_c'),
    alpha: getNumeric('_cell_angle_alpha'),
    beta: getNumeric('_cell_angle_beta'),
    gamma: getNumeric('_cell_angle_gamma'),
  };
}

/**
 * Extract atom sites from a CIF block
 */
function extractAtomSites(block: JsCifBlock): AtomSite[] {
  const loop: JsCifLoop | undefined = block.find_loop('_atom_site_label');
  if (!loop) {
    return [];
  }

  const atomSites: AtomSite[] = [];

  for (let i = 0; i < loop.numRows; i++) {
    const getValue = (tag: string, isNumeric = false): string | number | undefined => {
      const value: JsCifValue | undefined = loop.get_value_by_tag(i, tag);
      if (!value) return undefined;

      if (isNumeric && value.is_numeric()) {
        return value.numeric_value;
      }
      if (!isNumeric && value.is_text()) {
        return value.text_value;
      }
      return undefined;
    };

    const element = getValue('_atom_site_type_symbol', false);
    const x = getValue('_atom_site_fract_x', true);
    const y = getValue('_atom_site_fract_y', true);
    const z = getValue('_atom_site_fract_z', true);
    const occupancy = getValue('_atom_site_occupancy', true);

    if (element && x !== undefined && y !== undefined && z !== undefined) {
      atomSites.push({
        id: i + 1,
        element,
        x,
        y,
        z,
        occupancy: occupancy ?? 1.0,
      });
    }
  }

  return atomSites;
}

// Run the example
main().catch(console.error);
