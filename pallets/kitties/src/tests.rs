use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
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
            KittiesModule::create(RuntimeOrigin::signed(account_id)),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn breed_kitty_works() {
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
        assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1 + 1);
        assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id+1));

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
            KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
            Error::<Test>::SameKittyId
        );
        assert_noop!(
            KittiesModule::breed(RuntimeOrigin::signed(account_id), 1, 9999),
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
        assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
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