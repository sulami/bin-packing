//! Online bin packing strategies.
//!
//! Online strategies pack items into bins as they arrive, without knowing the sizes of future
//! items. Consequently, the API allows for sorting one item at a time.

use super::*;

/// Packs bins with items using a given online strategy, creating new bins as needed.
///
/// This is a convenience function to pack a lot of items at once.
pub fn pack_bins<B: Bin>(
    strategy: impl Strategy,
    bins: &mut Vec<B>,
    items: impl IntoIterator<Item = impl Item>,
) {
    for item in items {
        debug_assert!(item.size() <= B::capacity());
        if let Some(i) = strategy.next_idx(bins, &item) {
            bins[i].pack(item);
        } else {
            bins.push(B::default());
            bins.last_mut().unwrap().pack(item);
        }
    }
}

/// Packs bins with items using a given online strategy.
///
/// If the strategy fails to find a suitable bin for an item, the function stops, leaving the
/// remaining items in the iterator.
pub fn pack_existing_bins<B: Bin>(
    strategy: impl Strategy,
    bins: &mut [B],
    items: impl IntoIterator<Item = impl Item>,
) {
    for item in items {
        debug_assert!(item.size() <= B::capacity());
        if let Some(i) = strategy.next_idx(bins, &item) {
            bins[i].pack(item);
        } else {
            break;
        }
    }
}

/// An online strategy for packing items into bins, inspecting one item at a time.
pub trait Strategy {
    /// Returns the index of the next bin to pack the item into, or `None` if no bin is suitable.
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize>;
}

/// An online strategy that packs items into the first bin that has enough capacity.
pub struct FirstFit;
impl Strategy for FirstFit {
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize> {
        for (i, bin) in bins.iter().enumerate() {
            if item.size() <= bin.available() {
                return Some(i);
            }
        }
        None
    }
}

/// An online strategy that packs items into the last bin if possible.
pub struct NextFit;
impl Strategy for NextFit {
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize> {
        if let Some(last_bin) = bins.last() {
            if item.size() <= last_bin.available() {
                return Some(bins.len() - 1);
            }
        }
        None
    }
}

/// An online strategy that packs items into the bin with the least available capacity.
pub struct BestFit;
impl Strategy for BestFit {
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize> {
        let mut best_fit = None;
        for (i, bin) in bins.iter().enumerate() {
            if item.size() <= bin.available() {
                match best_fit {
                    None => best_fit = Some(i),
                    Some(j) => {
                        if bin.available() < bins[j].available() {
                            best_fit = Some(i);
                        }
                    }
                }
            }
        }
        best_fit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct BinImpl {
        used: usize,
    }
    impl Bin for BinImpl {
        fn capacity() -> usize {
            10
        }
        fn available(&self) -> usize {
            Self::capacity() - self.used
        }
        fn pack(&mut self, item: impl Item) {
            assert!(item.size() <= self.available(), "item too large");
            self.used += item.size();
        }
    }

    struct ItemImpl {
        size: usize,
    }
    impl ItemImpl {
        fn new(size: usize) -> Self {
            ItemImpl { size }
        }
    }
    impl Item for ItemImpl {
        fn size(&self) -> usize {
            self.size
        }
    }

    #[test]
    fn first_fit_uses_first_empty_bin() {
        let bins = vec![BinImpl::default(), BinImpl::default()];
        let item = ItemImpl::new(5);
        let strategy = FirstFit;
        assert_eq!(strategy.next_idx(&bins, &item), Some(0));
    }

    #[test]
    fn first_fit_uses_first_bin_with_enough_capacity() {
        let mut bins = vec![BinImpl::default(), BinImpl::default()];
        bins[0].used = 5;
        let item = ItemImpl::new(6);
        let strategy = FirstFit;
        assert_eq!(strategy.next_idx(&bins, &item), Some(1));
    }

    #[test]
    fn pack_all_with_first_fit() {
        let mut bins: Vec<BinImpl> = vec![];
        let items = vec![ItemImpl::new(5), ItemImpl::new(6)];
        pack_bins(FirstFit, &mut bins, items);
        assert_eq!(bins.len(), 2);
    }
}
