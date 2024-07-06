use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
mod simple_contract {
    enable_method_auth! {
        roles {
            main => updatable_by: [];
        },
        methods {
            exchange => restrict_to: [main];
            withdraw_assets => restrict_to: [main];
            cancel_escrow => restrict_to: [main];
        }
    }
    struct AvagoSwapSimple {
        id: u64,
        vault: NonFungibleVault,
        service_royality: Decimal,
        owner: NonFungibleLocalId,
        taker: Option<NonFungibleLocalId>,
        royality_data: Option<RoyalityData>,
        status: Status,
        price: Decimal,
        xrd_vault: Vault,
    }

    impl AvagoSwapSimple {
        pub fn instantiate(
            service_royality: Decimal,
            owner: NonFungibleLocalId,
            main: ComponentAddress,
            id: u64,
            args: ArgsSimpleSwap,
        ) -> Global<AvagoSwapSimple> {
            Self {
                id,
                owner,
                price: args.price,
                royality_data: args.royality_data,
                service_royality,
                taker: None,
                status: Status {
                    is_sold: false,
                    is_cancelled: false,
                    is_took: false,
                },
                vault: NonFungibleVault::with_bucket(args.asset),
                xrd_vault: Vault::new(XRD),
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
            badge: NonFungibleLocalId,
            required_assets: Assets,
        ) -> (Option<Vec<Assets>>, Option<RoyalityData>) {
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

            let mut xrd_bucket = required_assets
                .fungible_buckets
                .unwrap()
                .into_iter()
                .nth(0)
                .unwrap();

            assert!(
                xrd_bucket.resource_address() == XRD,
                "Asset Forbiden, you must pay with XRD"
            );

            assert!(
                xrd_bucket.amount() > self.price,
                "You dont have enough XRD to buy"
            );

            self.status.is_sold = true;
            self.xrd_vault.put(xrd_bucket.take(self.price).into());
            self.taker = Some(badge);

            let asset_to_return = Assets {
                non_fungible_buckets: Some(vec![self.vault.take_all()]),
                fungible_buckets: Some(vec![xrd_bucket]),
            };

            (Some(vec![asset_to_return]), self.royality_data.to_owned())
        }

        pub fn withdraw_assets(
            &mut self,
            badge: NonFungibleLocalId,
        ) -> (Option<Vec<Assets>>, Option<RoyalityData>) {
            assert!(
                badge == self.owner,
                "You are not the owner and not allowed to withdraw"
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

            let asset_to_return = Assets {
                non_fungible_buckets: None,
                fungible_buckets: Some(vec![self.xrd_vault.as_fungible().take_all()]),
            };

            (
                Some(vec![asset_to_return]),
                Some(RoyalityData {
                    addresses: vec![],
                    amount: self.service_royality,
                }),
            )
        }

        pub fn cancel_escrow(&mut self, badge: NonFungibleLocalId) -> Option<Vec<Assets>> {
            assert!(
                badge == self.owner,
                "You are not the owner and not allowed to cancel"
            );

            assert!(
                !self.status.is_sold,
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

            self.status.is_cancelled = true;
            let to_return = Assets {
                non_fungible_buckets: Some(vec![self.vault.take_all()]),
                fungible_buckets: None,
            };

            Some(vec![to_return])
        }
    }
}
