/**
 * Basic tests for CIF Parser WASM bindings
 *
 * Run with: wasm-pack test --node
 */

const assert = require('node:assert');
const { parse, version } = require('../pkg-node/cif_parser.js');

describe('CIF Parser Tests', () => {
  describe('Module', () => {
    it('should export version function', () => {
      const ver = version();
      assert.ok(ver);
      assert.strictEqual(typeof ver, 'string');
    });

    it('should export parse function', () => {
      assert.strictEqual(typeof parse, 'function');
    });
  });

  describe('Document Parsing', () => {
    it('should parse simple CIF', () => {
      const cif = 'data_test\n_item value\n';
      const doc = parse(cif);
      assert.strictEqual(doc.blockCount, 1);
    });

    it('should parse multiple blocks', () => {
      const cif = 'data_block1\n_item1 val1\ndata_block2\n_item2 val2\n';
      const doc = parse(cif);
      assert.strictEqual(doc.blockCount, 2);
    });

    // Note: The parser is intentionally permissive (see docs/reference/grammar-notes.md)
    // Invalid CIF returns an empty document rather than throwing an error
    it('should return empty document for invalid CIF', () => {
      const invalidCif = 'invalid cif content';
      const doc = parse(invalidCif);
      assert.strictEqual(doc.blockCount, 0);
    });
  });

  describe('Block Access', () => {
    const cif = 'data_example\n_cell_a 10.0\n_cell_b 20.0\n';
    let doc;
    let block;

    beforeEach(() => {
      doc = parse(cif);
      block = doc.first_block();
    });

    it('should get block by index', () => {
      const b = doc.get_block(0);
      assert.ok(b);
      assert.strictEqual(b.name, 'example');
    });

    it('should get block by name', () => {
      const b = doc.get_block_by_name('example');
      assert.ok(b);
      assert.strictEqual(b.name, 'example');
    });

    it('should have correct block properties', () => {
      assert.strictEqual(block.name, 'example');
      assert.ok(block.itemKeys.length > 0);
    });
  });

  describe('Value Access', () => {
    const cif = `data_test
_text 'hello'
_numeric 42.5
_unknown ?
_not_applicable .
`;
    let block;

    beforeEach(() => {
      const doc = parse(cif);
      block = doc.first_block();
    });

    it('should get text value', () => {
      const val = block.get_item('_text');
      assert.ok(val);
      assert.ok(val.is_text());
      assert.strictEqual(val.text_value, 'hello');
    });

    it('should get numeric value', () => {
      const val = block.get_item('_numeric');
      assert.ok(val);
      assert.ok(val.is_numeric());
      assert.strictEqual(val.numeric_value, 42.5);
    });

    it('should get unknown value', () => {
      const val = block.get_item('_unknown');
      assert.ok(val);
      assert.ok(val.is_unknown());
    });

    it('should get not-applicable value', () => {
      const val = block.get_item('_not_applicable');
      assert.ok(val);
      assert.ok(val.is_not_applicable());
    });
  });

  describe('Loop Access', () => {
    const cif = `data_test
loop_
_col1 _col2 _col3
val1  1.0   2.0
val2  3.0   4.0
`;
    let block;
    let loop;

    beforeEach(() => {
      const doc = parse(cif);
      block = doc.first_block();
      loop = block.get_loop(0);
    });

    it('should have correct dimensions', () => {
      assert.strictEqual(loop.numRows, 2);
      assert.strictEqual(loop.numColumns, 3);
    });

    it('should have correct tags', () => {
      const tags = loop.tags;
      assert.deepStrictEqual(tags, ['_col1', '_col2', '_col3']);
    });

    it('should get value by position', () => {
      const val = loop.get_value(0, 0);
      assert.ok(val);
      assert.ok(val.is_text());
      assert.strictEqual(val.text_value, 'val1');
    });

    it('should get value by tag', () => {
      const val = loop.get_value_by_tag(0, '_col2');
      assert.ok(val);
      assert.ok(val.is_numeric());
      assert.strictEqual(val.numeric_value, 1.0);
    });

    it('should get column', () => {
      const col = loop.get_column('_col1');
      assert.ok(col);
      assert.strictEqual(col.length, 2);
    });

    it('should get row dict', () => {
      const row = loop.get_row_dict(0);
      assert.ok(row);
      assert.ok(row._col1);
    });
  });
});
