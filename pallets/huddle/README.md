# Huddle Pallet - PBA 2022

```
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
```