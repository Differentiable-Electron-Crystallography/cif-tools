# CIF 1.1 vs CIF 2.0: Differences and Implementation Guide

This document serves as the definitive reference for understanding the differences between CIF 1.1 and CIF 2.0, and how they are implemented in this parser.

## Overview

**CIF 2.0** is based on the formal EBNF specification published by IUCr:
- **Specification**: https://www.iucr.org/__data/assets/text_file/0009/112131/CIF2-ENBF.txt
- **Standard**: ISO/IEC 14977 EBNF syntax
- **Character Set**: Full Unicode (UTF-8)

**CIF 1.1** is based on the paragraph-style specification:
- **Specification**: https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax
- **Character Set**: ASCII only

**Parser Approach**: This parser uses **CIF 2.0 EBNF as the source of truth**, with notes where CIF 1.1 differs.

---

## 1. Naming Convention Changes

CIF 2.0 uses hyphenated lowercase names (EBNF convention), while CIF 1.1 used PascalCase names.

| CIF 1.1 Name | CIF 2.0 EBNF Name | Grammar Rule | Notes |
|---|---|---|---|
| `DataBlockHeading` | `data-heading` | `data_heading` | Pest uses snake_case |
| `DataBlockName` | `container-code` | `container_code` | Used for both data blocks and save frames |
| `SaveFrame` | `save-frame` | `save_frame` | |
| `Tag` | `data-name` | `data_name` | Starts with `_` |
| `UnquotedString` | `wsdelim-string` | `wsdelim_string` | Whitespace-delimited string |
| `SingleQuotedString` | `quoted-string` (apostrophe variant) | `quoted_string` | |
| `DoubleQuotedString` | `quoted-string` (quote variant) | `quoted_string` | |
| `TextField` | `text-field` | `text_field` | Semicolon-delimited |
| `WhiteSpace` | `wspace` | `wspace` | |
| `NonBlankChar` | `non-blank-char` | `non_blank_char` | |
| `AnyPrintChar` | `char` | `char` | CIF2: any allowed char except line terminators |
| `OrdinaryChar` | (subset of `lead-char`) | `ordinary_char` | CIF1.1 concept, not in CIF2 |

**Decision**: Grammar rules use snake_case (Pest convention), matching CIF2 hyphenated names.

---

## 2. Character Set Differences

### CIF 1.1: ASCII Only
```pest
// CIF 1.1 (current implementation)
nonblank_ch = { '\x21'..'\x7E' }  // ASCII printable (! through ~)
anyprint_ch = { ' '..'~' | "\t" }  // ASCII space through ~, plus tab
```

### CIF 2.0: Full Unicode
```
// CIF 2.0 EBNF: allchars
allchars = ?U+0009? | ?U+000A? | ?U+000D? | ?U+0020 - U+007E?
  | ?U+00A0 - U+D7FF? | ?U+E000 - U+FDCF? | ?U+FDF0 - U+FFFD?
  | ?U+10000 - U+1FFFD? | ?U+20000 - U+2FFFD? | ... (all planes)
```

**Excluded code points**:
- U+0000 through U+0008 (control characters except tab)
- U+000B, U+000C (vertical tab, form feed)
- U+000E through U+001F (other control characters)
- U+007F through U+009F (DEL and C1 control characters)
- U+D800 through U+DFFF (UTF-16 surrogates)
- U+FDD0 through U+FDEF (non-characters)
- U+xFFFE and U+xFFFF for any plane x (non-characters)

**Pest Implementation**:
```pest
// Simplified for Pest (full Unicode support via ANY token and exclusions)
char = { !line_term ~ ANY }  // Any character except line terminators
non_blank_char = { !inline_wspace ~ char }  // Any char except space/tab/newlines
```

---

## 3. Grammar Structure Differences

### Data Block / Save Frame Names

**CIF 2.0 EBNF**:
```
container-code = non-blank-char, { non-blank-char } ;
```
- **Requires**: 1 or more characters

**CIF 1.1 Pest (buggy)**:
```pest
datablockname = { nonblank_ch* }
```
- **Allows**: 0 or more characters (empty names!)

**Fix**: Adopt CIF 2.0 requirement:
```pest
container_code = { non_blank_char+ }  // 1 or more
```

### Data Names (Tags)

