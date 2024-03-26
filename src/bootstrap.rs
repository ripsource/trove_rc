use scrypto::prelude::*;

#[blueprint]
mod bootstrap {
    /// This is a bootstrap struct which creates all of the resources which we need to use to test the NFT marketplace.
    struct Bootstrap {
        nft1: ResourceAddress,
        nft2: ResourceAddress,
        nft1_ids: IndexSet<NonFungibleLocalId>,
        nft2_ids: IndexSet<NonFungibleLocalId>,
    }

    impl Bootstrap {
        /// Creates a number of NFT collections used for testing of the NFT marketplace blueprints.
        pub fn bootstrap() -> (Global<Bootstrap>, Vec<NonFungibleBucket>) {
            // Creating the resources used for our non-fungible tokens

            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Bootstrap::blueprint_id());

            let cars = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Cars NFT".to_owned(), locked;
                        "description" => "An NFT of the fastest cars known to mankind!".to_owned(), locked;
                        "symbol" => "FAST".to_owned(), locked;
                    }
                ))
                .mint_initial_supply([
                    Car {
                        name: "Camry".to_string(),
                        manufacturer: "Toyota".to_string(),
                    },
                    Car {
                        name: "Altima".to_string(),
                        manufacturer: "Nissan".to_string(),
                    },
                    Car {
                        // Any raptor lovers?
                        name: "Raptor".to_string(),
                        manufacturer: "Ford".to_string(),
                    },
                    Car {
                        name: "Yukon".to_string(),
                        manufacturer: "GMC".to_string(),
                    },
                ]);

            let phones = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Phones NFT".to_owned(), locked;
                        "description" => "Do you really want me to describe to you what a phone is?".to_owned(), locked;
                        "symbol" => "PHONE".to_owned(), locked;
                    }
                ))
                .mint_initial_supply([
                    Phone {
                        name: "iPhone".to_string(),
                        manufacturer: "Apple".to_string(),
                    },
                    Phone {
                        name: "Galaxy".to_string(),
                        manufacturer: "Samsung".to_string(),
                    },
                    Phone {
                        name: "Pixel".to_string(),
                        manufacturer: "Google".to_string(),
                    },
                    Phone {
                        name: "P50".to_string(),
                        manufacturer: "Huawei".to_string(),
                    },
                ]);

            let laptops = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Laptops NFT".to_owned(), locked;
                        "description" => 
                        "Do you really want me to describe to you what a laptop is? I'm a bit concerned...".to_owned(), locked;
                        "symbol" => "LTOP".to_owned(), locked;
                    }
                ))
                .mint_initial_supply([
                    Laptop {
                        name: "MacBook".to_string(),
                        manufacturer: "Apple".to_string()
                    },
                    Laptop {
                        name: "Thinkpad".to_string(),
                        manufacturer: "Lenovo".to_string()
                    },
                    Laptop {
                        name: "Surface".to_string(),
                        manufacturer: "Microsoft".to_string()
                    },
                    Laptop {
                        name: "Chromebook".to_string(),
                        manufacturer: "Google".to_string()
                    }
                ]);
            let ids1 = cars.as_non_fungible().non_fungible_local_ids();
            let ids2 = phones.as_non_fungible().non_fungible_local_ids();

            let barter_component = Self {
                nft1: cars.resource_address(),
                nft2: phones.resource_address(),
                nft1_ids: ids1,
                nft2_ids: ids2,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize();
            // With all of the NFTs created, we can now return the buckets of tokens
            (barter_component, vec![cars, phones, laptops])
        }

        pub fn local_ids_1(&mut self) -> (ResourceAddress, Vec<NonFungibleLocalId>) {
            let mut new_thing = Vec::new();

            for id in self.nft1_ids.clone().into_iter() {
                new_thing.push(id)
            }

            (self.nft1.clone(), new_thing)
        }

        pub fn local_ids_2(&mut self) -> (ResourceAddress, Vec<NonFungibleLocalId>) {
            let mut new_thing = Vec::new();

            for id in self.nft2_ids.clone().into_iter() {
                new_thing.push(id)
            }

            (self.nft2.clone(), new_thing)
        }
    }
}

#[derive(NonFungibleData, ScryptoSbor)]
struct Car {
    name: String,
    manufacturer: String,
}

#[derive(NonFungibleData, ScryptoSbor)]
struct Phone {
    name: String,
    manufacturer: String,
}

#[derive(NonFungibleData, ScryptoSbor)]
struct Laptop {
    name: String,
    manufacturer: String,
}
