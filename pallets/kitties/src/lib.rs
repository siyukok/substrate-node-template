#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;

pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    pub type KittyId = u32;

    #[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
    pub struct Kitty(pub [u8; 16]);

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittieOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidKittyId
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let kitty_id = Self::get_next_id()?;
            let kitty = Kitty(Default::default());

            Kitties::<T>::insert(kitty_id, &kitty);
            KittieOwner::<T>::insert(kitty_id, &who);

            Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
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
    }
}
