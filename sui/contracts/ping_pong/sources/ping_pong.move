module ping_pong::ping_pong {
    use gateway::gateway_contract;
    use std::string::{Self,  String };
    use sui::sui::SUI;
    use sui::coin::Coin;
    use sui::transfer::Receiving;
    use route_token::route;
    use sui::event;
    use router_protocol::{writer, reader};

    /// Errors
    const ErrOnlyOwner: u64 = 0;
    const MetaNotExist: u64 = 1;
    const ERR_PAUSED: u64 = 2;
    const ERR_NotPAUSED: u64 = 3;
    const ONLY_OWNER: u64 = 4;

    /// Events
    public struct NewPing has drop, copy {
        request_id: u256,
    }

    public struct AckFromDestination has drop, copy {
        request_id: u256,
        ack_message: String,
        request_identifier: u256,
        exec_flag: bool
    }

    public struct PingFromSource has drop, copy {
        src_chain_id: String,
        request_id: u256,
        sample_str: String,
    }

    public struct Paused has drop, copy {}
    public struct UnPaused has drop, copy {}


    /// State 
    public struct PingPongContract has key, store {
        id: UID,
        i_send_cnt: u256,    
        i_receive_cnt: u256, 
        i_ack_cnt : u256,     
        chain_id: String, 
        pause: bool,             
        owner: address,
        metadata: option::Option<gateway_contract::Metadata>
    }

    fun init(
        ctx: &mut TxContext
    ) {
        let idx = object::new(ctx);
        let dapp = PingPongContract {
            id: idx,
            i_send_cnt: 0,
            i_receive_cnt:0 ,
            i_ack_cnt:0,
            chain_id: string::utf8(vector[]),
            pause: false,
            owner: tx_context::sender(ctx),
            metadata: option::none()
        };
        gateway_contract::new_metadata(
            object::uid_to_address(&dapp.id),
            ctx
        );
        transfer::share_object(dapp)
    }

    public entry fun initialize(
        self: &mut PingPongContract,
        chain_id: String,
        sent: Receiving<gateway_contract::Metadata>,
        _ctx: &mut TxContext
    ) {
        let metadata: gateway_contract::Metadata = transfer::public_receive(&mut self.id, sent);
        option::fill(&mut self.metadata, metadata);
        self.chain_id = chain_id
    }

    public entry fun i_ping(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        version: u128,
        route_amount: u256,
        route_recipient: String,
        dest_chain_id: String,
        dst_contract: String,
        request_metadata: vector<u8>,
        ctx: &mut TxContext
    ) {
        when_not_paused_(self);
        self.i_send_cnt = self.i_send_cnt + 1;

        // encode(u256, string)
        let packet = get_dst_packet_(self.i_send_cnt);

        // encode(string, bytes) to get request packet
        let mut request_packet = writer::new_writer(2);
        request_packet.write_string(dst_contract);
        request_packet.write_bytes(packet);

        let mut option_coin = option::none();
        let mut none_route = option::none();
        let request_id = i_send_(
            self,
            gateway_contract_obj,
            version,
            route_amount,
            route_recipient,
            dest_chain_id,
            request_metadata,
            request_packet.into_bytes(),
            &mut option_coin,
            &mut none_route,
            ctx
        );
        option::destroy_none(none_route);
        option::destroy_none(option_coin);
        event::emit(
            NewPing {
                request_id,
            }
        )
    }

    // SET DAPP METADATA
    public entry fun set_dapp_metadata(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        dapp_module_address: address,
        fee_payer_address: String,
        ctx: &mut TxContext
    ) {
        let mut option_coin = option::none();
        set_dapp_metadata_(
            self,
            gateway_contract_obj,
            dapp_module_address,
            fee_payer_address,
            &mut option_coin,
            ctx
        );
        option::destroy_none(option_coin)
    }

    public entry fun i_receive(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        sent: Receiving<gateway_contract::ExecuteDappIReceive>,
        ctx: &mut TxContext
    ) {
        let execute_dapp: gateway_contract::ExecuteDappIReceive = transfer::public_receive(
            &mut self.id, sent
        );
        let (_request_sender, packet, src_chain_id) = gateway_contract::get_i_receive_args(&execute_dapp);
        let reader = reader::new_reader(packet);
        let request_id = reader::read_u256(&reader, 0);
        let sample_str = string::utf8(reader::read_bytes(&reader, 1));

        self.i_receive_cnt = self.i_receive_cnt + 1;
        event::emit(
            PingFromSource {
                src_chain_id,
                request_id,
                sample_str
            }
        );
        gateway_contract::executed_i_receive_dapp(
            gateway_contract_obj,
            execute_dapp,
            // encode(u256, string)
             get_dst_packet_(self.i_receive_cnt),
            true,
            ctx
        )
    }

    public entry fun i_ack(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        sent: Receiving<gateway_contract::ExecuteDappIAck>,
        _ctx: &mut TxContext
    ) {
        let execute_dapp: gateway_contract::ExecuteDappIAck = transfer::public_receive(
            &mut self.id, sent
        );
        let (request_identifier, exec_flag, exec_data) = gateway_contract::get_i_ack_args(&execute_dapp);
        if(!exec_flag) {
            event::emit(
            AckFromDestination {
                    request_id: 0,
                    ack_message: string::utf8(vector[]),
                    request_identifier,
                    exec_flag,
                }
            );
            return gateway_contract::executed_i_ack_dapp(
                gateway_contract_obj,
                execute_dapp,
                vector[],
                true
            )
        };
        let reader = reader::new_reader(exec_data);
        let request_id = reader::read_u256(&reader, 0);
        let ack_message = string::utf8(reader::read_bytes(&reader, 1));
        self.i_ack_cnt = self.i_ack_cnt + 1;

        event::emit(
        AckFromDestination {
                request_id,
                ack_message,
                request_identifier,
                exec_flag,
            }
        );
        gateway_contract::executed_i_ack_dapp(
            gateway_contract_obj,
            execute_dapp,
            vector[],
            true
        )
    }

    public entry fun pause(self: &mut PingPongContract, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == self.owner, ONLY_OWNER);
        when_not_paused_(self);
        self.pause = true;
        event::emit(Paused {})
    }

    public entry fun unpause(self: &mut PingPongContract, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == self.owner, ONLY_OWNER);
        when_paused_(self);
        self.pause = false;
        event::emit(UnPaused {})
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///  Internal
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    fun i_send_(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        version: u128,
        route_amount: u256,
        route_recipient: String,
        dest_chain_id: String,
        request_metadata: vector<u8>,
        request_packet: vector<u8>,
        coin: &mut option::Option<Coin<SUI>>,
        route: &mut option::Option<Coin<route::ROUTE>>,
        ctx: &mut TxContext
    ): u256 {
        gateway_contract::i_send(
            gateway_contract_obj,
            option::borrow_mut(&mut self.metadata),
            version,
            route_amount,
            route_recipient,
            dest_chain_id,
            request_metadata,
            request_packet,
            coin,
            route,
            ctx
        )
    }

    fun set_dapp_metadata_(
        self: &mut PingPongContract,
        gateway_contract_obj: &mut gateway_contract::GatewayContract,
        dapp_module_address: address,
        fee_payer_address: String,
        coin: &mut option::Option<Coin<SUI>>,
        ctx: &mut TxContext
    ) {
        assert!(
            tx_context::sender(ctx) == self.owner,
            ErrOnlyOwner
        );
        assert!(
            option::is_some(&self.metadata),
            MetaNotExist
        );
        gateway_contract::set_dapp_metadata(
            gateway_contract_obj,
            option::borrow_mut(&mut self.metadata),
            dapp_module_address,
            object::uid_to_address(&self.id),
            fee_payer_address,
            coin,
            ctx
        )
    }


    fun when_not_paused_(self: &PingPongContract) {
        assert!(!self.pause, ERR_PAUSED)
    } 
    
    fun when_paused_(self: &PingPongContract) {
        assert!(self.pause, ERR_NotPAUSED)
    } 

    fun get_dst_packet_(cnt: u256) : vector<u8> {
        let mut writer = writer::new_writer(2);
        writer.write_u256(cnt);
        writer.write_string(string::utf8(vector[72, 101, 108, 108, 111, 32,  70, 114, 111, 109, 32,  83, 117, 105])); // Hello From Sui
        writer.into_bytes()
    } 
}
