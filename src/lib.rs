//! One-dimensional bin-packing strategies
//!
//! Strategies are dividided into two categories: online and offline. Online strategies pack items
//! into bins as they arrive, while offline strategies have access to all items in advance.

use std::cmp::Reverse;

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

/// Packs bins with items using a given online strategy, creating new bins as needed.
pub fn pack_bins<B: Bin>(
    strategy: impl OnlineStrategy,
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
    strategy: impl OnlineStrategy,
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
pub trait OnlineStrategy {
    /// Returns the index of the next bin to pack the item into, or `None` if no bin is suitable.
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize>;
}

/// An online strategy that packs items into the first bin that has enough capacity.
pub struct FirstFit;
impl OnlineStrategy for FirstFit {
    fn next_idx(&self, bins: &[impl Bin], item: &impl Item) -> Option<usize> {
        for (i, bin) in bins.iter().enumerate() {
            if item.size() <= bin.available() {
                return Some(i);
            }
        }
        None
    }
}

/// An online strategy that packs items into the bin with the least available capacity.
pub struct BestFit;
impl OnlineStrategy for BestFit {
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

/// An offline strategy that packs items into bins, having access all items in advance.
pub trait OfflineStrategy {
    /// Packs all items into bins, draining the items vector.
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>);
}

/// An offline strategy that orders the item by descending size and packs them using [`FirstFit`].
pub struct FirstFitDecreasing;
impl OfflineStrategy for FirstFitDecreasing {
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>) {
        items.sort_unstable_by_key(|item| Reverse(item.size()));
        while let Some(item) = items.pop() {
            match FirstFit.next_idx(bins, &item) {
                Some(i) => bins[i].pack(item),
                None => {
                    bins.push(Default::default());
                    bins.last_mut().unwrap().pack(item);
                }
            }
        }
    }
}

/// An offline strategy that orders the item by descending size and packs them using [`BestFit`].
pub struct BestFitDecreasing;
impl OfflineStrategy for BestFitDecreasing {
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>) {
        items.sort_unstable_by_key(|item| Reverse(item.size()));
        while let Some(item) = items.pop() {
            match BestFit.next_idx(bins, &item) {
                Some(i) => bins[i].pack(item),
                None => {
                    bins.push(Default::default());
                    bins.last_mut().unwrap().pack(item);
                }
            }
        }
    }
}

/// An offline strategy that orders the item by descending size and packs them using a modified
/// version of [`FirstFitDecreasing`], which classifies items by size and improves on regular FFD
/// for items larger than half the bin capacity.
pub struct ModifiedFirstFitDecreasing;
impl OfflineStrategy for ModifiedFirstFitDecreasing {
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>) {
        // Group items by size.
        let mut large = vec![];
        let mut medium = vec![];
        let mut small = vec![];
        let mut tiny = vec![];
        for item in items.drain(..) {
            match item.size() {
                s if s > B::capacity() / 2 => large.push(item),
                s if s > B::capacity() / 3 => medium.push(item),
                s if s > B::capacity() / 6 => small.push(item),
                _ => tiny.push(item),
            }
        }

        // Sort all large items into separate bins, adding new ones as needed.
        large.sort_unstable_by_key(|item| Reverse(item.size()));
        let mut idx = 0;
        for large_item in large {
            loop {
                if large_item.size() < bins[idx].available() {
                    bins[idx].pack(large_item);
                    break;
                }
                idx += 1;
                if idx == bins.len() {
                    bins.push(Default::default());
                    bins.last_mut().unwrap().pack(large_item);
                    break;
                }
            }
        }

        // Place the largest remaining medium item that fits in each bin.
        medium.sort_unstable_by_key(|item| Reverse(item.size()));
        for bin in bins.iter_mut() {
            if let Some(item_idx) = medium
                .iter()
                .position(|item| item.size() <= bin.available())
            {
                bin.pack(medium.remove(item_idx));
                if medium.is_empty() {
                    break;
                }
            }
        }

        // Place the smallest and largest remaining small items that fit in each bin, going
        // backwards.
        small.sort_unstable_by_key(|item| Reverse(item.size()));
        for bin in bins.iter_mut().rev() {
            if small.is_empty() {
                break;
            }
            if small.iter().rev().take(2).map(Item::size).sum::<usize>() > bin.available() {
                continue;
            }
            bin.pack(small.pop().unwrap());
            if let Some(largest_idx) = small.iter().position(|item| item.size() <= bin.available())
            {
                bin.pack(small.remove(largest_idx));
            }
        }

        // Place the largest remaining items that fits in each bin.
        tiny.sort_unstable_by_key(|item| Reverse(item.size()));
        for bin in bins.iter_mut() {
            while !medium.is_empty() && medium.first().unwrap().size() <= bin.available() {
                bin.pack(medium.remove(0));
            }
            while !small.is_empty() && small.first().unwrap().size() <= bin.available() {
                bin.pack(small.remove(0));
            }
            while !tiny.is_empty() && tiny.first().unwrap().size() <= bin.available() {
                bin.pack(tiny.remove(0));
            }
        }

        // Use FFD to pack the remaining items into new bins.
        let mut remainder = medium
            .into_iter()
            .chain(small)
            .chain(tiny)
            .collect::<Vec<_>>();
        FirstFitDecreasing.pack_all(bins, &mut remainder);
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