**CIF 2.0 EBNF**:
```
data-name = '_', non-blank-char, { non-blank-char } ;
```

**CIF 1.1 Pest**:
```pest
tag = { "_" ~ nonblank_ch+ }
```

**Status**: ✅ These match! No changes needed.

### Quoted Strings

**CIF 2.0 EBNF**:
```
quoted-string = ( quote-delim, quote-content, quote-delim )
  | ( apostrophe-delim, apostrophe-content, apostrophe-delim ) ;
quote-content = { char - quote-delim } ;
```
- **Allows**: Newlines inside quoted strings
- **char** includes newlines (just not line terminators at end)

**CIF 1.1 Pest (current)**:
```pest
singlequoted = { "'" ~ (!endq_single ~ !"\n" ~ ANY)* ~ endq_single }
```
- **Disallows**: Newlines (`!"\n"`)

**Decision**: Follow CIF 2.0 - allow newlines in quoted strings:
```pest
quoted_string = {
    (quote_delim ~ quote_content ~ quote_delim) |
    (apostrophe_delim ~ apostrophe_content ~ apostrophe_delim)
}
quote_content = { (!quote_delim ~ char)* }
apostrophe_content = { (!apostrophe_delim ~ char)* }
```

### Whitespace-Delimited Strings (Unquoted)

**CIF 2.0 EBNF**:
```
wsdelim-string = ( lead-char, {restrict-char} )
  - ( data-token | save-token | loop-token | global-token | stop-token ) ;

lead-char = restrict-char - ( '"' | '#' | "'" | '_' ) ;
restrict-char = non-blank-char - ( '[' | ']' | '{' | '}' ) ;
```
- **Excludes**: `[`, `]`, `{`, `}` (reserved for lists/tables)
- **Cannot start with**: `"`, `#`, `'`, `_`
- **Cannot match**: Keywords (data_, save_, loop_, global_, stop_)

**Implementation**:
```pest
restrict_char = { !("[" | "]" | "{" | "}") ~ non_blank_char }
lead_char = { !("\"" | "#" | "'" | "_") ~ restrict_char }
wsdelim_string = {
    !keyword ~
    lead_char ~
    restrict_char*
}
```

**Important**: The parser follows the CIF 2.0 spec strictly by excluding `[`, `]`, `{`, `}` from unquoted strings **even in CIF 1.1 mode**. This is necessary because:
1. PEG parsers cannot have different character rules based on runtime conditions
2. Without this restriction, empty lists/tables (`[]`, `{}`) cannot be parsed correctly
3. The parser would incorrectly consume `]` as part of an unquoted string when parsing `[]`

**Impact**: In both CIF 1.1 and CIF 2.0 files, these characters must be quoted:
```
# ❌ Invalid (will fail to parse)
_item [brackets]
_item {braces}

# ✅ Valid (must quote these characters)
_item '[brackets]'
_item '{braces}'
```

---

## 4. New Features in CIF 2.0

### 4.1 Magic Comment (Version Identifier)

**CIF 2.0 EBNF**:
```
file-heading = [ ?U+FEFF? ], magic-code, { inline-wspace } ;
magic-code = '#\#CIF_2.0' ;
```

**Implementation**:
```pest
magic_code = { "#\\#CIF_2.0" }
file_heading = { BYTE_ORDER_MARK? ~ magic_code ~ inline_wspace* }
```

**Version Detection**:
- If file starts with `#\#CIF_2.0` → CIF 2.0
- Otherwise → CIF 1.1 (backward compatibility)

### 4.2 Triple-Quoted Strings

**CIF 2.0 EBNF**:
```
triple-quoted-string = ( quote3-delim, quote3-content, quote3-delim )
  | ( apostrophe3-delim, apostrophe3-content, apostrophe3-delim ) ;

quote3-delim = '"""' ;
quote3-content = { [ '"', [ '"' ] ], not-quote, { not-quote } } ;
not-quote = allchars - '"' ;

apostrophe3-delim = "'''" ;
apostrophe3-content = { [ "'", [ "'" ] ], not-apostrophe, { not-apostrophe } } ;
not-apostrophe = allchars - "'" ;
```

