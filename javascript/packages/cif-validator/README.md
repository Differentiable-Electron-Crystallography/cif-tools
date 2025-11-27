# @cif-tools/validator

DDLm-based CIF (Crystallographic Information File) validation library compiled to WebAssembly.

## Features

- **Fast validation** - Rust compiled to WebAssembly for near-native performance
- **Precise error locations** - Every error includes exact line/column span information
- **Multiple validation modes** - Strict, Lenient, and Pedantic modes
- **Multi-dictionary support** - Combine core + powder + restraints dictionaries
- **Type checking** - Validates DDLm types (Integer, Real, DateTime, etc.)
- **Works everywhere** - Browser, Node.js, and bundler targets available

## Installation

```bash
npm install @cif-tools/validator
# or
yarn add @cif-tools/validator
# or
pnpm add @cif-tools/validator
```

## Quick Start

### Browser/ES Module

```javascript
import init, { validate, JsValidator, JsValidationMode } from '@cif-tools/validator';

// Initialize WASM module
await init();

// Simple one-shot validation
const result = validate(cifContent, dictionaryContent);

if (result.isValid) {
  console.log('Validation passed!');
} else {
  for (let i = 0; i < result.errorCount; i++) {
    const error = result.get_error(i);
    console.log(`Line ${error.span.startLine}: ${error.message}`);
  }
}
```

### Using the Validator Class

For validating multiple files against the same dictionary:

```javascript
import init, { JsValidator, JsValidationMode } from '@cif-tools/validator';

await init();

// Create validator and load dictionaries
const validator = new JsValidator();
validator.addDictionary(coreDictContent);
validator.addDictionary(powderDictContent);  // Optional: add more
validator.setMode(JsValidationMode.Strict);

// Validate multiple documents
for (const cifContent of cifFiles) {
  const result = validator.validate(cifContent);
  if (!result.isValid) {
    console.log(`${result.errorCount} errors found`);
  }
}
```

### Node.js

```javascript
const { validate, JsValidator } = require('@cif-tools/validator/node');

const result = validate(cifContent, dictionaryContent);
```

## API Reference

### Functions

#### `validate(cifContent: string, dictionaryContent: string): JsValidationResult`

One-shot validation of CIF content against a dictionary.

#### `validatorVersion(): string`

Get the version of the validator library.

#### `testValidatorWasm(): string`

Test function to verify WASM is working.

### Classes

#### `JsValidator`

Reusable validator for validating multiple CIF documents.

```typescript
const validator = new JsValidator();
validator.addDictionary(dictContent);     // Add dictionary from string
validator.setMode(JsValidationMode.Strict); // Set validation mode
const result = validator.validate(cifContent);
```

#### `JsValidationResult`

Result of validation containing errors and warnings.

```typescript
result.isValid       // boolean: true if no errors
result.errors        // any[]: Array of error objects
result.warnings      // any[]: Array of warning objects
result.errorCount    // number: Number of errors
result.warningCount  // number: Number of warnings
result.errorMessages // string[]: Error messages as strings
result.warningMessages // string[]: Warning messages as strings

result.get_error(index)   // JsValidationError | undefined
result.get_warning(index) // JsValidationWarning | undefined
```

#### `JsValidationError`

A validation error with precise location information.

```typescript
error.category    // JsErrorCategory: Type of error
error.message     // string: Human-readable message
error.span        // ValidatorSpan: Location in source file
error.dataName    // string | undefined: The data name involved
error.expected    // string | undefined: Expected value/type
error.actual      // string | undefined: Actual value found
error.suggestions // string[]: Fix suggestions
error.toString()  // string: Formatted error message
```

#### `JsValidationWarning`

A validation warning (non-fatal).

```typescript
warning.category  // JsWarningCategory: Type of warning
warning.message   // string: Human-readable message
warning.span      // ValidatorSpan: Location in source file
warning.toString() // string: Formatted warning message
```

#### `ValidatorSpan`

Source location information (1-indexed).

```typescript
span.startLine  // number: Starting line (1-indexed)
span.startCol   // number: Starting column (1-indexed)
span.endLine    // number: Ending line (1-indexed)
span.endCol     // number: Ending column (1-indexed)
```

### Enums

#### `JsValidationMode`

```typescript
JsValidationMode.Strict   // All checks enabled, unknown items are errors
JsValidationMode.Lenient  // Unknown items are warnings
JsValidationMode.Pedantic // Extra style checks enabled
```

#### `JsErrorCategory`

```typescript
JsErrorCategory.UnknownDataName  // Data name not in dictionary
JsErrorCategory.TypeError        // Type mismatch
JsErrorCategory.RangeError       // Value outside allowed range
JsErrorCategory.EnumerationError // Value not in allowed set
JsErrorCategory.MissingMandatory // Required item missing
JsErrorCategory.LoopStructure    // Invalid loop structure
JsErrorCategory.LinkError        // Foreign key reference error
JsErrorCategory.DictionaryError  // Dictionary loading error
```

#### `JsWarningCategory`

```typescript
JsWarningCategory.MixedCategories // Loop has mixed categories
JsWarningCategory.DeprecatedItem  // Using deprecated item
JsWarningCategory.Style           // Style recommendation
JsWarningCategory.UnknownItem     // Unknown item (lenient mode)
```

## Example: Monaco Editor Integration

The precise span information enables editor features like error highlighting:

```javascript
import init, { JsValidator, JsValidationMode } from '@cif-tools/validator';

await init();

const validator = new JsValidator();
validator.addDictionary(dictionaryContent);

function validateDocument(model) {
  const result = validator.validate(model.getValue());

  const markers = [];
  for (let i = 0; i < result.errorCount; i++) {
    const error = result.get_error(i);
    markers.push({
      severity: monaco.MarkerSeverity.Error,
      startLineNumber: error.span.startLine,
      startColumn: error.span.startCol,
      endLineNumber: error.span.endLine,
      endColumn: error.span.endCol,
      message: error.message,
      source: 'cif-validator'
    });
  }

  monaco.editor.setModelMarkers(model, 'cif-validator', markers);
}
```

## Package Exports

The package provides different builds for different environments:

```javascript
// Browser/ES Module (default)
import init, { validate } from '@cif-tools/validator';

// Node.js
import { validate } from '@cif-tools/validator/node';

// Bundler (webpack, vite, etc.)
import init, { validate } from '@cif-tools/validator/bundler';
```

## TypeScript

Full TypeScript definitions are included. The `.d.ts` files provide complete type information for all exported functions, classes, and enums.

## Development

This package is part of the [cif-tools](https://github.com/Differentiable-Electron-Crystallography/cif-tools) monorepo.

```bash
# Build WASM
just wasm-build-validator-web

# Run tests
just js-test
```

## License

MIT OR Apache-2.0
