use scrypto::prelude::*;

/// Hello Beem, welcome to my blueprint. Make yourself at home, but don't touch anything you can't afford.

// Each Swap Creator gets give a Swap Key - this is the non-fungible struct for that key
// I don't actually use all this data thats being stored on it on Trove, but it does mean there's an easy access full-record.
#[derive(NonFungibleData, ScryptoSbor, Debug)]
pub struct Escroceipt {
    name: String,
    description: String,
    key_image_url: Url,
    swap_component: ComponentAddress,
    nfts_offered: Vec<NonFungibleGlobalId>,
    tokens_offered: HashMap<ResourceAddress, Decimal>,
    nfts_requested: Vec<NonFungibleGlobalId>,
    tokens_requested: HashMap<ResourceAddress, Decimal>,
}

// Just some events for the front end to nab data from.
#[derive(ScryptoSbor, ScryptoEvent)]
struct ComponentCreated {
    component: ComponentAddress,
    creator_badge: ResourceAddress,
    creator_badge_local: NonFungibleLocalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct PartnerLocked {
    partner: ComponentAddress,
    partner_badge: ResourceAddress,
    partner_local_id: NonFungibleLocalId,
}

#[blueprint]
#[events(ComponentCreated, PartnerLocked)]
mod barter {

    // Potential bug here, should cost users $1k USD to swap IMO.
    enable_package_royalties! {
        new_trade_proposal => Free;
        partner_deposit_tokens => Free;
        partner_deposit_nfts => Free;
        partner_claims_creator_assets => Xrd(20.into());
        creator_claims_partner_assets => Xrd(20.into());
        creator_cancel => Free;
        partner_cancel => Free;
        burn_creator_badge => Free;
        get_badge => Free;
        burn_partner_badge => Free;
    }

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            creator_cancel => restrict_to: [admin];
            creator_claims_partner_assets => restrict_to: [admin];
            burn_creator_badge => PUBLIC;
            partner_deposit_tokens => PUBLIC;
            partner_deposit_nfts => PUBLIC;
            partner_claims_creator_assets => PUBLIC;
            partner_cancel => PUBLIC;
            get_badge => PUBLIC;
            burn_partner_badge => PUBLIC;
        }
    }

    struct Barter {
        // creator assets and badge
        creator_vaults: HashMap<ResourceAddress, Vault>,
        a_vault_key: ResourceAddress,
        a_vault_key_id: NonFungibleLocalId,
        a_vault_key_global: NonFungibleGlobalId,

        // partner assets and badge (optional)
        partner_vaults: HashMap<ResourceAddress, Vault>,
        badge_partner: Option<ResourceAddress>,
        badge_partner_local: Option<NonFungibleLocalId>,

        // state record of expected assets from partner
        expected_nfts: Vec<NonFungibleGlobalId>,
        expected_tokens: HashMap<ResourceAddress, Decimal>,

        // resource manager for burning badges
        proposal_resource_manager: ResourceManager,

        // state bools for validation
        tokens_validated: bool,
        nfts_validated: bool,
        private: bool,
        swapped: bool,
        category: Option<ResourceAddress>,
    }

    impl Barter {
        pub fn new_trade_proposal(
            // Just a random name that users can give their swaps
            custom_trade_name: String,
            // Optional to add a partner by including their account address - this triggers various access controls,
            // including an 'anti-pattern' airdrop deposit of a badge to the partner address automatically.
            partner: Option<ComponentAddress>,
            // Optional to include a variety of fungibles in your offer
            a_tokens: Option<Vec<Bucket>>,
            // Optional to include a variety of non fungibles in your offer
            a_nfts: Option<Vec<Bucket>>,
            // Optional to include a list of non fungible assets you're requesting in return
            b_nft_deposits: Option<Vec<NonFungibleGlobalId>>,
            // Optional to include a list of fungible assets you'ree requesting in retunr
            b_token_deposits: Option<HashMap<ResourceAddress, Decimal>>,
            // return component address, creator's badge resource address + local id, partners badge (optional) and the creator's badge itself
        ) -> (Global<Barter>, NonFungibleBucket) {
            let mut b_nft_deposits_unwrap: Vec<NonFungibleGlobalId> = Vec::new();
            let mut b_token_deposits_unwrap: HashMap<ResourceAddress, Decimal> = HashMap::new();

            // all of this section is calculating how many unique assets have been included in the swap proposal
            // then checking its less than 50.
            // Somewhat arbitrary limit on size... 256 events per tx - approx. 60 resource moves in a single trade
            // but 50 is a 'nicer' number.
            // Also I have no idea how to handle if the NFTs themselves have tonnes of data on them which could also potentially screw
            // the total amount of assets that can be exchanged... in anycase there's a failssafe method for the creator
            // to withdraw their assets once the swap's created.

            //There is an oversight here though..... I should really check that something is actually being requested or offered.
            //I set it up on the frontend, but I should really do it here too.

            if (b_nft_deposits.is_none() && b_token_deposits.is_none())
                || (a_nfts.is_none() && a_tokens.is_none())
            {
                panic!("You need to offer/request something")
            }

           

            let mut a_nft_len = 0;
            let mut a_tokens_len = 0;
            let mut b_nft_len = 0;
            let mut b_tokens_len = 0;

            if b_nft_deposits.is_some() {
                b_nft_len = b_nft_deposits.as_ref().unwrap().len();
                b_nft_deposits_unwrap = b_nft_deposits.unwrap();
            }
            if b_token_deposits.is_some() {
                b_tokens_len = b_token_deposits.as_ref().unwrap().len();
                b_token_deposits_unwrap = b_token_deposits.unwrap();
            }
            if a_tokens.is_some() {
                a_tokens_len = a_tokens.as_ref().unwrap().len();
            }
            if a_nfts.is_some() {
                let a_nft_buckets = a_nfts.as_ref().unwrap();
                for ids in a_nft_buckets.iter() {
                    let count_of_nfts = ids.as_non_fungible().non_fungible_local_ids().len();
                    a_nft_len = a_nft_len + count_of_nfts;
                }
            }

            assert!(
                b_nft_len + b_tokens_len + a_tokens_len + a_nft_len <= 50,
                "Reached single transaction event limit"
            );

            // end of limits checking

            let mut a_key: NonFungibleBucket;
            let mut a_token_deposits: HashMap<ResourceAddress, Decimal> = HashMap::new();
            let mut user_a_vaults = HashMap::new();


            let mut category:  Option<ResourceAddress> = None as Option<ResourceAddress>;

            //Deposit all creator fungible assets in map of vaults
            if a_tokens.is_some() {

                let a_token_buckets = a_tokens.unwrap();
                category = Some(a_token_buckets[0].resource_address());
                for i in a_token_buckets.iter() {
                    a_token_deposits.insert(i.resource_address(), i.amount());
                }
                for bucket in a_token_buckets.into_iter() {
                    user_a_vaults
                        .entry(bucket.resource_address())
                        .or_insert_with(|| Vault::new(bucket.resource_address()))
                        .put(bucket)
                }
            } 

            //Deposit all creator non-fungible assets in map of vaults
            // Don't ask me why I do this wierd if and else thing in the for loop, I just know it works and
            // without it... it didn't work. I tried Omar's nft marketplace blueprint which has a similar
            // loop iterating buckets into a hashmap of vaults, but didn't work for me in my blueprint.

            let mut a_nft_deposits: Vec<NonFungibleGlobalId> = Vec::new();

            if a_nfts.is_some() {
                let a_nft_buckets = a_nfts.unwrap();
                category = Some(a_nft_buckets[0].resource_address());
                for bucket in a_nft_buckets.iter() {
                    let fucket = bucket.as_non_fungible().non_fungible_local_ids();
                    let resource = bucket.resource_address();
                    if bucket.as_non_fungible().non_fungible_local_ids().len() > 1 {
                        for nft_id in fucket.into_iter() {
                            let global_ar: NonFungibleGlobalId =
                                NonFungibleGlobalId::new(resource, nft_id);
                            a_nft_deposits.push(global_ar)
                        }
                    } else {
                        let local: NonFungibleLocalId =
                            bucket.as_non_fungible().non_fungible_local_id();
                        let global = NonFungibleGlobalId::new(resource, local);
                        a_nft_deposits.push(global);
                    }
                }
                for bucket in a_nft_buckets.into_iter() {
                    user_a_vaults
                        .entry(bucket.resource_address())
                        .or_insert_with(|| Vault::new(bucket.resource_address()))
                        .put(bucket)
                }
            }

            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Barter::blueprint_id());

            let expected_b_nft_deposits = b_nft_deposits_unwrap.clone();
            let expected_b_token_deposits = b_token_deposits_unwrap.clone();

            let mut private_bool = false;
            let mut badge_option = None as Option<ResourceAddress>;
            let mut badge_local_id = None as Option<NonFungibleLocalId>;

            let key_custom_name = String::from("TROVE Key: ");

            let key_name = String::from(&custom_trade_name.to_string());

            let key_custom_label = key_custom_name + &key_name;

            let key_name_clone = key_custom_label.clone();

            // This is perhaps over complicated - but basically if you set the swap to private - which you'd only really need to do
            // if you were only requesting a fungible token and you were doing a 'good deal' for someone.
            // If you were requesting an NFT, it doesn't really matter as only one account can have that NFT.
            // The way I handle this here to offer so conditional logic that if private swap is ticked, then a creator key and partner
            // key are minted. If not, then just a creator key is minted.

            if !partner.is_some() {
                a_key = ResourceBuilder::new_string_non_fungible::<Escroceipt>(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                        metadata_setter => rule!(deny_all);
                        metadata_setter_updater => rule!(deny_all);
                    },
                    init {
                        "name" => key_name_clone.to_owned(), locked;
                        "description" => "Your Swap Proposal on trove.tools".to_owned(), locked;
                        "key_image_url" => Url::of("https://trove.tools/TroveSquare.png"), locked;
                        "icon_url" => Url::of("https://trove.tools/TroveSquare.png"), locked;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(deny_all);
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .mint_initial_supply([("Trove_Creator_Key".try_into().unwrap(),
                     Escroceipt {
                    name: "Trove Swap".to_owned(),
                    description: "This NFT contains details of your Swap on Trove".to_owned(),
                    key_image_url: Url::of("https://trove.tools/multiple.png"),
                    swap_component: component_address.clone(),
                    nfts_offered: a_nft_deposits.clone(),
                    tokens_offered: a_token_deposits.clone(),
                    nfts_requested: b_nft_deposits_unwrap.clone(),
                    tokens_requested: b_token_deposits_unwrap.clone(),
                })]);
            } else {
                let account_address = partner.unwrap();

                a_key = ResourceBuilder::new_string_non_fungible::<Escroceipt>(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                        metadata_setter => rule!(deny_all);
                        metadata_setter_updater => rule!(deny_all);
                    },
                    init {
                        "name" => key_name_clone.to_owned(), locked;
                        "description" => "Your Swap Proposal on trove.tools".to_owned(), locked;
                        "key_image_url" => Url::of("https://trove.tools/TroveSquare.png"), locked;
                        "icon_url" => Url::of("https://trove.tools/TroveSquare.png"), locked;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(deny_all);
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .mint_initial_supply([("Trove_Creator_Key".try_into().unwrap(),
                     Escroceipt {
                    name: "Trove Swap".to_owned(),
                    description: "This NFT contains details of your Swap on Trove".to_owned(),
                    key_image_url: Url::of("https://trove.tools/multiple.png"),
                    swap_component: component_address.clone(),
                    nfts_offered: a_nft_deposits.clone(),
                    tokens_offered: a_token_deposits.clone(),
                    nfts_requested: b_nft_deposits_unwrap.clone(),
                    tokens_requested: b_token_deposits_unwrap.clone(),
                }), ("Trove_Partner_Key".try_into().unwrap(),
                Escroceipt {
               name: "Trove Swap".to_owned(),
               description: "This NFT's metadata contains details of the requested Swap on Trove".to_owned(),
               key_image_url: Url::of("https://trove.tools/multiple.png"),
               swap_component: component_address.clone(),
               nfts_offered: a_nft_deposits.clone(),
               tokens_offered: a_token_deposits.clone(),
               nfts_requested: b_nft_deposits_unwrap.clone(),
               tokens_requested: b_token_deposits_unwrap.clone(),
           })
                ]);
                let partner_local_quick_id =
                    NonFungibleLocalId::String("Trove_Partner_Key".try_into().unwrap());
                let partner_nft_badge = a_key.take_non_fungible(&partner_local_quick_id);

                let partner_nft_badge_resource = partner_nft_badge.resource_address();
                let partner_nft_badge_local =
                    partner_nft_badge.as_non_fungible().non_fungible_local_id();
                let badge_bucket: Vec<Bucket> = vec![partner_nft_badge.into()];

                Global::<Account>::from(account_address)
                    .try_deposit_batch_or_abort(badge_bucket, None);
                
                private_bool = true;
                badge_option = Some(partner_nft_badge_resource.clone());
                badge_local_id = Some(partner_nft_badge_local.clone());

                Runtime::emit_event(PartnerLocked {
                    partner: account_address,
                    partner_badge: partner_nft_badge_resource.clone(),
                    partner_local_id: partner_nft_badge_local.clone(),
                });
            }

            let global_key_id = NonFungibleGlobalId::new(
                a_key.resource_address(),
                a_key.as_non_fungible().non_fungible_local_id(),
            );

            let partner_vaults: HashMap<ResourceAddress, Vault> = HashMap::new();

            Runtime::emit_event(ComponentCreated {
                component: component_address,
                creator_badge: a_key.resource_address(),
                creator_badge_local: a_key.as_non_fungible().non_fungible_local_id(),
            });

            let barter_component = Self {
                a_vault_key: a_key.resource_address(),
                a_vault_key_id: a_key.as_non_fungible().non_fungible_local_id(),
                a_vault_key_global: global_key_id.clone(),
                creator_vaults: user_a_vaults,
                partner_vaults,
                expected_nfts: expected_b_nft_deposits,
                expected_tokens: expected_b_token_deposits,
                tokens_validated: false,
                nfts_validated: false,
                proposal_resource_manager: a_key.resource_manager(),
                // partner_badge_manager: b_badge_manager,
                private: private_bool,
                badge_partner: badge_option,
                badge_partner_local: badge_local_id,
                swapped: false,
                category,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .metadata(metadata! (
                roles {
                    metadata_setter => rule!(deny_all);
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name" => "Trove Swap".to_owned(), locked;
                    "description" => "Find your swap proposal on https://trove.tools".to_owned(), locked;
                    "tags" => vec!["Swap".to_string()], locked;
                    "icon_url" => Url::of("https://trove.tools/TroveSquare.png"), locked;
                }
            ))
            .roles(roles!(
                admin => rule!(require(global_key_id));
            ))
            .with_address(address_reservation)
            .globalize();
            // return badge to a
            (barter_component, a_key)
        }

        // getter used for testing

        pub fn get_badge(
            &mut self,
        ) -> (
            ResourceAddress,
            NonFungibleLocalId,
            NonFungibleGlobalId,
            Option<ResourceAddress>,
            Option<NonFungibleLocalId>,
        ) {
            (
                self.a_vault_key,
                self.a_vault_key_id.clone(),
                self.a_vault_key_global.clone(),
                self.badge_partner.clone(),
                self.badge_partner_local.clone(),
            )
        }

        pub fn partner_deposit_nfts(&mut self, b_nft_assets: Vec<Bucket>, b_badge: Option<Proof>) {
            assert!(!self.swapped, "Swap has occurred already occured");
            // if swap required badge
            if self.private {
                // check badge has been passed
                assert!(b_badge.is_some(), "Badge required");

                // check passes badge is valid
                if b_badge.is_some() {
                    let badge_to_validate = b_badge.unwrap();
                    let badge_ref = badge_to_validate.clone();

                    let validation_resource = self.badge_partner.unwrap();

                    let badge_local_to_validate = badge_ref.check(validation_resource);

                    let badge_proof_local = badge_local_to_validate
                        .as_non_fungible()
                        .non_fungible_local_id();
                    let validation_local = self.badge_partner_local.as_ref().unwrap();
                    assert!(&badge_proof_local == validation_local, "Incorrect badge")
                }
            }

            let mut nft_record: Vec<NonFungibleGlobalId> = Vec::new();

            for i in &b_nft_assets {
                let nft_resource = i.resource_address();

                if i.as_non_fungible().non_fungible_local_ids().len() > 1 {
                    let fucket = i.as_non_fungible().non_fungible_local_ids();
                    for nft_id in fucket.into_iter() {
                        let global_ar: NonFungibleGlobalId =
                            NonFungibleGlobalId::new(nft_resource, nft_id);
                        nft_record.push(global_ar)
                    }
                } else {
                    let nft_id = i.as_non_fungible().non_fungible_local_id();
                    let nft_global = NonFungibleGlobalId::new(nft_resource, nft_id);
                    nft_record.push(nft_global)
                }
            }

            assert!(
                self.expected_nfts
                    .iter()
                    .all(|item| nft_record.contains(item)),
                "no match: {:?} vs {:?}",
                nft_record,
                self.expected_nfts
            );

            for bucket in b_nft_assets.into_iter() {
                self.partner_vaults
                    .entry(bucket.resource_address())
                    .or_insert_with(|| Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            self.nfts_validated = true
        }

        pub fn partner_deposit_tokens(
            &mut self,
            b_token_assets: Vec<Bucket>,
            b_badge: Option<Proof>,
        ) {
            assert!(!self.swapped, "Swap has occurred already occured");
            // if swap required badge
            if self.private {
                // check badge has been passed
                assert!(b_badge.is_some(), "Badge required");

                // check passes badge is valid
                if b_badge.is_some() {
                    let badge_to_validate = b_badge.unwrap();
                    let badge_ref = badge_to_validate.clone();

                    let validation_resource = self.badge_partner.unwrap();

                    let badge_local_to_validate = badge_ref.check(validation_resource);

                    let badge_proof_local = badge_local_to_validate
                        .as_non_fungible()
                        .non_fungible_local_id();
                    let validation_local = self.badge_partner_local.as_ref().unwrap();
                    assert!(&badge_proof_local == validation_local, "Incorrect badge")
                }
            }

            let token_criteria = self.expected_tokens.clone();
            let mut b_deposit_hm: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for bucket in b_token_assets.iter() {
                b_deposit_hm.insert(bucket.resource_address(), bucket.amount());
            }

            assert!(b_deposit_hm == token_criteria, "Token deposits don't match");

            for bucket in b_token_assets.into_iter() {
                self.partner_vaults
                    .entry(bucket.resource_address())
                    .or_insert_with(|| Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            self.tokens_validated = true
        }

        /// Of note - There is a possibility that someone could snipe the offered assets if the partner was to just
        /// manually send the right assets and not claim the new assets all in the same transaction. Couldn't happen if the swap
        /// is set to private. However, if we're talking atomic transactions and you do it right, there's no issue.
        /// Only an issue if people try to fool around with dev console submitting things wrong, but then the risk is on then imo XD

        pub fn partner_claims_creator_assets(&mut self, b_badge: Option<Proof>) -> Vec<Bucket> {
            assert!(!self.swapped, "Swap has occurred already occured");
            // if swap required badge
            if self.private {
                // check badge has been passed
                assert!(b_badge.is_some(), "Badge required");

                // check passes badge is valid
                if b_badge.is_some() {
                    let badge_to_validate = b_badge.unwrap();
                    let badge_ref = badge_to_validate.clone();

                    let validation_resource = self.badge_partner.unwrap();

                    let badge_local_to_validate = badge_ref.check(validation_resource);

                    let badge_proof_local = badge_local_to_validate
                        .as_non_fungible()
                        .non_fungible_local_id();
                    let validation_local = self.badge_partner_local.as_ref().unwrap();
                    assert!(&badge_proof_local == validation_local, "Incorrect badge")
                }
            }

            if self.expected_nfts.len() > 0 {
                assert!(
                    self.nfts_validated,
                    "Insufficient assets deposited for trade"
                )
            };
            if self.expected_tokens.len() > 0 {
                assert!(
                    self.tokens_validated,
                    "Insufficient assets deposited for trade"
                )
            };

            let a_assets: Vec<ResourceAddress> = self.creator_vaults.keys().cloned().collect();

            let mut buckets: Vec<Bucket> = Vec::new();

            for resource_address in a_assets.into_iter() {
                buckets.push(
                    self.creator_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }
            self.swapped = true;

            return buckets;
        }

        pub fn partner_cancel(&mut self, b_badge: Option<Proof>) -> Vec<Bucket> {
            assert!(!self.swapped, "Swap has occurred already occured");
            if self.private {
                // check badge has been passed
                assert!(b_badge.is_some(), "Badge required");

                // check passes badge is valid
                if b_badge.is_some() {
                    let badge_to_validate = b_badge.unwrap();
                    let badge_ref = badge_to_validate.clone();

                    let validation_resource = self.badge_partner.unwrap();

                    let badge_local_to_validate = badge_ref.check(validation_resource);

                    let badge_proof_local = badge_local_to_validate
                        .as_non_fungible()
                        .non_fungible_local_id();
                    let validation_local = self.badge_partner_local.as_ref().unwrap();
                    assert!(&badge_proof_local == validation_local, "Incorrect badge")
                }
            }

            let b_assets: Vec<ResourceAddress> = self.partner_vaults.keys().cloned().collect();

            let mut buckets: Vec<Bucket> = Vec::new();

            for resource_address in b_assets.into_iter() {
                buckets.push(
                    self.partner_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return buckets;
        }

        pub fn creator_cancel(&mut self) -> Vec<Bucket> {
            assert!(!self.swapped, "Swap has occurred");

            // update state such that swap has occurred - ie. block deposits.
            self.swapped = true;
            let a_assets: Vec<ResourceAddress> = self.creator_vaults.keys().cloned().collect();

            let mut buckets: Vec<Bucket> = Vec::new();

            for resource_address in a_assets.into_iter() {
                buckets.push(
                    self.creator_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }
            self.swapped = true;
            return buckets;
        }

        pub fn creator_claims_partner_assets(&mut self) -> Vec<Bucket> {
            assert!(self.swapped, "Swap hasn't occurred yet");

            let b_assets: Vec<ResourceAddress> = self.partner_vaults.keys().cloned().collect();

            let mut buckets: Vec<Bucket> = Vec::new();

            for resource_address in b_assets.into_iter() {
                buckets.push(
                    self.partner_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return buckets;
        }

        // after accepted
        pub fn burn_partner_badge(&mut self, burn_token: Bucket) {
            assert!(
                burn_token.resource_address() == self.badge_partner.unwrap(),
                "invalid key"
            );
            assert!(
                &burn_token.as_non_fungible().non_fungible_local_id()
                    == self.badge_partner_local.as_ref().unwrap(),
                "invalid key"
            );
            let resource_manager: ResourceManager = self.proposal_resource_manager;
            resource_manager.burn(burn_token);
        }

        pub fn burn_creator_badge(&mut self, burn_token: Bucket) {
            assert!(
                burn_token.as_non_fungible().non_fungible_local_id() == self.a_vault_key_id,
                "invalid key"
            );
            assert!(
                burn_token.resource_address() == self.a_vault_key,
                "invalid key"
            );

            let resource_manager: ResourceManager = self.proposal_resource_manager;
            resource_manager.burn(burn_token);

          
        }
    }
}
