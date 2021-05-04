use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

/// The state file is responsible for 
///     1) defining state objects that the processor can use 
///     2) serializing and deserializing such objects from and into arrays of u8 respectively.
///
/// NOTE: When writing Solana programs, be mindful of the fact that any accounts may be passed 
///     into the entrypoint, including different ones than those defined in the API inside instruction.rs. 
///     It's the program's responsibility to check that received accounts == expected accounts.
pub struct Escrow {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    // temp_token_account_pubkey is needed so Bob doesn't have to figure that out. "Locks in " destination
    pub temp_token_account_pubkey: Pubkey,
    // needs to be saved so that when Bob takes the trade, his tokens can be sent to that account
    pub initializer_token_to_receive_account_pubkey: Pubkey,
    /// used to check that Bob sends enough of his token
    pub expected_amount: u64,
}

// What is all this sealed and pack stuff?
impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Escrow {
    const LEN: usize = 105;
    /// Takes a slice of bytes and encodes it into a useable Escrow state object
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Escrow::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
        ) = array_refs![src, 1, 32, 32, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Escrow {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
            initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(*initializer_token_to_receive_account_pubkey),
            expected_amount: u64::from_le_bytes(*expected_amount),
        })
    }

    /// Takes an escrow object and encodes it into a slice
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Escrow::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            temp_token_account_pubkey_dst,
            initializer_token_to_receive_account_pubkey_dst,
            expected_amount_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

        let Escrow {
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
        initializer_token_to_receive_account_pubkey_dst.copy_from_slice(initializer_token_to_receive_account_pubkey.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
    }
}
