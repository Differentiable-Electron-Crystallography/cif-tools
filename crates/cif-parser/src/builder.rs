//! Builder utilities for constructing CIF blocks with proper state management.

use crate::ast::{CifBlock, CifFrame, CifLoop, CifValue};

/// Internal helper for building CIF blocks while managing pending loop state.
///
/// # The "Pending Loop State" Problem
///
/// In CIF files, loops can be interrupted by other elements, creating a complex
/// state management challenge. Consider this example:
///
/// ```text
/// data_test
/// loop_
/// _atom.id
/// _atom.type
/// _some_other_item  other_value    # This interrupts the loop!
/// 1 C
/// 2 N
/// ```
///
/// The parser encounters elements in this sequence:
/// 1. `loop_` keyword - signals start of a loop
/// 2. Loop tags (`_atom.id`, `_atom.type`) - defines the columns
/// 3. **Data item** (`_some_other_item other_value`) - interrupts!
/// 4. Loop values (`1 C`, `2 N`) - completes the original loop
///
/// # State Management Challenge
///
/// When the parser sees `_some_other_item`, it must:
/// 1. Finalize the incomplete loop (with tags but no values yet)
/// 2. Add it to the block
/// 3. Process the data item
/// 4. Continue parsing
///
/// Without proper state management, the parser would either:
/// - Lose the incomplete loop
/// - Mix the data item with the loop
/// - Crash on malformed structure
///
/// # Solution: BlockBuilder
///
/// The `BlockBuilder` encapsulates this state management:
///
/// ```rust,ignore
/// let mut builder = BlockBuilder::new("test".to_string());
///
/// // Start a loop
/// builder.start_loop(incomplete_loop);
///
/// // Add a data item - this automatically finalizes the pending loop
/// builder.add_item("_other_item".to_string(), value);
///
/// // The pending loop has been safely added to the block
/// let block = builder.finish();
/// ```
///
/// # Key Benefits
///
/// - **Automatic finalization**: Pending loops are automatically added when needed
/// - **No manual state tracking**: Eliminates repetitive `if let Some(loop_) = ...` code
/// - **Error prevention**: Impossible to forget to finalize a pending loop
/// - **Clean code**: Parse methods focus on their core logic, not state management
///
/// # Usage Pattern
///
/// The builder follows a clear pattern for each type of element:
/// - **Data items**: `add_item()` - finalizes pending loop, then adds item
/// - **New loops**: `start_loop()` - finalizes pending loop, then starts new one
/// - **Save frames**: `add_frame()` - finalizes pending loop, then adds frame
/// - **Block completion**: `finish()` - finalizes any remaining pending loop
pub(crate) struct BlockBuilder {
    /// The block being constructed
    block: CifBlock,
    /// Current incomplete loop waiting for values or finalization
    pending_loop: Option<CifLoop>,
}

impl BlockBuilder {
    /// Create a new builder for a block with the given name
    pub(crate) fn new(name: String) -> Self {
        Self {
            block: CifBlock::new(name),
            pending_loop: None,
        }
    }

    /// Get a mutable reference to the block (for direct name updates, etc.)
    pub(crate) fn block_mut(&mut self) -> &mut CifBlock {
        &mut self.block
    }

    /// Finalize any pending loop and add a data item
    pub(crate) fn add_item(&mut self, tag: String, value: CifValue) {
        self.finalize_pending_loop();
        self.block.items.insert(tag, value);
    }

    /// Finalize any pending loop and start a new one
    pub(crate) fn start_loop(&mut self, loop_: CifLoop) {
        self.finalize_pending_loop();
        self.pending_loop = Some(loop_);
    }

    /// Finalize any pending loop and add a frame
    pub(crate) fn add_frame(&mut self, frame: CifFrame) {
        self.finalize_pending_loop();
        self.block.frames.push(frame);
    }

    /// Finalize any pending loop by adding it to the block
    fn finalize_pending_loop(&mut self) {
        if let Some(loop_) = self.pending_loop.take() {
            self.block.loops.push(loop_);
        }
    }

    /// Consume the builder and return the completed block
    pub(crate) fn finish(mut self) -> CifBlock {
        self.finalize_pending_loop();
        self.block
    }
}
