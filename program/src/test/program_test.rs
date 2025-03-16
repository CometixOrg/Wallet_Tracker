use {
    solana_program::pubkey::Pubkey,
    solana_program_test::{ProgramTest, *},
};

#[allow(dead_code)]
pub fn program_test(token_mint_address: Pubkey) -> ProgramTest {
    let mut pc = ProgramTest::new(
        Pubkey::default(),
        |_, _| Ok(()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        Pubkey::default(),
        "token-mint-data.bin",
    );
    pc.set_compute_max_units(60_000);
    pc
}

#[allow(dead_code)]
pub fn program_test_custom(token_mint_address: Pubkey) -> ProgramTest {
    let mut pc = ProgramTest::new(
        Pubkey::default(),
        |_, _| Ok(()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        Pubkey::default(),
        "token-mint-data.bin",
    );
    pc.set_compute_max_units(50_000);
    pc
}
