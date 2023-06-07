use frame_support::{
    pallet_prelude::*,
    storage::StoragePrefixedMap,
    traits::GetStorageVersion,
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};

use crate::{Config, Pallet, Kitties, KittyId};

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct V0Kitty(pub [u8; 16]);

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct V1Kitty {
    pub dna: [u8; 16],
    pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
    let on_chain_version: StorageVersion = Pallet::<T>::on_chain_storage_version();
    let current_version: StorageVersion = Pallet::<T>::current_storage_version();

    if current_version != StorageVersion::new(2) {
        return Weight::zero();
    }

    let module = Kitties::<T>::module_prefix();
    let item = Kitties::<T>::storage_prefix();

    match on_chain_version {
        v0 if v0 == StorageVersion::new(0) => migrate_from_v0::<T>(module, item),
        v1 if v1 == StorageVersion::new(1) => migrate_from_v1::<T>(module, item),
        _ => {}
    }

    Weight::zero()
}

fn migrate_from_v0<T: Config>(module: &[u8], item: &[u8]) {
    for (index, kitty) in storage_key_iter::<KittyId, V0Kitty, Blake2_128Concat>(module, item).drain() {
        let new_kitty = crate::Kitty {
            dna: kitty.0,
            name: *b"abcdefgh",
        };
        Kitties::<T>::insert(index, &new_kitty);
    }
}

fn migrate_from_v1<T: Config>(module: &[u8], item: &[u8]) {
    for (index, kitty) in storage_key_iter::<KittyId, V1Kitty, Blake2_128Concat>(module, item).drain() {
        let new_kitty = crate::Kitty {
            dna: kitty.dna,
            name: *b"abcdefgh",
        };
        Kitties::<T>::insert(index, &new_kitty);
    }
}