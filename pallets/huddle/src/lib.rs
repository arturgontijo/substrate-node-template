#![cfg_attr(not(feature = "std"), no_std)]

//! # Huddle Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Huddle pallet is an auction where the winners is able to schedule meetings with people
//! they want to talk to.
//!
//! ### User Types
//!
//! * Hosts - Users that can create Huddles (must register a Social Network Account).
//! * Bidders - Users that are willing to pay for a meeting with Hosts.
//!
//! ### Mechanics
//!
//! 1 - Users register (bind) their AccountId with a Social Network Account (eg Twitter).
//! 	1.1 - the inputs are:
//! 		1.1.1 - AccountId (extrinsic's signer)
//! 		1.1.1 - Twitter handle (eg @arturgontijo)
//! 		1.1.1 - A tweet link with the AccountId (https://twitter.com/arturgontijo/status/XXXXX)
//! 				"My huddle's Account is 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
//! 2 - Registered users (hosts) can create Huddles, by setting:
//! 	2.1 - a timestamp, telling when the Huddle goes live and
//! 	2.2 - a floor price.
//! 3 - Other users can now bid for that Huddle, as soon as the bid's value is greater than:
//! 	3.1 - the floor price (for a new Huddle) or
//! 	3.2 - the current winning bid's value for already in auction Huddles.
//! 4 - Guests can also open Huddles for registered hosts:
//! 	4.1 - the Huddle is created with timestamp 0.
//! 	4.2 - the Host can accept it, setting a timestamp.
//! 	4.3 - other users can bid even without host acceptance.
//! 5 - After the timestamp is reached:
//! 	5.1 - the Huddle cannot receive bids.
//! 	5.2 - the Host is able to claim the winner bid's value.
//! 6 - We ensure the following scenarios:
//! 	6.1 - only registered Hosts can create Huddles;
//! 	6.2 - the timestamp must be somewhere in the future;
//! 	6.3 - Huddles with timestamp in the pass cannot receive new bids.
//! 	6.4 - new bids must have greater values than the current winning one.
//! 7 - Reputation System (number of stars):
//! 	7.1 - after the Huddle, guest participant is able to rate it.
//! 	7.2 - a reputation score will be always available to the whole network.

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	pallet_prelude::*,
	traits::{BalanceStatus, Currency, ReservableCurrency},
	PalletId,
};

use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

