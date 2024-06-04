use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
mod escrow {
    struct Escrow {
        status: Status,
        owner: NonFungibleGlobalId,
        offered_resource: AssetsAccumulator,
        requested_resource: RequiredResources,
        requested_resource_vault: Option<AssetsAccumulator>,
    }

    impl Escrow {
        pub fn instantiate_escrow(owner: NonFungibleGlobalId, args: Args) -> Global<Escrow> {
            Self {
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
            .globalize()
        }

        pub fn exchange(&mut self, mut required_assets: Assets) -> (Assets, Assets) {
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

            // taking needed assets for the component and give back the asset if user give more than required assets
            let for_component_to_take = self
                .requested_resource
                .take_needed_assets(&mut required_assets);

            self.requested_resource_vault = Some(AssetsAccumulator::new(for_component_to_take));

            (self.offered_resource.take(), required_assets)
        }

        pub fn withdraw_assets(&mut self, badge: NonFungibleBucket) -> Assets {
            self.check_badge(badge);

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
            self.requested_resource_vault.as_mut().unwrap().take()
        }

        pub fn cancel_escrow(&mut self, badge: NonFungibleBucket) -> Assets {
            self.check_badge(badge);

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
            self.offered_resource.take()
        }

        fn check_badge(&self, bucket: NonFungibleBucket) {
            assert!(
                bucket.amount() == dec!(1),
                "The contract only process one Nft badge"
            );
            assert!(
                bucket.resource_address() == self.owner.resource_address(),
                "Invalid badge address"
            );
            assert!(
                &bucket.non_fungible_local_id() == self.owner.local_id(),
                "Invalid badge address"
            );
            bucket.burn()
        }
    }
}

// Types //

#[derive(ScryptoSbor)]
pub struct Args {
    offered_resource: Assets,
    requested_resource: RequiredResources,
}

#[derive(ScryptoSbor)]
pub struct Status {
    is_sold: bool,
    is_cancelled: bool,
    is_took: bool,
}

#[derive(ScryptoSbor)]
pub struct RequiredResources {
    fungible_token_rs: Option<HashMap<ResourceAddress, Decimal>>,
    non_fungible_token_rs: Option<HashMap<ResourceAddress, IndexSet<NonFungibleLocalId>>>,
}

impl RequiredResources {
    pub fn take_needed_assets(&mut self, required_assets: &mut Assets) -> Assets {
        let mut for_component_to_take = Assets {
            fungible_buckets: None,
            non_fungible_buckets: None,
        };

        // taking needed assets
        if self.non_fungible_token_rs.is_some() {
            assert!(
                required_assets.non_fungible_buckets.is_some(),
                "[Forbiden] : Required NFT assets"
            );

            for_component_to_take.non_fungible_buckets = Some(
                required_assets
                    .non_fungible_buckets
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .map(|nft_bucket| self.extract_nft(nft_bucket))
                    .collect::<Vec<NonFungibleBucket>>(),
            );
        }

        if self.fungible_token_rs.is_some() {
            assert!(
                required_assets.fungible_buckets.is_some(),
                "[Forbiden] : Required NFT assets"
            );

            for_component_to_take.fungible_buckets = Some(
                required_assets
                    .fungible_buckets
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .map(|ft_bucket| self.extract_ft(ft_bucket))
                    .collect::<Vec<FungibleBucket>>(),
            );
        }

        // if there are still required assets on the HashMap, it will panic
        // if the owner doesn't require nft/ft it will pass the assertion
        assert!(
            self.non_fungible_token_rs
                .as_ref()
                .map_or(true, |f| f.is_empty()),
            "[Forbidden] : NFT is not sufficient"
        );

        assert!(
            self.fungible_token_rs
                .as_ref()
                .map_or(true, |f| f.is_empty()),
            "[Forbidden] : FT is not sufficient"
        );

        for_component_to_take
    }

    // extracting needed assets from every bucket that buyer passes to the method
    fn extract_nft(&mut self, nft_bucket: &mut NonFungibleBucket) -> NonFungibleBucket {
        let needed_nft_ids = self
            .non_fungible_token_rs
            .as_mut()
            .unwrap()
            .remove(&nft_bucket.resource_address())
            .expect("[Forbiden] : NFT is not match");

        nft_bucket.take_non_fungibles(&needed_nft_ids)
    }

    fn extract_ft(&mut self, ft_bucket: &mut FungibleBucket) -> FungibleBucket {
        let needed_amount = self
            .fungible_token_rs
            .as_mut()
            .unwrap()
            .remove(&ft_bucket.resource_address())
            .expect("[Forbiden] : FT is not match");

        ft_bucket.take(needed_amount)
    }
}

#[derive(ScryptoSbor)]
pub struct AssetsAccumulator {
    fungible_vault: Option<Vec<FungibleVault>>,
    non_fungible_vault: Option<Vec<NonFungibleVault>>,
}

// AssetsAccumulator will store the assets from temporary container (Bucket) to permanent container (Vault).
impl AssetsAccumulator {
    pub fn new(assets: Assets) -> Self {
        Self {
            fungible_vault: assets.fungible_buckets.map(|ft_bucket| {
                ft_bucket
                    .into_iter()
                    .map(FungibleVault::with_bucket)
                    .collect()
            }),
            non_fungible_vault: assets.non_fungible_buckets.map(|nft_bucket| {
                nft_bucket
                    .into_iter()
                    .map(NonFungibleVault::with_bucket)
                    .collect()
            }),
        }
    }

    // take method will take all assets from vault to bucket that will give it to the owner.
    pub fn take(&mut self) -> Assets {
        Assets {
            fungible_buckets: self
                .fungible_vault
                .as_mut()
                .map(|ft_bucket| ft_bucket.iter_mut().map(|x| x.take_all()).collect()),
            non_fungible_buckets: self
                .non_fungible_vault
                .as_mut()
                .map(|nft_bucket| nft_bucket.iter_mut().map(|x| x.take_all()).collect()),
        }
    }
}
