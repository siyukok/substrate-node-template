#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;

pub use weights::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use sp_io::hashing::blake2_128;
    use frame_support::traits::{Randomness, Currency, ExistenceRequirement};
    use frame_support::PalletId;

    use crate::migrations;
    use sp_runtime::traits::AccountIdConversion;

    pub type KittyId = u32;
    pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
    // v0 pub struct Kitty(pub [u8; 16]);
    // v1 pub struct Kitty {
    //     pub dna: [u8; 16],
    //     pub name: [u8; 4],
    // }
    pub struct Kitty {
        pub dna: [u8; 16],
        pub name: [u8; 8],
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type Currency: Currency<Self::AccountId>;
        #[pallet::constant]
        type KittyPrice: Get<BalanceOf<Self>>;
        type PalletId: Get<PalletId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_parents)]
    pub type KittyParents<T> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_on_sale)]
    pub type KittyOnSale<T> = StorageMap<_, Blake2_128Concat, KittyId, BalanceOf<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
        KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
        KittyTransferred { from: T::AccountId, to: T::AccountId, kitty_id: KittyId },
        KittyOnSale { who: T::AccountId, kitty_id: KittyId, price: BalanceOf<T> },
        KittyBought { who: T::AccountId, kitty_id: KittyId },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidKittyId,
        SameKittyId,
        NotKittyOwner,
        AlreadyIsKittyOwner,
        KittyOnSale,
        KittyNotOnSale,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            migrations::v2::migrate::<T>()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let kitty_id = Self::get_next_id()?;
            let kitty = Kitty{
                dna: Self::random_value(&who),
                name,
            };

            let price = T::KittyPrice::get();
            T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?; // 保证金

            Kitties::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &who);

            Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId,name:[u8;8]) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(kitty_id_1!=kitty_id_2,Error::<T>::SameKittyId);

            ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
            ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);

            let kitty_id = Self::get_next_id()?;
            let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
            let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

            let selector = Self::random_value(&who);
            let mut data = [0u8; 16];
            for i in 0..kitty_1.dna.len() {
                data[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & !selector[i]);
            }

            let kitty = Kitty{
                dna: data,
                name,
            };

            let price = T::KittyPrice::get();
            T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

            Kitties::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &who);
            KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

            Self::deposit_event(Event::KittyBreed { who, kitty_id, kitty });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn transfer(origin: OriginFor<T>, recipient: T::AccountId, kitty_id: KittyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            ensure!(owner==who,Error::<T>::NotKittyOwner);

            KittyOwner::<T>::insert(kitty_id, &recipient);

            Self::deposit_event(Event::KittyTransferred { from: who, to: recipient, kitty_id });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn sale(origin: OriginFor<T>, kitty_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            ensure!(owner==who,Error::<T>::NotKittyOwner);

            ensure!(Self::kitty_on_sale(kitty_id).is_some(),Error::<T>::KittyOnSale);

            let price = T::KittyPrice::get();
            <KittyOnSale<T>>::insert(kitty_id, &price);

            Self::deposit_event(Event::KittyOnSale { who, kitty_id, price });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn buy(origin: OriginFor<T>, kitty_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            ensure!(owner!=who,Error::<T>::AlreadyIsKittyOwner);

            let price = Self::kitty_on_sale(kitty_id).ok_or(Error::<T>::KittyNotOnSale)?;
            T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

            KittyOwner::<T>::insert(kitty_id, &who);
            <KittyOnSale<T>>::remove(kitty_id);

            Self::deposit_event(Event::KittyBought { who, kitty_id });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn get_next_id() -> Result<KittyId, DispatchError> {
            NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError>{
                let current_id = *next_id;
                *next_id = next_id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
                Ok(current_id)
            })
        }

        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }

        fn get_account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