use pallet_timestamp::{self as timestamp};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Huddle's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Currency: ReservableCurrency<Self::AccountId>;

		/// The maximum length of a Social Account.
		#[pallet::constant]
		type MaxSocialAccountLength: Get<u32>;

		/// The maximum length of a Social Proof (eg link/keybase).
		#[pallet::constant]
		type MaxSocialProofLength: Get<u32>;

		/// The maximum number of Huddles a Host can create.
		#[pallet::constant]
		type MaxHuddlesPerHost: Get<u32>;

		/// The maximum number of Bids users can create.
		#[pallet::constant]
		type MaxBidsPerUser: Get<u32>;

		/// The minimum time threshold, from now, to schedule a Huddle.
		#[pallet::constant]
		type MinTimestampThreshold: Get<Self::Moment>;

		/// The minimum bid value threshold to surpass the current winning one.
		#[pallet::constant]
		type MinBidValueThreshold: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event for Host registration.
		HostRegistered(T::AccountId, SocialAccount<T>, SocialProof<T>),
		/// Event for Huddles created by hosts.
		HuddleCreated(T::AccountId, T::Moment, BalanceOf<T>),
		/// Event for Huddles accepted by hosts.
		HuddleAccepted(T::AccountId, T::Moment, BalanceOf<T>),
		/// Event for Huddles created by guests.
		HuddleOpen(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Event for Bid creation.
		BidCreated(T::AccountId, HuddleId, BalanceOf<T>),
		/// Event for Bid creation.
		Claimed(T::AccountId, HuddleId, BalanceOf<T>),
		/// Event for rating.
		RatingSent(T::AccountId, HuddleId, u8),
	}

	// Errors
	#[pallet::error]
	pub enum Error<T> {
		/// Error for non registered Hosts.
		HostNotRegistered,
		/// Host has created too many Huddles.
		TooManyHuddles,
		/// User has created too many Bids.
		TooManyBids,
		/// Social Account length is too long.
		SocialAccountTooLong,
		/// Social Proof length is too long.
		SocialProofTooLong,
		/// Overflow in HuddleId.
		OverflowHuddleId,
		/// Invalid HuddleId.
		InvalidHuddleId,
		/// Invalid HuddleId for a given Host.
		HostInvalidHuddleId,
		/// Error for invalid timestamps, it must be at least now + 24h.
		InvalidTimestamp,
		/// Error for low value Bids.
		BidIsTooLow,
		/// Error if hosts try to open a huddle to themselves.
		HostsCannotOpenTheirHuddles,
		/// Error if hosts bids their own huddles.
		HostsCannotBidTheirHuddles,
		/// Not the winner Bid.
		NotWinnerBid,
		/// Error while trying to unreserve Bid's value.
		UnreserveError,
		/// Error while trying to repatriate Bid's value to the Host.
		RepatriateError,
		/// Error while trying to claim, timestamp not reached yet.
		TimestampNotReached,
		/// Error while trying to claim the Winner Bid's value.
		InvalidClaim,
		/// Error while trying to unwrap a Vec into BoundedVec.
		UnwrapErrorVec,
		/// Error if hosts rates their own huddles.
		HostsCannotRateTheirHuddles,
		/// Error if guest sends more than 5 stars to the rate() function.
		MaxStarValueIsFive,
	}

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type SocialAccount<T> = BoundedVec<u8, <T as Config>::MaxSocialAccountLength>;
	pub type SocialProof<T> = BoundedVec<u8, <T as Config>::MaxSocialProofLength>;
	pub type HuddleId = u64;

	#[derive(PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum HuddleStatus {
		/// Huddle was created by host and it's open for bids.
		Created,
		/// Huddle was opened by guest and it's open for bids.
		Open,
		/// Huddle is set and has one or more bids.
		InAuction,
		/// Huddle was concluded.
		Concluded,
	}

	#[derive(PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum BidStatus {
		/// Current winning Bid.
		Winning,
		/// Bid was surpassed.
		Surpassed,
		/// Winner Bid.
		Winner,
	}

	/// Struct for Registered User (Host) information.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct UserProfile<SocialAccount, SocialProof> {
		pub social_account: SocialAccount,
		pub social_proof: SocialProof,
	}

	/// Struct for Bid's information.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Bid<Balance> {
		pub huddle: HuddleId,
		pub value: Balance,
		pub status: BidStatus,
	}

	/// Struct for Huddle's information.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Huddle<AccountId, Balance, Moment> {
		pub id: HuddleId,
		pub timestamp: Moment,
		pub guest: Option<AccountId>,
		pub value: Balance,
		pub status: HuddleStatus,
		pub stars: u8,
	}

	/// UUID for Huddles.
	#[pallet::storage]
	#[pallet::getter(fn huddle_counter)]
	pub(super) type HuddleCounter<T: Config> = StorageValue<_, HuddleId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn hosts)]
	/// Binds an AccountId to a SubSocial Account.
	pub(super) type Hosts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		UserProfile<SocialAccount<T>, SocialProof<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn huddles)]
	/// Stores a Huddles' data.
	pub(super) type Huddles<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<Huddle<T::AccountId, BalanceOf<T>, T::Moment>, T::MaxHuddlesPerHost>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn bids)]
	/// Stores a Bids' data.
	pub(super) type Bids<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<Bid<BalanceOf<T>>, T::MaxBidsPerUser>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Origin can register themselves by binding a SocialAccount and a SocialProof to their accounts.
		#[pallet::weight(T::DbWeight::get().reads(2) + T::DbWeight::get().writes(1))]
		pub fn register(
			origin: OriginFor<T>,
			social_account: SocialAccount<T>,
			social_proof: SocialProof<T>,
		) -> DispatchResult {
			let host = ensure_signed(origin)?;

			ensure!(
				social_account.len() <= T::MaxSocialAccountLength::get() as usize,
				Error::<T>::SocialAccountTooLong
			);

			ensure!(
				social_proof.len() <= T::MaxSocialProofLength::get() as usize,
				Error::<T>::SocialProofTooLong
			);

			let user_profile = UserProfile {
				social_account: social_account.clone(),
				social_proof: social_proof.clone(),
			};

			// Insert/Update the Social Account of the origin's AccountId.
			<Hosts<T>>::insert(&host, &user_profile);

			// Emit an event.
			Self::deposit_event(Event::HostRegistered(host, social_account, social_proof));

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(5) + T::DbWeight::get().writes(2))]
		/// Hosts (registered users) can create a Huddle.
		pub fn create(
			origin: OriginFor<T>,
			timestamp: T::Moment,
			min_value: BalanceOf<T>,
		) -> DispatchResult {
			let host = ensure_signed(origin)?;
			ensure!(<Hosts<T>>::contains_key(&host), Error::<T>::HostNotRegistered);

			// Check if the given timestamp is at least now + MinTimestampThreshold.
			let now = <timestamp::Pallet<T>>::get();
			ensure!(
				timestamp >= now + T::MinTimestampThreshold::get(),
				Error::<T>::InvalidTimestamp
			);

			// Check if we can add a new HuddleId.
			let next_uuid =
				Self::huddle_counter().checked_add(1).ok_or(Error::<T>::OverflowHuddleId)?;

			let new_huddle = Huddle {
				id: next_uuid,
				timestamp: timestamp.clone(),
				guest: None,
				value: min_value,
				status: HuddleStatus::Created,
				stars: 0,
			};

			insert_huddle::<T>(&host, new_huddle)?;

			// Update the Huddle counter.
			<HuddleCounter<T>>::put(next_uuid);
			// Emit an event
			Self::deposit_event(Event::HuddleCreated(host, timestamp, min_value));

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(5) + T::DbWeight::get().writes(3))]
		/// Users can open a Huddle to talk to any Hosts.
		pub fn open(
			origin: OriginFor<T>,
			host: AccountOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let guest = ensure_signed(origin)?;

			ensure!(host != guest, Error::<T>::HostsCannotOpenTheirHuddles);

			// Guests can only open huddles to talk to registered hosts.
			ensure!(<Hosts<T>>::contains_key(&host), Error::<T>::HostNotRegistered);

			// Check if we can add a new HuddleId.
			let next_uuid =
				Self::huddle_counter().checked_add(1).ok_or(Error::<T>::OverflowHuddleId)?;

			// In order to open a Huddle, guest must surpass the last bid of a host's huddle
			if let Some(huddles) = <Huddles<T>>::get(&host) {
				if let Some(last_huddle) = huddles.last() {
					ensure!(value >= last_huddle.value, Error::<T>::BidIsTooLow);
				}
			}

			// Reserve the value of the Bid.
			T::Currency::reserve(&guest, value.clone())?;

			let new_huddle = Huddle {
				id: next_uuid,
				timestamp: 0u32.into(),
				guest: Some(guest.clone()),
				value: value.clone(),
				status: HuddleStatus::Open,
				stars: 0,
			};

			insert_huddle::<T>(&host, new_huddle)?;
			insert_update_bid::<T>(&guest, next_uuid.clone(), value.clone());

			// Update the Huddle counter.
			<HuddleCounter<T>>::put(next_uuid);
			// Emit an event
			Self::deposit_event(Event::HuddleOpen(guest, host, value));

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(5) + T::DbWeight::get().writes(4))]
		/// Host can accept an open Huddle.
		pub fn accept(
			origin: OriginFor<T>,
			huddle: HuddleId,
			timestamp: T::Moment,
		) -> DispatchResult {
			let host = ensure_signed(origin)?;

			// Check if HuddleId is valid.
			ensure!(0 < huddle && huddle <= Self::huddle_counter(), Error::<T>::InvalidHuddleId);

			let mut found = false;
			if let Some(mut huddles) = <Huddles<T>>::get(&host) {
				match huddles.binary_search_by(|h| h.id.cmp(&huddle)) {
					Ok(pos) => {
						// Check if the given timestamp is at least now + MinTimestampThreshold.
						let now = <timestamp::Pallet<T>>::get();
						ensure!(
							timestamp >= now + T::MinTimestampThreshold::get(),
							Error::<T>::InvalidTimestamp
						);

						// It is InAuction now (accepted by host)
						huddles[pos].status = HuddleStatus::InAuction;
						huddles[pos].timestamp = timestamp;

						let value = huddles[pos].value.clone();

						// Update the Host's Huddles.
						<Huddles<T>>::insert(&host, huddles);

						found = true;

						// Emit an event.
						Self::deposit_event(Event::HuddleAccepted(host, timestamp, value));
					},
					Err(_) => {},
				}
			}

			ensure!(found, Error::<T>::HostInvalidHuddleId);

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(5) + T::DbWeight::get().writes(4))]
		/// Users can bid to talk to a host.
		pub fn bid(
			origin: OriginFor<T>,
			host: AccountOf<T>,
			huddle: HuddleId,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let guest = ensure_signed(origin)?;

			ensure!(host != guest, Error::<T>::HostsCannotBidTheirHuddles);

			// Check if HuddleId is valid.
			ensure!(0 < huddle && huddle <= Self::huddle_counter(), Error::<T>::InvalidHuddleId);

			let mut found = false;
			if let Some(mut huddles) = <Huddles<T>>::get(&host) {
				match huddles.binary_search_by(|h| h.id.cmp(&huddle)) {
					Ok(pos) => {
						// Check the Timestamp (is the Huddle still valid?).
						// If it is Open, we do not check its timestamp.
						if huddles[pos].status != HuddleStatus::Open {
							let now = <timestamp::Pallet<T>>::get();
							ensure!(huddles[pos].timestamp >= now, Error::<T>::InvalidTimestamp);
						}

						// Check if Bid's value is greater than the winning one.
						let value_threshold =
							<BalanceOf<T>>::from(T::MinBidValueThreshold::get() as u8);
						ensure!(
							value > huddles[pos].value + value_threshold,
							Error::<T>::BidIsTooLow
						);

						// We need to release the reserve value of the current winning Bid.
						if let Some(last_guest) = huddles[pos].guest.clone() {
							ensure!(
								release_value::<T>(&last_guest, huddle),
								Error::<T>::UnreserveError
							);
						}

						insert_update_bid::<T>(&guest, huddle, value);

						// Reserve the value of the Bid.
						T::Currency::reserve(&guest, value.clone())?;

						// Update the Huddle's data.
						huddles[pos].value = value;
						huddles[pos].guest = Some(guest.clone());

						// We only set it to InAuction if last status != Open (created by guest)
						if huddles[pos].status != HuddleStatus::Open {
							huddles[pos].status = HuddleStatus::InAuction;
						}

						// Update the Host's Huddles.
						<Huddles<T>>::insert(&host, huddles);

						found = true;

						// Emit an event.
						Self::deposit_event(Event::BidCreated(guest, huddle, value));
					},
					Err(_) => {},
				}
			}

			ensure!(found, Error::<T>::HostInvalidHuddleId);

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(3) + T::DbWeight::get().writes(2))]
		/// Host can claim the winner bid's amount after the Huddle's timestamp is reached.
		pub fn claim(origin: OriginFor<T>, huddle: HuddleId) -> DispatchResult {
			let host = ensure_signed(origin)?;
			ensure!(0 < huddle && huddle <= Self::huddle_counter(), Error::<T>::InvalidHuddleId);

			let mut found = false;
			if let Some(mut huddles) = <Huddles<T>>::get(&host) {
				match huddles.binary_search_by(|h| h.id.cmp(&huddle)) {
					Ok(pos) => {
						// Check if it can be claimed by verifying the Timestamp.
						let now = <timestamp::Pallet<T>>::get();
						ensure!(huddles[pos].timestamp < now, Error::<T>::TimestampNotReached);

						// We need to repatriate the reserve value of the winner Bid (if any) to the
						// Host.
						if let Some(guest) = huddles[pos].guest.clone() {
							ensure!(
								repatriate_value::<T>(&guest, &host, huddle),
								Error::<T>::RepatriateError
							);
						}

						// Update the Huddle's status.
						huddles[pos].status = HuddleStatus::Concluded;
						let value = huddles[pos].value.clone();

						// Update the Host's Huddles.
						<Huddles<T>>::insert(&host, huddles);

						found = true;

						// Emit an event.
						Self::deposit_event(Event::Claimed(host, huddle, value));
					},
					Err(_) => {},
				}
			}

			ensure!(found, Error::<T>::InvalidClaim);

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads(3) + T::DbWeight::get().writes(1))]
		/// Winner's Bid can rate how was the Huddle (0-5 stars).
		pub fn rate(
			origin: OriginFor<T>,
			host: AccountOf<T>,
			huddle: HuddleId,
			stars: u8,
		) -> DispatchResult {
			let guest = ensure_signed(origin)?;

			ensure!(host != guest, Error::<T>::HostsCannotRateTheirHuddles);
			ensure!(stars <= 5, Error::<T>::MaxStarValueIsFive);

			// Check if HuddleId is valid.
			ensure!(0 < huddle && huddle <= Self::huddle_counter(), Error::<T>::InvalidHuddleId);

			let mut found = false;
			let mut winner = false;
			if let Some(mut huddles) = <Huddles<T>>::get(&host) {
				match huddles.binary_search_by(|h| h.id.cmp(&huddle)) {
					Ok(pos) => {
						// Check the Timestamp.
						let now = <timestamp::Pallet<T>>::get();
						ensure!(huddles[pos].timestamp < now, Error::<T>::TimestampNotReached);

						// Check if the guest was the winner (huddle must be already claimed).
						if let Some(bids) = <Bids<T>>::get(&guest) {
							match bids.binary_search_by(|b| b.huddle.cmp(&huddle)) {
								Ok(pos) =>
									if bids[pos].status == BidStatus::Winner {
										winner = true;
									},
								Err(_) => {},
							};
						};

						if winner {
							// Update the Huddle's data.
							huddles[pos].stars = stars.clone();

							// Update the Host's Huddles.
							<Huddles<T>>::insert(&host, huddles);

							// Emit an event.
							Self::deposit_event(Event::RatingSent(guest, huddle, stars));
						}

						found = true;
					},
					Err(_) => {},
				}
			}

			ensure!(found, Error::<T>::HostInvalidHuddleId);
			ensure!(winner, Error::<T>::NotWinnerBid);

			Ok(())
		}
	}

	/// Insert a new Huddle into the storage
	fn insert_huddle<T: Config>(
		host: &AccountOf<T>,
		new_huddle: Huddle<T::AccountId, BalanceOf<T>, T::Moment>,
	) -> DispatchResult {
		if let Some(mut huddles) = <Huddles<T>>::get(&host) {
			huddles.try_push(new_huddle).map_err(|()| Error::<T>::TooManyHuddles)?;
			// Update the Host's Huddles.
			<Huddles<T>>::insert(&host, huddles);
		} else {
			// Update the Host's Huddles.
			<Huddles<T>>::insert(
				&host,
				BoundedVec::try_from(vec![new_huddle]).map_err(|()| Error::<T>::UnwrapErrorVec)?,
			);
		}
		Ok(())
	}

	/// Insert a new Bid or Update an existing one.
	fn insert_update_bid<T: Config>(
		guest: &AccountOf<T>,
		huddle: HuddleId,
		value: BalanceOf<T>,
	) -> bool {
		if let Some(mut bids) = <Bids<T>>::get(guest) {
			match bids.binary_search_by(|b| b.huddle.cmp(&huddle)) {
				Ok(pos) => {
					bids[pos].value = value;
					bids[pos].status = BidStatus::Winning;
				},
				Err(_) => {
					// Insert a Bid entry.
					let res = bids
						.try_push(Bid { huddle: huddle.clone(), value, status: BidStatus::Winning })
						.map_err(|()| Error::<T>::TooManyBids);
					if !res.is_ok() {
						return false
					}
				},
			}
			// Update the Guest's Bids.
			<Bids<T>>::insert(guest, bids);
		} else {
			// Update the Guest's Bids.
			<Bids<T>>::insert(
				guest,
				BoundedVec::try_from(vec![Bid {
					huddle: huddle.clone(),
					value,
					status: BidStatus::Winning,
				}])
				.unwrap_or(BoundedVec::default()),
			);
		}
		true
	}

	/// Release the value of a Surpassed Bid.
	fn release_value<T: Config>(guest: &AccountOf<T>, huddle: HuddleId) -> bool {
		if let Some(mut bids) = <Bids<T>>::get(guest) {
			match bids.binary_search_by(|b| b.huddle.cmp(&huddle)) {
				Ok(pos) => {
					T::Currency::unreserve(guest, bids[pos].value);
					bids[pos].status = BidStatus::Surpassed;
					// Update the Guest's Bids.
					<Bids<T>>::insert(guest, bids);
				},
				Err(_) => return false,
			}
		}
		true
	}

	/// Repatriate the winning Bid's value to the Huddle's Host.
	fn repatriate_value<T: Config>(
		guest: &AccountOf<T>,
		host: &AccountOf<T>,
		huddle: HuddleId,
	) -> bool {
		if let Some(mut bids) = <Bids<T>>::get(guest) {
			match bids.binary_search_by(|b| b.huddle.cmp(&huddle)) {
				Ok(pos) => {
					// Repatriate the value of the Bid to the Host.
					let res = T::Currency::repatriate_reserved(
						guest,
						host,
						bids[pos].value,
						BalanceStatus::Free,
					);
					if !res.is_ok() {
						return false
					}
					bids[pos].status = BidStatus::Winner;
					// Update the Guest's Bids.
					<Bids<T>>::insert(guest, bids);
				},
				Err(_) => return false,
			}
		}
		true
	}
}
