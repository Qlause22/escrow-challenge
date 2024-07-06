use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
mod avago_swap_basic {
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
    struct AvagoSwapBasic {
        id: u64,
        nft_royalities: Option<RoyalityData>,
        service_royality: Decimal,
        status: Status,
        owner: NonFungibleGlobalId,
        taker: Option<NonFungibleGlobalId>,
        offered_resource: AssetsAccumulator,
        requested_resource: RequiredResources,
        requested_resource_vault: Option<AssetsAccumulator>,
    }

    impl AvagoSwapBasic {
        pub fn instantiate(
            service_royality: Decimal,
            owner: NonFungibleGlobalId,
            main: ComponentAddress,
            id: u64,
            args: Args,
        ) -> Global<AvagoSwapBasic> {
            Self {
                id,
                service_royality,
                taker: None,
                nft_royalities: args.offered_resource.get_royality_data(),
                owner: owner.clone(),
                requested_resource: args.requested_resource,
                requested_resource_vault: None,
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
            badge: NonFungibleGlobalId,
            mut required_assets: Assets,
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

            self.status.is_sold = true;

            let for_component_to_take = self
                .requested_resource
                .take_needed_assets(&mut required_assets);

            let royality_data = self.nft_royalities.take().map_or(
                for_component_to_take.get_royality_data(),
                |mut f| {
                    if let Some(x) = for_component_to_take.get_royality_data() {
                        f.extend(x);
                    }
                    Some(f)
                },
            );

            self.requested_resource_vault = Some(AssetsAccumulator::new(for_component_to_take));
            self.taker = Some(badge);
            (
                Some(vec![self.offered_resource.take(), required_assets]),
                royality_data,
            )
        }

        pub fn withdraw_assets(
            &mut self,
            badge: NonFungibleGlobalId,
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
            (
                Some(vec![self.requested_resource_vault.as_mut().unwrap().take()]),
                Some(RoyalityData {
                    addresses: vec![],
                    amount: self.service_royality,
                }),
            )
        }

        pub fn cancel_escrow(&mut self, badge: NonFungibleGlobalId) -> Option<Vec<Assets>> {
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
            Some(vec![self.offered_resource.take()])
        }
    }
}

#[derive(ScryptoSbor)]
pub struct Args {
    pub offered_resource: Assets,
    pub requested_resource: RequiredResources,
}
