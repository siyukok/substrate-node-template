use frame_support::{assert_noop, assert_ok, BoundedVec};

use crate::{Error, mock::*};

use super::*;

const ACCOUNT_ONE: u64 = 1;
const ACCOUNT_TWO: u64 = 2;

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_eq!(
				Proofs::<Test>::get(&claim),
				Some((ACCOUNT_ONE, frame_system::Pallet::<Test>::block_number()))
			);
		}
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_noop!(
				PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()),
				Error::<Test>::ProofAlreadyExist
			);
		}
	})
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
		}
	});
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_noop!(
				PoeModule::revoke_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()),
				Error::<Test>::ClaimNotExist
			);
		}
	})
}

#[test]
fn revoke_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_noop!(
                PoeModule::revoke_claim(RuntimeOrigin::signed(ACCOUNT_TWO), claim.clone()),
                Error::<Test>::NotClaimOwner
			);
		}
	});
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_eq!(
				Proofs::<Test>::get(&claim),
				Some((ACCOUNT_ONE, frame_system::Pallet::<Test>::block_number()))
			);
			assert_ok!(
				PoeModule::transfer_claim(
					RuntimeOrigin::signed(ACCOUNT_ONE),
					ACCOUNT_TWO,
					claim.clone()
				)
			);
			assert_eq!(
				Proofs::<Test>::get(&claim),
				Some((ACCOUNT_TWO, frame_system::Pallet::<Test>::block_number()))
			);
		}
	})
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_noop!(
				PoeModule::transfer_claim(
					RuntimeOrigin::signed(ACCOUNT_ONE),
					ACCOUNT_TWO,
					claim.clone()
				),
				Error::<Test>::ClaimNotExist
			);
		}
	})
}

#[test]
fn transfer_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_eq!(
				Proofs::<Test>::get(&claim),
				Some((ACCOUNT_ONE, frame_system::Pallet::<Test>::block_number()))
			);
			assert_noop!(
				PoeModule::transfer_claim(
					RuntimeOrigin::signed(ACCOUNT_TWO),
					ACCOUNT_ONE,
					claim.clone()
				),
				Error::<Test>::NotClaimOwner
			);
		}
	})
}

#[test]
fn transfer_failed_with_self_transfer() {
	new_test_ext().execute_with(|| {
		if let Ok(claim) = BoundedVec::try_from(vec![0, 1]) {
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(ACCOUNT_ONE), claim.clone()));
			assert_noop!(
				PoeModule::transfer_claim(
						RuntimeOrigin::signed(ACCOUNT_ONE),
						ACCOUNT_ONE,
						claim.clone()
				),
				Error::<Test>::TransferToSelf
			);
		}
	})
}
