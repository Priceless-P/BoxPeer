module BoxPeer_addr::BoxPeer {
    use std::signer;
    use std::string;
    use std::vector;
    use aptos_framework::coin;
    use aptos_framework::event;
    use aptos_framework::aptos_coin::AptosCoin;

    const CONTRACT_ADDRESS: address = @0xbca47e0e304b5dcd2b54c9d6683d1cd11010d6453798da34acd1ae5065c4ff5f;

    // Error Codes
    const ECONTENT_NOT_FOUND: u64 = 1000;
    const EINSUFFICIENT_FUNDS: u64 = 1001;
    const ENODES_NOT_REGISTERED: u64 = 1002;

    struct NodeRegistered has store, copy, drop {
        node_address: address,
    }

    // Holds content metadata and ownership
    #[event]
    struct Content has store, drop {
        owner: address,
        cid: string::String,
        nodes: vector<address>,
        fee_paid: u64,
        consumer_fee: u64,
        file_type: string::String,
    }

    // Holds a collection of contents
    #[event]
    struct ContentRegistry has key {
        contents: vector<Content>,
    }

    // Initializes a ContentRegistry for an account
    public entry fun init_registry(account: &signer) {
        move_to(account, ContentRegistry {
            contents: vector::empty<Content>(),
        });
    }

    // Upload Content
    public entry fun upload_content(
        account: &signer,
        cid: string::String,
        nodes: vector<address>,
        fee_paid: u64,
        consumer_fee: u64,
        file_type: string::String,
    ) acquires ContentRegistry {

    if (!exists<ContentRegistry>(signer::address_of(account))) {
        init_registry(account);
    };

        let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));
        // Check if there are nodes to distribute the fee
    let num_nodes = vector::length(&nodes);
    assert!(num_nodes > 0, ENODES_NOT_REGISTERED);

        let balance = coin::balance<AptosCoin>(signer::address_of(account));
        assert!(balance >= fee_paid, EINSUFFICIENT_FUNDS);


        // Calculate the fee per node
    let fee_per_node = fee_paid / num_nodes;
    let remainder = fee_paid % num_nodes;

    // Transfer fee to each node
    let i = 0;
    while (i < num_nodes) {
        let node_address = vector::borrow(&nodes, i);
        coin::transfer<AptosCoin>(account, *node_address, fee_per_node);
        i = i + 1;
    };

    if (remainder > 0) {
        let first_node_address = vector::borrow(&nodes, 0);
        coin::transfer<AptosCoin>(account, *first_node_address, remainder);
    };
        //coin::transfer<AptosCoin>(account, CONTRACT_ADDRESS, fee_paid);

        let new_content = Content {
            owner: signer::address_of(account),
            cid,
            nodes,
            fee_paid,
            consumer_fee,
            file_type,
        };
        

        vector::push_back(&mut registry.contents,new_content);
        
        let event =  Content {
            owner: signer::address_of(account),
            cid,
            nodes,
            fee_paid,
            consumer_fee,
            file_type,
        };

        event::emit(event);
    }
}
