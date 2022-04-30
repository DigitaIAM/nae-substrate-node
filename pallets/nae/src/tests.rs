use crate::{mock::*, Error, *};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn correct_mutations() {
	new_test_ext().execute_with(|| {
		let zeros = ID::string("0"); // H256::zero();

		let subject = zeros;
		let relation = BoundedVec::default(); // TODO [zeros].to_vec();

		let v1 = Value::ID(ID::string("1"));
		let v2 = Value::ID(ID::string("2"));

		let changes = [Change::<Test> {
			primary: subject.clone(),
			relation: relation.clone(),
			before: None,
			after: Some(v1.clone()),
		}]
		.to_vec()
		.try_into()
		.unwrap();

		assert_ok!(Nae::modify(Origin::signed(1), changes));
		assert_eq!(Nae::memory(&subject, &relation), Some(v1.clone()));

		let changes = [Change::<Test> {
			primary: subject.clone(),
			relation: relation.clone(),
			before: Some(v1.clone()),
			after: Some(v2.clone()),
		}]
		.to_vec()
		.try_into()
		.unwrap();

		assert_ok!(Nae::modify(Origin::signed(1), changes));
		assert_eq!(Nae::memory(&subject, &relation), Some(v2.clone()));

		let changes = [Change::<Test> {
			primary: subject.clone(),
			relation: relation.clone(),
			before: Some(v2.clone()),
			after: None,
		}]
		.to_vec()
		.try_into()
		.unwrap();

		assert_ok!(Nae::modify(Origin::signed(1), changes));
		assert_eq!(Nae::memory(&subject, &relation), None);
	});
}

#[test]
fn empty_changes_failss() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no changes.
		let changes = [].to_vec().try_into().unwrap();
		assert_noop!(Nae::modify(Origin::signed(1), changes), Error::<Test>::EmptyChanges);
	});
}
