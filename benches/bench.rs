use std::iter::repeat_with;

use bin_packing::{offline::*, online::*, *};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, PlotConfiguration};

const BIN_SIZE: usize = 10;

#[derive(Default)]
struct BinImpl {
    used: usize,
}
impl Bin for BinImpl {
    fn capacity() -> usize {
        BIN_SIZE
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

/// Packs `size` random items into bins using the given strategy.
fn pack_online_with_strategy(strategy: impl crate::online::Strategy, size: usize) -> Vec<BinImpl> {
    let mut bins = Vec::<BinImpl>::new();
    pack_bins(
        strategy,
        &mut bins,
        repeat_with(|| ItemImpl::new(rand::random::<usize>() % BIN_SIZE)).take(size),
    );
    bins
}

pub fn compare_online_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("online_comparison");

    for size in [5, 10, 25, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("FF", size), size, |b, size| {
            b.iter(|| pack_online_with_strategy(FirstFit, *size))
        });
        group.bench_with_input(BenchmarkId::new("NF", size), size, |b, size| {
            b.iter(|| pack_online_with_strategy(NextFit, *size))
        });
        group.bench_with_input(BenchmarkId::new("BF", size), size, |b, size| {
            b.iter(|| pack_online_with_strategy(BestFit, *size))
        });
        group.bench_with_input(BenchmarkId::new("WF", size), size, |b, size| {
            b.iter(|| pack_online_with_strategy(WorstFit, *size))
        });
        group.bench_with_input(BenchmarkId::new("AWF", size), size, |b, size| {
            b.iter(|| pack_online_with_strategy(AlmostWorstFit, *size))
        });
    }

    group.finish();
}

fn pack_offline_with_strategy(
    strategy: impl crate::offline::Strategy,
    size: usize,
) -> Vec<BinImpl> {
    let mut bins = Vec::<BinImpl>::new();
    strategy.pack_all(
        &mut bins,
        &mut repeat_with(|| ItemImpl::new(rand::random::<usize>() % BIN_SIZE))
            .take(size)
            .collect(),
    );
    bins
}

pub fn compare_offline_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("offline_comparison");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    for size in [
        5, 10, 25, 50, 100, 150, 200, 250, 500, 750, 1_000, 1_500, 2_000, 2_500, 5_000, 7_500,
        10_000,
    ]
    .iter()
    {
        group.bench_with_input(BenchmarkId::new("FFD", size), size, |b, size| {
            b.iter(|| pack_offline_with_strategy(FirstFitDecreasing, *size))
        });
        group.bench_with_input(BenchmarkId::new("BFD", size), size, |b, size| {
            b.iter(|| pack_offline_with_strategy(BestFitDecreasing, *size))
        });
        group.bench_with_input(BenchmarkId::new("MFFD", size), size, |b, size| {
            b.iter(|| pack_offline_with_strategy(ModifiedFirstFitDecreasing, *size))
        });
    }

    group.finish();
}

criterion_group!(online, compare_online_strategies);
criterion_group!(offline, compare_offline_strategies);
criterion_main!(online, offline);
