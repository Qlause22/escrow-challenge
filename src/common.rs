use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct Assets {
    pub fungible_buckets: Option<Vec<FungibleBucket>>,
    pub non_fungible_buckets: Option<Vec<NonFungibleBucket>>,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct EscrowBadge {
    pub component_owned: ComponentAddress,
}
