use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use gateway::program::Gateway;
use gateway::{self, DappAccount as GatewayDappAccount, GatewayAccount, PacketAccount};
use internal::*;

declare_id!("7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3");

pub const AUTHORIZED_DEPLOYER: &str = "AcSZ7ECK7yQk9TedzsjkT8dzhyuxi7TzXNhwuGADD9N";
const PING_PONG_ACCOUNT: &[u8] = b"ping_pong";

#[event]
pub struct AckFromDestination {
    pub request_id: Option<u128>,
    pub ack_message: Option<String>,
    pub request_identifier: u128,
    pub exec_flag: bool,
}

#[event]
pub struct PingFromSource {
    pub src_chain_id: String,
    pub request_id: u128,
    pub sample_str: String,
}

#[event]
pub struct NewPing {
    pub request_id: u128,
}

//NOTE: No Audit Required, Just A Test Dapp
mod internal {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////////////
    /// Pausable
    ////////////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn _when_not_pause(dapp_acc: Account<'_, PingPongAccount>) -> Result<()> {
        if dapp_acc.pause {
            return Err(PausableError::Paused.into());
        }
        Ok(())
    }
    pub(crate) fn _when_pause(dapp_acc: Account<'_, PingPongAccount>) -> Result<()> {
        if !dapp_acc.pause {
            return Err(PausableError::Paused.into());
        }
        Ok(())
    }

    fn encode_u256(value: u128) -> Vec<u8> {
        let mut encoded = vec![0u8; 32];
        encoded.splice(16.., value.to_be_bytes()); // place u64 into the last 8 bytes (big endian)
        encoded
    }

    fn encode_string(s: &str) -> Vec<u8> {
        let mut result = vec![];
        let mut string_data = s.as_bytes().to_vec();

        let mut length = vec![0u8; 32];
        length.splice(24.., (string_data.len() as u64).to_be_bytes()); // length of the string
        result.extend(length);

        let padding = (32 - (string_data.len() % 32)) % 32;
        string_data.extend(vec![0u8; padding]); // padding with zeros
        result.extend(string_data);

        result
    }

    pub(crate) fn abi_encode_u128_string(value: u128, text: String) -> Vec<u8> {
        let mut encoded = vec![];
        encoded.extend(encode_u256(value));
        let mut string_offset = vec![0u8; 32];
        string_offset.splice(24.., 64u64.to_be_bytes()); // Offset for the string
        encoded.extend(string_offset);
        encoded.extend(encode_string(&text));
        encoded
    }

    fn decode_u256(data: &[u8]) -> u128 {
        u128::from_be_bytes(data[16..32].try_into().unwrap())
    }

    fn decode_string(data: &[u8], offset: usize) -> String {
        let length_data = &data[offset..offset + 32];
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&length_data[24..32]); // Extract the last 8 bytes for the string length
        let string_length = u64::from_be_bytes(buffer) as usize;
        let string_data = &data[offset + 32..offset + 32 + string_length];
        String::from_utf8(string_data.to_vec()).expect("Invalid UTF-8 string")
    }

    pub(crate) fn abi_decode_u128_string(encoded: &[u8]) -> (u128, String) {
        let decoded_u256 = decode_u256(&encoded[0..32]);
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&encoded[56..64]); // Get the last 8 bytes for the string offset
        let string_offset = u64::from_be_bytes(buffer) as usize;
        let decoded_string = decode_string(&encoded, string_offset);
        (decoded_u256, decoded_string)
    }
}

#[account]
#[derive(Default, InitSpace)]
pub struct PingPongAccount {
    pub isend_cnt: u128,    // 16
    pub ireceive_cnt: u128, // 16
    pub iack_cnt: u128,     // 16
    #[max_len(50)]
    pub chain_id: String, // 4 + size = 54
    pub gateway_authority: Pubkey,
    pub pause: bool,             //1
    pub i_send_default_fee: u64, //8
    pub owner: Pubkey,           // 32
}

#[program]
pub mod ping_pong {
    use super::*;
    use std::str::FromStr;

