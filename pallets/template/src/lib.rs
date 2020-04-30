#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{
    decl_module, decl_storage, decl_event, dispatch,
    traits::{Currency, Imbalance},
    weights::{DispatchClass, Weight, WeighData, ClassifyDispatch, PaysFee},
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Currency: Currency<Self::AccountId>;
}

// 32 bytes seems to be a maximum value that can be stored as the key; if the proof is larger,
// it can be hashed to that size, securing the proof in the process
type Proof = [u8; 32];

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(fn something): Option<u32>;
		// a hashmap between a hash of the proof and the current block number (much easier
		// and arguably more meaningful than a timestamp in the context of a blockchain)
		ProofAndStamp: map hasher(identity) Proof => Option<T::BlockNumber>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
	    Balance = BalanceOf<T>,
	    AccountId = <T as system::Trait>::AccountId,
	{
		/// Just a dummy event.
		/// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		/// To emit this event, we call the deposit function, from our runtime functions
		SomethingStored(u32, AccountId),
		/// Some money was issued by `AccountId`
		MoneyIssued(AccountId, Balance),
		/// Some money was issued by `AccountId` and it was added to its balance
		FreeMoneyGiven(AccountId, Balance),
	}
);

pub struct FreeInitialTx;

impl<T> WeighData<T> for FreeInitialTx {
	fn weigh_data(&self, _: T) -> Weight {
	    0
	}
}

impl<T> PaysFee<T> for FreeInitialTx {
	fn pays_fee(&self, _: T) -> bool {
		false
	}
}

impl<T> ClassifyDispatch<T> for FreeInitialTx {
	fn classify_dispatch(&self, _: T) -> DispatchClass {
		Default::default()
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			// Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			Something::put(something);

			// Here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn issue_money(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult {
    		let who = ensure_signed(origin)?;
		    T::Currency::issue(amount);

            Self::deposit_event(RawEvent::MoneyIssued(who, amount));
		    Ok(())
		}

		#[weight = FreeInitialTx]
        pub fn give_me_money(origin, proof: Proof, amount: BalanceOf<T>) -> dispatch::DispatchResult {
          	let who = ensure_signed(origin)?;

          	// verify the proof here, possibly only if this is the very first tx posted by the
            // `who` account in order to avoid a DoS vector

            // issue the desired amount
          	T::Currency::issue(amount);

            // endow own account with the issued amount
            if T::Currency::deposit_creating(&who, amount).peek() != amount {
                // undo the issuance if the account was not endowed with the desired amount
                T::Currency::burn(amount);
            } else {
                // save the proof along with the current block number
                <ProofAndStamp<T>>::insert(proof, <system::Module<T>>::block_number());
                // report success if everything went well
                Self::deposit_event(RawEvent::FreeMoneyGiven(who, amount));
            }

            Ok(())
        }
	}
}