**Implementation**:
```pest
triple_quoted_string = {
    (quote3_delim ~ quote3_content ~ quote3_delim) |
    (apostrophe3_delim ~ apostrophe3_content ~ apostrophe3_delim)
}

quote3_delim = { "\"\"\"" }
quote3_content = { (!"\"\"\"" ~ ANY)* }

apostrophe3_delim = { "'''" }
apostrophe3_content = { (!"'''" ~ ANY)* }
```

**Purpose**: Multi-line strings without escape sequences or line-initial semicolons.

### 4.3 Lists

**CIF 2.0 EBNF**:
```
list = '[', [ list-values-start, { wspace-data-value } ], [ wspace ], ']' ;
```

**Implementation**:
```pest
list = {
    "[" ~ wspace? ~
    (data_value ~ (wspace ~ data_value)*)? ~
    wspace? ~ "]"
}
```

**Example**:
```
_atom_types [C N O H]
_coordinates [1.0 2.0 3.0]
```

### 4.4 Tables (Dictionaries)

**CIF 2.0 EBNF**:
```
table = '{', [ wspace-any, table-entry, { wspace, table-entry } ], [ wspace ], '}' ;
table-entry = ( quoted-string | triple-quoted-string ), ':',
              ( nospace-value | wsdelim-string | wspace-data-value ) ;
```

**Implementation**:
```pest
table = {
    "{" ~ wspace? ~
    (table_entry ~ (wspace? ~ "," ~ wspace? ~ table_entry)*)? ~
    wspace? ~ "}"
}

table_entry = {
    (quoted_string | triple_quoted_string) ~
    wspace? ~ ":" ~ wspace? ~
    data_value
}
```

**Example**:
```
_symmetry_operations {
    'x,y,z': 1
    '-x,-y,-z': 2
}
```

---

## 5. CIF 2.0 EBNF → Pest Mapping

### EBNF Syntax Conventions

| EBNF | Meaning | Pest Equivalent |
|---|---|---|
| `A, B` | Sequence (concatenation) | `A ~ B` |
| `A \| B` | Alternation (choice) | `A \| B` |
| `{ A }` | Zero or more repetitions | `A*` |
| `[ A ]` | Optional (zero or one) | `A?` |
| `( A )` | Grouping | `(A)` |
| `'text'` or `"text"` | Terminal string | `"text"` |
| `A - B` | Exception (A except B) | `A ~ !B` or lookahead |
| `?U+XXXX?` | Unicode code point | Character literal or `ANY` |

### Example Translations

**EBNF**:
```
data-name = '_', non-blank-char, { non-blank-char } ;
```

**Pest**:
```pest
data_name = { "_" ~ non_blank_char ~ non_blank_char* }
// Or simplified:
data_name = { "_" ~ non_blank_char+ }
```

---

**EBNF**:
```
wsdelim-string = ( lead-char, {restrict-char} )
  - ( data-token | save-token | loop-token ) ;
```

**Pest** (using negative lookahead):
```pest
wsdelim_string = {
    !(data_token | save_token | loop_token) ~
    lead_char ~
    restrict_char*
}
```

---

**EBNF**:
```
list = '[', [ wspace-any, data-value, { wspace, data-value } ], [ wspace ], ']' ;
```

**Pest**:
```pest
list = {
    "[" ~ wspace? ~
    (data_value ~ (wspace ~ data_value)*)? ~
    wspace? ~ "]"
}
```

---

## 6. Whitespace Handling Differences

### CIF 2.0 Whitespace Productions

**EBNF**:
```
wspace = ( inline-wspace | line-term ), wspace-any ;
wspace-any = { wspace-to-eol }, { inline-wspace } ;
wspace-to-eol = { inline-wspace }, [ comment ], line-term ;
inline-wspace = ?U+0020? | ?U+0009? ;  // Space or tab
line-term = ( ?U+000D?, [ ?U+000A? ] ) | ?U+000A? ;  // CR, LF, or CRLF
comment = '#', { char } ;
```

**CIF 1.1 Implementation**:
```pest
ws_char = { " " | "\t" | "\n" | "\r" }
comment = { "#" ~ (!"\n" ~ ANY)* }
whitespace = { (ws_char | comment)+ }
```

**Key Difference**: CIF 2.0 normalizes line endings (CR, LF, CRLF → LF) before parsing. Pest handles this natively with `\n` and `\r`.

