use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct Assets {
    pub fungible_buckets: Option<Vec<FungibleBucket>>,
    pub non_fungible_buckets: Option<Vec<NonFungibleBucket>>,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AvagoBadge {
    pub owner: ComponentAddress,
}

#[derive(ScryptoSbor)]
pub struct Status {
    pub is_sold: bool,
    pub is_cancelled: bool,
    pub is_took: bool,
}

#[derive(ScryptoSbor)]
pub struct RequiredResources {
    pub fungible_token_rs: Option<HashMap<ResourceAddress, Decimal>>,
    pub non_fungible_token_rs: Option<HashMap<ResourceAddress, IndexSet<NonFungibleLocalId>>>,
}

impl RequiredResources {
    pub fn take_needed_assets(&mut self, required_assets: &mut Assets) -> Assets {
        let mut for_component_to_take = Assets {
            fungible_buckets: None,
            non_fungible_buckets: None,
        };

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
    pub fungible_vault: Option<Vec<FungibleVault>>,
    pub non_fungible_vault: Option<Vec<NonFungibleVault>>,
}

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

#[derive(ScryptoSbor)]
pub struct Args {
    pub offered_resource: Assets,
    pub requested_resource: RequiredResources,
}

#[derive(ScryptoSbor)]
pub struct ArgsSimpleSwap {
    pub asset: NonFungibleBucket,
    pub price: Decimal,
}
