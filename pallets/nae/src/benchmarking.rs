//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::benchmarks;
use frame_support::{pallet_prelude::*, storage::bounded_vec::BoundedVec};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
	sort_vector {
		let x in 0 .. 10000;
		let mut m = Vec::<u32>::new();
		for i in (0..x).rev() {
			m.push(i);
		}
	}: {
		// The benchmark execution phase could also be a closure with custom code
		m.sort();
	}

	impl_benchmark_test_suite!(Nae, crate::mock::new_test_ext(), crate::mock::Test);
}
