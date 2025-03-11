use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use token_account_client::instruction::*;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum TokenAccountInstruction {
    Create,
    CreateIdempotent,
    RecoverNested,
}
