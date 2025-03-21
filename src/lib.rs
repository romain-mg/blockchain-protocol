pub mod blockchain;
pub mod miner;
pub mod network;

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use crate::blockchain::{utils::convert_public_key_to_bytes, Blockchain};
    use crate::miner::{AccountKeys, Miner, Transaction};
    use crate::network::Network;
    use k256::ecdsa::Signature;
    use primitive_types::U256;

    #[test]
    fn test_mine_one_block() {
        let (mut blockchain, mut miner, mut sender_account, receiver_account) = setup();
        let sender_account_public_key = sender_account.public_key;
        let receiver_account_public_key = receiver_account.public_key;
        let transaction_0: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 0,
        };

        let signature_0: Signature = sender_account.sign_transaction(&transaction_0);

        let transaction_1: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 1,
        };

        let signature_1: Signature = sender_account.sign_transaction(&transaction_1);

        let transaction_2: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 2,
        };

        let signature_2: Signature = sender_account.sign_transaction(&transaction_2);

        miner.add_transaction_to_mempool(transaction_0, &signature_0, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_1, &signature_1, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_2, &signature_2, &mut blockchain);

        miner
            .compute_next_block(&mut blockchain, String::from(""))
            .expect("Block must have been built");
        assert_eq!(sender_account.get_balance(&mut blockchain), U256::from(994));
        assert_eq!(receiver_account.get_balance(&mut blockchain), U256::from(3));
        assert_eq!(
            blockchain.get_balance(&miner.account_keys.public_key),
            U256::add(U256::from(3), U256::from(blockchain.mining_reward))
        );
    }

    #[test]
    fn test_chain_reorg() {
        let (mut blockchain, miner, sender_account, receiver_account) = setup();
        let (new_blockchain, fork_block_hash) = mine_initial_blockchain_helper(
            blockchain,
            miner.clone(),
            sender_account.clone(),
            &receiver_account,
        );
        blockchain = new_blockchain;
        let sender_account_balance = sender_account.get_balance(&mut blockchain);
        assert_eq!(sender_account_balance, U256::from(988));
        let receiver_account_balance = receiver_account.get_balance(&mut blockchain);
        assert_eq!(receiver_account_balance, U256::from(6));
        let mut_sender = blockchain
            .accounts
            .get_mut(&convert_public_key_to_bytes(
                &sender_account.get_public_key(),
            ))
            .unwrap();
        mut_sender.nonce = 3;
        let (forked_blockchain, _) = mine_fork_helper(
            blockchain,
            miner,
            sender_account.clone(),
            &receiver_account,
            fork_block_hash,
        );
        blockchain = forked_blockchain;
        let receiver_account_balance = receiver_account.get_balance(&mut blockchain);
        assert_eq!(receiver_account_balance, U256::from(28));
        let sender_account_balance = sender_account.get_balance(&mut blockchain);
        assert_eq!(sender_account_balance, U256::from(959));
    }

    fn setup() -> (Blockchain, Miner, AccountKeys, AccountKeys) {
        let difficulty_divisor: i32 = 20000;
        let difficulty: U256 = U256::MAX / difficulty_divisor;
        let target_duration_between_blocks = 5;
        let max_transactions_per_block = 3;
        let blocks_between_deifficulty_adjustment = 10;
        let mut blockchain: Blockchain = Blockchain::create_blockchain(
            difficulty,
            target_duration_between_blocks,
            max_transactions_per_block,
            blocks_between_deifficulty_adjustment,
        );

        let miner: Miner = Miner::new(&mut blockchain, Network::new());

        let sender_account = AccountKeys::new();
        let sender_account_public_key = sender_account.get_public_key();
        blockchain.create_account(&sender_account_public_key);

        let receiver_account = AccountKeys::new();
        let receiver_account_public_key = receiver_account.get_public_key();
        blockchain.create_account(&receiver_account_public_key);

        blockchain.mint(&sender_account_public_key, U256::from(1000));
        return (blockchain, miner, sender_account, receiver_account);
    }

    fn mine_initial_blockchain_helper(
        mut blockchain: Blockchain,
        mut miner: Miner,
        mut sender_account: AccountKeys,
        receiver_account: &AccountKeys,
    ) -> (Blockchain, String) {
        let sender_account_public_key = sender_account.public_key;
        let receiver_account_public_key = receiver_account.public_key;
        let transaction_0: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 0,
        };

        let signature_0: Signature = sender_account.sign_transaction(&transaction_0);

        let transaction_1: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 1,
        };

        let signature_1: Signature = sender_account.sign_transaction(&transaction_1);

        let transaction_2: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 2,
        };

        let signature_2: Signature = sender_account.sign_transaction(&transaction_2);

        miner.add_transaction_to_mempool(transaction_0, &signature_0, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_1, &signature_1, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_2, &signature_2, &mut blockchain);

        let first_block_hash = miner
            .compute_next_block(&mut blockchain, String::from(""))
            .unwrap();

        let transaction_3: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 3,
        };

        let signature_3: Signature = sender_account.sign_transaction(&transaction_3);

        let transaction_4: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 4,
        };

        let signature_4: Signature = sender_account.sign_transaction(&transaction_4);

        let transaction_5: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(1),
            fee: U256::from(1),
            nonce: 5,
        };

        let signature_5: Signature = sender_account.sign_transaction(&transaction_5);

        miner.add_transaction_to_mempool(transaction_3, &signature_3, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_4, &signature_4, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_5, &signature_5, &mut blockchain);

        miner.compute_next_block(&mut blockchain, first_block_hash.clone());
        return (blockchain, first_block_hash);
    }

    fn mine_fork_helper(
        mut blockchain: Blockchain,
        mut miner: Miner,
        mut sender_account: AccountKeys,
        receiver_account: &AccountKeys,
        fork_block_hash: String,
    ) -> (Blockchain, String) {
        let sender_account_public_key = sender_account.public_key;
        let receiver_account_public_key = receiver_account.public_key;

        let transaction_3: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(5),
            fee: U256::from(2),
            nonce: 3,
        };

        let signature_3: Signature = sender_account.sign_transaction(&transaction_3);

        let transaction_4: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(5),
            fee: U256::from(2),
            nonce: 4,
        };

        let signature_4: Signature = sender_account.sign_transaction(&transaction_4);

        let transaction_5: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(5),
            fee: U256::from(2),
            nonce: 5,
        };

        let signature_5: Signature = sender_account.sign_transaction(&transaction_5);

        miner.add_transaction_to_mempool(transaction_3, &signature_3, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_4, &signature_4, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_5, &signature_5, &mut blockchain);

        let concurrent_block_hash = miner
            .compute_next_block(&mut blockchain, fork_block_hash)
            .unwrap();

        let mut_sender = blockchain
            .accounts
            .get_mut(&convert_public_key_to_bytes(&sender_account_public_key))
            .unwrap();
        mut_sender.nonce = 6;
        let transaction_6: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(5),
            fee: U256::from(2),
            nonce: 6,
        };

        let signature_6: Signature = sender_account.sign_transaction(&transaction_6);

        let transaction_7: Transaction = Transaction {
            public_key_from: sender_account_public_key,
            public_key_to: receiver_account_public_key,
            amount: U256::from(5),
            fee: U256::from(2),
            nonce: 7,
        };

        let signature_7: Signature = sender_account.sign_transaction(&transaction_7);

        miner.add_transaction_to_mempool(transaction_6, &signature_6, &mut blockchain);
        miner.add_transaction_to_mempool(transaction_7, &signature_7, &mut blockchain);

        let dominant_block_hash = miner
            .compute_next_block(&mut blockchain, concurrent_block_hash)
            .unwrap();

        return (blockchain, dominant_block_hash);
    }

    #[test]
    fn test_network_block_propagation() {
        let (base_blockchain, base_miner, mut sender_account, receiver_account) = setup();

        // Clone the base blockchain to simulate separate states for two different nodes.
        let mut blockchain1 = base_blockchain.clone(); // Miner1's view
        let mut blockchain2 = base_blockchain; // Miner2's view

        // Create a common simulated network instance.
        let mut network = Network::new();

        // Create two miners, each with its own blockchain state but using the same network.
        let mut miner1 = Miner::new(&mut blockchain1, network.clone());
        let mut miner2 = Miner::new(&mut blockchain2, network.clone());

        // Add both miners to the network.
        network.add_miner(miner1.clone());
        network.add_miner(miner2.clone());

        // Use the same accounts in both blockchain states.
        let sender_pub = sender_account.get_public_key();
        let receiver_pub = receiver_account.get_public_key();
        blockchain1.create_account(&sender_pub);
        blockchain1.create_account(&receiver_pub);
        blockchain1.mint(&sender_pub, U256::from(1000));
        // Assume blockchain2 is identical to blockchain1 initially.

        // Create a transaction: sender sends 10 tokens (fee 1) to receiver.
        let transaction: Transaction = Transaction {
            public_key_from: sender_pub,
            public_key_to: receiver_pub,
            amount: U256::from(10),
            fee: U256::from(1),
            nonce: 0,
        };
        let signature: Signature = sender_account.sign_transaction(&transaction);

        // Add the transaction to miner1's mempool (which is part of blockchain1).
        miner1.add_transaction_to_mempool(transaction, &signature, &mut blockchain1);

        // Miner1 mines a block on its blockchain.
        let parent_hash = blockchain1.current_longest_chain_latest_block_hash.clone();
        let new_block_hash = miner1
            .compute_next_block(&mut blockchain1, parent_hash)
            .expect("Block must be mined");

        // Retrieve the newly mined block from blockchain1.
        let new_block = blockchain1
            .get_block(&new_block_hash)
            .expect("Mined block should exist")
            .clone();

        // --- Propagation Phase ---
        // Simulate network propagation: broadcast the block from miner1 to miner2's blockchain.
        network.broadcast_block(&miner1, new_block, &mut blockchain2);

        // --- Verification Phase ---
        // Verify that miner2's blockchain now has the new block as its tip.
        assert_eq!(
            blockchain2.current_longest_chain_latest_block_hash,
            new_block_hash
        );

        // Verify that receiver's balance is updated in blockchain2.
        // In this transaction, receiver should receive 10 tokens.
        assert_eq!(blockchain2.get_balance(&receiver_pub), U256::from(10));
    }
}
