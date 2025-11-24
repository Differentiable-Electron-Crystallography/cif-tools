//! Loop structures representing tabular data in CIF files.

use super::CifValue;

/// Represents a loop structure in a CIF file (tabular data).
///
/// Loops are one of the most important structures in CIF files, representing
/// tabular data with named columns (tags) and multiple rows of values.
///
/// # Structure
///
/// ```text
/// loop_
/// _atom_site_label          # Column 1
/// _atom_site_type_symbol    # Column 2
/// _atom_site_fract_x        # Column 3
/// C1  C  0.1234            # Row 1
/// N1  N  0.5678            # Row 2
/// O1  O  0.9012            # Row 3
/// ```
///
/// # Data Organization
///
/// - **Tags**: Column headers (always start with `_`)
/// - **Values**: Organized as a vector of rows, each row containing values for all columns
/// - **Type safety**: Each value is parsed into a [`CifValue`] with appropriate type
///
/// # Access Patterns
///
/// ```
/// use cif_parser::Document;
///
/// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\nval3 val4\n";
/// # let doc = Document::parse(cif).unwrap();
/// # let loop_ = &doc.blocks[0].loops[0];
/// // By position
/// let value = loop_.get(0, 1);  // Row 0, Column 1
///
/// // By tag name
/// let value = loop_.get_by_tag(0, "_col2");  // Row 0, "_col2" column
///
/// // Get entire column
/// let column = loop_.get_column("_col1");
/// ```
///
/// # Validation
///
/// The parser ensures that:
/// - Number of values is divisible by number of tags
/// - Each row has exactly the right number of values
/// - Empty loops (tags but no values) are valid
#[derive(Debug, Clone)]
pub struct CifLoop {
    /// Column names/headers (CIF tags starting with `_`)
    pub tags: Vec<String>,
    /// Data organized as rows, each containing one value per tag
    pub values: Vec<Vec<CifValue>>,
}

impl Default for CifLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl CifLoop {
    /// Create a new empty loop
    pub fn new() -> Self {
        CifLoop {
            tags: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Get the number of rows in the loop
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the loop is empty (no rows)
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get a specific value by row and column index
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let loop_ = &doc.blocks[0].loops[0];
    /// let value = loop_.get(0, 1);  // First row, second column
    /// ```
    pub fn get(&self, row: usize, col: usize) -> Option<&CifValue> {
        self.values.get(row)?.get(col)
    }

    /// Get a specific value by row index and tag name
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let loop_ = &doc.blocks[0].loops[0];
    /// let value = loop_.get_by_tag(0, "_col1");  // First row, "_col1" column
    /// ```
    pub fn get_by_tag(&self, row: usize, tag: &str) -> Option<&CifValue> {
        let col = self.tags.iter().position(|t| t == tag)?;
        self.get(row, col)
    }

    /// Get all values for a specific tag (column)
    ///
    /// Returns `None` if the tag doesn't exist.
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\nval3 val4\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let loop_ = &doc.blocks[0].loops[0];
    /// let column = loop_.get_column("_col1");  // All values in "_col1"
    /// ```
    pub fn get_column(&self, tag: &str) -> Option<Vec<&CifValue>> {
        let col = self.tags.iter().position(|t| t == tag)?;
        Some(self.values.iter().map(|row| &row[col]).collect())
    }

    /// Iterate over rows as vectors of values
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\nval3 val4\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let loop_ = &doc.blocks[0].loops[0];
    /// for row in loop_.rows() {
    ///     // Process each row
    ///     assert_eq!(row.len(), 2);
    /// }
    /// ```
    pub fn rows(&self) -> impl Iterator<Item = &Vec<CifValue>> {
        self.values.iter()
    }

    /// Iterate over all tags (column names)
    ///
    /// # Examples
    /// ```
    /// # use cif_parser::Document;
    /// # let cif = "data_test\nloop_\n_col1\n_col2\nval1 val2\n";
    /// # let doc = Document::parse(cif).unwrap();
    /// # let loop_ = &doc.blocks[0].loops[0];
    /// let tags: Vec<_> = loop_.tags_iter().collect();
    /// assert_eq!(tags, vec![&"_col1".to_string(), &"_col2".to_string()]);
    /// ```
    pub fn tags_iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
}