    pub fn initialize(
        ctx: Context<Initialize>,
        chain_id: String,
        gateway_authority: Pubkey,
        i_send_default_fee: u64,
        owner: Pubkey,
    ) -> Result<()> {
        let dapp_account = &mut ctx.accounts.ping_pong_account;
        if Pubkey::from_str(AUTHORIZED_DEPLOYER).unwrap_or_default() != ctx.accounts.signer.key() {
            return Err(DappError::UnAuthorized.into());
        }
        dapp_account.isend_cnt = 0;
        dapp_account.iack_cnt = 0;
        dapp_account.ireceive_cnt = 0;
        dapp_account.chain_id = chain_id;
        dapp_account.gateway_authority = gateway_authority;
        dapp_account.owner = owner;
        dapp_account.pause = false;
        dapp_account.i_send_default_fee = i_send_default_fee;
        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Setter
    //////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn pause(ctx: Context<Execute>) -> Result<()> {
        let dapp_acc = &mut ctx.accounts.dapp_account;
        _when_not_pause(dapp_acc.clone())?;
        dapp_acc.pause = true;
        Ok(())
    }

    pub fn unpause(ctx: Context<Execute>) -> Result<()> {
        let dapp_acc = &mut ctx.accounts.dapp_account;
        _when_pause(dapp_acc.clone())?;
        dapp_acc.pause = false;
        Ok(())
    }

