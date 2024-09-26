module BoxPeer_addr::BoxPeer {
    use std::signer;
    use std::string;
    use std::vector;
    use aptos_framework::coin;
    use aptos_framework::event;
    use aptos_framework::aptos_coin::AptosCoin;

    const CONTRACT_ADDRESS: address = @0x001cdc16ec30101c48a9c0f069c7570f6ca50a66a67615676e90490219d77a08;
    
    // Error Codes
    const ECONTENT_NOT_FOUND: u64 = 1000;
    const EINSUFFICIENT_FUNDS: u64 = 1001;
    const ENODES_NOT_REGISTERED: u64 = 1002;
    
    struct NodeRegistered has store, copy, drop {
        node_address: address,
    }

    // Holds content metadata and ownership
    #[event]
    struct Content has store {
        owner: address,
        content_hash: string::String,
        nodes: vector<address>,
        fee_paid: u64,
        consumer_fee: u64,
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
        content_hash: string::String,
        nodes: vector<address>,
        fee_paid: u64,
        consumer_fee: u64,
    ) acquires ContentRegistry {

        let registry = borrow_global_mut<ContentRegistry>(signer::address_of(account));
        // Check if there are nodes to distribute the fee
    let num_nodes = vector::length(&nodes);
    assert!(num_nodes > 0, ENODES_NOT_REGISTERED);

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
            content_hash,
            nodes,
            fee_paid,
            consumer_fee,
        };

        vector::push_back(&mut registry.contents, new_content);
    }
}