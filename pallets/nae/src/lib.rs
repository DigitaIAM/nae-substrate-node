#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*, storage::bounded_vec::BoundedVec, CloneNoBound, EqNoBound,
		PartialEqNoBound, RuntimeDebugNoBound,
	};
	use frame_system::{pallet_prelude::*, RawOrigin};
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_io::hashing::blake2_256;
	use sp_std::prelude::*;

	#[derive(
		Encode, Decode, TypeInfo, CloneNoBound, RuntimeDebugNoBound, PartialEqNoBound, EqNoBound,
	)] // TODO MaxEncodedLen
	#[scale_info(skip_type_params(T))]
	pub struct Change<T: Config> {
		/// primary object of relation
		pub primary: ID,

		/// description of relation between primary object and value
		pub relation: BoundedVec<ID, T::MaxRelations>,

		/// value before modification
		pub before: Option<Value>,

		/// value after modification
		pub after: Option<Value>,
	}

	impl<T: Config> Change<T> {
		fn new(
			primary: ID,
			relation: BoundedVec<ID, T::MaxRelations>,
			before: Option<Value>,
			after: Option<Value>,
		) -> Self {
			Change { primary, relation, before, after }
		}
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(
		Encode, Decode, TypeInfo, CloneNoBound, RuntimeDebugNoBound, PartialEqNoBound, EqNoBound,
	)] // TODO MaxEncodedLen
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub enum Value {
		ID(ID),
		IDS(Vec<ID>),
		// TODO IDS(BoundedVec<ID, T::MaxIDS>),
		String(Vec<u8>),
		// TODO String(BoundedVec<u8, T::MaxString>),
	}

	impl Value {
		pub fn id(str: &str) -> Value {
			Value::ID(str.into())
		}

		pub fn string(str: &str) -> Value {
			Value::String(str.as_bytes().to_vec())
		}
	}

	impl Into<Value> for &str {
		fn into(self) -> Value {
			let len = self.len();
			if self.starts_with("\"") && self.ends_with("\"") && len > 2 {
				Value::string(&self[1..len-1])
			} else {
				Value::id(self)
			}
		}
	}

	// pub type ID = u128;

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
		CloneNoBound,
		RuntimeDebugNoBound,
		PartialEqNoBound,
		EqNoBound,
	)]
	pub struct ID {
		bits: [u8; 32],
	}

	impl Into<ID> for &str {
		fn into(self) -> ID {
			ID { bits: blake2_256(self.as_bytes()) }
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info] // TODO remove after switch to BoundedVec at Value
	pub struct Pallet<T>(_);

	// Pallet's genesis configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub memories: Vec<Vec<(ID, Vec<ID>, Value)>>,
		pub root: Option<T::AccountId>,
	}

	// Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			use std::io::BufRead;

			let mut memories: Vec<Vec<(ID, Vec<ID>, Value)>> = Vec::with_capacity(1_000_000);
			let mut changes: Vec<(ID, Vec<ID>, Value)> = Vec::with_capacity(256);

			let file = std::fs::File::open("data/prototype.nae").expect("cannot open genesis config file");
			let reader = std::io::BufReader::new(file);

			for data in reader.lines() {
				let data = data.expect("data");

				// ignore comments
				if data.starts_with("#") {
					continue;
				}

				let line = data.trim();
				if line.is_empty() {
					if changes.is_empty() {
						continue;
					} else {
						memories.push(changes.clone());
						changes.clear();
					}
				} else {
					println!("{}", line);

					let mut parts = line.split(" ").collect::<Vec<&str>>();
					if parts.len() < 2 {
						panic!("unexpected number of items in record: {}", line);
					}

					let primary: ID = parts.remove(0).into();
					let value: Value = parts.remove(parts.len() - 1).into();

					let relation = parts.iter().map(|id| (*id).into()).collect();

					changes.push((primary, relation, value));
				}
			}

			GenesisConfig { memories, root: None }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let mut count: u32 = 0;
			let root = self.root.as_ref().expect("root account");
			for tuples in &self.memories {
				let mut changes = Vec::with_capacity(tuples.len());
				for (primary, relation, value) in tuples {
					changes.push(Change::new(
						primary.clone(),
						relation.clone().try_into().unwrap(),
						None,
						Some(value.clone()),
					))
				}

				println!("{}", count);
				count += 1;

				assert!(Pallet::<T>::modify(
					RawOrigin::Signed(root.clone()).into(),
					changes.try_into().unwrap()
				)
				.is_ok());
			}
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///
		type MaxChanges: Get<u32>;
		///
		type MaxRelations: Get<u32>;
		///
		type MaxIDS: Get<u32>;
		///
		type MaxString: Get<u32>;
	}

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when changes has been accepted.
		MutationAccepted(T::AccountId, BoundedVec<Change<T>, T::MaxChanges>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// no changes
		EmptyChanges,
		/// too many changes
		TooManyChanges,
		/// no relations
		EmptyRelations,
		/// relation vector is not ordered
		RelationsIsNotOrdered,
		/// change must have state mutation
		BeforeAndAfterStatesAreEqual,
		/// before state mismatch
		BeforeStateMismatch,
	}

	#[pallet::storage]
	#[pallet::getter(fn memory)]
	pub(super) type Memory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ID,
		Blake2_128Concat,
		BoundedVec<ID, T::MaxRelations>,
		Value, // TODO (T::BlockNumber, Value),
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn modify(
			origin: OriginFor<T>,
			changes: BoundedVec<Change<T>, T::MaxChanges>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

			ensure!(!changes.is_empty(), Error::<T>::EmptyChanges);

			// Get the block number from the FRAME System pallet.
			// let current_block = <frame_system::Pallet<T>>::block_number();

			// let mut mutations = Vec::with_capacity(changes.len());

			for change in changes.clone() {
				// Verify that before states correct
				let current = Memory::<T>::get(&change.primary, &change.relation);
				ensure!(current == change.before, Error::<T>::BeforeStateMismatch);

				// mutate storage
				match change.after {
					None => Memory::<T>::remove(&change.primary, &change.relation),
					Some(v) => Memory::<T>::insert(&change.primary, &change.relation, v),
				}

				// mutations.push(change);
			}

			// let mutations: BoundedVec<_, _> =
			// 	mutations.try_into().map_err(|()| Error::<T>::TooManyChanges)?;

			// Emit an event that the claim was created.
			Self::deposit_event(Event::MutationAccepted(sender, changes));

			Ok(())
		}
	}
}
