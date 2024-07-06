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
            pay_royality => PUBLIC;
            call_component_and_set_royality => PUBLIC;
            create_swap => PUBLIC;
            claim_royality => PUBLIC;
        }
    }
    struct AvagoProxy {
        xrd_vault: FungibleVault,
        service_royality: Decimal,
        resource_manager: ResourceManager,
        swap_style: HashMap<u8, BlueprintId>,
        users: KeyValueStore<ComponentAddress, ()>,
        royality_list: KeyValueStore<ComponentAddress, Decimal>,
        id: ID,
        ticket: ResourceManager,
        transient_royality_data: Option<RoyalityData>,
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
                    blueprint_name: String::from("AvagoSwapBasic"),
                },
            );
            swap_style.insert(
                2u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapBid"),
                },
            );
            swap_style.insert(
                3u8,
                BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: String::from("AvagoSwapOffer"),
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
                royality_list: KeyValueStore::<ComponentAddress, Decimal>::new_with_registered_type(
                ),
                ticket: ResourceBuilder::new_fungible(owner.clone())
                    .divisibility(DIVISIBILITY_NONE)
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(global_caller(component_address)));
                        burner_updater => rule!(deny_all);
                    })
                    .deposit_roles(deposit_roles! {
                        depositor => rule!(deny_all);
                        depositor_updater => rule!(deny_all);
                    })
                    .withdraw_roles(withdraw_roles! {
                        withdrawer => rule!(deny_all);
                        withdrawer_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply(),
                reward_address: None,
                transient_royality_data: None,
                id: ID {
                    member: 0u64,
                    basic: 0u64,
                    bid: 0u64,
                    offer: 0u64,
                },
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
                    call_component_and_set_royality => Free, locked;
                    claim_royality => Free, locked;
                    claim_custom_royality => Free, locked;
                    change_service_royality => Free, locked;
                    change_reward_address => Free, locked;
                    create_swap => Free, updatable;
                    pay_royality => Free, updatable;
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
            let global_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .global_id()
                .to_owned();

            self.increase_id(&style);

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
                    global_id,
                    Runtime::global_address(),
                    self.get_current_id_of_style(&style),
                    args
                ),
            ))
            .unwrap()
        }

        pub fn call_component(
            &self,
            proof: Proof,
            component_address: ComponentAddress,
            method: String,
            args: Option<ScryptoValue>,
        ) -> Option<Vec<Assets>> {
            let global_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .global_id()
                .to_owned();

            let scrypto_args = args.map_or(scrypto_args!(global_id.clone()), |arg| {
                scrypto_args!(global_id, arg)
            });

            scrypto_decode(&ScryptoVmV1Api::object_call(
                component_address.as_node_id(),
                method.as_str(),
                scrypto_args,
            ))
            .unwrap()
        }

        pub fn call_component_and_set_royality(
            &mut self,
            proof: Proof,
            component_address: ComponentAddress,
            method: String,
            args: Option<ScryptoValue>,
        ) -> (Option<Vec<Assets>>, Bucket) {
            let global_id = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .global_id()
                .to_owned();

            let scrypto_args = args.map_or(scrypto_args!(global_id.clone()), |arg| {
                scrypto_args!(global_id, arg)
            });

            let (assets, royality_data): (Option<Vec<Assets>>, Option<RoyalityData>) =
                scrypto_decode(&ScryptoVmV1Api::object_call(
                    component_address.as_node_id(),
                    method.as_str(),
                    scrypto_args,
                ))
                .unwrap();

            match royality_data {
                Some(royality_data) => {
                    Runtime::global_component().set_royalty(
                        String::from("pay_royality"),
                        RoyaltyAmount::Xrd(royality_data.amount),
                    );
                    self.transient_royality_data = Some(royality_data);
                }
                None => {
                    Runtime::global_component()
                        .set_royalty(String::from("pay_royality"), RoyaltyAmount::Xrd(dec!(0)));
                    self.transient_royality_data = None;
                }
            };

            (assets, self.ticket.mint(1))
        }

        pub fn pay_royality(&mut self, ticket: Bucket) -> Option<Assets> {
            assert!(
                ticket.resource_address() == self.ticket.address(),
                "invalid ticket"
            );

            match self.transient_royality_data.as_ref() {
                Some(royality_data) => {
                    royality_data
                        .addresses
                        .iter()
                        .for_each(|component_address| {
                            if self.royality_list.get(component_address).is_some() {
                                *self.royality_list.get_mut(component_address).unwrap() += dec!(1);
                            } else {
                                self.royality_list.insert(*component_address, dec!(1));
                            }
                        });
                }
                None => {
                    Runtime::global_component()
                        .set_royalty(String::from("pay_royality"), RoyaltyAmount::Xrd(dec!(0)));
                }
            }

            let reward: Option<Assets> = if let Some(reward_address) = self.reward_address {
                scrypto_decode(&ScryptoVmV1Api::object_call(
                    reward_address.as_node_id(),
                    "claim_reward",
                    scrypto_args!(),
                ))
                .unwrap()
            } else {
                None
            };

            self.transient_royality_data = None;
            ticket.burn();
            reward
        }

        pub fn claim_royality(&mut self, proof: Proof) -> FungibleBucket {
            let nft_data = proof
                .check_with_message(self.resource_manager.address(), "Invalid proof")
                .as_non_fungible()
                .non_fungible::<AvagoBadge>()
                .data();

            match self.royality_list.remove(&nft_data.owner) {
                Some(amount) => {
                    self.xrd_vault.put(
                        Runtime::global_component()
                            .claim_component_royalties()
                            .as_fungible(),
                    );
                    self.xrd_vault.take(amount)
                }
                None => Runtime::panic(String::from("You dont have royalties")),
            }
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

        fn increase_id(&mut self, style: &u8) {
            match style {
                1u8 => self.id.basic += 1u64,
                2u8 => self.id.bid += 1u64,
                3u8 => self.id.offer += 1u64,
                _ => Runtime::panic(String::from("Invalid style")),
            }
        }

        fn get_current_id_of_style(&self, style: &u8) -> u64 {
            match style {
                1u8 => self.id.basic,
                2u8 => self.id.bid,
                3u8 => self.id.offer,
                _ => Runtime::panic(String::from("Invalid style")),
            }
        }
    }
}

#[derive(ScryptoSbor)]
struct ID {
    member: u64,
    basic: u64,
    bid: u64,
    offer: u64,
}
