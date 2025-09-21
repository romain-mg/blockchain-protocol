## blockchain-protocol

A Rust workspace featuring:
- A fully-featured in-memory Proof-of-Work blockchain with mempool, transaction validation, block production, dynamic difficulty, and chain reorg to the longest chain by cumulative difficulty.
- A libp2p-based P2P node where a bootnode can continuously produce blocks and peers can sync and download the entire blockchain.

### Repository layout

- `blockchain_core/`
  - `src/blockchain/`
    - `account.rs`: `AccountKeys` and account state management (balances, nonces).
    - `block.rs`: `Transaction`, `Header`, `Block`, Merkle tree, hashing, (de)serialization.
    - `utils.rs`: Transaction hashing, key utilities.
  - `src/blockchain.rs`: `Blockchain` data structure, state transition rules, cumulative difficulty, reorg logic, difficulty adjustment.
  - `src/miner.rs`: `Miner` with mempool, transaction validation, PoW block production, and simulated peer propagation.
  - `src/mock/`: `mock_network.rs`, `mock_miner.rs` for in-memory network simulation in tests/examples.
  - `src/lib.rs`: Test suite covering block mining, chain reorg, simulated propagation, and multithreading.
- `node/`
  - `src/p2p_node.rs`: libp2p swarm (Kademlia + request/response CBOR protocol) for blockchain sync.
  - `src/main.rs`: CLI, bootnode mining loop, inbound sync request handling, dialing/syncing.

### Features

- **PoW blockchain core**
  - Accounts with balances and nonces (`U256`), keyed by compressed ECDSA secp256k1 public keys (`k256`).
  - Transactions signed with ECDSA; mempool prioritized by fee, filtered by nonce/balance validity.
  - Blocks contain serialized transactions and a Merkle root; block hash includes nonce/timestamp/prev/merkle.
  - PoW mining: iterate nonce and timestamp until `hash(header) <= difficulty`.
  - Dynamic difficulty: adjusts every N blocks to target a configured block time.
  - Longest chain selection by cumulative difficulty; full reorg applies/reverts transactions as needed.
  - Miner rewards (block reward + fees) applied on apply, reverted on reorg.
- **P2P node**
  - libp2p TCP + Noise + Yamux + Kademlia for discovery plus a custom Request/Response protocol (`/blockchain/1.0.0`) to sync.
  - Bootnode mines continuously in a background thread and serves full-chain sync upon request.
  - Syncing nodes dial the bootnode and request the serialized blockchain (JSON) in one shot.
  - Mining is temporarily paused while serving a sync to avoid prolonged lock contention.


### Run the tests (core)

- `cargo test -p blockchain_core`

Tests include:
- Mining and account state updates.
- Chain reorg to a heavier fork.
- Simulated block propagation across in-memory miners.
- Multithreaded serialization while mining.

### Running the P2P demo

There are two roles:
- Bootnode: mines blocks and serves blockchain sync to peers.
- Sync node: dials the bootnode and downloads the blockchain.


- Open two terminals.
- Terminal 1 (bootnode):
  - Edit `BOOTNODE_MULTIADDR` to a reachable IP/port (e.g., your LAN IP).
  - Run: `cargo run -p node -- --bootnode true --secret-key-seed 1`

- Terminal 2 (sync node):
  - `cargo run -p node -- --sync true`
  - You should see the blockchain printed after sync completes.

Optionally, set the `BOOTSTRAP_NODE_KEYS` environment variable to a base64-encoded libp2p keypair (protobuf-encoded). If the bootnode’s printed PeerId or address differs from the hardcoded values in `node/src/main.rs`, update:
- `BOOTNODE_ID`: the bootnode’s PeerId.
- `BOOTNODE_MULTIADDR`: the full listen `Multiaddr`, including `/p2p/<peer-id>`.



### CLI flags (node)

- `--bootnode <bool>`: When true, starts in mining + serve mode.
- `--sync <bool>`: When true, dials the bootnode and requests a chain sync.
- `--secret-key-seed <u8>`: Deterministic keypair for stable PeerId (overrides env/bootnode key path).
- `--listen-address <Multiaddr>`: Listening address (ignored when `--bootnode true`, which uses `BOOTNODE_MULTIADDR` constant).

