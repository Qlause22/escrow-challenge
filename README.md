## Table of Contents

- [Introduction](#introduction)
- [Types](#types)
  - [Resource Specifier](#resource-specifier)
  - [Escrow Badge](#escrowbadge)
- [State](#state)
- [Interface](#interface)
  - [instantiate_escrow](#instantiate_escrow)
  - [exchange](#exchange)
  - [withdraw_resource](#withdraw_resource)
  - [cancel_escrow](#cancel_escrow)
- [Submission Guidelines](#submission-guidelines)


## Introduction

This Scrypto package contains a a boilerplate blueprint to assist you with the Escrow Challenge exercise, but you are not required to use it. The boilerplate blueprint contains predefined types and interface for you to help you reason about the necessary logic to facilitate an escrow exchange.

Please read the documentation below to help you understand how to best use the boilerplate for your needs.

## Types

### Resource Specifier

We define a `ResourceSpecifier` enum that can be either `Fungible` or `NonFungible`. We use an enum to represent the different types of 
resources that can be used in the escrow. For example, when we instantiate the Escrow Blueprint, we will request from the user what resource
they want to request from the other party. Because we are using an enum type, we are able to handle both types of resource (fungibles and non-fungible) 
the user may specify.

The `ResourceSpecifier` type implements a method called `get_resource_address`. We do this so it's easier to retrieve the value of the `ResourceAddress` whether it comes from the `Fungible` or `NonFungible` variant. 

```rust
#[derive(ScryptoSbor, Clone)]
pub enum ResourceSpecifier {
    Fungible {
        resource_address: ResourceAddress,
        amount: Decimal
    },
    NonFungible {
        resource_address: ResourceAddress,
        non_fungible_local_id: NonFungibleLocalId
    }
}

impl ResourceSpecifier {

    pub fn get_resource_address(&self) -> ResourceAddress {
        match self {
            Self::Fungible {
                resource_address, ..
            }
            | Self::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }
}
```

Example use:

```rust

pub fn verify_requested_resource(&self, requested_resource: ResourceSpecifier) {

    let requested_resource: ResourceAddress = requested_resource.get_resource_address();

    assert_eq!(
        requested_resource,
        self.requested_resource_vault.resource_address(),
        "Incorrect resource specified!"
    );

}


```


### EscrowBadge

The `EscrowBadge` is a `Struct` type that will represent the `NonFungibleData` of a non-fungible resource we will create for the instantiator. Essentially, this non-fungible resource is created to allow the instantiatior, who defines the terms of the escrow, to cancel the escrow and receive
their resource back if the other party does not fulfill their end or to withdraw the resource from the escrow once the other party has fulfilled the terms of the escrow. The `NonFungibleData` will simply contain the `ResourceAddress` of the resource the instantiator is offering in the escrow.


```rust
#[derive(ScryptoSbor, NonFungibleData)]
pub struct EscrowBadge {
    offered_resource: ResourceAddress
}
```


## State

The `Escrow` blueprint contains 4 state defined in its `Struct` to record information about the escrow. These states are:

```rust
struct Escrow {
    requested_resource: ResourceSpecifier,
    offered_resource: Vault,
    requested_resource_vault: Vault,
    escrow_nft: ResourceAddress,
}
```

| Field | Type  | Description |
| ----- | ----- | ----------- |
| `requested_resource` | `ResourceSpecifier` |  The `requested_resource` is a field which is meant to capture the instantiator's requested resource in the exchange. The instantiatior will be requested what resource they would like for the resource they will offer to the other party. The `ResourceSpecifier` is the value that will capture the instatiator's request to allow flexibility for if the instantiator prefers a `Fungible` or `NonFungible` resource.
| `offered_resource` | `Vault` | The `offered_resource` is a field that will contain the resource the instantiator is offering to the other party as part of the exchange. At instantiation, the instantiator is required to send their offered resource to the escrow component as part of their end of the deal and will be contained in the `Vault` value.
| `requested_resource_vault` | `Vault` | The `requested_resource_vault` is a field that will contain the resource offered by the other party. When the other party sends the resource requested by the instantiatior, the resource will be contained in the `Vault` value.
| `escrow_nft` | `ResourceAddress` | The `escrow_nft` is a field that will allow the component to know the identifier address of the `EscrowBadge` nft. At instantiation, the instantiator will receive this minted NFT to allow them to 

## Interface

The boilerplate blueprint defines the following interface for its instantiated component:


### instantiate_escrow
| Name            | Type            | Arguments       | Description  
| --------------- | --------------- | ----------------- | --------------- |
| `instantiate_escrow` | Function | `requested_resource`<br>`offered_resource` | An instantiation function which will create a component from the `Escrow` blueprint. The function takes two arguments which will determine the instantiator's requested resource and the offered resource in the exchange. The function will return a `Global<Escrow>` and `NonFungibleBucket` which represents that instantiated component and the instantiator's `EscrowBadge` NFT.

```rust
pub fn instantiate_escrow(
    requested_resource: ResourceSpecifier,
    offered_resource: Bucket
) -> (Global<Escrow>, NonFungibleBucket) {

    // * Instantiation logic * //

}
```

### exchange
| Name            | Type            | Arguments       | Description  
| --------------- | --------------- | ----------------- | --------------- |
| `exchange` | Method | `bucket_of_resource` | A method that faciliates the escrow exchange. The other party will send their part of the deal (the requested resource) and returns a `Bucket` containing the offered resource.

```rust
pub fn exchange(&mut self, bucket_of_resource: Bucket) -> Bucket {

    // * Exchange logic * //

}
```

### withdraw_resource
| Name            | Type            | Arguments       | Description  
| --------------- | --------------- | ----------------- | --------------- |
| `withdraw_resource` | Method | `escrow_nft` | A method that will allow the instantiator to withdraw the requested resource. The `EscrowBadge` NFT needs to be sent to the component to verify that the caller is the person that is allowed to redeem the requested resource. Once verified, the method will return a `Bucket` of the requested resource.

```rust
pub fn withdraw_resource(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {

    // * Withdraw logic * //

}
```

### cancel_escrow
| Name            | Type            | Arguments       | Description  
| --------------- | --------------- | ----------------- | --------------- |
| `cancel_escrow` | Method | `escrow_nft` | A method that closes the escrow if the other party has rejected the exchange. The `EscrowBadge` NFT needs to be sent to the component to verify the caller is the person allowed to close the escrow and withdraw the offered resource. Once verified, the method will reutnr a `Bucket` of the offered resource. Once the escrow is cancelled, the other party or anyone else cannot be allowed to deposit the requested resource to the component.

```rust
pub fn cancel_escrow(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {

    // * Cancel escrow logic * //

}
```

## Submission 

To submit your exercise:

1. Run `scrypto build` while in your project directory. You should have a `.rpd` and `.wasm` file in your `/target/wasm32-unknown-unknown/release` folder.
2. Deploy your package on [Stokenet](https://stokenet-console.radixdlt.com/deploy-package)
3. Fill out the submission form on [LearnWorlds](https://academy.radixdlt.com/path-player?courseid=scrypto101&unit=663e315c23537a334304e8aeUnit)

If you have any questions, feel free to head over to the `#scrypto-rust-beginners` channel on [Discord](https://discord.com/channels/417762285172555786/936736798342803528)!
