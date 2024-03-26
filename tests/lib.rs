use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

// Contains three functions, each with transcations that are either expected to pass or fail.
// Test 1: test_basic_swap - 
// ---- account 1 creates swap 
// ---- account 2 accepts
// ---- account 2 attempts to cancel swap after its accepted || failure
// ---- account 1 attempts to cancel swap after its accepted || failure
// ---- account 1 completes swap, collecting their new assets

// Test 2: test_partner_badge_basic_swap -
// ---- account 1 creates swap with partner address
// ---- account 2 attempts to accept swap without badge || failure
// ---- account 2 accepts with badge 
// ---- account 2 attempts to cancel swap after its accepted || failure
// ---- account 1 attempts to cancel swap after its accepted || failure
// ---- account 1 completes swap, collecting their new assets

// Test 3: cancel_swap_test -
// ---- account 1 creates swap
// ---- account 1 attempts to cancel swap without badge || failure
// ---- account 1 cancels swap with badge
// ---- account 2 attempts to do swap after its been cancelled || failure

#[test]
fn test_basic_swap() {
    // Setup the environment
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    // Create an account 2
    let (public_key2, _private_key2, account_component2) = test_runner.new_allocated_account();
    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // supply account 1 with NFTs
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // get the component address of the bootstrap
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // getter method for ids
    let manifest = ManifestBuilder::new()
        .call_method(component, "local_ids_1", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    let firstrs: (ResourceAddress, Vec<NonFungibleLocalId>) = receipt.expect_commit(true).output(1);


    // supply account 2 with NFTs
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
        .call_method(
            account_component2,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    // get component address of bootstrap
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // getter method for ids
    let manifest = ManifestBuilder::new()
        .call_method(component, "local_ids_1", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    let secondrs: (ResourceAddress, Vec<NonFungibleLocalId>) =
        receipt.expect_commit(true).output(1);

    // creating global ids and hashmaps to be supplied as what is requested in the swap - could do with some tidying/was experimenting a bit
    let second_id1 = secondrs.1[0].clone();
    let second_id2 = secondrs.1[1].clone();
    let global_1 = NonFungibleGlobalId::new(secondrs.0, second_id1);
    let global_2 = NonFungibleGlobalId::new(secondrs.0, second_id2);
    let locallist2 = secondrs.1.clone();
    let locallist = firstrs.1.clone();
    let btree1: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(firstrs.1);
    let btree2: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(secondrs.1);
    let none_hashmap_fungibles: Option<HashMap<ResourceAddress, Decimal>> = None as Option<HashMap<ResourceAddress, Decimal>>;
    let request_global = vec![global_1, global_2];
    let request: Option<Vec<NonFungibleGlobalId>> = Some(request_global);
    let partner_option = None as Option<ComponentAddress>;
    let partner_badge = None as Option<ManifestProof>;
    let partner_badge2 = None as Option<ManifestProof>;

    // NFT set up complete


    // account 1 creates swap - trade of NFTs + tokens for NFTs | non private, so no partner address included

    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "withdraw_non_fungibles",
            manifest_args!(firstrs.0, locallist),
        )
        .call_method(account_component, "withdraw", manifest_args!(
            XRD,
            dec!(1000)
        ))
        .take_non_fungibles_from_worktop(firstrs.0, btree1, "bucket1")
        .take_from_worktop(XRD, dec!(1000), "bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Barter",
                "new_trade_proposal",
                manifest_args!(
                    "My new trade!", // String name
                    partner_option,
                    Some(vec![lookup.bucket("bucket2")]),
                    Some(vec![lookup.bucket("bucket1")]),
                    request.clone(),
                    none_hashmap_fungibles.clone()
                ),
            )
        })
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt_new_trade_proposal = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt_new_trade_proposal.expect_commit_success();

let blank_tokens = None as Option<Vec<ManifestBucket>>;
let blank_nfts = None as Option<Vec<ManifestBucket>>;

// try create trade without anything to offer 

let manifest = ManifestBuilder::new()
.call_function(
        package_address,
        "Barter",
        "new_trade_proposal",
        manifest_args!(
            "My new trade!", // String name
            partner_option,
            blank_tokens,
            blank_nfts,
            request,
            none_hashmap_fungibles
        ),
    )
.call_method(
    account_component,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_new_bad_trade_proposal = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
receipt_new_bad_trade_proposal.expect_commit_failure();



    // get component of the swap
    let component = receipt_new_trade_proposal
    .expect_commit(true).new_component_addresses()[0];


// get badge address of swap creator
let manifest = ManifestBuilder::new()
.call_method(component, "get_badge", manifest_args!()).build();
let receipty = test_runner.execute_manifest_ignoring_fee(
    manifest,
    vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipty.expect_commit_success();
    let output: (ResourceAddress, NonFungibleLocalId, NonFungibleGlobalId, Option<ResourceAddress>, Option<NonFungibleLocalId>) =
    receipty.expect_commit(true).output(1);
    let localid_badge = output.1.clone();
    let localid_badge2 = localid_badge.clone();
    let globalid_badge: NonFungibleGlobalId = output.2.clone();
    let localtry = globalid_badge.local_id();
    let rebadge = globalid_badge.resource_address();
    let btrbadge: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(vec![output.1]);


// account 2 sends NFTs requested and receives assets

let manifest = ManifestBuilder::new()

.call_method(
    account_component2,
    "withdraw_non_fungibles",
    manifest_args!(secondrs.0, locallist2),
)

.take_non_fungibles_from_worktop(secondrs.0, btree2, "bucket1")
.with_name_lookup(|builder, lookup| {
    builder.call_method(
        component,
        "partner_deposit_nfts",
        manifest_args!(vec![lookup.bucket("bucket1")], partner_badge)
    )
})
.call_method(component, "partner_claims_creator_assets", manifest_args!(partner_badge2))
.call_method(
    account_component2,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_accept.expect_commit_success();


// account 2 attempts to cancel and withdraw their assets after swap | expect failure

let manifest = ManifestBuilder::new()
.call_method(
    account_component2,
    "partner_cancel",
    manifest_args!(),
)
.call_method(
    account_component2,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_cancel = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_cancel.expect_commit_failure();


// account 1 attempts to cancel swap after it has occured | expect failure

let manifest = ManifestBuilder::new()
.call_method(account_component, "create_proof_of_non_fungibles", manifest_args!(
    rebadge,
    vec![localtry]
))
.call_method(
        component,
        "creator_cancel",
        manifest_args!(),
    )
.call_method(
    account_component,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let a_claim_early = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
a_claim_early.expect_commit_failure();



// account 1 completes the swap, claiming their requested assets

let manifest = ManifestBuilder::new()
.call_method(account_component, "create_proof_of_non_fungibles", manifest_args!(
    rebadge,
    vec![localtry]
))
.call_method(
        component,
        "creator_claims_partner_assets",
        manifest_args!(),
    )
    .pop_from_auth_zone("new_proof")

.with_name_lookup(|builder, lookup | {builder.drop_proof(lookup.proof("new_proof"))})
.call_method(account_component, "withdraw_non_fungibles", manifest_args!(output.0, vec![localid_badge2]
))
.take_non_fungibles_from_worktop(output.0, btrbadge, "bucket1")
.with_name_lookup(|builder, lookup| {
    builder.call_method(
        component,
        "burn_creator_badge",
        manifest_args!(lookup.bucket("bucket1"))
    )
    })
.call_method(
    account_component,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let a_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
a_accept.expect_commit_success();

}


#[test]
fn test_partner_badge_basic_swap() {
    // Setup the environment
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    // Create an account 2
    let (public_key2, _private_key2, account_component2) = test_runner.new_allocated_account();
    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // account 1 NFTs
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .call_method(component, "local_ids_1", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );
    let firstrs: (ResourceAddress, Vec<NonFungibleLocalId>) = receipt.expect_commit(true).output(1);
    let _first_id1 = firstrs.1[0].clone();
   

    //account 2 NFTs

    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
        .call_method(
            account_component2,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .call_method(component, "local_ids_1", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    let secondrs: (ResourceAddress, Vec<NonFungibleLocalId>) =
        receipt.expect_commit(true).output(1);
    let second_id1 = secondrs.1[0].clone();
    let second_id2 = secondrs.1[1].clone();
    let global_1 = NonFungibleGlobalId::new(secondrs.0, second_id1);
    let global_2 = NonFungibleGlobalId::new(secondrs.0, second_id2);
    let locallist2 = secondrs.1.clone();
    let locallist = firstrs.1.clone();
    let btree1: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(firstrs.1);
    let btree2: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(secondrs.1.clone());
    let btree3: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(secondrs.1.clone());
    let none_hashmap_fungibles: Option<HashMap<ResourceAddress, Decimal>> = None as Option<HashMap<ResourceAddress, Decimal>>;
    let request_global = vec![global_1, global_2];
    let request: Option<Vec<NonFungibleGlobalId>> = Some(request_global);

    // NFT set up complete


    // swap created with partner badge

    let manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            "withdraw_non_fungibles",
            manifest_args!(firstrs.0, locallist),
        )
        .call_method(account_component, "withdraw", manifest_args!(
            XRD,
            dec!(1000)
        ))
        .take_non_fungibles_from_worktop(firstrs.0, btree1, "bucket1")
        .take_from_worktop(XRD, dec!(1000), "bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "Barter",
                "new_trade_proposal",
                manifest_args!(
                    "My new trade!", // String name
                    Some(account_component2),
                    Some(vec![lookup.bucket("bucket2")]),
                    Some(vec![lookup.bucket("bucket1")]),
                    request,
                    none_hashmap_fungibles
                ),
            )
        })
        
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt_new_trade_proposal = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt_new_trade_proposal);
    receipt_new_trade_proposal.expect_commit_success();

    let component = receipt_new_trade_proposal
    .expect_commit(true).new_component_addresses()[0];


// get badge address 
let manifest = ManifestBuilder::new()
.call_method(component, "get_badge", manifest_args!()).build();
let receipty = test_runner.execute_manifest_ignoring_fee(
    manifest,
    vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipty.expect_commit_success();
    let output: (ResourceAddress, NonFungibleLocalId, NonFungibleGlobalId, Option<ResourceAddress>, Option<NonFungibleLocalId>) =
    receipty.expect_commit(true).output(1);
    let localid_badge = output.1.clone();
    let localid_badge2 = localid_badge.clone();
    let globalid_badge: NonFungibleGlobalId = output.2.clone();
    let partner_badge = output.3.unwrap();
    let partner_badge_local = output.4.unwrap();
    println!("partner badge {:?}\n", partner_badge);
    println!("partner local badge {:?}\n", partner_badge_local);
    let localtry = globalid_badge.local_id();
    let rebadge = globalid_badge.resource_address();
    let btrbadge: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(vec![output.1]);

// account 2 does not use badge to send and receive assets || expect failure

let manifest = ManifestBuilder::new()
.call_method(
    account_component2,
    "withdraw_non_fungibles",
    manifest_args!(secondrs.0.clone(), locallist2.clone()),
)
.take_non_fungibles_from_worktop(secondrs.0, btree2, "bucket1")
.call_method(
        component,
        "partner_deposit_nfts",
        manifest_args!()
    )
.call_method(component, "partner_claims_creator_assets", manifest_args!())
.call_method(
    account_component2,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_accept.expect_commit_failure();



// account 2 uses badge to send and receive assets

let manifest = ManifestBuilder::new()
.call_method(account_component2, "create_proof_of_non_fungibles", manifest_args!(
    partner_badge.clone(),
    vec![partner_badge_local.clone()]
))
.pop_from_auth_zone("partner_proof")

.call_method(
    account_component2,
    "withdraw_non_fungibles",
    manifest_args!(secondrs.0.clone(), locallist2.clone()),
)
.take_non_fungibles_from_worktop(secondrs.0, btree3, "bucket1")
.with_name_lookup(|builder, lookup| {
    builder.call_method(
        component,
        "partner_deposit_nfts",
        manifest_args!(vec![lookup.bucket("bucket1")], Some(lookup.proof("partner_proof")))
    )
})
.call_method(account_component2, "create_proof_of_non_fungibles", manifest_args!(
    partner_badge.clone(),
    vec![partner_badge_local.clone()]
))
.pop_from_auth_zone("partner_proof2")

.with_name_lookup(|builder, lookup| {
    builder.call_method(component, "partner_claims_creator_assets", manifest_args!(Some(lookup.proof("partner_proof2"))))
})
.call_method(
    account_component2,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_accept.expect_commit_success();


// account 2 attemps to withdraw after swap | expect failure

let manifest = ManifestBuilder::new()
.call_method(account_component2, "create_proof_of_non_fungibles", manifest_args!(
    partner_badge.clone(),
    vec![partner_badge_local.clone()]
))
.pop_from_auth_zone("partner_proof")
.with_name_lookup(|builder, lookup| {
    builder.call_method(
    account_component2,
    "partner_cancel",
    manifest_args!(lookup.proof("partner_proof")),
)})
.call_method(
    account_component2,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_cancel = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_cancel.expect_commit_failure();


// account 1 attempts to withdraw after swap | expect failure

let manifest = ManifestBuilder::new()
.call_method(account_component, "create_proof_of_non_fungibles", manifest_args!(
    rebadge,
    vec![localtry]
))
.call_method(
        component,
        "creator_cancel",
        manifest_args!(),
    )
.call_method(
    account_component,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let a_claim_early = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
a_claim_early.expect_commit_failure();



// account 1 completes swap, claiming their requested assets

let manifest = ManifestBuilder::new()
.call_method(account_component, "create_proof_of_non_fungibles", manifest_args!(
    rebadge,
    vec![localtry]
))
.call_method(
        component,
        "creator_claims_partner_assets",
        manifest_args!(),
    )
    .pop_from_auth_zone("new_proof")

.with_name_lookup(|builder, lookup | {builder.drop_proof(lookup.proof("new_proof"))})
.call_method(account_component, "withdraw_non_fungibles", manifest_args!(output.0, vec![localid_badge2]
))
.take_non_fungibles_from_worktop(output.0, btrbadge, "bucket1")
.with_name_lookup(|builder, lookup| {
    builder.call_method(
        component,
        "burn_creator_badge",
        manifest_args!(lookup.bucket("bucket1"))
    )
    })
.call_method(
    account_component,
    "deposit_batch",
    manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let a_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
a_accept.expect_commit_success();

}

#[test]
fn cancel_swap_test () {
  // Setup the environment
  let mut test_runner = TestRunnerBuilder::new().without_trace().build();
  // Create an account
  let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
  // Create an account 2
  let (public_key2, _private_key2, account_component2) = test_runner.new_allocated_account();
  // Publish package
  let package_address = test_runner.compile_and_publish(this_package!());


  // account 1 NFTs

  let manifest = ManifestBuilder::new()
      .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
      .call_method(
          account_component,
          "deposit_batch",
          manifest_args!(ManifestExpression::EntireWorktop),
      )
      .build();
  let receipt = test_runner.execute_manifest_ignoring_fee(
      manifest,
      vec![NonFungibleGlobalId::from_public_key(&public_key)],
  );
  receipt.expect_commit_success();
  let component = receipt.expect_commit(true).new_component_addresses()[0];
  let manifest = ManifestBuilder::new()
      .call_method(component, "local_ids_1", manifest_args!())
      .build();
  let receipt = test_runner.execute_manifest_ignoring_fee(
      manifest,
      vec![NonFungibleGlobalId::from_public_key(&public_key2)],
  );

  let firstrs: (ResourceAddress, Vec<NonFungibleLocalId>) = receipt.expect_commit(true).output(1);
  let _first_id1 = firstrs.1[0].clone();


  let manifest = ManifestBuilder::new()
      .call_function(package_address, "Bootstrap", "bootstrap", manifest_args!())
      .call_method(
          account_component2,
          "deposit_batch",
          manifest_args!(ManifestExpression::EntireWorktop),
      )
      .build();
  let receipt = test_runner.execute_manifest_ignoring_fee(
      manifest,
      vec![NonFungibleGlobalId::from_public_key(&public_key2)],
  );
  let component = receipt.expect_commit(true).new_component_addresses()[0];
  let manifest = ManifestBuilder::new()
      .call_method(component, "local_ids_1", manifest_args!())
      .build();
  let receipt = test_runner.execute_manifest_ignoring_fee(
      manifest,
      vec![NonFungibleGlobalId::from_public_key(&public_key2)],
  );
 
  let secondrs: (ResourceAddress, Vec<NonFungibleLocalId>) =
      receipt.expect_commit(true).output(1);
  let second_id1 = secondrs.1[0].clone();
  let second_id2 = secondrs.1[1].clone();
  let global_1 = NonFungibleGlobalId::new(secondrs.0, second_id1);
  let global_2 = NonFungibleGlobalId::new(secondrs.0, second_id2);
  let locallist2 = secondrs.1.clone();
  let locallist = firstrs.1.clone();
  let btree1: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(firstrs.1);
  let btree2: BTreeSet<NonFungibleLocalId> = FromIterator::from_iter(secondrs.1);
  let none_hashmap_fungibles: Option<HashMap<ResourceAddress, Decimal>> = None as Option<HashMap<ResourceAddress, Decimal>>;
  let request_global = vec![global_1, global_2];
  let request: Option<Vec<NonFungibleGlobalId>> = Some(request_global);
  let partner_option = None as Option<ComponentAddress>;
  let partner_badge = None as Option<ManifestProof>;
  let partner_badge2 = None as Option<ManifestProof>;

  // NFT set up complete

  // account 1 creates swap, no partner

  let manifest = ManifestBuilder::new()
      .call_method(
          account_component,
          "withdraw_non_fungibles",
          manifest_args!(firstrs.0, locallist),
      )
      .call_method(account_component, "withdraw", manifest_args!(
          XRD,
          dec!(1000)
      ))
      .take_non_fungibles_from_worktop(firstrs.0, btree1, "bucket1")
      .take_from_worktop(XRD, dec!(1000), "bucket2")
      .with_name_lookup(|builder, lookup| {
          builder.call_function(
              package_address,
              "Barter",
              "new_trade_proposal",
              manifest_args!(
                  "My new trade!", // String name
                  partner_option,
                  Some(vec![lookup.bucket("bucket2")]),
                  Some(vec![lookup.bucket("bucket1")]),
                  request,
                  none_hashmap_fungibles
              ),
          )
      })
      .call_method(
          account_component,
          "deposit_batch",
          manifest_args!(ManifestExpression::EntireWorktop),
      )
      .build();
  let receipt_new_trade_proposal = test_runner.execute_manifest_ignoring_fee(
      manifest,
      vec![NonFungibleGlobalId::from_public_key(&public_key)],
  );
  receipt_new_trade_proposal.expect_commit_success();

  let component = receipt_new_trade_proposal
  .expect_commit(true).new_component_addresses()[0];


// attempt cancel without badge

  let manifest = ManifestBuilder::new()
  .call_method(component, "creator_cancel", manifest_args!())
  .call_method(account_component, "deposit_batch", manifest_args!(ManifestExpression::EntireWorktop))
  .build();
let receipt_new_trade_proposal = test_runner.execute_manifest_ignoring_fee(
    manifest,
    vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
receipt_new_trade_proposal.expect_commit_failure();
  

// get badge address 
let manifest = ManifestBuilder::new()
.call_method(component, "get_badge", manifest_args!()).build();
let receipty = test_runner.execute_manifest_ignoring_fee(
  manifest,
  vec![NonFungibleGlobalId::from_public_key(&public_key)],
  );
  receipty.expect_commit_success();

  let output: (ResourceAddress, NonFungibleLocalId, NonFungibleGlobalId, Option<ResourceAddress>, Option<NonFungibleLocalId>) =
  receipty.expect_commit(true).output(1);
  let globalid_badge: NonFungibleGlobalId = output.2.clone();
  let localtry = globalid_badge.local_id();
  let rebadge = globalid_badge.resource_address();


// account 1 cancels swap with badge

  let manifest = ManifestBuilder::new()
  .call_method(account_component, "create_proof_of_non_fungibles", manifest_args!(
    rebadge,
    vec![localtry]
  ))
  .call_method(component, "creator_cancel", manifest_args!())
  .call_method(account_component, "deposit_batch", manifest_args!(ManifestExpression::EntireWorktop))
  .build();
let receipt_new_trade_proposal = test_runner.execute_manifest_ignoring_fee(
    manifest,
    vec![NonFungibleGlobalId::from_public_key(&public_key)],
);
receipt_new_trade_proposal.expect_commit_success();

// account 2 attempts to do swap after its been cancelled | expect failure

let manifest = ManifestBuilder::new()
.call_method(
  account_component2,
  "withdraw_non_fungibles",
  manifest_args!(secondrs.0, locallist2),
)
.take_non_fungibles_from_worktop(secondrs.0, btree2, "bucket1")
.with_name_lookup(|builder, lookup| {
  builder.call_method(
      component,
      "partner_deposit_nfts",
      manifest_args!(vec![lookup.bucket("bucket1")], partner_badge)
  )
})
.call_method(component, "partner_claims_creator_assets", manifest_args!(partner_badge2))
.call_method(
  account_component2,
  "deposit_batch",
  manifest_args!(ManifestExpression::EntireWorktop),
)
.build();
let receipt_b_accept = test_runner.execute_manifest_ignoring_fee(
manifest,
vec![NonFungibleGlobalId::from_public_key(&public_key2)],
);
receipt_b_accept.expect_commit_failure();


}
