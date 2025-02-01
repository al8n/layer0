#[cfg(feature = "xxhash3")]
use bloomur::hasher::Xxh3;
use bloomur::{bits_per_key, Filter};

#[cfg(feature = "xxhash32")]
use bloomur::hasher::Xxh32;

use fastbloom_rs::Membership;
use rand::{thread_rng, RngCore};

use divan::Bencher;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

fn keys() -> Vec<Vec<u8>> {
  const LEN: usize = 128;
  const NUM: usize = 1024;

  (0..NUM)
    .map(|_| {
      let mut rng = thread_rng();
      let mut k = std::vec![0; LEN];
      rng.fill_bytes(&mut k);
      k
    })
    .collect()
}

#[divan::bench]
fn bloomur(bencher: Bencher) {
  bencher
    .with_inputs(|| {
      let b = bits_per_key(100000, 0.001);
      (b, keys())
    })
    .bench_local_values(|(bpk, keys)| {
      let mut f = Filter::<512>::with_bits_per_key(bpk);

      for k in keys.iter() {
        f.insert(k);
      }

      f.finalize();
    });
}

#[divan::bench]
#[cfg(feature = "xxhash32")]
fn bloomur_xxhash32(bencher: Bencher) {
  bencher
    .with_inputs(|| {
      let b = bits_per_key(100000, 0.001);
      (b, keys())
    })
    .bench_local_values(|(bpk, keys)| {
      let mut f = Filter::<512, Xxh32>::with_bits_per_key_and_hasher(bpk, Xxh32::default());

      for k in keys.iter() {
        f.insert(k);
      }

      f.finalize();
    });
}

#[divan::bench]
#[cfg(feature = "xxhash3")]
fn bloomur_xxhash3(bencher: Bencher) {
  bencher
    .with_inputs(|| {
      let b = bits_per_key(100000, 0.001);
      (b, keys())
    })
    .bench_local_values(|(bpk, keys)| {
      let mut f = Filter::<512, Xxh3>::with_bits_per_key_and_hasher(bpk, Xxh3::default());

      for k in keys.iter() {
        f.insert(k);
      }

      f.finalize();
    });
}

#[divan::bench]
fn bloomfilter(bencher: Bencher) {
  bencher.with_inputs(keys).bench_local_values(|keys| {
    let mut f = bloomfilter::Bloom::new_for_fp_rate(100000, 0.001).unwrap();

    for k in keys.iter() {
      f.set(k);
    }

    f.to_bytes();
  });
}

#[divan::bench]
fn fastbloom(bencher: Bencher) {
  bencher.with_inputs(keys).bench_local_values(|keys| {
    let mut f = fastbloom::BloomFilter::with_false_pos(0.001).expected_items(100000);

    for k in keys.iter() {
      f.insert(k);
    }

    f.as_slice().to_vec();
  });
}

#[divan::bench]
fn fastbloom_rs(bencher: Bencher) {
  bencher.with_inputs(keys).bench_local_values(|keys| {
    let mut f = fastbloom_rs::FilterBuilder::new(100000, 0.001).build_bloom_filter();

    for k in keys.iter() {
      f.add(k);
    }

    f.get_u8_array().to_vec();
  });
}
