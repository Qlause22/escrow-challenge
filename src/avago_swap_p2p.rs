use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
mod avago_swap_p2p {
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
    struct AvagoSwapP2P {
        id: u128,
        status: Status,
        owner: NonFungibleLocalId,
        taker: NonFungibleLocalId,
        offered_resource: AssetsAccumulator,
        requested_resource: RequiredResources,
        requested_resource_vault: Option<AssetsAccumulator>,
    }

    impl AvagoSwapP2P {
        pub fn instantiate(
            owner: NonFungibleLocalId,
            main: ComponentAddress,
            id: u128,
            args: ArgsP2P,
        ) -> Global<AvagoSwapP2P> {
            Self {
                id,
                taker: args.taker,
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
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles! {
                main => rule!(require(global_caller(main)));
            })
            .globalize()
        }

        pub fn exchange(
            &mut self,
            badge: NonFungibleLocalId,
            mut required_assets: Assets,
        ) -> Option<Vec<Assets>> {
            assert!(
                self.taker == badge,
                "Your id have no access to this contract."
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

            let for_component_to_take = self
                .requested_resource
                .take_needed_assets(&mut required_assets);

            self.requested_resource_vault = Some(AssetsAccumulator::new(for_component_to_take));
            Some(vec![self.offered_resource.take(), required_assets])
        }

        pub fn withdraw_assets(&mut self, badge: NonFungibleLocalId) -> Option<Vec<Assets>> {
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
            Some(vec![self.requested_resource_vault.as_mut().unwrap().take()])
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
            Some(vec![self.offered_resource.take()])
        }
    }
}

#[derive(ScryptoSbor)]
pub struct ArgsP2P {
    offered_resource: Assets,
    requested_resource: RequiredResources,
    taker: NonFungibleLocalId,
}
