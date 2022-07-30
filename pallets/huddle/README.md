# Huddle Pallet - PBA 2022

## Overview

The Huddle pallet is an auction where the winners is able to schedule meetings with people
they want to talk to.

## User Types

* Hosts - Users that can create Huddles (must register a Social Network Account).
* Bidder - Users that are willing to pay for a meeting with Hosts.

## Mechanics

- Users register (bind) their AccountId with a Social Network Account (eg Twitter).
  - the inputs are:
    - AccountId (extrinsic's signer)
    - Twitter handle (eg @arturgontijo)
    - A tweet link with the AccountId (https://twitter.com/arturgontijo/status/XXXXX)
    - eg "My huddle's Account is 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

- Registered users (hosts) can create Huddles, by setting:
  - a timestamp, telling when the Huddle goes live and
  - a floor-price.

- Other users can now bid for that Huddle, as soon as the bid's value is greater than:
  - the floor price (for a new Huddle) or
  - the current winning bid's value for already in auction Huddles.

- After the timestamp is reached:
  - the Huddle cannot receive bids.
  - the Host is able to claim the winner bid's value.

- We ensure the following scenarios:
  - only registered Hosts can create Huddles;
  - the timestamp must be somewhere in the future;
  - Huddles with timestamp in the pass cannot receive new bids.
  - new bids must have greater values than the current winning one.

- Reputation System:
  - after the Huddle, both participants are able to rate it.
  - a reputation score will be always available to the whole network.
