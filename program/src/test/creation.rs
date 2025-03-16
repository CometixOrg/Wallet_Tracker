#![cfg(feature = "test")]

mod program_test;

use {
    program_test::program_test_2022,
    solana_program::{instruction::*, pubkey::Pubkey, system_instruction, sysvar},
    solana_program_test::*,
    solana_sdk::{
        signature::Signer,
        transaction::{Transaction, TransactionError},
    },
    solana_program::system_program::id as system_program_id,
};

#[tokio::test]
async fn test_associated_token_address() {
    let wallet_address = Pubkey::new_unique();
    let token_mint_address = Pubkey::new_unique();
    let associated_token_address = get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    let (banks_client, payer, recent_blockhash) =
        program_test_2022(token_mint_address).start().await;
    let rent = banks_client.get_rent().await.unwrap();

    let expected_token_account_len = 165;
    let expected_token_account_balance = rent.minimum_balance(expected_token_account_len);

    assert_eq!(
        banks_client
            .get_account(associated_token_address)
            .await
            .expect("get_account"),
        None,
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &wallet_address,
            &token_mint_address,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let associated_account = banks_client
        .get_account(associated_token_address)
        .await
        .expect("get_account")
        .expect("associated_account not none");
    assert_eq!(associated_account.data.len(), expected_token_account_len);
    assert_eq!(associated_account.owner, system_program_id());
    assert_eq!(associated_account.lamports, expected_token_account_balance);
}

#[tokio::test]
async fn test_create_with_fewer_lamports() {
    let wallet_address = Pubkey::new_unique();
    let token_mint_address = Pubkey::new_unique();
    let associated_token_address = get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    let (banks_client, payer, recent_blockhash) =
        program_test_2022(token_mint_address).start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_len = 165;
    let expected_token_account_balance = rent.minimum_balance(expected_token_account_len);

    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::transfer(
            &payer.pubkey(),
            &associated_token_address,
            rent.minimum_balance(0),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client
            .get_balance(associated_token_address)
            .await
            .unwrap(),
        rent.minimum_balance(0)
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &wallet_address,
            &token_mint_address,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client
            .get_balance(associated_token_address)
            .await
            .unwrap(),
        expected_token_account_balance,
    );
}

#[tokio::test]
async fn test_create_with_excess_lamports() {
    let wallet_address = Pubkey::new_unique();
    let token_mint_address = Pubkey::new_unique();
    let associated_token_address = get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    let (banks_client, payer, recent_blockhash) =
        program_test_2022(token_mint_address).start().await;
    let rent = banks_client.get_rent().await.unwrap();

    let expected_token_account_len = 165;
    let expected_token_account_balance = rent.minimum_balance(expected_token_account_len);

    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::transfer(
            &payer.pubkey(),
            &associated_token_address,
            expected_token_account_balance + 1,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client
            .get_balance(associated_token_address)
            .await
            .unwrap(),
        expected_token_account_balance + 1
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &wallet_address,
            &token_mint_address,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client
            .get_balance(associated_token_address)
            .await
            .unwrap(),
        expected_token_account_balance + 1
    );
}

#[tokio::test]
async fn test_create_account_mismatch() {
    let wallet_address = Pubkey::new_unique();
    let token_mint_address = Pubkey::new_unique();
    let _associated_token_address = get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    let (banks_client, payer, recent_blockhash) =
        program_test_2022(token_mint_address).start().await;

    let mut instruction = create_associated_token_account(
        &payer.pubkey(),
        &wallet_address,
        &token_mint_address,
    );
    instruction.accounts[1] = AccountMeta::new(Pubkey::default(), false);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::InvalidSeeds)
    );

    let mut instruction = create_associated_token_account(
        &payer.pubkey(),
        &wallet_address,
        &token_mint_address,
    );
    instruction.accounts[2] = AccountMeta::new(Pubkey::default(), false);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::InvalidSeeds)
    );

    let mut instruction = create_associated_token_account(
        &payer.pubkey(),
        &wallet_address,
        &token_mint_address,
    );
    instruction.accounts[3] = AccountMeta::new(Pubkey::default(), false);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::InvalidSeeds)
    );
}

#[tokio::test]
async fn test_create_associated_token_account_using_legacy_implicit_instruction() {
    let wallet_address = Pubkey::new_unique();
    let token_mint_address = Pubkey::new_unique();
    let associated_token_address = get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    let (banks_client, payer, recent_blockhash) =
        program_test_2022(token_mint_address).start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_len = 165;
    let expected_token_account_balance = rent.minimum_balance(expected_token_account_len);

    assert_eq!(
        banks_client
            .get_account(associated_token_address)
            .await
            .expect("get_account"),
        None,
    );

    let mut create_associated_token_account_ix = create_associated_token_account(
        &payer.pubkey(),
        &wallet_address,
        &token_mint_address,
    );

    create_associated_token_account_ix.data = vec![];
    create_associated_token_account_ix
        .accounts
        .push(AccountMeta::new_readonly(sysvar::rent::id(), false));

    let mut transaction =
        Transaction::new_with_payer(&[create_associated_token_account_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let associated_account = banks_client
        .get_account(associated_token_address)
        .await
        .expect("get_account")
        .expect("associated_account not none");
    assert_eq!(associated_account.data.len(), expected_token_account_len);
    assert_eq!(associated_account.owner, system_program_id());
    assert_eq!(associated_account.lamports, expected_token_account_balance);
}
