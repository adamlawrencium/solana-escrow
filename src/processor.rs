use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Grab the data that contains the required escrow 'amount'
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        //
        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }
        }
    }

    /// This function is validating each of the inputs from EscrowInstruction
    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        // INITIALIZER
        // Check if initializer is the signer
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // TEMP TOKEN ACCOUNT
        // Temp token account that will be transfered to the escrow program. Note: needs to be writable.
        // Note: we don't check this is owned by Token Program because we transfer this account to the PDA.
        let temp_token_account = next_account_info(account_info_iter)?;

        // TOKEN TO RECEIVE ACCOUNT
        // Confirm that receiving token account is owned by Token Program
        // When Bob submits his coins, Escrow will send those to this account.
        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // ESCROW ACCOUNT
        // Validate the escrow account
        let escrow_account = next_account_info(account_info_iter)?;

        // VALIDATE RENT
        // Calculate the rent cost. Programs disappear if account balance goes to 0.
        // So we check to make sure there are enough funds to prevent the program disappearing.
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}
