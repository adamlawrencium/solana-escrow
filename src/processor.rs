use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::invoke
};

use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};

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

        // Now create the Escrow object
        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
        escrow_info.expected_amount = amount;

        Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

        // Transfer (user space) ownership of the temporary token account to the Program-derived address
        // Get the token program, then 
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let token_program = next_account_info(account_info_iter)?;
        let owner_change_instruction = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_instruction,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }
}
