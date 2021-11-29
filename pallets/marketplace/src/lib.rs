// This file is part of Web3Games.

// Copyright (C) 2021 Web3Games https://web3games.org
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]


use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::ExistenceRequirement::AllowDeath;
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec,
};
use frame_support::sp_runtime::traits::AccountIdConversion;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;
use primitives::{Balance, TokenId};
use scale_info::TypeInfo;
use sp_runtime::{traits::One, RuntimeDebug};
use sp_std::prelude::*;

pub use pallet::*;
use crate::NftType::{MultiToken, NonFungibleToken};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type CollectionId = u32;
pub type NftId = u32;
pub type SaleId = u32;


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum NftType {
	NonFungibleToken,
	MultiToken,
}

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Collection<AccountId, BoundedString> {
	/// Class owner
	pub owner: AccountId,
	// The type of nft
	pub nft_type: NftType,
	/// The account of nft
	pub nft_id: NftId,
	/// Metadata from ipfs
	pub metadata: BoundedString,
	/// Escrow account
	pub escrow_account:AccountId
}

/// Collection Sale
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Sale<AccountId> {
	owner_id:AccountId,
	token_id:TokenId,
	price:Balance,
	amount :Balance,
	// pub is_auction: bool,

}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::PalletId;
	use frame_system::pallet_prelude::*;
	use crate::NftType::{MultiToken, NonFungibleToken};

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_token_non_fungible::Config + pallet_token_multi::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;


		#[pallet::constant]
		type PalletId: Get<PalletId>;


		/// The maximum length of metadata stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Collections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Collection<T::AccountId, BoundedVec<u8, <T as pallet::Config>::StringLimit>>,
	>;

	#[pallet::storage]
	pub(super) type CollectionSale<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Blake2_128Concat,
		SaleId,
		Sale<T::AccountId>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_collectionsale_id)]
	pub(super) type NextCollectionSaleId<T: Config> = StorageMap<_,Blake2_128Concat,CollectionId,SaleId,ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;



	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated(CollectionId, T::AccountId),
		CollectionSaleCreated(CollectionId,SaleId,T::AccountId),
		CollectionSaleDestroyed(CollectionId,SaleId,T::AccountId),
		CollectionSaleUpdated(CollectionId,SaleId,T::AccountId),
		CollectionDestroyed(CollectionId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		TokenNotFound,
		CollectionFound,
		NumOverflow,
		NoAvailableCollectionId,
		CollectionNotFound,
		NoAvailableAssetId,
		AssetNotFound,
		InvalidQuantity,
		NoPermission,
		NoOwnerShip,
		ConfuseBehavior,
		CannotDestroyCollection,
		BadMetadata,
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_collection(
			origin: OriginFor<T>,
			nft_type: NftType,
			nft_id: NftId,
			metadata: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if nft_type == NonFungibleToken{
				let nft_id:T::NonFungibleTokenId = nft_id.into();
				ensure!(
				pallet_token_non_fungible::Pallet::<T>::exists(nft_id),
				Error::<T>::TokenNotFound,);
			}else if nft_type == MultiToken {
				let nft_id:T::MultiTokenId = nft_id.into();
				ensure!(
				pallet_token_multi::Pallet::<T>::exists(nft_id),
				Error::<T>::TokenNotFound);
			}
			Self::do_create_collection(&who, nft_type,nft_id, metadata)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn add_sale(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
			price: Balance,
			amount:Balance
		) -> DispatchResult {
			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
			Self::do_add_sale(origin, collection_id, token_id, price,amount)?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn remove_sale(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			sale_id:SaleId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_remove_sale(&who, collection_id, sale_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn update_price(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			sale_id:SaleId,
			price: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
			let owner_id = CollectionSale::<T>::get(collection_id,sale_id).unwrap().owner_id;
			ensure!(who == owner_id, Error::<T>::NoOwnerShip);

			Self::do_update_price(&who, collection_id, sale_id, price)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn update_amount(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			sale_id:SaleId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
			let owner_id = CollectionSale::<T>::get(collection_id,sale_id).unwrap().owner_id;
			ensure!(who == owner_id, Error::<T>::NoOwnerShip);

			Self::do_update_amount(&who, collection_id, sale_id, amount)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			sale_id:SaleId,
			price:Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
			let owner_id = CollectionSale::<T>::get(collection_id,sale_id).unwrap().owner_id;
			ensure!(who != owner_id, Error::<T>::ConfuseBehavior);

			Self::do_offer(origin, collection_id, sale_id,price)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn accept_offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			token_id: TokenId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_accept_offer(&who, collection_id, token_id)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_destroy_collection(&who, collection_id)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T>  {

	pub fn do_create_collection(
		who: &T::AccountId,
		nft_type: NftType,
		nft_id: NftId,
		metadata: Vec<u8>,
	) -> Result<CollectionId, DispatchError> {
		let bounded_metadata: BoundedVec<u8, <T as pallet::Config>::StringLimit> =
			metadata.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let collection_id =
			NextCollectionId::<T>::try_mutate(|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;

		let deposit = T::CreateCollectionDeposit::get();
		<T as pallet::Config>::Currency::reserve(who, deposit.clone())?;

		// Generate account from collection_id
		let next_collection_id = NextCollectionId::<T>::get();
		let escrow_account:<T as frame_system::Config>::AccountId =
			<T as pallet::Config>::PalletId::get().into_sub_account(next_collection_id);

		let collection = Collection {
			owner: who.clone(),
			nft_type,
			nft_id,
			metadata: bounded_metadata,
			escrow_account
		};

		Collections::<T>::insert(collection_id, collection);

		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	}

	pub fn do_add_sale(
		origin: OriginFor<T>,
		collection_id: CollectionId,
		token_id: TokenId,
		price: Balance,
		amount:Balance
	) -> DispatchResult {
		let who= ensure_signed(origin.clone())?;
		let escrow_account = Collections::<T>::get(collection_id).unwrap().escrow_account;
		let nft_id = Collections::<T>::get(collection_id).unwrap().nft_id;
		let nft_type = Collections::<T>::get(collection_id).unwrap().nft_type;
		let mut new_sale = Sale{
			owner_id: who.clone(),
			token_id,
			price,
			amount: 0
		};
		if nft_type == NonFungibleToken{
			let nft_id:<T as pallet_token_non_fungible::Config>::NonFungibleTokenId = nft_id.into();
			pallet_token_non_fungible::Pallet::<T>::approve(
				origin.clone(),
				nft_id,
				escrow_account,
				token_id,
			)?;
			new_sale = Sale{
				owner_id: who.clone(),
				token_id,
				price,
				amount: 1
			};
		}else if nft_type == MultiToken {
			let nft_id:<T as pallet_token_multi::Config>::MultiTokenId = nft_id.into();
			pallet_token_multi::Pallet::<T>::set_approval_for_all(
				origin.clone(),
				nft_id,
				escrow_account,
				true,
			)?;
			new_sale = Sale{
				owner_id: who.clone(),
				token_id,
				price,
				amount
			};
		}
		let sale_id =
			NextCollectionSaleId::<T>::try_mutate(collection_id, |id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;
		CollectionSale::<T>::insert(collection_id,sale_id,new_sale);
        Self::deposit_event(Event::CollectionSaleCreated(collection_id,sale_id, who));
		Ok(())
	}

	pub fn do_remove_sale(
		who: &T::AccountId,
		collection_id: CollectionId,
		sale_id:SaleId,
	) -> DispatchResult {
		Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
		let owner_id = CollectionSale::<T>::get(collection_id,sale_id).unwrap().owner_id;
		ensure!(*who == owner_id, Error::<T>::NoOwnerShip);
		CollectionSale::<T>::remove(collection_id,sale_id);
		Self::deposit_event(Event::CollectionSaleDestroyed(collection_id, sale_id,who.clone()));
		Ok(())
	}

	pub fn do_update_price(
		who: &T::AccountId,
		collection_id: CollectionId,
		sale_id:SaleId,
		price: Balance,
	) -> DispatchResult {
		let old_sale = CollectionSale::<T>::get(collection_id,sale_id).unwrap();
		let new_sale = Sale{
			owner_id: old_sale.owner_id,
			token_id:old_sale.token_id,
			price,
			amount: old_sale.amount
		};
		CollectionSale::<T>::insert(collection_id,sale_id,new_sale);
		Self::deposit_event(Event::CollectionSaleUpdated(collection_id, sale_id,who.clone()));
		Ok(())
	}


	pub fn do_update_amount(
		who: &T::AccountId,
		collection_id: CollectionId,
		sale_id:SaleId,
		amount: Balance,
	) -> DispatchResult {
		let old_sale = CollectionSale::<T>::get(collection_id,sale_id).unwrap();
		let new_sale = Sale{
			owner_id: old_sale.owner_id,
			token_id:old_sale.token_id,
			price:old_sale.price,
			amount
		};
		CollectionSale::<T>::insert(collection_id,sale_id,new_sale);
		Self::deposit_event(Event::CollectionSaleUpdated(collection_id, sale_id,who.clone()));
		Ok(())
	}

	pub fn do_offer(
		origin: OriginFor<T>,
		collection_id: CollectionId,
		sale_id:SaleId,
		price:Balance,
	) -> DispatchResult  {
		let who = ensure_signed(origin)?;
		let sale_price:Balance = CollectionSale::<T>::get(collection_id,sale_id).unwrap().price;
		let escrow_account = Collections::<T>::get(collection_id).unwrap().escrow_account;
		let nft_type = Collections::<T>::get(collection_id).unwrap().nft_type;
		let nft_id = Collections::<T>::get(collection_id).unwrap().nft_id;
		let token_id = CollectionSale::<T>::get(collection_id,sale_id).unwrap().token_id;
		if sale_price == price{
			<T as Config>::Currency::transfer(&who, &escrow_account, sale_price, AllowDeath)?;
			if nft_type == NonFungibleToken{
				let nft_id:<T as pallet_token_non_fungible::Config>::NonFungibleTokenId = nft_id.into();
				pallet_token_non_fungible::Pallet::<T>::transfer_from(
					origin,
					nft_id,
					escrow_account.clone(),
					who,
					token_id
				)?;
			}else if nft_type == MultiToken {
				let nft_id:<T as pallet_token_multi::Config>::MultiTokenId = nft_id.into();
				let amount = CollectionSale::<T>::get(collection_id,sale_id).unwrap().amount;
				pallet_token_multi::Pallet::<T>::transfer_from(
					origin,
					nft_id,
					escrow_account.clone(),
					who,
					token_id,
					amount
				)?;
			}
		}else{
			<T as Config>::Currency::transfer(&who, &escrow_account, price, AllowDeath)?;
		}
		Ok(())
	}

	pub fn do_accept_offer(
		_who: &T::AccountId,
		_collection_id: CollectionId,
		_token_id: TokenId,
	) -> DispatchResult {
		Ok(())
	}

	pub fn do_destroy_collection(
		_who: &T::AccountId,
		_collection_id: CollectionId,
	) -> DispatchResult {
		// Collections::<T>::try_mutate_exists(collection_id, |collection| -> DispatchResult {
		// 	let c = collection.take().ok_or(Error::<T>::CollectionNotFound)?;
		// 	ensure!(c.owner == *who, Error::<T>::NoPermission);
		//
		// 	let deposit = T::CreateCollectionDeposit::get();
		// 	<T as pallet::Config>::Currency::unreserve(who, deposit);
		//
		// 	Self::deposit_event(Event::CollectionDestroyed(collection_id, who.clone()));
		//
		// 	Ok(())
		// })
		todo!()

	}
}
