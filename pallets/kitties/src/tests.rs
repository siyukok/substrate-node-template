use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};
use frame_support::traits::StorageVersion;

const DEFAULT_KITTY_NAME: [u8; 8] = *b"abcdefgh";

#[test]
fn create_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME));
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
        assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
        System::assert_last_event(Event::KittyCreated
        {
            who: account_id,
            kitty_id,
            kitty: KittiesModule::kitties(kitty_id).unwrap(),
        }.into());

        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
        assert_eq!(KittiesModule::kitty_parents(kitty_id), None);
        NextKittyId::<Test>::set(KittyId::MAX);
        assert_noop!(
            KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn breed_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME));
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME));
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1 + 1);
        assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id+1,DEFAULT_KITTY_NAME));

        let breed_kitty_id = 2;
        System::assert_last_event(Event::KittyBreed
        {
            who: account_id,
            kitty_id: breed_kitty_id,
            kitty: KittiesModule::kitties(breed_kitty_id).unwrap(),
        }.into());
        assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
        assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));
        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id,DEFAULT_KITTY_NAME),
            Error::<Test>::SameKittyId
        );
        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id), 1, 9999,DEFAULT_KITTY_NAME),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn transfer_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let recipient = 2;
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME));
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
        assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));
        System::assert_last_event(Event::KittyTransferred
        {
            from: account_id,
            to: recipient,
            kitty_id,
        }.into());
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));
        assert_noop!(
            KittiesModule::transfer(RuntimeOrigin::signed(9999), recipient, kitty_id),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn check_version_works() {
    let v0_1 = StorageVersion::new(0);
    let v1 = StorageVersion::new(1);
    let v0_2 = StorageVersion::new(0);
    assert_eq!(v0_1, v0_2);
    assert_ne!(v0_1, v1);
}

#[test]
fn sale_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let price = 0;
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id),DEFAULT_KITTY_NAME));
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

        assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));

        System::assert_last_event(Event::KittyOnSale
        {
            who: account_id,
            kitty_id,
            price,
        }.into());
        assert_eq!(KittiesModule::kitty_on_sale(kitty_id), Some(price));
        assert_noop!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),Error::<Test>::KittyOnSale);
    })
}

#[test]
fn buy_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id_seller = 1;
        let account_id_buyer = 2;
        let price = 0;
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id_seller),DEFAULT_KITTY_NAME));
        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id_seller));

        assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(account_id_seller), kitty_id),Error::<Test>::AlreadyIsKittyOwner);

        assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(account_id_buyer), kitty_id),Error::<Test>::KittyNotOnSale);

        assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id_seller), kitty_id));

        System::assert_last_event(Event::KittyOnSale
        {
            who: account_id_seller,
            kitty_id,
            price,
        }.into());

        assert_eq!(KittiesModule::kitty_on_sale(kitty_id), Some(price));

        assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(account_id_buyer), kitty_id));
        System::assert_last_event(Event::KittyBought
        {
            who: account_id_buyer,
            kitty_id,
        }.into());

        assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id_buyer));
    })
}