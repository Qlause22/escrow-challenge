use crate::common::*;
use scrypto::prelude::*;

#[blueprint]
#[types(AvagoBadge, ComponentAddress, Decimal)]
mod avago_proxy {

    enable_method_auth! {
        methods {
            change_swap_style => restrict_to : [OWNER];
            change_service_royality => restrict_to : [OWNER];
            claim_custom_royality => restrict_to : [OWNER];
            change_reward_address => restrict_to : [OWNER];
            register => PUBLIC;
            unregister => PUBLIC;
            call_component => PUBLIC;
            create_swap => PUBLIC;
            create_multiple_swap => PUBLIC;
        }
    }
    struct AvagoProxy {
        xrd_vault: FungibleVault,
        service_royality: Decimal,
        resource_manager: ResourceManager,
        swap_style: HashMap<u8, BlueprintId>,
        users: KeyValueStore<ComponentAddress, ()>,
        id: Ids,
        reward_address: Option<ComponentAddress>,
    }

    impl AvagoProxy {
        pub fn instantiate_proxy(role: NonFungibleGlobalId) -> Global<AvagoProxy> {
            let owner = OwnerRole::Updatable(rule!(require(role)));
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(AvagoProxy::blueprint_id());

            let mut swap_style: HashMap<u8, BlueprintId> = HashMap::new();

            swap_style.insert(
                1u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapSimple"),
                },
            );
            swap_style.insert(
                2u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapBasic"),
                },
            );
            swap_style.insert(
                3u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapBid"),
                },
            );
            swap_style.insert(
                4u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapOffer"),
                },
            );
            swap_style.insert(
                5u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapP2P"),
                },
            );

            Self {
                xrd_vault: FungibleVault::new(XRD),
                service_royality: dec!(20),
                resource_manager: ResourceBuilder::new_integer_non_fungible_with_registered_type::<
                    AvagoBadge,
                >(owner.clone())
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor => rule!(require(require(global_caller(component_address))));
                    depositor_updater => rule!(deny_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply(),
                users: KeyValueStore::new(),
                reward_address: None,
                id: Ids::new(),
                swap_style,
            }
            .instantiate()
            .prepare_to_globalize(owner)
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_setter => rule!(require(global_caller(component_address)));
                    royalty_setter_updater => OWNER;
                    royalty_locker => OWNER;
                    royalty_locker_updater => OWNER;
                    royalty_claimer =>  rule!(require(global_caller(component_address)));
                    royalty_claimer_updater => OWNER;
                },
                init {
                    change_swap_style => Free, locked;
                    register => Free, locked;
                    unregister => Free, locked;
                    call_component => Free, locked;
                    claim_custom_royality => Free, locked;
                    change_service_royality => Free, locked;
                    change_reward_address => Free, locked;
                    create_swap => Free, updatable;
                    create_multiple_swap => Free, updatable;
                }
            })
            .with_address(address_reservation)
            .globalize()
        }

        pub fn change_swap_style(
            &mut self,
            id: u8,
            package_address: PackageAddress,
            blueprint_name: String,
        ) {
            if let Some(style) = self.swap_style.get_mut(&id) {
                *style = BlueprintId {
                    package_address,
                    blueprint_name,
                };
            } else {
                self.swap_style.insert(
                    id,
                    BlueprintId {
                        package_address,
                        blueprint_name,
                    },
                );
            }
        }

        pub fn register(&mut self, mut account: Global<Account>) {
            assert!(
                self.users.get(&account.address()).is_none(),
                "Account already registered"
            );

            self.id.member += 1u64;

            let nft = vec![self.resource_manager.mint_non_fungible(
                &NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(self.id.member)),
                AvagoBadge {
                    owner: account.address(),
                },
            )];

            self.users.insert(account.address(), ());

            account.try_deposit_batch_or_abort(nft, None);
        }

        pub fn unregister(&mut self, badge: NonFungibleBucket) {
            assert!(
                badge.resource_address() == self.resource_manager.address(),
                "Invalid Resource Address"
            );

            let data: AvagoBadge = badge.as_non_fungible().non_fungible().data();

            let deleted_account = self.users.remove(&data.owner);

            assert!(deleted_account.is_some(), "Account is not registered");

            badge.burn();
        }

        pub fn create_swap(
            &mut self,
            proof: Proof,
            style: u8,
            args: ScryptoValue,
        ) -> Global<AnyComponent> {
            assert!(style != 0, "simple swap is not supported with this method");

            let local_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .local_id()
                .clone();

            self.id.swap += 1u128;

            let BlueprintId {
                package_address,
                blueprint_name,
            } = self.swap_style.get(&style).unwrap().to_owned();

            scrypto_decode(&ScryptoVmV1Api::blueprint_call(
                package_address,
                &blueprint_name,
                "instantiate",
                scrypto_args!(
                    self.service_royality,
                    local_id,
                    Runtime::global_address(),
                    self.id.swap,
                    args
                ),
            ))
            .unwrap()
        }

        pub fn create_multiple_swap(
            &mut self,
            proof: Proof,
            mut assets: NonFungibleBucket,
            price: Decimal,
        ) {
            let local_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .local_id()
                .clone();

            let BlueprintId {
                package_address,
                blueprint_name,
            } = self.swap_style.get(&1u8).unwrap().to_owned();

            let component_address: ComponentAddress = Runtime::global_address();

            while !assets.is_empty() {
                self.id.swap += 1u128;

                ScryptoVmV1Api::blueprint_call(
                    package_address,
                    &blueprint_name,
                    "instantiate",
                    scrypto_args!(
                        self.service_royality,
                        &local_id,
                        component_address,
                        self.id.swap,
                        ArgsSimpleSwap {
                            price,
                            asset: assets.take(1),
                        }
                    ),
                );
            }
            assets.drop_empty();
        }

        pub fn call_component(
            &self,
            proof: Proof,
            component_address: ComponentAddress,
            method: String,
            args: Option<ScryptoValue>,
        ) -> Option<Vec<Assets>> {
            let local_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .local_id()
                .clone();

            let scrypto_args = args.map_or(scrypto_args!(local_id.clone()), |arg| {
                scrypto_args!(local_id, arg)
            });

            scrypto_decode(&ScryptoVmV1Api::object_call(
                component_address.as_node_id(),
                method.as_str(),
                scrypto_args,
            ))
            .unwrap()
        }

        pub fn claim_custom_royality(&mut self, amount: Decimal) -> FungibleBucket {
            self.xrd_vault.put(
                Runtime::global_component()
                    .claim_component_royalties()
                    .as_fungible(),
            );
            self.xrd_vault.take(amount)
        }

        pub fn change_service_royality(&mut self, amount: Decimal) {
            self.service_royality = amount;
        }

        pub fn change_reward_address(&mut self, address: Option<ComponentAddress>) {
            self.reward_address = address;
        }
    }
}

#[derive(ScryptoSbor)]
pub struct Ids {
    member: u64,
    swap: u128,
}

impl Ids {
    pub fn new() -> Self {
        Self {
            member: 0u64,
            swap: 0u128,
        }
    }
}