    pub fn update_owner(ctx: Context<Execute>, new_owner: Pubkey) -> Result<()> {
        let dapp_account: &mut Account<'_, PingPongAccount> = &mut ctx.accounts.dapp_account;
        _when_not_pause(dapp_account.clone())?;
        dapp_account.owner = new_owner;
        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////
    /// IDapp
    //////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn i_ping(
        ctx: Context<DappISend>,
        packet_seed: Vec<u8>,
        version: u128,
        route_amount: u64,
        route_recipient: String,
        dest_chain_id: String,
        dst_contract: Vec<u8>,
        request_metadata: Vec<u8>,
    ) -> Result<()> {
        let gateway_account = &ctx.accounts.gateway_account;
        let request_packet = &ctx.accounts.request_packet;
        let ping_pong_account = &mut ctx.accounts.ping_pong_account;
        _when_not_pause(ping_pong_account.clone())?;
        let system_program = &ctx.accounts.system_program;

        let op_signer_associate_account = &ctx.accounts.signer_associate_account;
        let mut signer_associate_account: Option<AccountInfo> = None;
        if op_signer_associate_account.is_some() {
            signer_associate_account = Some(
                op_signer_associate_account
                    .as_ref()
                    .unwrap()
                    .to_account_info(),
            );
        }
        let op_mint = &ctx.accounts.mint;
        let mut mint: Option<AccountInfo> = None;
        if op_mint.is_some() {
            mint = Some(op_mint.as_ref().unwrap().to_account_info());
        }
        let op_associated_token_program = &ctx.accounts.associated_token_program;
        let mut associated_token_program: Option<AccountInfo> = None;
        if op_associated_token_program.is_some() {
            associated_token_program = Some(
                op_associated_token_program
                    .as_ref()
                    .unwrap()
                    .to_account_info(),
            );
        }
        let op_token_program = &ctx.accounts.token_program;
        let mut token_program: Option<AccountInfo> = None;
        if op_token_program.is_some() {
            token_program = Some(op_token_program.as_ref().unwrap().to_account_info());
        }
        ping_pong_account.isend_cnt += 1;

        // initialize request packet account
        let payload = abi_encode_u128_string(
            ping_pong_account.isend_cnt,
            String::from("Hello From Solana"),
        );
        let len = (4 + dst_contract.len()) + (4 + payload.len());
        let mut output = vec![0u8; len];
        let mut offset = 0;
        output[offset..offset + 4].copy_from_slice(&(dst_contract.len() as u32).to_le_bytes());
        offset += 4;
        // set dst contract
        output[offset..offset + dst_contract.len()].copy_from_slice(&dst_contract);
        offset += dst_contract.len();

        // set payload length
        output[offset..offset + 4].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        offset += 4;
        output[offset..offset + payload.len()].copy_from_slice(&payload);
        gateway::cpi::initialize_packet_account(
            CpiContext::new(
                ctx.accounts.gateway_program.to_account_info(),
                gateway::cpi::accounts::InitializePacketAccount {
                    packet_account: ctx.accounts.request_packet.to_account_info(),
                    signer: ctx.accounts.signer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
            ),
            packet_seed.clone(),
            Some(output),
        )?;

        let bump = ctx.bumps.ping_pong_account;
        let binding = [PING_PONG_ACCOUNT, &[bump]];
        let signer_seeds = [binding.as_slice()];
        let isend_ctx = CpiContext::new_with_signer(
            ctx.accounts.gateway_program.to_account_info(),
            gateway::cpi::accounts::ISend {
                gateway_account: gateway_account.to_account_info(),
                request_packet: request_packet.to_account_info(),
                dapp_signer_account: ping_pong_account.to_account_info(),
                dapp_account: ctx.accounts.gateway_dapp_account.to_account_info(),
                signer: ctx.accounts.signer.to_account_info(),
                signer_associate_account,
                mint,
                associated_token_program,
                token_program,
                system_program: system_program.to_account_info(),
                event_authority: ctx.accounts.gateway_event_authority.to_account_info(),
                program: ctx.accounts.gateway_program.to_account_info(),
            },
            signer_seeds.as_slice(),
        );
        let request_id = gateway::cpi::i_send(
            isend_ctx,
            packet_seed,
            version,
            route_amount,
            route_recipient,
            dest_chain_id,
            request_metadata,
        )?
        .get();
        emit_cpi!(NewPing { request_id });
        Ok(())
    }

    pub fn set_dapp_metadata(ctx: Context<DappSetDappMetadata>, fee_payer: String) -> Result<()> {
        let gateway_account = &ctx.accounts.gateway_account;
        let dapp_acc = &ctx.accounts.ping_pong_account;
        _when_not_pause(*dapp_acc.clone())?;
        let signer = &mut ctx.accounts.signer;
        let system_program = &mut ctx.accounts.system_program;
        let bump = ctx.bumps.ping_pong_account;
        let binding = [PING_PONG_ACCOUNT, &[bump]];
        let signer_seeds = [binding.as_slice()];
        let set_dapp_metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.gateway_program.to_account_info(),
            gateway::cpi::accounts::SetDappMetadata {
                gateway_account: gateway_account.to_account_info(),
                dapp_signer_account: dapp_acc.to_account_info(),
                dapp_account: ctx.accounts.gateway_dapp_account.to_account_info(),
                fee_payer_account: signer.to_account_info(),
                system_program: system_program.to_account_info(),
                event_authority: ctx.accounts.gateway_event_authority.to_account_info(),
                program: ctx.accounts.gateway_program.to_account_info(),
            },
            signer_seeds.as_slice(),
        );
        gateway::cpi::set_dapp_metadata(
            set_dapp_metadata_ctx,
            fee_payer,
            ctx.program_id.key(),
            dapp_acc.key(),
        )?;
        Ok(())
    }

    pub fn i_receive(
        ctx: Context<DappIReceive>,
        _request_sender: String,
        src_chain_id: String,
    ) -> Result<Vec<u8>> {
        let ping_pong_account = &mut ctx.accounts.ping_pong_account;
        let packet_account = &ctx.accounts.packet_account.load()?;
        let (request_id, sample_str) = abi_decode_u128_string(
            &packet_account.get_packet_slice(0, packet_account.packet_len()),
        );

        ping_pong_account.ireceive_cnt += 1;
        emit_cpi!(PingFromSource {
            src_chain_id,
            request_id,
            sample_str
        });
        Ok(abi_encode_u128_string(
            ping_pong_account.ireceive_cnt,
            format!("Hello From Solana"),
        ))
    }

