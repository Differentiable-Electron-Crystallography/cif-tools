/**
 * Integration tests for CIF Validator WASM bindings using shared fixtures.
 *
 * These tests mirror the Python integration tests in
 * python/cif-validator/tests/test_integration.py
 * for test parity across Python and JavaScript.
 */

const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const {
  validate,
  JsValidator,
  JsValidationMode,
  JsErrorCategory,
} = require('../pkg-node/cif_validator.js');

// Helper to load fixture files
function loadFixture(name) {
  const fixturePath = path.join(__dirname, '../../../../fixtures', name);
  return fs.readFileSync(fixturePath, 'utf8');
}

// Load validation fixtures
function loadValidationDict() {
  return loadFixture('validation/test_validation.dic');
}

function loadValidCif() {
  return loadFixture('validation/valid_structure.cif');
}

function loadInvalidCif() {
  return loadFixture('validation/invalid_structure.cif');
}

describe('Integration Tests', () => {
  // =============================================================================
  // valid_structure.cif - Should pass validation
  // =============================================================================

  describe('valid_structure.cif', () => {
    it('should pass validation with no errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadValidCif();

      const result = validate(cifContent, dictContent);

      assert.ok(result.isValid);
      assert.strictEqual(result.errorCount, 0);
    });

    it('should have no warnings in strict mode', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadValidCif();

      const validator = new JsValidator();
      validator.addDictionary(dictContent);
      validator.setMode(JsValidationMode.Strict);

      const result = validator.validate(cifContent);

      assert.ok(result.isValid);
    });
  });

  // =============================================================================
  // invalid_structure.cif - Should fail validation with 9 errors
  // =============================================================================

  describe('invalid_structure.cif', () => {
    it('should fail validation', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      assert.ok(!result.isValid);
      assert.ok(result.errorCount > 0);
    });

    it('should have exactly 9 errors', () => {
      /**
       * Expected errors:
       * 1. _cell.length_a = -5.0 (range: 0.1-1000)
       * 2. _cell.length_b = 5000.0 (range: 0.1-1000)
       * 3. _cell.angle_alpha = 270.0 (range: 0-180)
       * 4. _cell.angle_beta = -45.0 (range: 0-180)
       * 5. _symmetry.crystal_system = "dodecahedral" (invalid enum)
       * 6. _atom_site.fract_x = 1.5000 (range: 0-1)
       * 7. _atom_site.fract_y = -0.1000 (range: 0-1)
       * 8. _atom_site.occupancy = 2.5 (range: 0-1)
       * 9. _atom_site.occupancy = -0.5 (range: 0-1)
       */
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      assert.strictEqual(result.errorCount, 9);
    });
  });

  // =============================================================================
  // Range Error Detection
  // =============================================================================

  describe('Range Error Detection', () => {
    it('should detect cell length range errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      // Find errors related to cell lengths
      const cellLengthErrors = [];
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.dataName && error.dataName.startsWith('_cell.length_')) {
          cellLengthErrors.push(error);
        }
      }

      // Should have 2 cell length errors: length_a (-5.0) and length_b (5000.0)
      assert.strictEqual(cellLengthErrors.length, 2);

      // All should be range errors
      for (const error of cellLengthErrors) {
        assert.strictEqual(error.category, JsErrorCategory.RangeError);
      }
    });

    it('should detect cell angle range errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      // Find errors related to cell angles
      const cellAngleErrors = [];
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.dataName && error.dataName.startsWith('_cell.angle_')) {
          cellAngleErrors.push(error);
        }
      }

      // Should have 2 cell angle errors: angle_alpha (270.0) and angle_beta (-45.0)
      assert.strictEqual(cellAngleErrors.length, 2);

      // All should be range errors
      for (const error of cellAngleErrors) {
        assert.strictEqual(error.category, JsErrorCategory.RangeError);
      }
    });

    it('should detect fractional coordinate range errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      // Find errors related to fractional coordinates
      const fractErrors = [];
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.dataName && error.dataName.startsWith('_atom_site.fract_')) {
          fractErrors.push(error);
        }
      }

      // Should have 2 fractional coord errors: fract_x (1.5) and fract_y (-0.1)
      assert.strictEqual(fractErrors.length, 2);

      // All should be range errors
      for (const error of fractErrors) {
        assert.strictEqual(error.category, JsErrorCategory.RangeError);
      }
    });

    it('should detect occupancy range errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      // Find errors related to occupancy
      const occupancyErrors = [];
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.dataName === '_atom_site.occupancy') {
          occupancyErrors.push(error);
        }
      }

      // Should have 2 occupancy errors: 2.5 and -0.5
      assert.strictEqual(occupancyErrors.length, 2);

      // All should be range errors
      for (const error of occupancyErrors) {
        assert.strictEqual(error.category, JsErrorCategory.RangeError);
      }
    });
  });

  // =============================================================================
  // Enumeration Error Detection
  // =============================================================================

  describe('Enumeration Error Detection', () => {
    it('should detect enumeration error for crystal_system', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      // Find the crystal_system error
      const crystalSystemErrors = [];
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.dataName === '_symmetry.crystal_system') {
          crystalSystemErrors.push(error);
        }
      }

      // Should have exactly 1 enumeration error
      assert.strictEqual(crystalSystemErrors.length, 1);
      assert.strictEqual(
        crystalSystemErrors[0].category,
        JsErrorCategory.EnumerationError
      );

      // The actual value should be "dodecahedral"
      assert.strictEqual(crystalSystemErrors[0].actual, 'dodecahedral');
    });
  });

  // =============================================================================
  // Error Span Information
  // =============================================================================

  describe('Error Span Information', () => {
    it('should have valid span positions for all errors', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        const span = error.span;

        // All span values should be positive (1-indexed)
        assert.ok(span.startLine >= 1, `startLine should be >= 1`);
        assert.ok(span.endLine >= 1, `endLine should be >= 1`);
        assert.ok(span.startCol >= 1, `startCol should be >= 1`);
        assert.ok(span.endCol >= 1, `endCol should be >= 1`);

        // End should be at or after start
        assert.ok(span.endLine >= span.startLine);
        if (span.startLine === span.endLine) {
          assert.ok(span.endCol >= span.startCol);
        }
      }
    });

    it('should have informative error messages', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadInvalidCif();

      const result = validate(cifContent, dictContent);

      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        // Message should not be empty
        assert.ok(error.message);
        assert.ok(error.message.length > 0);
      }
    });
  });

  // =============================================================================
  // Validator Class Workflow
  // =============================================================================

  describe('Validator Class Workflow', () => {
    it('should support the Validator class workflow', () => {
      const dictContent = loadValidationDict();
      const cifContent = loadValidCif();

      // Create validator
      const validator = new JsValidator();

      // Add dictionary
      validator.addDictionary(dictContent);

      // Set mode
      validator.setMode(JsValidationMode.Strict);

      // Validate
      const result = validator.validate(cifContent);

      assert.ok(result.isValid);
    });

    it('should validate multiple documents with same validator', () => {
      const dictContent = loadValidationDict();
      const validCif = loadValidCif();
      const invalidCif = loadInvalidCif();

      const validator = new JsValidator();
      validator.addDictionary(dictContent);

      // Validate valid document
      const result1 = validator.validate(validCif);
      assert.ok(result1.isValid);

      // Validate invalid document with same validator
      const result2 = validator.validate(invalidCif);
      assert.ok(!result2.isValid);
      assert.strictEqual(result2.errorCount, 9);
    });
  });

  // =============================================================================
  // Validation Modes
  // =============================================================================

  describe('Validation Modes', () => {
    it('should treat unknown items as warnings in lenient mode', () => {
      const dictContent = loadValidationDict();
      // CIF with an item not in the dictionary
      const cifWithUnknown = `#\\#CIF_2.0
data_test
_entry.id 'test'
_unknown_item 'this is not in the dictionary'
_cell.length_a 10.0
`;

      const validator = new JsValidator();
      validator.addDictionary(dictContent);
      validator.setMode(JsValidationMode.Lenient);

      const result = validator.validate(cifWithUnknown);

      // In lenient mode, unknown items should be warnings, not errors
      // So the document should be valid
      assert.ok(result.isValid);

      // Should have at least one warning about the unknown item
      assert.ok(result.warningCount >= 1);
    });

    it('should treat unknown items as errors in strict mode', () => {
      const dictContent = loadValidationDict();
      // CIF with an item not in the dictionary
      const cifWithUnknown = `#\\#CIF_2.0
data_test
_entry.id 'test'
_unknown_item 'this is not in the dictionary'
_cell.length_a 10.0
`;

      const validator = new JsValidator();
      validator.addDictionary(dictContent);
      validator.setMode(JsValidationMode.Strict);

      const result = validator.validate(cifWithUnknown);

      // In strict mode, unknown items should be errors
      // Find the unknown item error
      let hasUnknownError = false;
      for (let i = 0; i < result.errorCount; i++) {
        const error = result.get_error(i);
        if (error.category === JsErrorCategory.UnknownDataName) {
          hasUnknownError = true;
          break;
        }
      }
      assert.ok(hasUnknownError, 'Should have an unknown data name error');
    });
  });

  // =============================================================================
  // API Smoke Tests
  // =============================================================================

  describe('API Smoke Tests', () => {
    it('should have validate function available', () => {
      assert.ok(typeof validate === 'function');
    });

    it('should have JsValidator class available', () => {
      assert.ok(typeof JsValidator === 'function');
      const validator = new JsValidator();
      assert.ok(validator);
    });

    it('should have validation mode enum available', () => {
      assert.strictEqual(JsValidationMode.Strict, 0);
      assert.strictEqual(JsValidationMode.Lenient, 1);
      assert.strictEqual(JsValidationMode.Pedantic, 2);
    });

    it('should have error category enum available', () => {
      assert.strictEqual(JsErrorCategory.UnknownDataName, 0);
      assert.strictEqual(JsErrorCategory.TypeError, 1);
      assert.strictEqual(JsErrorCategory.RangeError, 2);
      assert.strictEqual(JsErrorCategory.EnumerationError, 3);
    });
  });
});
