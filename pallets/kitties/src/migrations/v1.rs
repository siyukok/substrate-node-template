use codec::EncodeLike;
use frame_support::{
    pallet_prelude::*,
    storage::StoragePrefixedMap,
    traits::GetStorageVersion,
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};

use crate::{Config, Pallet, Kitties, KittyId, Kitty};

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct V0Kitty(pub [u8; 16]);

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct V1Kitty {
    pub dna: [u8; 16],
    pub name: [u8; 4],
}

impl EncodeLike<Kitty> for V1Kitty {}

pub fn migrate<T: Config>() -> Weight {
    let on_chain_version = Pallet::<T>::on_chain_storage_version();
    let current_version = Pallet::<T>::current_storage_version();

    if on_chain_version != StorageVersion::new(0) {
        return Weight::zero();
    }

    if current_version != StorageVersion::new(1) {
        return Weight::zero();
    }

    let module = Kitties::<T>::module_prefix();
    let item = Kitties::<T>::storage_prefix();

    // remove and insert new struct kitty
    for (index, kitty) in storage_key_iter::<KittyId, V0Kitty, Blake2_128Concat>(module, item).drain() {
        let new_kitty = V1Kitty {
            dna: kitty.0,
            name: *b"abcd",
        };
        Kitties::<T>::insert(index, new_kitty);
    }

    Weight::zero()
}