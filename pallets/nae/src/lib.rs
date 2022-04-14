#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, storage::bounded_vec::BoundedVec};
	use frame_system::pallet_prelude::*;
	use sp_runtime::RuntimeDebug;

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Clone, PartialEq, Eq, PartialOrd, Ord)]
	// #[scale_info(skip_type_params(T))]
	pub struct Change { // TODO <T: Config> {
		/// primary object of relation
		pub primary: ID,

		/// description of relation between primary object and value
		pub relation: ID, // TODO BoundedVec<ID, T::MaxContent>,

		/// value before modification
		pub before: Option<Value>,

		/// value after modification
		pub after: Option<Value>,
	}

	// impl<T: Config> Debug for Change<T> {
	// impl std::fmt::Debug for Change {
	// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	// 		write!(f, "Change {} {:?} > {:?}", self.primary, self.before, self.after)
	// 	}
	// }

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Clone, PartialEq, Eq, PartialOrd, Ord)]
	pub enum Value {
		ID(ID)
	}

	pub type ID = u128;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///
		type MaxChanges: Get<u32>;
		///
		type MaxContent: Get<u32>;
	}

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when changes has been accepted.
		MutationAccepted(T::AccountId), // TODO BoundedVec<Change<T>, T::MaxChanges>
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
	pub(super) type Memory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ID,
		Blake2_128Concat,
		ID, // TODO BoundedVec<ID, T::MaxContent>,
		Value // TODO (T::BlockNumber, Value),
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn modify(
			origin: OriginFor<T>,
			change: Change // TODO BoundedVec<Change<T>, T::MaxChanges>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

			// Get the block number from the FRAME System pallet.
			// let current_block = <frame_system::Pallet<T>>::block_number();

			// for change in changes {

				// Verify that before states correct
				let current = Memory::<T>::get(change.primary, &change.relation);
				ensure!(current == change.before, Error::<T>::BeforeStateMismatch);

				// mutate storage
				match change.after {
					None => Memory::<T>::remove(change.primary, change.relation.clone()),
					Some(v) => Memory::<T>::insert(change.primary, change.relation.clone(), v),
				}
			// }

			// Emit an event that the claim was created.
			Self::deposit_event(Event::MutationAccepted(sender)); // TODO , changes));

			Ok(())
		}
	}
}
