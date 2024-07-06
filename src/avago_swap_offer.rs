use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
#[types(NonFungibleLocalId, AssetsAccumulator)]
mod avago_swap_bid {

    enable_method_auth! {
        roles {
            main => updatable_by: [];
        },
        methods {
            exchange => restrict_to: [main];
            withdraw_assets => restrict_to: [main];
            cancel_escrow => restrict_to: [main];
            offer => restrict_to: [main];
            cancel_offer => restrict_to: [main];
        }
    }

    struct AvagoSwapOffer {
        id: u64,
        status: Status,
        owner: NonFungibleLocalId,
        offered_resource: AssetsAccumulator,
        offers: KeyValueStore<NonFungibleLocalId, AssetsAccumulator>,
        selected: Option<NonFungibleLocalId>,
        nft_royalities: Option<RoyalityData>,
        service_royality: Decimal,
    }

    impl AvagoSwapOffer {
        pub fn instantiate(
            service_royality: Decimal,
            owner: NonFungibleLocalId,
            main: ComponentAddress,
            id: u64,
            args: Args,
        ) -> Global<AvagoSwapOffer> {
            Self {
                id,
                service_royality,
                nft_royalities: args.offered_resource.get_royality_data(),
                owner: owner.clone(),
                offers: KeyValueStore::<NonFungibleLocalId, AssetsAccumulator>::new_with_registered_type(),
                offered_resource: AssetsAccumulator::new(args.offered_resource),
                selected: None,
                status: Status {
                    is_sold: false,
                    is_cancelled: false,
                    is_took: false,
                },
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles! {
                main => rule!(require(global_caller(main)));
            })
            .globalize()
        }

        pub fn exchange(
            &mut self,
            owner: NonFungibleLocalId,
            selected: NonFungibleLocalId,
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
            assert!(
                !self.status.is_took,
                "Contract has already been bought by other."
            );

            self.status.is_sold = true;
            self.selected = Some(selected.clone());

            if let Some(mut assets) = self.offers.get_mut(&selected) {
                let assets = assets.take();
                self.nft_royalities =
                    self.nft_royalities
                        .take()
                        .map_or(assets.get_royality_data(), |mut f| {
                            if let Some(x) = assets.get_royality_data() {
                                f.extend(x);
                            }
                            Some(f)
                        });
                (
                    Some(vec![assets]),
                    Some(RoyalityData {
                        addresses: vec![],
                        amount: self.service_royality,
                    }),
                )
            } else {
                Runtime::panic(String::from("Selected offers is not available."))
            }
        }

        pub fn cancel_escrow(&mut self, owner: NonFungibleLocalId) -> Option<Vec<Assets>> {
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

        pub fn offer(
            &mut self,
            offerer: NonFungibleLocalId,
            offer_assets: Assets,
        ) -> Option<Vec<Assets>> {
            assert!(
                !self.status.is_cancelled,
                "Contract has already been cancelled."
            );
            assert!(
                !self.status.is_sold,
                "Contract has already been bought by other."
            );
            assert!(
                !self.status.is_took,
                "Contract has already been bought by other."
            );

            self.offers
                .insert(offerer, AssetsAccumulator::new(offer_assets));

            None
        }

        pub fn withdraw_assets(
            &mut self,
            offer_winner: NonFungibleLocalId,
        ) -> (Option<Vec<Assets>>, Option<RoyalityData>) {
            assert!(
                self.selected.is_some(),
                "The owner is not selected the offer contract yet."
            );
            assert!(
                &offer_winner == self.selected.as_ref().unwrap(),
                "You are not the offer winner of this contract."
            );
            assert!(
                self.status.is_sold,
                "Contract hasn't already been sold by other."
            );
            assert!(
                !self.status.is_cancelled,
                "Contract has already been cancelled."
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

        pub fn cancel_offer(&mut self, offerer: NonFungibleLocalId) -> Option<Vec<Assets>> {
            assert!(
                self.selected.is_some(),
                "The owner is not selected the offer contract yet."
            );
            assert!(
                &offerer != self.selected.as_ref().unwrap(),
                "your offer has been selected by owner contract, can not be withdrawed."
            );

            if let Some(mut assets_to_take) = self.offers.remove(&offerer) {
                Some(vec![assets_to_take.take()])
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
