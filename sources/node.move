module CDN::NodeRegistry {
    use std::signer;
    use std::string;
    use std::vector;

    struct Node has store, copy {
        address: address,
        ip_address: string::String,
        is_active: bool,
        served_content_hashes: vector<string::String>,
        preferences: NodePreferences,
    }


    struct NodePreferences has store, copy {
        max_storage: u64,
        max_bandwidth: u64,
        location: string::String,
        preferred_content_types: vector<string::String>,
    }

    struct NodeRegistry has key {
        nodes: vector<Node>
    }

    public fun init_registry(account: &signer) {
        move_to(account, NodeRegistry {
            nodes: vector::empty<Node>(),
        });
    }

    public entry fun register_node(
        account: &signer,
        ip_address: string::String,
        max_storage: u64,
        max_bandwidth: u64,
        location: string::String,
        preferred_content_types: vector<string::String>) acquires NodeRegistry {

            let registry = borrow_global_mut<NodeRegistry>(signer::address_of(account));

            let new_node = Node {
                address: signer::address_of(account),
                ip_address,
                is_active: true,
                served_content_hashes: vector::empty<string::String>(),
                preferences: NodePreferences {
                    max_storage,
                    max_bandwidth,
                    location,
                    preferred_content_types
                },
            };
        vector::push_back(&mut registry.nodes, new_node);
    }

    public entry fun opt_out_content(account: &signer, content_hash: string::String) acquires NodeRegistry {
        let registry = borrow_global_mut<NodeRegistry>(signer::address_of(signer));
        let len = vector::length(registry.nodes);
        let i = 0;

        while (i < len) {
            let node = vector::borrow_mut(&mut registry.nodes, i);
            if (node.address == signer::address_of(account)) {
                let content_len = vector::length(&node.served_content_hashes);
                let j = 0;
                while (j < content_len) {
                    if (vector::borrow(&node.served_content_hashes, j) == content_hash) {
                        vector::remove(&mut node.served_content_hashes, j);
                        break;
                    };
                    j = j + 1;
                };
                break;
            };
            i = i + 1;
        }
    }
}