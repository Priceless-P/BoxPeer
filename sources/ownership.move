module BoxPeer_addr::BoxPeer {
    use std::signer;
    use std::string;
    use std::vector;
    use aptos_framework::coin;
    use aptos_framework::aptos_coin::AptosCoin;
    // use pyth::pyth;
    // use pyth::price_identifier;
    //
    // use pyth::price::{Self, Price};

    const USD_COST_PER_GB: u64 = 50;

    const CONTRACT_ADDRESS: address = @0xbaee1a20f32189d921e7ea94c2a886e065fecc69ba6cb953bb55969ae4ae3cd3;
    // Error Codes
    const ECONTENT_NOT_FOUND: u64 = 1000;
    const EINSUFFICIENT_FUNDS: u64 = 1001;
    const ENODES_NOT_REGISTERED: u64 = 1002;
    const EINVALID_BILLING_PERIOD: u64 = 1003;

    // Billing Period Constants
    const BILLING_MONTHLY: u8 = 1;
    const BILLING_QUARTERLY: u8 = 3;
    const BILLING_BI_ANNUAL: u8 = 6;
    const BILLING_YEARLY: u8 = 12;

    const APTOS_USD_PRICE_FEED_IDENTIFIER : vector<u8> = x"44a93dddd8effa54ea51076c4e851b6cbbfd938e82eb90197de38fe8876bb66e";

    // Event Definitions
    struct ContentUploaded has store, copy, drop {
        content_owner: address,
        content_hash: string::String,
    }

    struct ContentPurchased has store, copy, drop {
        buyer: address,
        content_owner: address,
        content_hash: string::String,
    }

    struct ContentOwnershipVerified has store, copy, drop {
        content_owner: address,
        content_hash: string::String,
        verified_owner: bool,
    }

    struct NodeRegistered has store, copy, drop {
        node_address: address,
    }

    // Holds content metadata and ownership
    struct Content has store {
        owner: address,
        content_hash: string::String,
        file_size: u64,
        num_chunks: u64,           // Number of chunks for this content
        remaining_fee: u64,        // Remaining fee to distribute to nodes
        payable: bool,             // If consumers must pay to access the content
        consumer_fee: u64,
        billing_period: u8,
        next_billing_month: u64,
    }

    // Holds a collection of contents
    struct ContentRegistry has key {
        contents: vector<Content>,
        content_uploaded_events: vector<ContentUploaded>,
        content_purchased_events: vector<ContentPurchased>,
        content_ownership_verified_events: vector<ContentOwnershipVerified>,
    }

    // Initializes a ContentRegistry for an account
    public entry fun init_registry(account: &signer) {
        move_to(account, ContentRegistry {
            contents: vector::empty<Content>(),
            content_uploaded_events: vector::empty<ContentUploaded>(),
            content_purchased_events: vector::empty<ContentPurchased>(),
            content_ownership_verified_events: vector::empty<ContentOwnershipVerified>(),
        });
    }

    // Upload Content
    public entry fun upload_content(
        account: &signer,
        content_hash: string::String,
        file_size: u64,
        num_chunks: u64,
        remaining_fee: u64,
        payable: bool,
        consumer_fee: u64,
        billing_period: u8,
    ) acquires ContentRegistry {

        let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));


        coin::transfer<AptosCoin>(account, CONTRACT_ADDRESS, content_fee);

        let new_content = Content {
            owner: signer::address_of(account),
            content_hash,
            file_size,
            num_chunks,
            remaining_fee: content_fee, // Remaining fee starts as the full content fee
            payable,
            consumer_fee,
            billing_period,
            next_billing_month: 2,
        };

        vector::push_back(&mut registry.contents, new_content);

        let upload_event = ContentUploaded {
            content_owner: signer::address_of(account),
            content_hash,
        };
        vector::push_back(&mut registry.content_uploaded_events, upload_event);
    }

    // Verify the owner of a content
    public entry fun verify_content_ownership(
        owner: address,
        content_hash: string::String
    ) acquires ContentRegistry {
        let registry = borrow_global_mut<ContentRegistry>(owner);

        let len = vector::length(&registry.contents);
        let i = 0;

        let  found = false;

        while (i < len) {
            let content = vector::borrow(&registry.contents, i);
            if (content.content_hash == content_hash) {
                found = true;
                vector::push_back(&mut registry.content_ownership_verified_events, ContentOwnershipVerified {
                    content_owner: owner,
                    content_hash,
                    verified_owner: true,
                });
                break
            };
            i = i + 1;
        };

        if (!found) {
            vector::push_back(&mut registry.content_ownership_verified_events, ContentOwnershipVerified {
                content_owner: owner,
                content_hash,
                verified_owner: false,
            });
            assert!(found, ECONTENT_NOT_FOUND);
        }
    }

    // Purchase content
    public entry fun purchase_content(
        buyer: &signer,
        content_owner: address,
        content_hash: string::String
    ) acquires ContentRegistry {
        let registry = borrow_global_mut<ContentRegistry>(content_owner);
        let len = vector::length(&registry.contents);
        let i = 0;

        let found = false;

        while (i < len) {
            let content = vector::borrow(&registry.contents, i);
            if (content.content_hash == content_hash) {
                assert!(content.cost > 0, EINSUFFICIENT_FUNDS);

                // Transfer coins from buyer to content owner
                coin::transfer<AptosCoin>(buyer, content_owner, content.cost);
                found = true;

                let purchase_event = ContentPurchased {
                    buyer: signer::address_of(buyer),
                    content_owner,
                    content_hash,
                };
                vector::push_back(&mut registry.content_purchased_events, purchase_event);
                break
            };
            i = i + 1;
        };

        assert!(found, ECONTENT_NOT_FOUND);
    }

    // fun fetch_price(receiver : &signer,  vaas : vector<vector<u8>>) : Price {
    //     let coins = coin::withdraw<AptosCoin>(receiver, pyth::get_update_fee(&vaas)); // Get coins to pay for the update
    //     pyth::update_price_feeds(vaas, coins); // Update price feed with the provided vaas
    //     pyth::get_price(price_identifier::from_byte_vec(APTOS_USD_PRICE_FEED_IDENTIFIER)) // Get recent price (will fail if price is too old)
    //
    // }
    //
    // fun calculate_fee_in_apt(content_size_gb: u64, account: &signer, vaas: vector<vector<u8>>): u64 {
    //     let usd_cost = content_size_gb * USD_COST_PER_GB;
    //     let _apt_price_usd = fetch_price(account, vaas);
    //     let fee_in_apt = usd_cost;
    //     fee_in_apt
    // }
}