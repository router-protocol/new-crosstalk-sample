#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod types;

#[ink::contract]
pub mod rate_limit {
    use crate::types::types::Result;
    use crate::types::Bytes;

    #[ink(storage)]
    pub struct RateLimit {
        pub time_period: u64,
        pub limit: u128,
        pub tx_count: u128,
        pub current_period_end: u64,
        pub gateway_contract: AccountId,
        pub owner: AccountId,
    }

    impl RateLimit {
        #[ink(constructor)]
        pub fn new(gateway_contract: AccountId, time_period: u64, limit: u128) -> Self {
            Self {
                gateway_contract,
                time_period,
                limit,
                tx_count: 0,
                current_period_end: Self::env().block_timestamp(),
                owner: Self::env().caller(),
            }
        }

        #[ink(message, selector = 0xb5cd5fd1)]
        pub fn verify_cross_chain_request(
            &mut self,
            _request_identifier: u128,
            _request_timestamp: u128,
            _request_sender: Bytes,
            _src_chain_id: Bytes,
            _packet: Bytes,
            _handler_address: Bytes,
        ) -> bool {
            let res = self.only_gateway();
            if res.is_err() {
                panic!("{:?}", res.err());
            }
            // TODO: add your business logic here
            self.execute_rate_limiting()
        }

        //////////////////////////////////////////////////////////////////////////////////////////////////////
        /// Internal
        //////////////////////////////////////////////////////////////////////////////////////////////////////

        fn execute_rate_limiting(&mut self) -> bool {
            if self.env().block_timestamp() >= self.current_period_end {
                self.current_period_end = self
                    .env()
                    .block_timestamp()
                    .checked_add(self.time_period)
                    .unwrap();
                self.tx_count = 0;
            }
            if self.tx_count < self.limit {
                self.tx_count = self.tx_count.checked_add(1).unwrap();
                true
            } else {
                false
            }
        }

        //////////////////////////////////////////////////////////////////////////////////////////////////////
        /// Setter
        //////////////////////////////////////////////////////////////////////////////////////////////////////

        #[ink(message)]
        pub fn update_limit(&mut self, limit: u128) -> Result<()> {
            self.only_owner()?;
            self.limit = limit;
            Ok(())
        }

        #[ink(message)]
        pub fn update_time_period(&mut self, time_period: u64) -> Result<()> {
            self.only_owner()?;
            self.time_period = time_period;
            Ok(())
        }

        //////////////////////////////////////////////////////////////////////////////////////////////////////
        /// Getter
        //////////////////////////////////////////////////////////////////////////////////////////////////////

        #[ink(message)]
        pub fn get_update_limit(&self) -> u128 {
            self.limit
        }

        #[ink(message)]
        pub fn get_time_period(&self) -> u64 {
            self.time_period
        }

        //////////////////////////////////////////////////////////////////////////////////////////////////////
        /// Modifiers
        //////////////////////////////////////////////////////////////////////////////////////////////////////

        fn only_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {}
            Ok(())
        }

        fn only_gateway(&self) -> Result<()> {
            if self.env().caller() != self.gateway_contract {}
            Ok(())
        }
    }
}
