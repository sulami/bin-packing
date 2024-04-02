//! Offline bin packing strategies.
//!
//! Offline strategies have access to all items in advance and can optimize the packing process
//! accordingly.

use std::cmp::Reverse;

use crate::online::Strategy as OnlineStrategy;
use crate::*;

/// An offline strategy that packs items into bins, having access all items in advance.
pub trait Strategy {
    /// Packs all items into bins, draining the items vector.
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>);
}

/// An offline strategy that orders the item by descending size and packs them using
/// [`crate::online::FirstFit`].
pub struct FirstFitDecreasing;
impl Strategy for FirstFitDecreasing {
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>) {
        items.sort_unstable_by_key(|item| Reverse(item.size()));
        while let Some(item) = items.pop() {
            match crate::online::FirstFit.next_idx(bins, &item) {
                Some(i) => bins[i].pack(item),
                None => {
                    bins.push(Default::default());
                    bins.last_mut().unwrap().pack(item);
                }
            }
        }
    }
}

/// An offline strategy that orders the item by descending size and packs them using
/// [`crate::online::BestFit`].
pub struct BestFitDecreasing;
impl Strategy for BestFitDecreasing {
    fn pack_all<B: Bin>(&self, bins: &mut Vec<B>, items: &mut Vec<impl Item>) {
        items.sort_unstable_by_key(|item| Reverse(item.size()));
        while let Some(item) = items.pop() {
            match crate::online::BestFit.next_idx(bins, &item) {
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
impl Strategy for ModifiedFirstFitDecreasing {
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
                if idx == bins.len() {
                    bins.push(Default::default());
                    bins.last_mut().unwrap().pack(large_item);
                    break;
                }
                if large_item.size() < bins[idx].available() {
                    bins[idx].pack(large_item);
                    break;
                }
                idx += 1;
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
