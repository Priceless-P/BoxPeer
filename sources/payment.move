// module CDN::ContentRegistry {
//     use std::signer;
//     use std::string;
//     use std::vector;
//     use aptos_framework::coin;
//     use aptos_framework::aptos_coin::AptosCoin;

//     // Error Codes
//     const ECONTENT_NOT_FOUND: u64 = 1000;
//     const EINSUFFICIENT_FUNDS: u64 = 1001;
//     const ENODES_NOT_REGISTERED: u64 = 1002;
//     const EINVALID_BILLING_PERIOD: u64 = 1003;

//     // Billing Period Constants (in months)
//     const BILLING_MONTHLY: u8 = 1;
//     const BILLING_QUARTERLY: u8 = 3;
//     const BILLING_BI_ANNUAL: u8 = 6;
//     const BILLING_YEARLY: u8 = 12;

//     // Fee Constants
//     const FEE_PER_GB: u64 = 8500; // micro-APT per GB

//     // Event Definitions
//     struct ContentUploaded has copy, drop {
//         content_owner: address,
//         content_hash: string::String,
//     }

//     struct ContentPurchased has copy, drop {
//         buyer: address,
//         content_owner: address,
//         content_hash: string::String,
//     }

//     struct ContentOwnershipVerified has copy, drop {
//         content_owner: address,
//         content_hash: string::String,
//         verified_owner: bool,
//     }

//     struct FeeDistributed has copy, drop {
//         content_owner: address,
//         fee_amount: u64, // micro-APT
//     }

//     struct NodeRegistered has copy, drop {
//         node_address: address,
//     }

//     // Holds content metadata and ownership
//     struct Content has store, copy {
//         owner: address,
//         title: string::String,
//         content_type: string::String,
//         content_hash: string::String,
//         size: u64, // in bytes
//         billing_period: u8, // in months
//         next_billing_month: u64, // timestamp or block number (implementation-dependent)
//     }

//     // Holds a collection of contents and nodes
//     struct ContentRegistry has key {
//         contents: vector<Content>,
//         content_uploaded_events: vector<ContentUploaded>,
//         content_purchased_events: vector<ContentPurchased>,
//         content_ownership_verified_events: vector<ContentOwnershipVerified>,
//         fee_distributed_events: vector<FeeDistributed>,
//         node_registered_events: vector<NodeRegistered>,
//         nodes: vector<address>, // Registered nodes
//     }

//     // Initializes a ContentRegistry for an account
//     public entry fun init_registry(account: &signer) {
//         move_to(account, ContentRegistry {
//             contents: vector::empty<Content>(),
//             content_uploaded_events: vector::empty<ContentUploaded>(),
//             content_purchased_events: vector::empty<ContentPurchased>(),
//             content_ownership_verified_events: vector::empty<ContentOwnershipVerified>(),
//             fee_distributed_events: vector::empty<FeeDistributed>(),
//             node_registered_events: vector::empty<NodeRegistered>(),
//             nodes: vector::empty<address>(),
//         });
//     }

//     // Node Registration
//     public entry fun register_node(account: &signer) acquires ContentRegistry {
//         let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));

//         // Check if node is already registered
//         let len = vector::length(&registry.nodes);
//         let mut i = 0;
//         while (i < len) {
//         let node = vector::borrow(&registry.nodes, i);
//         if (*node == signer::address_of(account)) {
//         // Node already registered; exit
//         return;
//         };
//         i = i + 1;
//         }

//         // Register the node
//         vector::push_back(&mut registry.nodes, signer::address_of(account));

//         // Emit event
//         let event = NodeRegistered {
//         node_address: signer::address_of(account),
//         };
//         vector::push_back(&mut registry.node_registered_events, event);
//     }

//     // Upload Content with Fee Payment
//     public entry fun upload_content(
//         account: &signer,
//         title: string::String,
//         content_type: string::String,
//         content_hash: string::String,
//         size: u64, // in bytes
//         billing_period: u8, // in months: 1, 3, 6, 12
//     ) acquires ContentRegistry {
//         // Validate billing period
//         let valid_billing = billing_period == BILLING_MONTHLY
//             || billing_period == BILLING_QUARTERLY
//             || billing_period == BILLING_BI_ANNUAL
//             || billing_period == BILLING_YEARLY;
//         assert!(valid_billing, EINVALID_BILLING_PERIOD);

//         let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));

//         // Calculate fee based on size and billing period
//         let size_in_gb = (size + 999_999_999) / 1_000_000_000; // Ceiling division for partial GB
//         let fee = size_in_gb * FEE_PER_GB * billing_period as u64;

//         // Check if there are registered nodes
//         let num_nodes = vector::length(&registry.nodes);
//         assert!(num_nodes > 0, ENODES_NOT_REGISTERED);

//         // Transfer fee from content owner to contract account
//         // Note: Replace `contract_address` with your contract's address
//         let contract_address = signer::address_of(account); // Assuming contract is owned by the registry account
//         coin::transfer<AptosCoin>(account, contract_address, fee);

//         // Create new content entry
//         let new_content = Content {
//             owner: signer::address_of(account),
//             title,
//             content_type,
//             content_hash,
//             size,
//             billing_period,
//             next_billing_month: current_month(), // Implement `current_month` based on your time tracking
//         };

//         vector::push_back(&mut registry.contents, new_content);

