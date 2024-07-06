use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
#[types(NonFungibleGlobalId, Decimal)]
mod avago_swap_bid {

    enable_method_auth! {
        roles {
            main => updatable_by: [];
        },
        methods {
            exchange => restrict_to: [main];
            withdraw_assets => restrict_to: [main];
            cancel_escrow => restrict_to: [main];
            bid => restrict_to: [main];
            cancel_bid => restrict_to: [main];
        }
    }

    struct AvagoSwapBid {
        id: u64,
        status: Status,
        owner: NonFungibleGlobalId,
        offered_resource: AssetsAccumulator,
        bids_vault: FungibleVault,
        bidder: KeyValueStore<NonFungibleGlobalId, Decimal>,
        highest_bid: Option<HighestBidder>,
        nft_royalities: Option<RoyalityData>,
        service_royality: Decimal,
    }

    impl AvagoSwapBid {
        pub fn instantiate(
            service_royality: Decimal,
            owner: NonFungibleGlobalId,
            main: ComponentAddress,
            id: u64,
            args: Args,
        ) -> Global<AvagoSwapBid> {
            Self {
                id,
                service_royality,
                nft_royalities: args.offered_resource.get_royality_data(),
                owner: owner.clone(),
                bids_vault: FungibleVault::new(XRD),
                bidder: KeyValueStore::<NonFungibleGlobalId, Decimal>::new_with_registered_type(),
                highest_bid: None,
                offered_resource: AssetsAccumulator::new(args.offered_resource),
                status: Status {
                    is_sold: false,
                    is_cancelled: false,
                    is_took: false,
                },
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner))))
            .roles(roles! {
                main => rule!(require(global_caller(main)));
            })
            .globalize()
        }

        pub fn exchange(
            &mut self,
            owner: NonFungibleGlobalId,
        ) -> (Option<Vec<Assets>>, Option<RoyalityData>) {
            assert!(
                owner == self.owner,
                "You are not the owner of this contract swap."
            );
            assert!(
                !self.status.is_cancelled,
                "Contract has already been cancelled."
            );
            assert!(
                !self.status.is_sold,
                "Contract has already been bought by other."
            );

            self.status.is_sold = true;

            (
                Some(vec![Assets {
                    fungible_buckets: Some(vec![self
                        .highest_bid
                        .as_mut()
                        .map(|bidder| self.bids_vault.take(bidder.amount))
                        .unwrap()]),
                    non_fungible_buckets: None,
                }]),
                Some(RoyalityData {
                    addresses: vec![],
                    amount: self.service_royality,
                }),
            )
        }

        pub fn cancel_escrow(&mut self, owner: NonFungibleGlobalId) -> Option<Vec<Assets>> {
            assert!(
                owner == self.owner,
                "You are not the owner of this contract swap."
            );
            assert!(
                !self.status.is_sold,
                "Contract has already been sold by other."
            );
            assert!(
                !self.status.is_cancelled,
                "Contract has already been cancelled."
            );

            self.status.is_cancelled = true;
            Some(vec![self.offered_resource.take()])
        }
        pub fn bid(
            &mut self,
            bidder: NonFungibleGlobalId,
            xrd_bucket: FungibleBucket,
        ) -> Option<Vec<Assets>> {
            assert!(
                !self.status.is_cancelled,
                "Contract has already been cancelled."
            );
            assert!(
                !self.status.is_sold,
                "Contract has already been bought by other."
            );

            if let Some(highest_bidder) = self.highest_bid.as_ref() {
                info!("{}, {}", highest_bidder.amount, xrd_bucket.amount());
                assert!(
                    highest_bidder.amount < xrd_bucket.amount(),
                    "Your bid has to be greater than current bid."
                );
            }

            self.highest_bid = Some(HighestBidder {
                non_fungible_global_id: bidder,
                amount: xrd_bucket.amount(),
            });

            self.bids_vault.put(xrd_bucket);

            None
        }

        pub fn withdraw_assets(
            &mut self,
            bid_winner: NonFungibleGlobalId,
        ) -> (Option<Vec<Assets>>, Option<RoyalityData>) {
            assert!(
                self.highest_bid.is_some(),
                "No one has bid this contract yet."
            );
            assert!(
                bid_winner == self.highest_bid.as_ref().unwrap().non_fungible_global_id,
                "You are not the bid winner of this contract."
            );
            assert!(
                !self.status.is_took,
                "Contract has already been withdrawed by owner."
            );

            self.status.is_took = true;
            (
                Some(vec![self.offered_resource.take()]),
                self.nft_royalities.take(),
            )
        }

        pub fn cancel_bid(&mut self, bidder: NonFungibleGlobalId) -> Option<Vec<Assets>> {
            assert!(
                self.highest_bid.is_some(),
                "No one has bid this contract yet."
            );
            assert!(
                bidder != self.highest_bid.as_ref().unwrap().non_fungible_global_id,
                "You are in the highest bid and can not withdraw."
            );

            if let Some(amount_to_take) = self.bidder.remove(&bidder) {
                Some(vec![Assets {
                    fungible_buckets: Some(vec![self.bids_vault.take(amount_to_take)]),
                    non_fungible_buckets: None,
                }])
            } else {
                Runtime::panic(String::from("You are not bid this contract"));
            }
        }
    }
}

#[derive(ScryptoSbor)]
pub struct Args {
    pub offered_resource: Assets,
}

#[derive(ScryptoSbor)]
pub struct HighestBidder {
    pub non_fungible_global_id: NonFungibleGlobalId,
    pub amount: Decimal,
}