---

## 7. Reserved Words

### CIF 2.0 Keywords (Case-Insensitive)

| Keyword | Purpose | Status |
|---|---|---|
| `data_` | Data block header | Active (CIF 1.1 & 2.0) |
| `save_` | Save frame delimiter | Active (CIF 1.1 & 2.0) |
| `loop_` | Loop structure | Active (CIF 1.1 & 2.0) |
| `global_` | Global block | Reserved (STAR, not used in CIF) |
| `stop_` | Loop terminator | Reserved (deprecated) |

**Implementation** (unchanged):
```pest
data_token = { ^"data_" }
save_token = { ^"save_" }
loop_token = { ^"loop_" }
global_token = { ^"global_" }
stop_token = { ^"stop_" }
```

---

## 8. Migration Guide: CIF 1.1 → CIF 2.0

### Grammar Changes Summary

1. **Character set**: ASCII → Unicode
2. **Container names**: Must be 1+ characters (not empty)
3. **Quoted strings**: Can contain newlines
4. **Unquoted strings**: Cannot contain `[`, `]`, `{`, `}` (must be quoted)
5. **New types**: Lists, tables, triple-quoted strings
6. **Magic comment**: `#\#CIF_2.0` required for CIF 2.0 files

### Backward Compatibility

**Files that will still parse**:
- ✅ Valid CIF 1.1 files (ASCII, no lists/tables)
- ✅ Data blocks with non-empty names
- ✅ Quoted strings without newlines
- ✅ Unquoted strings without brackets/braces

**Files that may break**:
- ❌ Empty data block names (`data_` with no name)
- ❌ Files using `[`, `]`, `{`, `}` in unquoted strings

**Reserved Characters**:

The characters `[`, `]`, `{`, `}` are **reserved** in both CIF 1.1 and CIF 2.0 modes. They cannot appear in unquoted strings and must be quoted:

```cif
# ❌ Invalid - will fail to parse
data_test
_item [text]
_value {data}

# ✅ Valid - quote the values
data_test
_item '[text]'
_value '{data}'
```

This restriction applies to all files parsed by this library, regardless of whether they have the `#\#CIF_2.0` magic comment. This is a technical requirement of the PEG parser implementation to correctly handle CIF 2.0 lists and tables.

### AST Changes

**New Value Types**:
```rust
pub enum CifValue {
    // Existing (CIF 1.1)
    Text(String),
    Numeric(f64),
    Unknown,          // ?
    NotApplicable,    // .

    // New (CIF 2.0)
    List(Vec<CifValue>),                    // [val1 val2 val3]
    Table(HashMap<String, CifValue>),       // {key1:val1 key2:val2}
}
```

**Version Tracking**:
```rust
pub enum CifVersion {
    V1_1,  // No magic comment or CIF 1.1 features only
    V2_0,  // Has magic comment #\#CIF_2.0
}

pub struct CifDocument {
    pub version: CifVersion,
    pub blocks: Vec<DataBlock>,
}
```

---

## 9. Reference Links

- **CIF 2.0 EBNF**: https://www.iucr.org/__data/assets/text_file/0009/112131/CIF2-ENBF.txt
- **CIF 1.1 Syntax**: https://www.iucr.org/resources/cif/spec/version1.1/cifsyntax
- **STAR2 Paper**: Spadaccini & Hall, J. Chem. Inf. Model., 2012, 52(8), 1901-1906
- **ISO/IEC 14977**: EBNF Standard
- **Pest Documentation**: https://pest.rs

---

## 10. Implementation Checklist

- [x] Document CIF 1.1 vs CIF 2.0 differences (this file)
- [ ] Rewrite `src/cif.pest` with CIF 2.0 EBNF naming
- [ ] Add Unicode character support
- [ ] Add triple-quoted strings
- [ ] Add list syntax
- [ ] Add table syntax
- [ ] Update quoted strings to allow newlines
- [ ] Extend `CifValue` enum
- [ ] Add `CifVersion` enum and version detection
- [ ] Update parser for new grammar rules
- [ ] Update Python bindings
- [ ] Update WASM/JS bindings
- [ ] Update documentation and examples

---

*This document is the source of truth for the CIF parser implementation. All grammar rules and AST types should reference this document.*
