### Core Components

#### **libp2p Networking Stack**
Libp2p provides the networking backbone for BoxPeer. Potocols used:
   - **Peer Discovery** (mDNS)
   - **Data Routing** (Kademlia DHT)
   - **Communication** (QUIC Transport)
   - **Data Exchange** (Beetswap)
   - **Peer Identification** (lip2p Identity)
---

### Details

- **QUIC**: Chosen for its low-latency, encrypted connections and better performance over unreliable networks ([libp2p QUIC](https://www.google.com/url?sa=t&source=web&rct=j&opi=89978449&url=https://docs.libp2p.io/concepts/transports/quic/&ved=2ahUKEwjohaXDtpGJAxWAR_EDHWnAJL4QFnoECBwQAQ&usg=AOvVaw3mtHjrkS8irUT1uDe50q2z)).
- **mDNS**: Used for local peer discovery. ([mDNS Documentation](https://www.google.com/url?sa=t&source=web&rct=j&opi=89978449&url=https://docs.libp2p.io/concepts/discovery-routing/mdns/&ved=2ahUKEwi4ncH3tpGJAxVoc_EDHW42DGIQFnoECBYQAQ&usg=AOvVaw0afLC01b_Rey85fIFEiAcb)).
- **Kademlia DHT**: Distributed hash table (DHT) that stores peer and content information, allowing decentralized data lookup ([Kademlia Explained](https://docs.libp2p.io/concepts/discovery-routing/kaddht/)).
- **Beetswap**: is a Rust-based implementation of the Bitswap protocol for the libp2p networking stack. ([Beetswap Protocol](https://github.com/eigerco/beetswap/tree/main)).
- **Libp2p Identity**: Used for generating node’s network identity keys.([libp2p identity Protocol](https://docs.rs/libp2p-identity))

---

### Data Flow

1. **User Requests Content**: A peer queries the DHT (Kademlia) to locate peers that hold the requested content.
2. **Content is Requested**: The requesting peer uses Beetswap to ask for specific data blocks from other peers.
3. **Content is Delivered**: Blocks are sent via the QUIC transport layer, ensuring fast and secure transmission.

---

### Blockchain Integration

The **Aptos Blockchain** ensures that ownership, access control, and payments are decentralized. Here are the primary functions:

1. **Registering Content**: When a peer uploads content, a transaction is recorded on the blockchain, associating the content with the peer’s identity.
2. **Fee Payments**: Users pay fees through smart contracts to access premium content. These payments are recorded immutably.
3. **Node Reward**: When a node locks a content, he is rewarded with APT.

---

### Security Considerations

- **Encryption**: All peer-to-peer communications use end-to-end encryption based on  [noise protocol](https://www.google.com/url?sa=t&source=web&rct=j&opi=89978449&url=https://docs.libp2p.io/concepts/secure-comm/noise/&ved=2ahUKEwii1oXyt5GJAxX1R_EDHUsxLu4QFnoECBcQAQ&usg=AOvVaw3bkt0M6vW0rmR17c4EBbAj).
- **Blockchain Immutability**: Transactions, ownership, and fees are secured by the Aptos blockchain, which prevents tampering.
- **Decentralization**: No single entity controls the content or network, reducing points of failure and risks of malicious attacks.

---


### Other References
 - Aptos Keyless - [Link](https://aptosfoundation.org/currents/going-keyless)
 - How closest peer is determined ~ [Link](https://ethportal.net/concepts/protocols/kademlia)

