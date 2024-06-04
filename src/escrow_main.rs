use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
mod escrow_proxy {

    struct EscrowProxy {
        swap_id: u64,
        resource_manager: ResourceManager,
        escrow_style: BlueprintId,
    }

    impl EscrowProxy {
        pub fn instantiate_proxy(role: Option<ResourceAddress>) -> Global<EscrowProxy> {
            let owner = role.map_or(OwnerRole::None, |r| OwnerRole::Updatable(rule!(require(r))));
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(EscrowProxy::blueprint_id());

            Self {
                resource_manager: ResourceBuilder::new_integer_non_fungible::<EscrowBadge>(
                    owner.clone(),
                )
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply(),
                swap_id: 0u64,
                escrow_style: BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("Escrow"),
                },
            }
            .instantiate()
            .prepare_to_globalize(owner)
            .with_address(address_reservation)
            .globalize()
        }

        // we can create other OTC style and apply it with method bellow
        pub fn change_swap_style(
            &mut self,
            package_address: PackageAddress,
            blueprint_name: String,
        ) {
            self.escrow_style = BlueprintId {
                package_address,
                blueprint_name,
            };
        }

        pub fn create_swap(&mut self, args: ScryptoValue) -> Bucket {
            let non_fungible_local_id =
                NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(self.swap_id));
            self.swap_id += 1;

            let non_fungible_global_id = NonFungibleGlobalId::new(
                self.resource_manager.address(),
                non_fungible_local_id.clone(),
            );

            let BlueprintId {
                package_address,
                blueprint_name,
            } = &self.escrow_style;

            let component: Global<AnyComponent> = scrypto_decode(&ScryptoVmV1Api::blueprint_call(
                package_address.to_owned(),
                blueprint_name,
                "instantiate_escrow",
                scrypto_args!(non_fungible_global_id, args),
            ))
            .unwrap();

            self.resource_manager.mint_non_fungible(
                &non_fungible_local_id,
                EscrowBadge {
                    component_owned: component.address(),
                },
            )
        }
    }
}
