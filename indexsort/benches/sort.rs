use divan::{Bencher, black_box};
use rand::RngExt;

fn main() {
  divan::main();
}

const SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

fn gen_random(n: usize) -> Vec<i64> {
  let mut rng = rand::rng();
  (0..n).map(|_| rng.random::<i64>()).collect()
}

fn gen_sorted(n: usize) -> Vec<i64> {
  (0..n as i64).collect()
}

fn gen_reversed(n: usize) -> Vec<i64> {
  (0..n as i64).rev().collect()
}

fn gen_few_unique(n: usize) -> Vec<i64> {
  let mut rng = rand::rng();
  (0..n)
    .map(|_| (rng.random_range(0i32..10)) as i64)
    .collect()
}

// --- Unstable sort: indexsort::sort_slice vs [T]::sort_unstable ---

#[divan::bench_group]
mod unstable_random {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_random(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort_unstable(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_random(n))
      .bench_local_values(|mut data| {
        data.sort_unstable();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod unstable_sorted {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_sorted(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort_unstable(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_sorted(n))
      .bench_local_values(|mut data| {
        data.sort_unstable();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod unstable_reversed {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_reversed(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort_unstable(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_reversed(n))
      .bench_local_values(|mut data| {
        data.sort_unstable();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod unstable_few_unique {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_few_unique(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort_unstable(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_few_unique(n))
      .bench_local_values(|mut data| {
        data.sort_unstable();
        black_box(data)
      });
  }
}

// --- Stable sort: indexsort::sort_slice_stable vs [T]::sort ---

#[divan::bench_group]
mod stable_random {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_random(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice_stable(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_random(n))
      .bench_local_values(|mut data| {
        data.sort();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod stable_sorted {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_sorted(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice_stable(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_sorted(n))
      .bench_local_values(|mut data| {
        data.sort();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod stable_reversed {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_reversed(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice_stable(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_reversed(n))
      .bench_local_values(|mut data| {
        data.sort();
        black_box(data)
      });
  }
}

#[divan::bench_group]
mod stable_few_unique {
  use super::*;

  #[divan::bench(args = SIZES)]
  fn indexsort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_few_unique(n))
      .bench_local_values(|mut data| {
        indexsort::sort_slice_stable(&mut data, |d, i, j| d[i] < d[j]);
        black_box(data)
      });
  }

  #[divan::bench(args = SIZES)]
  fn std_sort(bencher: Bencher, n: usize) {
    bencher
      .with_inputs(|| gen_few_unique(n))
      .bench_local_values(|mut data| {
        data.sort();
        black_box(data)
      });
  }
}