//         // Emit ContentUploaded event
//         let upload_event = ContentUploaded {
//             content_owner: signer::address_of(account),
//             content_hash,
//         };
//         vector::push_back(&mut registry.content_uploaded_events, upload_event);

//         // Distribute fee to nodes
//         distribute_fee(registry, fee);
//     }

//     // Function to Distribute Fee to Nodes
//     fun distribute_fee(registry: &mut ContentRegistry, fee: u64) {
//         let num_nodes = vector::length(&registry.nodes);
//         assert!(num_nodes > 0, ENODES_NOT_REGISTERED);

//         let fee_per_node = fee / num_nodes;
//         let remainder = fee % num_nodes; // Handle any remaining fee

//         let mut i = 0;
//         while (i < num_nodes) {
//         let node = vector::borrow(&registry.nodes, i);
//         // Transfer fee_per_node to the node
//         // Assuming the contract holds the funds and can transfer to nodes
//         coin::transfer<AptosCoin>(&self, *node, fee_per_node); // Replace `&self` with appropriate signer if needed

//         i = i + 1;
//         }

//         // Handle remainder (optional): Could be retained by the contract or distributed as needed

//         // Emit FeeDistributed event
//         let event = FeeDistributed {
//         content_owner: signer::address_of(&self), // Replace with actual content owner if tracked
//         fee_amount: fee,
//         };
//         vector::push_back(&mut registry.fee_distributed_events, event);
//     }

//     // Helper Function to Get Current Month
//     // Note: Move doesn't have built-in time functions. You'll need to manage this via block numbers or external triggers.
//     fun current_month(): u64 {
//         // Placeholder implementation
//         // Implement based on your blockchain's time tracking
//         0
//     }

//     // Verify the owner of a content
//     public entry fun verify_content_ownership(
//         owner: address,
//         content_hash: string::String
//     ) acquires ContentRegistry {
//         let registry = borrow_global<ContentRegistry>(owner);

//         let len = vector::length(&registry.contents);
//         let mut i = 0;

//         let mut found = false;

//         while (i < len) {
//         let content = vector::borrow(&registry.contents, i);
//         if (content.content_hash == content_hash) {
//         found = true;
//         vector::push_back(&mut registry.content_ownership_verified_events, ContentOwnershipVerified {
//         content_owner: owner,
//         content_hash,
//         verified_owner: true,
//         });
//         break;
//         };
//         i = i + 1;
//         };

//         if (!found) {
//         vector::push_back(&mut registry.content_ownership_verified_events, ContentOwnershipVerified {
//         content_owner: owner,
//         content_hash,
//         verified_owner: false,
//         });
//         assert!(found, ECONTENT_NOT_FOUND);
//         }
//     }

//     // Purchase content
//     public entry fun purchase_content(
//         buyer: &signer,
//         content_owner: address,
//         content_hash: string::String
//     ) acquires ContentRegistry {
//         let registry = borrow_global_mut<ContentRegistry>(content_owner);
//         let len = vector::length(&registry.contents);
//         let mut i = 0;

//         let mut found = false;

//         while (i < len) {
//         let content = vector::borrow(&registry.contents, i);
//         if (content.content_hash == content_hash) {
//         assert!(coin::value(&content.cost) > 0, EINSUFFICIENT_FUNDS);

//         // Transfer coins from buyer to content owner
//         coin::transfer<AptosCoin>(buyer, content_owner, coin::value(&content.cost));
//         found = true;

//         let purchase_event = ContentPurchased {
//         buyer: signer::address_of(buyer),
//         content_owner,
//         content_hash,
//         };
//         vector::push_back(&mut registry.content_purchased_events, purchase_event);
//         break;
//         };
//         i = i + 1;
//         };

//         assert!(found, ECONTENT_NOT_FOUND);
//     }

//     // Function to Renew Billing Period
//     public entry fun renew_billing(
//         account: &signer,
//         content_hash: string::String,
//         additional_months: u8, // 1, 3, 6, 12
//     ) acquires ContentRegistry {
//         // Validate billing period
//         let valid_billing = additional_months == BILLING_MONTHLY
//             || additional_months == BILLING_QUARTERLY
//             || additional_months == BILLING_BI_ANNUAL
//             || additional_months == BILLING_YEARLY;
//         assert!(valid_billing, EINVALID_BILLING_PERIOD);

//         let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));

//         let len = vector::length(&registry.contents);
//         let mut i = 0;
//         let mut found = false;
//         let mut fee = 0;

//         while (i < len) {
//         let content = vector::borrow_mut(&mut registry.contents, i);
//         if (content.content_hash == content_hash) {
//         // Calculate additional fee
//         let size_in_gb = (content.size + 999_999_999) / 1_000_000_000; // Ceiling division
//         fee = size_in_gb * FEE_PER_GB * additional_months as u64;

//         // Transfer fee from content owner to contract account
//         let contract_address = signer::address_of(account); // Assuming contract is owned by the registry account
//         coin::transfer<AptosCoin>(account, contract_address, fee);

//         // Update billing period and next billing month
//         content.billing_period += additional_months;
//         content.next_billing_month += additional_months as u64; // Implement proper time tracking

//         found = true;

//         // Emit FeeDistributed event after distribution
//         distribute_fee(registry, fee);

//         break;
//         };
//         i = i + 1;
//         };

//         assert!(found, ECONTENT_NOT_FOUND);
//     }
// }
