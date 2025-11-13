//! Abstract Syntax Tree (AST) types for CIF documents.
//!
//! This module contains the data structures that represent parsed CIF content.
//! The AST is constructed from PEST's parse tree by the parser module.
//!
//! # Architecture
//!
//! The AST follows the hierarchical structure of CIF files:
//!
//! ```text
//! CifDocument
//!  └─ CifBlock (data blocks)
//!      ├─ items: HashMap<String, CifValue>  (key-value pairs)
//!      ├─ loops: Vec<CifLoop>               (tabular data)
//!      └─ frames: Vec<CifFrame>             (named sub-containers)
//!          ├─ items: HashMap<String, CifValue>
//!          └─ loops: Vec<CifLoop>
//! ```
//!
//! # Design Principles
//!
//! - **Decoupled from parsing**: AST types don't know how to parse themselves
//! - **Pattern matching friendly**: Use enums with exhaustive matching
//! - **Iterator-based traversal**: Provide iterator methods for queries
//! - **Public fields**: Direct field access for flexibility (struct types)

pub mod block;
pub mod document;
pub mod frame;
pub mod loop_struct;
pub mod value;

pub use block::CifBlock;
pub use document::CifDocument;
pub use frame::CifFrame;
pub use loop_struct::CifLoop;
pub use value::CifValue;
