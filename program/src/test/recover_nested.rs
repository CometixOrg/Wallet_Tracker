#![cfg(feature = "test")]

mod program_test;

use {
    program_test::{program_test, program_test_custom},
    solana_program::{pubkey::Pubkey, system_instruction},
    solana_program_test::*,
    solana_sdk::{
        instruction::{AccountMeta, InstructionError},
        signature::Signer,
        signer::keypair::Keypair,
        transaction::{Transaction, TransactionError},
    },
};

async fn create_mint(context: &mut ProgramTestContext, program_id: &Pubkey) -> (Pubkey, Keypair) {
    let mint_account = Keypair::new();
    let token_mint_address = mint_account.pubkey();
    let mint_authority = Keypair::new();
    let space = 0;  // Simplified, no extension type
    let rent = context.banks_client.get_rent().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint_account.pubkey(),
                rent.minimum_balance(space),
                space as u64,
                program_id,
            ),
            system_instruction::create_account(
                program_id,
                &token_mint_address,
                &mint_authority.pubkey(),
                Some(&mint_authority.pubkey()),
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_account],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    (token_mint_address, mint_authority)
}

async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    owner: &Pubkey,
    mint: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            owner,
            mint,
            program_id,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    *owner
}

#[allow(clippy::too_many_arguments)]
async fn try_recover_nested(
    context: &mut ProgramTestContext,
    program_id: &Pubkey,
    nested_mint: Pubkey,
    nested_mint_authority: Keypair,
    nested_associated_token_address: Pubkey,
    destination_token_address: Pubkey,
    wallet: Keypair,
    recover_transaction: Transaction,
    expected_error: Option<InstructionError>,
) {
    let nested_account = context
        .banks_client
        .get_account(nested_associated_token_address)
        .await
        .unwrap()
        .unwrap();
    let lamports = nested_account.lamports;

    let amount = 100;
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &program_id,
                &nested_associated_token_address,
                &nested_mint_authority.pubkey(),
                &[],
                amount,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &nested_mint_authority],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let result = context
        .banks_client
        .process_transaction(recover_transaction)
        .await;

    if let Some(expected_error) = expected_error {
        let error = result.unwrap_err().unwrap();
        assert_eq!(error, TransactionError::InstructionError(0, expected_error));
    } else {
        result.unwrap();
        assert!(context
            .banks_client
            .get_account(nested_associated_token_address)
            .await
            .unwrap()
            .is_none());
        let destination_account = context
            .banks_client
            .get_account(destination_token_address)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(destination_account.lamports, amount);
        let wallet_account = context
            .banks_client
            .get_account(wallet.pubkey())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wallet_account.lamports, lamports);
    }
}

async fn check_same_mint(context: &mut ProgramTestContext, program_id: &Pubkey) {
    let wallet = Keypair::new();
    let (mint, mint_authority) = create_mint(context, program_id).await;

    let owner_associated_token_address =
        create_associated_token_account(context, &wallet.pubkey(), &mint, program_id).await;
    let nested_associated_token_address = create_associated_token_account(
        context,
        &owner_associated_token_address,
        &mint,
        program_id,
    )
    .await;

    context.last_blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &wallet.pubkey(),
            &mint,
            &mint,
            program_id,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &wallet],
        context.last_blockhash,
    );
    try_recover_nested(
        context,
        program_id,
        mint,
        mint_authority,
        nested_associated_token_address,
        owner_associated_token_address,
        wallet,
        transaction,
        None,
    )
    .await;
}

#[tokio::test]
async fn success_same_mint_custom() {
    let dummy_mint = Pubkey::new_unique();
    let pt = program_test_custom(dummy_mint);
    let mut context = pt.start_with_context().await;
    check_same_mint(&mut context, &Pubkey::default()).await;
}

#[tokio::test]
async fn success_same_mint() {
    let dummy_mint = Pubkey::new_unique();
    let pt = program_test(dummy_mint);
    let mut context = pt.start_with_context().await;
    check_same_mint(&mut context, &Pubkey::default()).await;
}

// Similarly refactor all other methods as above
