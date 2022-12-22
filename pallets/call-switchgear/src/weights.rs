// This file is part of Web3Games.

// Copyright (C) 2021-2022 Web3Games https://web3games.org
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

//! Autogenerated weights for web3games_call_switchgear
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-09, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/web3games-node
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=web3games_call_switchgear
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --output=./pallets/call-switchgear/src/weights.rs
// --template=./.maintain/w3g-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for web3games_call_switchgear.
pub trait WeightInfo {
	fn switchoff_transaction() -> Weight;
	fn switchon_transaction() -> Weight;
}

/// Weights for web3games_call_switchgear using the Web3Games node and recommended hardware.
pub struct W3GWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for W3GWeight<T> {
	// Storage: CallSwitchgear SwitchedOffTransactions (r:1 w:1)
	fn switchoff_transaction() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: CallSwitchgear SwitchedOffTransactions (r:1 w:1)
	fn switchon_transaction() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn switchoff_transaction() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn switchon_transaction() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
