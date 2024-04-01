//! One-dimensional bin-packing strategies
//!
//! Strategies are dividided into two categories: online and offline. Online strategies pack items
//! into bins as they arrive, while offline strategies have access to all items in advance.

pub mod offline;
pub mod online;

/// A bin that can hold items.
pub trait Bin: Default {
    /// Returns the total capacity of the bin.
    fn capacity() -> usize;
    /// Returns the available capacity of the bin.
    fn available(&self) -> usize;
    /// Packs an item into the bin.
    fn pack(&mut self, item: impl Item);
}

/// An item that can be packed into a bin.
pub trait Item {
    /// Returns the size of the item.
    fn size(&self) -> usize;
}
