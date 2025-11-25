/**
 * Integration tests for CIF Parser WASM bindings using shared fixtures.
 *
 * These tests mirror the Rust integration tests in shared_fixtures.rs
 * for test parity across Rust, Python, and JavaScript.
 */

const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const { parse } = require('../pkg-node/cif_parser.js');

// Helper to load fixture files
function loadFixture(name) {
  const fixturePath = path.join(__dirname, '../../../../fixtures', name);
  return fs.readFileSync(fixturePath, 'utf8');
}

describe('Integration Tests', () => {
  // =============================================================================
  // simple.cif - Basic CIF with unknown (?) and not-applicable (.) values
  // =============================================================================

  describe('simple.cif', () => {
    it('should parse simple.cif', () => {
      const content = loadFixture('simple.cif');
      const doc = parse(content);

      assert.strictEqual(doc.blockCount, 1);
      assert.strictEqual(doc.first_block().name, 'simple');
    });

    it('should detect unknown value (?)', () => {
      const content = loadFixture('simple.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_temperature_kelvin');
      assert.ok(value.is_unknown());
    });

    it('should detect not applicable value (.)', () => {
      const content = loadFixture('simple.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_pressure');
      assert.ok(value.is_not_applicable());
    });

    it('should get text value', () => {
      const content = loadFixture('simple.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_title');
      assert.strictEqual(value.text_value, 'Simple Test Structure');
    });

    it('should get numeric value', () => {
      const content = loadFixture('simple.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_cell_length_a');
      assert.strictEqual(value.numeric_value, 10.0);
    });
  });

  // =============================================================================
  // loops.cif - Multiple loops (atom sites, bonds)
  // =============================================================================

  describe('loops.cif', () => {
    it('should parse loops.cif', () => {
      const content = loadFixture('loops.cif');
      const doc = parse(content);

      assert.strictEqual(doc.blockCount, 1);
      assert.strictEqual(doc.first_block().name, 'loops');
    });

    it('should have multiple loops', () => {
      const content = loadFixture('loops.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // Should have 2 loops: atom_site and bond
      assert.strictEqual(block.numLoops, 2);
    });

    it('should access atom site loop', () => {
      const content = loadFixture('loops.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const atomLoop = block.find_loop('_atom_site_label');
      assert.strictEqual(atomLoop.numRows, 5); // C1, C2, N1, O1, O2

      // Test accessing by tag
      const firstLabel = atomLoop.get_value_by_tag(0, '_atom_site_label');
      assert.strictEqual(firstLabel.text_value, 'C1');

      // Test getting a column
      const xCoords = atomLoop.get_column('_atom_site_fract_x');
      assert.strictEqual(xCoords.length, 5);
    });

    it('should access bond loop', () => {
      const content = loadFixture('loops.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const bondLoop = block.find_loop('_bond_type');
      assert.strictEqual(bondLoop.numRows, 3); // single, double, triple

      const firstType = bondLoop.get_value_by_tag(0, '_bond_type');
      assert.strictEqual(firstType.text_value, 'single');

      const firstLength = bondLoop.get_value_by_tag(0, '_bond_length');
      assert.ok(Math.abs(firstLength.numeric_value - 1.54) < 0.01);
    });
  });

  // =============================================================================
  // complex.cif - Save frames, multiple blocks
  // =============================================================================

  describe('complex.cif', () => {
    it('should parse complex.cif with multiple blocks', () => {
      const content = loadFixture('complex.cif');
      const doc = parse(content);

      // Should have 2 data blocks
      assert.strictEqual(doc.blockCount, 2);
    });

    it('should access multiple blocks', () => {
      const content = loadFixture('complex.cif');
      const doc = parse(content);

      assert.strictEqual(doc.get_block(0).name, 'block1');
      assert.strictEqual(doc.get_block(1).name, 'block2');

      // Access by name
      const block2 = doc.get_block_by_name('block2');
      assert.strictEqual(block2.get_item('_title').text_value, 'Second Data Block');
    });

    it('should access save frames', () => {
      const content = loadFixture('complex.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // Should have 1 save frame
      assert.strictEqual(block.numFrames, 1);
      const frame = block.get_frame(0);
      assert.strictEqual(frame.name, 'frame1');

      // Access frame items
      assert.strictEqual(frame.get_item('_frame_category').text_value, 'restraints');
    });
  });

  // =============================================================================
  // pycifrw_xanthine.cif - Uncertainty values (NumericWithUncertainty)
  // =============================================================================

  describe('pycifrw_xanthine.cif (uncertainty)', () => {
    it('should detect numeric with uncertainty type', () => {
      const content = loadFixture('pycifrw_xanthine.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // Cell length a has uncertainty: 10.01(11)
      const cellA = block.get_item('_cell_length_a');
      assert.ok(cellA.is_numeric_with_uncertainty());
    });

    it('should extract uncertainty value', () => {
      const content = loadFixture('pycifrw_xanthine.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // 10.01(11) means value=10.01, uncertainty=0.11
      const cellA = block.get_item('_cell_length_a');
      assert.ok(Math.abs(cellA.numeric_value - 10.01) < 0.001);
      assert.ok(Math.abs(cellA.uncertainty_value - 0.11) < 0.001);
    });

    it('should have correct value_type', () => {
      const content = loadFixture('pycifrw_xanthine.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const cellA = block.get_item('_cell_length_a');
      assert.strictEqual(cellA.value_type, 'NumericWithUncertainty');
    });

    it('should handle multiple uncertainties', () => {
      const content = loadFixture('pycifrw_xanthine.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // _cell_length_b: 18.23(8) -> value=18.23, uncertainty=0.08
      const cellB = block.get_item('_cell_length_b');
      assert.ok(Math.abs(cellB.numeric_value - 18.23) < 0.001);
      assert.ok(Math.abs(cellB.uncertainty_value - 0.08) < 0.001);

      // _cell_length_c: 6.93(13) -> value=6.93, uncertainty=0.13
      const cellC = block.get_item('_cell_length_c');
      assert.ok(Math.abs(cellC.numeric_value - 6.93) < 0.001);
      assert.ok(Math.abs(cellC.uncertainty_value - 0.13) < 0.001);

      // _cell_angle_beta: 107.5(9) -> value=107.5, uncertainty=0.9
      const beta = block.get_item('_cell_angle_beta');
      assert.ok(Math.abs(beta.numeric_value - 107.5) < 0.1);
      assert.ok(Math.abs(beta.uncertainty_value - 0.9) < 0.1);
    });

    it('should handle plain numeric without uncertainty', () => {
      const content = loadFixture('pycifrw_xanthine.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // _cell_angle_alpha is plain 90.0 (no uncertainty)
      const alpha = block.get_item('_cell_angle_alpha');
      assert.ok(alpha.is_numeric());
      assert.ok(!alpha.is_numeric_with_uncertainty());
      assert.strictEqual(alpha.uncertainty_value, undefined);
    });
  });

  // =============================================================================
  // crystalmaker_LuAG.cif - High precision uncertainty values
  // =============================================================================

  describe('crystalmaker_LuAG.cif (high precision)', () => {
    it('should handle high-precision uncertainty', () => {
      const content = loadFixture('crystalmaker_LuAG.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // 11.910400(4) -> value=11.9104, uncertainty=0.000004
      const cellA = block.get_item('_cell_length_a');
      assert.ok(Math.abs(cellA.numeric_value - 11.9104) < 0.0001);
      assert.ok(Math.abs(cellA.uncertainty_value - 0.000004) < 0.0000001);
    });

    it('should handle zero uncertainty', () => {
      const content = loadFixture('crystalmaker_LuAG.cif');
      const doc = parse(content);
      const block = doc.first_block();

      // 90.000000(0) -> value=90.0, uncertainty=0.0
      const alpha = block.get_item('_cell_angle_alpha');
      assert.ok(Math.abs(alpha.numeric_value - 90.0) < 0.0001);
      assert.ok(Math.abs(alpha.uncertainty_value) < 0.0000001);
    });
  });

  // =============================================================================
  // cif2_lists.cif - CIF 2.0 list syntax
  // =============================================================================

  describe('cif2_lists.cif (CIF 2.0)', () => {
    it('should detect CIF 2.0 version', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);

      assert.ok(doc.isCif2());
    });

    it('should parse empty list', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_empty_list');
      assert.ok(value.is_list());
      const list = value.list_value;
      assert.strictEqual(list.length, 0);
    });

    it('should parse single-item list', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_single_item');
      assert.ok(value.is_list());
      const list = value.list_value;
      assert.strictEqual(list.length, 1);
      assert.strictEqual(list[0].numeric_value, 42.0);
    });

    it('should parse numeric list', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_numeric_list');
      assert.ok(value.is_list());
      const list = value.list_value;
      assert.strictEqual(list.length, 5);
      for (let i = 0; i < list.length; i++) {
        assert.strictEqual(list[i].numeric_value, i + 1);
      }
    });

    it('should parse nested list', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_nested_list');
      assert.ok(value.is_list());
      const list = value.list_value;
      assert.strictEqual(list.length, 2);
      // First nested list [1 2]
      assert.strictEqual(list[0].list_value[0].numeric_value, 1.0);
      assert.strictEqual(list[0].list_value[1].numeric_value, 2.0);
      // Second nested list [3 4]
      assert.strictEqual(list[1].list_value[0].numeric_value, 3.0);
      assert.strictEqual(list[1].list_value[1].numeric_value, 4.0);
    });

    it('should parse list with unknown value', () => {
      const content = loadFixture('cif2_lists.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_mixed_with_unknown');
      assert.ok(value.is_list());
      const list = value.list_value;
      assert.strictEqual(list.length, 4);
      assert.strictEqual(list[0].numeric_value, 1.0);
      assert.strictEqual(list[1].numeric_value, 2.0);
      assert.strictEqual(list[2].value_type, 'Unknown');
      assert.strictEqual(list[3].numeric_value, 4.0);
    });
  });

  // =============================================================================
  // cif2_tables.cif - CIF 2.0 table syntax
  // =============================================================================

  describe('cif2_tables.cif (CIF 2.0)', () => {
    it('should detect CIF 2.0 version', () => {
      const content = loadFixture('cif2_tables.cif');
      const doc = parse(content);

      assert.ok(doc.isCif2());
    });

    it('should parse empty table', () => {
      const content = loadFixture('cif2_tables.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_empty_table');
      assert.ok(value.is_table());
      const table = value.table_value;
      // serde_wasm_bindgen serializes HashMap as JavaScript Map
      assert.ok(table instanceof Map);
      assert.strictEqual(table.size, 0);
    });

    it('should parse simple table', () => {
      const content = loadFixture('cif2_tables.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_simple_table');
      assert.ok(value.is_table());
      const table = value.table_value;
      // serde_wasm_bindgen serializes HashMap as JavaScript Map
      assert.ok(table instanceof Map);
      assert.strictEqual(table.size, 2);
      assert.strictEqual(table.get('a').numeric_value, 1.0);
      assert.strictEqual(table.get('b').numeric_value, 2.0);
    });

    it('should parse coordinates table', () => {
      const content = loadFixture('cif2_tables.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_coordinates');
      assert.ok(value.is_table());
      const table = value.table_value;
      // serde_wasm_bindgen serializes HashMap as JavaScript Map
      assert.ok(table instanceof Map);
      assert.strictEqual(table.size, 3);
      assert.strictEqual(table.get('x').numeric_value, 1.5);
      assert.strictEqual(table.get('y').numeric_value, 2.5);
      assert.strictEqual(table.get('z').numeric_value, 3.5);
    });

    it('should parse table with unknown value', () => {
      const content = loadFixture('cif2_tables.cif');
      const doc = parse(content);
      const block = doc.first_block();

      const value = block.get_item('_with_unknown');
      assert.ok(value.is_table());
      const table = value.table_value;
      // serde_wasm_bindgen serializes HashMap as JavaScript Map
      assert.ok(table instanceof Map);
      assert.strictEqual(table.size, 2);
      assert.strictEqual(table.get('value').numeric_value, 42.0);
      assert.strictEqual(table.get('error').value_type, 'Unknown');
    });
  });

  // =============================================================================
  // Span Tests - Source location tracking for LSP/IDE features
  // =============================================================================

  describe('Span Tests', () => {
    describe('simple.cif spans', () => {
      it('should have span with all required properties', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        const value = block.get_item('_cell_length_a');
        const span = value.span;

        // All properties should be accessible (camelCase in JS)
        assert.ok(typeof span.startLine === 'number');
        assert.ok(typeof span.startCol === 'number');
        assert.ok(typeof span.endLine === 'number');
        assert.ok(typeof span.endCol === 'number');

        // Lines are 1-indexed and should be positive
        assert.ok(span.startLine >= 1);
        assert.ok(span.endLine >= 1);
        assert.ok(span.startCol >= 1);
        assert.ok(span.endCol >= 1);
      });

      it('should have correct span for numeric value', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // Line 2: _cell_length_a    10.0
        const value = block.get_item('_cell_length_a');
        const span = value.span;

        assert.strictEqual(span.startLine, 2);
        assert.strictEqual(span.endLine, 2);
        // Value "10.0" should span 4 characters (endCol is exclusive)
        assert.strictEqual(span.endCol - span.startCol, 4);
      });

      it('should have correct span for text value', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // Line 8: _title 'Simple Test Structure'
        const value = block.get_item('_title');
        const span = value.span;

        assert.strictEqual(span.startLine, 8);
        assert.strictEqual(span.endLine, 8);
      });

      it('should have correct span for unknown value (?)', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // Line 9: _temperature_kelvin ?
        const value = block.get_item('_temperature_kelvin');
        const span = value.span;

        assert.strictEqual(span.startLine, 9);
        assert.strictEqual(span.endLine, 9);
        // Single character '?' (endCol is exclusive)
        assert.strictEqual(span.endCol - span.startCol, 1);
      });

      it('should have correct span for not applicable value (.)', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // Line 10: _pressure .
        const value = block.get_item('_pressure');
        const span = value.span;

        assert.strictEqual(span.startLine, 10);
        assert.strictEqual(span.endLine, 10);
        // Single character '.' (endCol is exclusive)
        assert.strictEqual(span.endCol - span.startCol, 1);
      });

      it('should support contains() method for hit testing', () => {
        const content = loadFixture('simple.cif');
        const doc = parse(content);
        const block = doc.first_block();

        const value = block.get_item('_cell_length_a');
        const span = value.span;

        // Position inside the span should return true
        assert.ok(span.contains(span.startLine, span.startCol));
        assert.ok(span.contains(span.endLine, span.endCol));

        // Position outside the span should return false
        assert.ok(!span.contains(span.startLine - 1, span.startCol));
        assert.ok(!span.contains(span.startLine, span.startCol - 1));
        assert.ok(!span.contains(span.endLine + 1, span.endCol));
        assert.ok(!span.contains(span.endLine, span.endCol + 1));
      });
    });

    describe('loops.cif spans', () => {
      it('should have correct spans for loop values', () => {
        const content = loadFixture('loops.cif');
        const doc = parse(content);
        const block = doc.first_block();

        const atomLoop = block.find_loop('_atom_site_label');

        // First row values should be on line 11
        const firstLabel = atomLoop.get_value_by_tag(0, '_atom_site_label');
        assert.strictEqual(firstLabel.span.startLine, 11);

        // Second row values should be on line 12
        const secondLabel = atomLoop.get_value_by_tag(1, '_atom_site_label');
        assert.strictEqual(secondLabel.span.startLine, 12);

        // Different columns on same row should have same line but different columns
        const firstX = atomLoop.get_value_by_tag(0, '_atom_site_fract_x');
        assert.strictEqual(firstX.span.startLine, firstLabel.span.startLine);
        assert.ok(firstX.span.startCol !== firstLabel.span.startCol);
      });

      it('should have distinct spans for loop column values', () => {
        const content = loadFixture('loops.cif');
        const doc = parse(content);
        const block = doc.first_block();

        const atomLoop = block.find_loop('_atom_site_label');
        const labels = atomLoop.get_column('_atom_site_label');

        // Each label should have a different line
        const lines = labels.map(label => label.span.startLine);
        const uniqueLines = new Set(lines);
        assert.strictEqual(uniqueLines.size, lines.length); // All unique lines

        // Lines should be consecutive (11, 12, 13, 14, 15)
        assert.deepStrictEqual(lines, [11, 12, 13, 14, 15]);
      });
    });

    describe('complex.cif spans', () => {
      it('should have spans across multiple data blocks', () => {
        const content = loadFixture('complex.cif');
        const doc = parse(content);

        const block1 = doc.get_block(0);
        const block2 = doc.get_block(1);

        // Values in block2 should have higher line numbers than block1
        // Both blocks have _entry_id
        const entry1 = block1.get_item('_entry_id');
        const entry2 = block2.get_item('_entry_id');

        assert.ok(entry2.span.startLine > entry1.span.startLine);
      });
    });

    describe('pycifrw_xanthine.cif spans', () => {
      it('should have correct span for uncertainty values', () => {
        const content = loadFixture('pycifrw_xanthine.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // Value with uncertainty like 10.01(11) should have span covering the whole notation
        const cellA = block.get_item('_cell_length_a');
        const span = cellA.span;

        assert.ok(span.startLine >= 1);
        assert.ok(span.endCol > span.startCol); // Should span multiple characters
      });
    });

    describe('CIF 2.0 spans', () => {
      it('should have spans for list values', () => {
        const content = loadFixture('cif2_lists.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // The list itself should have a span
        const numericList = block.get_item('_numeric_list');
        assert.ok(numericList.span.startLine >= 1);

        // Nested list should also have spans
        const nestedList = block.get_item('_nested_list');
        assert.ok(nestedList.span.startLine >= 1);
      });

      it('should have spans for table values', () => {
        const content = loadFixture('cif2_tables.cif');
        const doc = parse(content);
        const block = doc.first_block();

        // The table itself should have a span
        const simpleTable = block.get_item('_simple_table');
        assert.ok(simpleTable.span.startLine >= 1);

        // Coordinates table should have a span
        const coords = block.get_item('_coordinates');
        assert.ok(coords.span.startLine >= 1);
      });
    });
  });
});