    pub fn i_ack(
        ctx: Context<DappIAck>,
        request_identifier: u128,
        exec_flag: bool,
    ) -> Result<Vec<u8>> {
        let ping_pong_acount = &mut ctx.accounts.ping_pong_account;
        let packet_account = ctx.accounts.packet_account.load()?;
        if !exec_flag {
            emit_cpi!(AckFromDestination {
                request_id: None,
                ack_message: None,
                request_identifier,
                exec_flag,
            });
            return Ok(vec![]);
        }
        let (request_id, ack_message) = abi_decode_u128_string(
            &packet_account.get_packet_slice(0, packet_account.packet_len()),
        );
        ping_pong_acount.iack_cnt += 1;
        emit_cpi!(AckFromDestination {
            request_id: Some(request_id),
            ack_message: Some(ack_message),
            request_identifier,
            exec_flag,
        });
        Ok(vec![])
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds = [PING_PONG_ACCOUNT], bump, payer = signer, space = 8 + PingPongAccount::INIT_SPACE)]
    pub ping_pong_account: Account<'info, PingPongAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(mut, seeds = [PING_PONG_ACCOUNT], bump)]
    pub dapp_account: Account<'info, PingPongAccount>,
    #[account(mut, constraint = signer.key() == dapp_account.owner)]
    pub signer: Signer<'info>,
}

#[error_code]
pub enum PausableError {
    #[msg("Paused")]
    Paused,
    #[msg("UnPaused")]
    UnPaused,
}

#[error_code]
pub enum DappError {
    #[msg("UnAuthorized")]
    UnAuthorized,
    #[msg("InSufficientBalance")]
    InSufficientBalance,
    #[msg("SolTransferFailed")]
    SolTransferFailed,
    #[msg("InvalidGatewayAccount")]
    InvalidGatewayAccount,
}

////////////////////////////////////////////////////////////////////////////////////
/// IDapp Accounts
////////////////////////////////////////////////////////////////////////////////////

#[event_cpi]
#[derive(Accounts)]
pub struct DappISend<'info> {
    #[account(mut, seeds = [PING_PONG_ACCOUNT], bump)]
    pub ping_pong_account: Account<'info, PingPongAccount>,
    #[account(mut)]
    pub gateway_account: Account<'info, GatewayAccount>,
    /// CHECK: _
    #[account(mut)]
    pub request_packet: UncheckedAccount<'info>,
    #[account()]
    pub gateway_dapp_account: Account<'info, GatewayDappAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub signer_associate_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        mut,
        constraint = gateway_account.route_token == mint.key()
    )]
    pub mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,
    pub token_program: Option<Interface<'info, TokenInterface>>,
    /// CHECK: _
    #[account(mut)]
    pub gateway_event_authority: UncheckedAccount<'info>,
    pub gateway_program: Program<'info, Gateway>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DappSetDappMetadata<'info> {
    #[account(mut, seeds = [PING_PONG_ACCOUNT], bump)]
    pub ping_pong_account: Box<Account<'info, PingPongAccount>>,
    #[account(mut, constraint = signer.key() == ping_pong_account.owner)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub gateway_program: Program<'info, Gateway>,
    #[account(mut)]
    pub gateway_account: Account<'info, GatewayAccount>,
    ///CHECK: _
    #[account(mut)]
    pub gateway_dapp_account: UncheckedAccount<'info>,
    ///CHECK: _
    #[account(mut)]
    gateway_event_authority: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct DappIAck<'info> {
    #[account(mut, seeds = [PING_PONG_ACCOUNT], bump)]
    pub ping_pong_account: Account<'info, PingPongAccount>,
    #[account()]
    pub packet_account: AccountLoader<'info, PacketAccount>,
    #[account(
        constraint = ping_pong_account.gateway_authority == gateway_authority.key()
    )]
    pub gateway_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct DappIReceive<'info> {
    #[account(mut, seeds = [PING_PONG_ACCOUNT], bump)]
    pub ping_pong_account: Box<Account<'info, PingPongAccount>>,
    #[account()]
    pub packet_account: AccountLoader<'info, PacketAccount>,
    #[account(
        constraint = ping_pong_account.gateway_authority == gateway_authority.key()
    )]
    pub gateway_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
