pub mod cpi {
    use crate::types::types::Result;
    use crate::types::Bytes;
    use ink::prelude::vec;
    use ink::{
        env::{
            call::{build_call, ExecutionInput, Selector},
            DefaultEnvironment,
        },
        primitives::AccountId,
    };
    const GATEWAY_I_SEND: [u8; 4] = [76, 39, 39, 59]; // 0x4c27273b
    const GATEWAY_SET_DAPP_METADATA: [u8; 4] = [61, 32, 223, 81]; // 0x3d20df51

    //  https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md
    const TRANSFER_FROM_SELECTOR: [u8; 4] = [84, 179, 199, 110]; // 0x54b3c76e;
    const APPROVE_SELECTOR: [u8; 4] = [178, 15, 27, 189]; // 0xb20f1bbd

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///  Cross Contract Call
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    /// internal function `_call_i_ack` is used to call the `i_ack` function on a dapp handler account and
    /// returns the execution data and status.
    pub(crate) fn _call_i_send(
        handler_acc_id: AccountId,
        transferred_value: u128,
        version: u128,
        route_amount: u128,
        route_recipient: Bytes,
        dest_chain_id: Bytes,
        request_metadata: Bytes,
        request_packet: Bytes,
    ) -> u128 {
        build_call::<DefaultEnvironment>()
            .call(handler_acc_id)
            .transferred_value(transferred_value)
            .exec_input(
                ExecutionInput::new(Selector::from(GATEWAY_I_SEND))
                    .push_arg::<u128>(version) // version
                    .push_arg::<u128>(route_amount) //route_amount
                    .push_arg::<&Bytes>(&route_recipient) // route_recipient
                    .push_arg::<&Bytes>(&dest_chain_id) //dest_chain_id
                    .push_arg::<&Bytes>(&request_metadata) //request_metadata
                    .push_arg::<&Bytes>(&request_packet), //request_packet
            )
            .returns::<u128>()
            .invoke()
    }

    pub(crate) fn _call_set_dapp_metadata(
        handler_acc_id: AccountId,
        transferred_value: u128,
        fee_payer_address: Bytes,
    ) -> () {
        build_call::<DefaultEnvironment>()
            .call(handler_acc_id)
            .transferred_value(transferred_value)
            .exec_input(
                ExecutionInput::new(Selector::from(GATEWAY_SET_DAPP_METADATA))
                    .push_arg::<&Bytes>(&fee_payer_address), // fee_payer_address
            )
            .returns::<()>()
            .invoke()
    }

    #[allow(dead_code)]
    pub(crate) fn _transfer_from(
        token: AccountId,
        from: AccountId,
        to: AccountId,
        amount: u128,
        data: &[u8],
    ) -> Result<()> {
        Ok(build_call::<DefaultEnvironment>()
            .call(token)
            .exec_input(
                ExecutionInput::new(Selector::from(TRANSFER_FROM_SELECTOR))
                    .push_arg::<&AccountId>(&from)
                    .push_arg::<&AccountId>(&to)
                    .push_arg::<u128>(amount)
                    .push_arg::<&[u8]>(data),
            )
            .returns::<()>()
            .invoke())
    }

    #[allow(dead_code)]
    pub(crate) fn _approve(token: AccountId, to: AccountId, amount: u128) -> Result<()> {
        Ok(build_call::<DefaultEnvironment>()
            .call(token)
            .exec_input(
                ExecutionInput::new(Selector::from(APPROVE_SELECTOR))
                    .push_arg::<&AccountId>(&to)
                    .push_arg::<u128>(amount),
            )
            .returns::<()>()
            .invoke())
    }
}

pub use cpi::*;
