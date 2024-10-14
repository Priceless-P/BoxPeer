module BoxPeer_addr::BoxPeer {
    use std::signer;
    use std::string;
    use std::vector;
    use aptos_framework::coin;
    use aptos_framework::account;
    use aptos_framework::event;
    use aptos_framework::aptos_coin::AptosCoin;

    const CONTRACT_ADDRESS: address = @0x61c8f3e7ecbcda5dd641c434b277a13b6076c09de32322ce197d2fe3f1e54ef1;

    // Error Codes
    const ECONTENT_NOT_FOUND: u64 = 1000;
    const EINSUFFICIENT_FUNDS: u64 = 1001;
    const ENODES_NOT_REGISTERED: u64 = 1002;
    const ECONTENT_ALREADY_PAID: u64 = 1003;
    const ENOT_CONTRACT_ADDRESS: u64 = 1004;
    const EPAYOUT_EXCEEDS_FEES: u64 = 1005;

    struct NodeRegistered has store, copy, drop {
        node_address: address,
    }

    // Holds content metadata and ownership
    #[event]
    struct Content has store, drop {
        owner: address,
        owner_name: string::String,
        description: string::String,
        title: string::String,
        cid: string::String,
        fee_paid: u64,
        consumer_fee: u64,
        file_type: string::String,
        purchasers: vector<address>,
    }

    // Holds a collection of contents
    #[event]
    struct ContentRegistry has key {
        contents: vector<Content>,
    }

    #[event]
    struct GlobalContentRegistry has key {
        contents: vector<Content>,
        content_event_handle: event::EventHandle<Content>,
    }
    struct ContentFee has store {
        cid: string::String,
        total_fee: u64,
    }


    struct ContractSigner has key {
        signer_capability: account::SignerCapability,
        content_fees: vector<ContentFee>,
    }


    #[event]
    struct PurchaserAddedEvent has store, key, drop {
        cid: string::String,
        purchaser: address,
    }

    // Initializes a ContentRegistry for an account
    public entry fun init_registry(account: &signer) {
        move_to(account, ContentRegistry {
            contents: vector::empty<Content>(),
        });
    }

    // Initialize event handle for the contract owner
    public entry fun initialize_contract_event_handle(account: &signer) {
        let account_address = signer::address_of(account);
        assert!(account_address == CONTRACT_ADDRESS, ENOT_CONTRACT_ADDRESS);

        if (!exists<GlobalContentRegistry>(CONTRACT_ADDRESS)) {
            move_to(account, GlobalContentRegistry {
                contents: vector::empty<Content>(),
                content_event_handle: account::new_event_handle<Content>(account),
            });
        }
    }
    public entry fun initialize_contract_signer(account: &signer, seed: vector<u8>) {
        let (_contract_signer, contract_signer_cap) = account::create_resource_account(account, seed);
        assert!(&signer::address_of(account) == &CONTRACT_ADDRESS, ENOT_CONTRACT_ADDRESS);
        // Store the signer capability in the contract
        move_to(account, ContractSigner {
            signer_capability: contract_signer_cap,
            content_fees: vector::empty<ContentFee>(),
        });
    }


    // Upload Content
    public entry fun upload_content(
        account: &signer,
        cid: string::String,
        fee_paid: u64,
        consumer_fee: u64,
        file_type: string::String,
        owner_name: string::String,
        description: string::String,
        title: string::String,
    ) acquires ContentRegistry, GlobalContentRegistry, ContractSigner {

    if (!exists<ContentRegistry>(signer::address_of(account))) {
        init_registry(account);
    };

        let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));

        // Transfer the fee to the contract address
        coin::transfer<AptosCoin>(account, CONTRACT_ADDRESS, fee_paid);

        let contract_signer = borrow_global_mut<ContractSigner>(CONTRACT_ADDRESS);
        let new_content_fee = ContentFee { cid, total_fee: fee_paid };
        vector::push_back(&mut contract_signer.content_fees, new_content_fee);

        let new_content = Content {
            owner: signer::address_of(account),
            cid,
            fee_paid,
            consumer_fee,
            file_type,
            owner_name,
            description,
            title,
            purchasers: vector::empty<address>(),
        };


        vector::push_back(&mut registry.contents,new_content);

        let event =  Content {
            owner: signer::address_of(account),
            cid,
            fee_paid,
            consumer_fee,
            file_type,
            owner_name,
            description,
            title,
            purchasers: vector::empty<address>(),
        };

        event::emit(event);

        let new_content2 =  Content {
            owner: signer::address_of(account),
            cid,
            fee_paid,
            consumer_fee,
            file_type,
            owner_name,
            description,
            title,
            purchasers: vector::empty<address>(),
        };

        let event2 =  Content {
            owner: signer::address_of(account),
            cid,
            fee_paid,
            consumer_fee,
            file_type,
            owner_name,
            description,
            title,
            purchasers: vector::empty<address>(),
        };

        let contract_event_handle = borrow_global_mut<GlobalContentRegistry>(CONTRACT_ADDRESS);
        vector::push_back(&mut contract_event_handle.contents, new_content2);
        event::emit_event(&mut contract_event_handle.content_event_handle, event2)
    }

    public entry fun pay_for_content(account: &signer, cid: string::String) acquires GlobalContentRegistry, ContentRegistry {
        let global_registry = borrow_global_mut<GlobalContentRegistry>(CONTRACT_ADDRESS);

        let i = 0;
        let found = false;

        while (i < vector::length(&global_registry.contents)) {
            let content = vector::borrow_mut(&mut global_registry.contents, i);

            if (content.cid == cid) {
                found = true;

                let purchaser_already_exists = vector::contains(&content.purchasers, &signer::address_of(account));
                assert!(!purchaser_already_exists, ECONTENT_ALREADY_PAID);

                let balance = coin::balance<AptosCoin>(signer::address_of(account));
                assert!(balance >= content.consumer_fee, EINSUFFICIENT_FUNDS);

                coin::transfer<AptosCoin>(account, content.owner, content.consumer_fee);

                // Update the purchasers in the GlobalContentRegistry
                vector::push_back(&mut content.purchasers, signer::address_of(account));

                event::emit(PurchaserAddedEvent {
                    cid: content.cid,
                    purchaser: signer::address_of(account),
                });

                // Check if the ContentRegistry exists for the user's address
                if (exists<ContentRegistry>(signer::address_of(account))) {
                    let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));

                    // Find the corresponding content in the ContentRegistry and update its purchasers
                    let j = 0;
                    while (j < vector::length(&registry.contents)) {
                        let registry_content = vector::borrow_mut(&mut registry.contents, j);
                        if (registry_content.cid == cid) {
                            // Update the purchasers in the ContentRegistry
                            vector::push_back(&mut registry_content.purchasers, signer::address_of(account));
                            break
                        };
                        j = j + 1;
                    };
                };

                break
            };
            i = i + 1;
        };
        assert!(found, ECONTENT_NOT_FOUND);
    }

    #[view]
    public fun get_purchasers_by_cid(cid: string::String): vector<address> acquires GlobalContentRegistry {
        let content_registry = borrow_global<GlobalContentRegistry>(CONTRACT_ADDRESS);

        let i = 0;
        while (i < vector::length(&content_registry.contents)) {
            let content = vector::borrow(&content_registry.contents, i);
            if (content.cid == cid) {
                return content.purchasers
            };
            i = i + 1;
        };

        vector::empty<address>()
    }

    #[view]
    public fun get_total_earned_by_owner(owner: address): u64 acquires GlobalContentRegistry {
        let global_registry = borrow_global<GlobalContentRegistry>(CONTRACT_ADDRESS);
        let total_earned = 0;
        let i = 0;

        while (i < vector::length(&global_registry.contents)) {
            let content = vector::borrow(&global_registry.contents, i);
            // If the content owner matches the given owner address, calculate earnings
            if (content.owner == owner) {
                let num_purchasers = vector::length(&content.purchasers);
                let content_earned = num_purchasers * content.consumer_fee;
                total_earned = total_earned + content_earned;
            };
            i = i + 1;
        };
        total_earned
    }

    public entry fun get_reward(account: &signer, cid: string::String, amount: u64) acquires ContractSigner {
        let contract_signer = borrow_global_mut<ContractSigner>(CONTRACT_ADDRESS);

        // Find the fees for the given cid
        let  found = false;
        let i = 0;
        let fee_for_cid = 0;

        while (i < vector::length(&contract_signer.content_fees)) {
        let content_fee = vector::borrow_mut(&mut contract_signer.content_fees, i);
        if (content_fee.cid == cid) {
        fee_for_cid = content_fee.total_fee;
        found = true;
        break
        };
        i = i + 1;
        };

        assert!(found, ECONTENT_NOT_FOUND);
        assert!(fee_for_cid >= amount, EINSUFFICIENT_FUNDS);

        // Use SignerCapability to create a signer for the contract address
        let contract_signer_instance = account::create_signer_with_capability(&contract_signer.signer_capability);

        // Transfer the amount to the user's account
        coin::transfer<AptosCoin>(&contract_signer_instance, signer::address_of(account), amount);

        // Deduct the amount from the cid's fees
        let content_fee = vector::borrow_mut(&mut contract_signer.content_fees, i);
        content_fee.total_fee = content_fee.total_fee - amount;
    }

}
