use super::*;

use mock::*;

use frame_support::{assert_noop, assert_ok};

#[test]
fn huddle_works() {
	new_test_ext().execute_with(|| {
		// All 3 users have funds.
		assert_eq!(Balances::free_balance(1), 50);
		assert_eq!(Balances::free_balance(2), 50);
		assert_eq!(Balances::free_balance(3), 50);

		// Dispatch a register extrinsic -> register(origin, social_account).
		let bounded_name: BoundedVec<_, _> = (b"alice").to_vec().try_into().unwrap();
		let bounded_proof: BoundedVec<_, _> = (b"alice's proof").to_vec().try_into().unwrap();
		assert_ok!(HuddlePallet::register(
			Origin::signed(1),
			bounded_name.clone(),
			bounded_proof.clone()
		));

		// Check if (1) is registered.
		assert_eq!(
			HuddlePallet::hosts(1),
			Some(UserProfile { social_account: bounded_name, social_proof: bounded_proof }),
		);

		// Creating a Huddle for an unregistered Host (2).
		assert_noop!(
			HuddlePallet::create_huddle(Origin::signed(2), 100, 2),
			Error::<Test>::HostNotRegistered,
		);

		// Creating a Huddle for a registered Host (1) -> create_huddle(origin, timestamp, value).
		assert_ok!(HuddlePallet::create_huddle(Origin::signed(1), 100, 2));

		// Checking the created Huddle.
		assert_eq!(
			HuddlePallet::huddles(1),
			Some(
				BoundedVec::try_from(vec![Huddle {
					id: 1,
					timestamp: 100,
					guest: None,
					value: 2,
					status: HuddleStatus::Open,
					stars: 0,
				}])
				.unwrap()
			),
		);

		// No Huddles for (2)
		assert_eq!(HuddlePallet::huddles(2), None);

		// (2) bids with less than the minimum value -> bid(origin, host, huddle, value).
		assert_noop!(HuddlePallet::bid(Origin::signed(2), 1, 1, 1), Error::<Test>::BidIsTooLow);

		// (3) bids for (1)'s Huddle -> bid(origin, host, huddle, value).
		assert_ok!(HuddlePallet::bid(Origin::signed(3), 1, 1, 5));
		// (3) has reserved 5 for the Bid's value.
		assert_eq!(Balances::free_balance(3), 45);

		// Checking the Huddle.
		assert_eq!(
			HuddlePallet::huddles(1),
			Some(
				BoundedVec::try_from(vec![Huddle {
					id: 1,
					timestamp: 100,
					guest: Some(3), // (3) is the winning guest.
					value: 5,
					status: HuddleStatus::InAuction,
					stars: 0,
				}])
				.unwrap()
			)
		);

		// (2) bids for (1)'s Huddle -> bid(origin, host, huddle, value).
		assert_ok!(HuddlePallet::bid(Origin::signed(2), 1, 1, 15));

		// Checking the Huddle.
		assert_eq!(
			HuddlePallet::huddles(1),
			Some(
				BoundedVec::try_from(vec![Huddle {
					id: 1,
					timestamp: 100,
					guest: Some(2), // (2) is the winning guest.
					value: 15,
					status: HuddleStatus::InAuction,
					stars: 0,
				}])
				.unwrap()
			)
		);

		// Check timestamp
		assert_eq!(pallet_timestamp::Pallet::<Test>::get(), 0);
		// Run till block 10 (6 secs per block)
		run_to_block(10);
		// Timestamp must be 60
		assert_eq!(pallet_timestamp::Pallet::<Test>::get(), 60);

		// User (1) tries to claim the winning Bid's funds -> claim(origin, huddle).
		assert_noop!(HuddlePallet::claim(Origin::signed(1), 1), Error::<Test>::TimestampNotReached);

		// Run 10 more blocks (6 secs per block)
		run_to_block(20);
		// Timestamp must be 120
		assert_eq!(pallet_timestamp::Pallet::<Test>::get(), 120);

		// (1) claims the funds -> claim(origin, huddle).
		assert_eq!(Balances::free_balance(1), 50);
		assert_ok!(HuddlePallet::claim(Origin::signed(1), 1));

		// (1) now has 50 (initial) + 15 (bid's value claimed) free balance.
		assert_eq!(Balances::free_balance(1), 65);

		// User (1) tries to rate his own huddle -> rate(origin, host, huddle, stars).
		assert_noop!(HuddlePallet::rate(Origin::signed(1), 1, 1, 5), Error::<Test>::HostsCannotRateTheirHuddles);
		// User (2), the winner, can rate the concluded huddle.
		assert_ok!(HuddlePallet::rate(Origin::signed(2), 1, 1, 3));
		// Checking the Huddle.
		assert_eq!(
			HuddlePallet::huddles(1),
			Some(
				BoundedVec::try_from(vec![Huddle {
					id: 1,
					timestamp: 100,
					guest: Some(2), // (2) is the winner guest.
					value: 15,
					status: HuddleStatus::Concluded,
					stars: 3,
				}])
				.unwrap()
			)
		);
	});
}
