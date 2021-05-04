use solana_program::pubkey::Pubkey;

/// The state file is responsible for 
///     1) defining state objects that the processor can use 
///     2) serializing and deserializing such objects from and into arrays of u8 respectively.
pub struct Escrow {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
    pub initializer_token_to_receive_account_pubkey: Pubkey,
    pub expected_amount: u64,
}
