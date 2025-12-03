# diffBloch Dictionary - Structured Items Implementation

## Status: IMPLEMENTED

**Date:** 2025-12-03

## Summary

Converted 5 values that were being parsed from free-text `_diffrn_measurement_details` into proper typed, validated CIF items.

## What Was Done

### 1. Dictionary Changes (`diffbloch_2.dic`)

Added 5 new mandatory items to the `DIFFRN_MEASUREMENT` category:

| Item | Type | Constraints | Lines |
|------|------|-------------|-------|
| `_diffrn_measurement.rotation_axis_position` | Real | - | 1067-1094 |
| `_diffrn_measurement.data_collection_geometry` | Code | enum: `continuous_rotation`, `precession` | 1096-1126 |
| `_diffrn_measurement.rc_width` | Real | >= 0 | 1128-1157 |
| `_diffrn_measurement.mosaicity` | Real | >= 0 | 1159-1188 |
| `_diffrn_measurement.dstarmax` | Real | >= 0 | 1190-1219 |

**Note:** `data_collection_geometry` uses underscored values (`continuous_rotation` not `continuous rotation`) because CIF `Code` type doesn't allow whitespace.

### 2. Example CIF Update (`urea_valid.cif`)

Added structured items (lines 35-40):
```cif
_diffrn_measurement_rotation_axis_position    272.325
_diffrn_measurement_data_collection_geometry  continuous_rotation
_diffrn_measurement_rc_width                  0.00490
_diffrn_measurement_mosaicity                 0.070
_diffrn_measurement_dstarmax                  1.900
```

### 3. Validation Results

| File | Status | Errors |
|------|--------|--------|
| `urea_valid.cif` | **VALID** | 0 |
| `urea_invalid.cif` | **INVALID** | 6 MissingMandatory errors |

---

## REQUIRED: Future diffBloch Changes

### File: `diffBloch/rotation_dataset.py`

#### Current Code (lines 705-729):
```python
def extract_data_params(pets_path):
    for block in parse_cif(pets_path):
        measurement_details = block.get('_diffrn_measurement_details', '')
        for line in measurement_details.split('\n'):
            if 'rotation axis position' in line:
                rotation_axis_position = float(line.split(':')[1].strip())
            # ... etc
```

#### Required Update:
```python
def extract_data_params(pets_path):
    for block in parse_cif(pets_path):
        # Try structured items first (new format)
        rotation_axis_position = block.get('_diffrn_measurement_rotation_axis_position')
        if rotation_axis_position is not None:
            rotation_axis_position = float(rotation_axis_position)
            rc_width = float(block.get('_diffrn_measurement_rc_width'))
            mosaicity = float(block.get('_diffrn_measurement_mosaicity'))
            dstarmax = float(block.get('_diffrn_measurement_dstarmax'))
            geometry = block.get('_diffrn_measurement_data_collection_geometry')
            # Convert underscore format back to space format for internal use
            data_collection_geometry = geometry.replace('_', ' ')
        else:
            # Fall back to legacy free-text parsing
            measurement_details = block.get('_diffrn_measurement_details', '')
            # ... existing parsing code ...

        return rotation_axis_position, rc_width, mosaicity, dstarmax, data_collection_geometry
```

### Enumeration Value Mapping

The CIF uses underscored values, but diffBloch internal code expects spaces:

| CIF Value | diffBloch Internal |
|-----------|-------------------|
| `continuous_rotation` | `'continuous rotation'` |
| `precession` | `'precession'` |

---

## Problem Statement (Background)

diffBloch was parsing structured data from free-text CIF fields, which:
- Bypassed dictionary validation (type checking, range checking, mandatory checking)
- Caused cryptic runtime errors when values are missing or malformed
- Made CIF files fragile (depended on exact string patterns like "rotation axis position")

### Critical Failure Mode

If "tilt axis position" instead of "rotation axis position":
1. `rotation_axis_position = None` (extraction fails silently)
2. `rotation_matrix_z(-None)` → **TypeError crash**

Now with proper validation:
- Dictionary catches missing `_diffrn_measurement.rotation_axis_position`
- Clear error message: "Missing mandatory item..."

---

## Other Patterns Found (Lower Priority)

| Location | Pattern | Risk Level | Notes |
|----------|---------|------------|-------|
| Cell lengths | `5.63083(80)` SU notation | Medium | Needs SU handling in validator |
| HKL strings | `"1 0 0"` split | Low | Internal DataFrame format |
| CSV constraints | Space-separated atoms | Low | Not CIF data |

---

## Files Modified

**Dictionary:**
- `diffbloch_2.dic`

**Example CIF:**
- `urea_valid.cif`

**Future (not yet modified):**
- `diffBloch/rotation_dataset.py` - needs update to read structured items
