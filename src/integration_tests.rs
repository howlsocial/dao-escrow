#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg, WithdrawalReadyResponse, WithdrawalRequestedResponse,
    };
    use crate::state::Config;

    use cw20_base::msg::InstantiateMsg as CW20InstantiateMsg;

    use cosmwasm_std::{coins, Addr, BlockInfo, Coin, Empty, Uint128};
    use cw_multi_test::{
        next_block, App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor,
    };

    use cw20::{Cw20Coin, MinterResponse};

    pub fn escrow_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    //const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ujuno";
    const OVERRIDE_ADDRESS: &str = "override-dao-or-multisig-address";
    const WITHDRAW_ADDRESS: &str = "gordon-gekko-address";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(3_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn mock_instantiate(
        days: u64,
        withdraw_immutable: bool,
    ) -> (App, CwTemplateContract, Addr, CwTemplateContract, Addr) {
        let mut app = mock_app();
        let escrow_contract_id = app.store_code(escrow_contract_template());
        let cw20_id = app.store_code(contract_cw20());

        let withdraw_address = String::from(WITHDRAW_ADDRESS); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = days; // this is what we are expecting to set it to
        let override_address = String::from(OVERRIDE_ADDRESS);

        let msg = InstantiateMsg {
            set_withdraw_as_immutable: withdraw_immutable,
            enable_cw20_receive: false,
            override_address,
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        let escrow_contract_addr = app
            .instantiate_contract(
                escrow_contract_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(3_000_000),
                }], // set a contract balance
                "dao-escrow",
                None,
            )
            .unwrap();

        let escrow_contract = CwTemplateContract(escrow_contract_addr.clone());

        app.update_block(next_block);

        let cw20_instantiate_msg = CW20InstantiateMsg {
            symbol: "HOWL".to_string(),
            name: "Howl Token".to_string(),
            mint: Some(MinterResponse {
                minter: USER.to_string(),
                cap: None,
            }),
            marketing: None,
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: escrow_contract_addr.to_string(),
                amount: Uint128::new(5_000_000),
            }],
        };

        let cw20_contract_addr = app
            .instantiate_contract(
                cw20_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &cw20_instantiate_msg,
                &[],
                "cw20",
                None,
            )
            .unwrap();

        let cw20_contract = CwTemplateContract(cw20_contract_addr.clone());

        (
            app,
            escrow_contract,
            escrow_contract_addr,
            cw20_contract,
            cw20_contract_addr,
        )
    }

    fn _mock_instantiate_no_balance() -> (App, CwTemplateContract, Addr, CwTemplateContract, Addr) {
        let mut app = mock_app();
        let escrow_contract_id = app.store_code(escrow_contract_template());
        let cw20_id = app.store_code(contract_cw20());

        let withdraw_address = String::from(WITHDRAW_ADDRESS); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to
        let override_address = String::from("override-dao-or-multisig-address");

        let msg = InstantiateMsg {
            set_withdraw_as_immutable: true,
            enable_cw20_receive: false,
            override_address,
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        let escrow_contract_addr = app
            .instantiate_contract(
                escrow_contract_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &msg,
                &[],
                "dao-escrow",
                None,
            )
            .unwrap();

        let escrow_contract = CwTemplateContract(escrow_contract_addr.clone());

        app.update_block(next_block);

        let cw20_instantiate_msg = CW20InstantiateMsg {
            symbol: "HOWL".to_string(),
            name: "Howl Token".to_string(),
            mint: Some(MinterResponse {
                minter: USER.to_string(),
                cap: None,
            }),
            marketing: None,
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: escrow_contract_addr.to_string(),
                amount: Uint128::new(100),
            }],
        };

        let cw20_contract_addr = app
            .instantiate_contract(
                cw20_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &cw20_instantiate_msg,
                &[],
                "cw20",
                None,
            )
            .unwrap();

        let cw20_contract = CwTemplateContract(cw20_contract_addr.clone());

        (
            app,
            escrow_contract,
            escrow_contract_addr,
            cw20_contract,
            cw20_contract_addr,
        )
    }

    fn advance_one_day_one_hour(block: &mut BlockInfo) {
        let one_day_one_hour_in_seconds = 90_000;
        block.time = block.time.plus_seconds(one_day_one_hour_in_seconds);
        // av of 1 per 5s
        let blocks_to_advance = one_day_one_hour_in_seconds / 5;
        block.height += blocks_to_advance;
    }

    fn get_config(app: &mut App, contract_address: Addr) -> Result<Config, cosmwasm_std::StdError> {
        let msg = QueryMsg::GetConfig {};
        app.wrap().query_wasm_smart(contract_address, &msg)
    }

    fn is_withdrawal_ready(
        app: &mut App,
        contract_address: Addr,
    ) -> Result<WithdrawalReadyResponse, cosmwasm_std::StdError> {
        let msg = QueryMsg::IsWithdrawalReady {};
        app.wrap().query_wasm_smart(contract_address, &msg)
    }

    fn withdrawal_requested(
        app: &mut App,
        contract_address: Addr,
    ) -> Result<WithdrawalRequestedResponse, cosmwasm_std::StdError> {
        let msg = QueryMsg::GetWithdrawalRequested {};

        app.wrap().query_wasm_smart(contract_address, &msg)
    }

    fn get_balance(app: &mut App, address: &Addr) -> Vec<Coin> {
        app.wrap().query_all_balances(address).unwrap()
    }

    fn exec_override(
        app: &mut App,
        address: String,
        contract_address: Addr,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::OverrideWithdraw {};

        app.execute_contract(Addr::unchecked(address), contract_address, &msg, &[])
    }

    fn exec_update_override_address(
        app: &mut App,
        address: String,
        contract_address: Addr,
        new_address: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::UpdateOverrideAddress {
            address: new_address,
        };

        app.execute_contract(Addr::unchecked(address), contract_address, &msg, &[])
    }

    fn exec_update_withdraw_address(
        app: &mut App,
        address: String,
        contract_address: Addr,
        new_address: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::UpdateWithdrawalAddress {
            address: new_address,
        };

        app.execute_contract(Addr::unchecked(address), contract_address, &msg, &[])
    }

    fn start_native_withdraw(
        app: &mut App,
        address: String,
        contract_address: Addr,
        amount: Uint128,
        denom_or_address: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::StartWithdraw {
            amount,
            denom_or_address,
        };

        app.execute_contract(Addr::unchecked(address), contract_address, &msg, &[])
    }

    fn start_cw20_withdraw(
        app: &mut App,
        address: String,
        contract_address: Addr,
        amount: Uint128,
        cw20_contract_address: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::StartWithdraw {
            amount,
            denom_or_address: cw20_contract_address,
        };

        app.execute_contract(Addr::unchecked(address), contract_address, &msg, &[])
    }

    fn get_cw20_balance(app: &mut App, cw20_addr: Addr, address: String) -> Uint128 {
        let msg = cw20_base::msg::QueryMsg::Balance { address };
        let result: cw20::BalanceResponse = app.wrap().query_wasm_smart(cw20_addr, &msg).unwrap();
        result.balance
    }

    mod withdraw {
        use super::*;

        #[test]
        fn start_withdraw_native() {
            let (mut app, _cw_template_contract, contract_addr, _, _) = mock_instantiate(28, true);

            let withdraw_address = String::from(WITHDRAW_ADDRESS);
            let _validated_addr = Addr::unchecked(&withdraw_address);

            start_native_withdraw(
                &mut app,
                withdraw_address,
                contract_addr.clone(),
                Uint128::new(2_000_000),
                NATIVE_DENOM.to_string(),
            )
            .unwrap();

            let withdrawal_ready = is_withdrawal_ready(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: false,
                }
            );

            let withdrawal_requested = withdrawal_requested(&mut app, contract_addr).unwrap();

            assert_eq!(
                withdrawal_requested,
                WithdrawalRequestedResponse {
                    withdrawal_requested: true,
                }
            );
        }

        #[test]
        fn start_withdraw_then_claim_native() {
            let (mut app, cw_template_contract, contract_addr, _, cw20_contract_addr) =
                mock_instantiate(1, true);

            let withdraw_address = String::from(WITHDRAW_ADDRESS);
            let validated_addr = Addr::unchecked(&withdraw_address);

            let withdrawal_requested_res =
                withdrawal_requested(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res,
                WithdrawalRequestedResponse {
                    withdrawal_requested: false,
                }
            );

            start_native_withdraw(
                &mut app,
                withdraw_address.clone(),
                contract_addr.clone(),
                Uint128::new(2_000_000),
                NATIVE_DENOM.to_string(),
            )
            .unwrap();

            let withdrawal_ready = is_withdrawal_ready(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: false,
                }
            );

            // move time forward
            app.update_block(advance_one_day_one_hour);

            // should be ready
            let withdrawal_ready_try_two =
                is_withdrawal_ready(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready_try_two,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: true,
                }
            );

            let withdrawal_requested_res_two =
                withdrawal_requested(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res_two,
                WithdrawalRequestedResponse {
                    withdrawal_requested: true,
                }
            );

            //now claim
            let claim_msg = ExecuteMsg::ExecuteNativeWithdraw {
                amount: Uint128::new(2_000_000),
                denom: NATIVE_DENOM.to_string(),
            };
            let claim_msg_res = cw_template_contract.call(claim_msg).unwrap();
            app.execute(validated_addr.clone(), claim_msg_res).unwrap();

            // contract balance should NOT be zero
            let contract_balance = get_balance(&mut app, &contract_addr);
            assert_eq!(contract_balance, coins(1_000_000, NATIVE_DENOM));

            // withdrawer should have 2/3 balance
            let withdrawer_balance = get_balance(&mut app, &validated_addr);
            assert_eq!(withdrawer_balance, coins(2_000_000, NATIVE_DENOM));

            // check cw20 balances are unaffected
            let escrow_contract_cw20_balance = get_cw20_balance(
                &mut app,
                cw20_contract_addr.clone(),
                contract_addr.to_string(),
            );
            assert_eq!(escrow_contract_cw20_balance, Uint128::new(5_000_000));

            // withdrawer should have no balance
            let withdrawer_balance =
                get_cw20_balance(&mut app, cw20_contract_addr, withdraw_address);
            assert_eq!(withdrawer_balance, Uint128::new(0));
        }

        #[test]
        fn start_withdraw_then_override_native() {
            let (mut app, cw_template_contract, contract_addr, _, cw20_contract_addr) =
                mock_instantiate(1, true);

            let withdraw_address = String::from(WITHDRAW_ADDRESS);
            let validated_addr = Addr::unchecked(&withdraw_address);

            let withdrawal_requested_res =
                withdrawal_requested(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res,
                WithdrawalRequestedResponse {
                    withdrawal_requested: false,
                }
            );

            start_native_withdraw(
                &mut app,
                withdraw_address.clone(),
                contract_addr.clone(),
                Uint128::new(2_000_000),
                NATIVE_DENOM.to_string(),
            )
            .unwrap();

            let withdrawal_ready = is_withdrawal_ready(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: false,
                }
            );

            // move to a time when withdrawal has been registered
            app.update_block(next_block);

            let withdrawal_requested_res_two =
                withdrawal_requested(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res_two,
                WithdrawalRequestedResponse {
                    withdrawal_requested: true,
                }
            );

            // oh no! this sucks, we hit the override button
            exec_override(
                &mut app,
                OVERRIDE_ADDRESS.to_string(),
                contract_addr.clone(),
            )
            .unwrap();

            // move time forward to claim time
            app.update_block(advance_one_day_one_hour);

            // withdrawal should not exist
            let withdrawal_requested_res_three =
                withdrawal_requested(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res_three,
                WithdrawalRequestedResponse {
                    withdrawal_requested: false,
                }
            );

            // should NOT be ready, hence error
            let _withdrawal_ready_try_two =
                is_withdrawal_ready(&mut app, contract_addr.clone()).unwrap_err();

            // now claim
            // this will also error
            let claim_msg = ExecuteMsg::ExecuteNativeWithdraw {
                amount: Uint128::new(2_000_000),
                denom: NATIVE_DENOM.to_string(),
            };
            let claim_msg_res = cw_template_contract.call(claim_msg).unwrap();
            app.execute(validated_addr.clone(), claim_msg_res)
                .unwrap_err();

            // contract balance should be full
            let contract_balance = get_balance(&mut app, &contract_addr);
            assert_eq!(contract_balance, coins(3_000_000, NATIVE_DENOM));

            // withdrawer should have no balance
            let withdrawer_balance = get_balance(&mut app, &validated_addr);
            assert_eq!(withdrawer_balance, &[]);

            // check cw20 balances are unaffected
            let escrow_contract_cw20_balance = get_cw20_balance(
                &mut app,
                cw20_contract_addr.clone(),
                contract_addr.to_string(),
            );
            assert_eq!(escrow_contract_cw20_balance, Uint128::new(5_000_000));

            // withdrawer should have no balance
            let withdrawer_balance =
                get_cw20_balance(&mut app, cw20_contract_addr, withdraw_address);
            assert_eq!(withdrawer_balance, Uint128::new(0));
        }

        #[test]
        fn start_withdraw_then_claim_cw20() {
            let (
                mut app,
                cw_template_contract,
                escrow_contract_addr,
                _cw20_contract,
                cw20_contract_addr,
            ) = mock_instantiate(1, true);

            let withdraw_address = String::from(WITHDRAW_ADDRESS);
            let validated_addr = Addr::unchecked(&withdraw_address);

            let withdrawal_requested_res =
                withdrawal_requested(&mut app, escrow_contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res,
                WithdrawalRequestedResponse {
                    withdrawal_requested: false,
                }
            );

            start_cw20_withdraw(
                &mut app,
                withdraw_address.clone(),
                escrow_contract_addr.clone(),
                Uint128::new(3_000_000),
                cw20_contract_addr.to_string(),
            )
            .unwrap();

            let withdrawal_ready =
                is_withdrawal_ready(&mut app, escrow_contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: false,
                }
            );

            // move time forward
            app.update_block(advance_one_day_one_hour);

            // should be ready
            let withdrawal_ready_try_two =
                is_withdrawal_ready(&mut app, escrow_contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_ready_try_two,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: true,
                }
            );

            let withdrawal_requested_res_two =
                withdrawal_requested(&mut app, escrow_contract_addr.clone()).unwrap();

            assert_eq!(
                withdrawal_requested_res_two,
                WithdrawalRequestedResponse {
                    withdrawal_requested: true,
                }
            );

            //now claim
            let claim_msg = ExecuteMsg::ExecuteCW20Withdraw {
                amount: Uint128::new(3_000_000),
                address: cw20_contract_addr.to_string(),
            };
            let claim_msg_res = cw_template_contract.call(claim_msg).unwrap();
            app.execute(validated_addr.clone(), claim_msg_res).unwrap();

            // contract balance should NOT be zero
            let escrow_contract_cw20_balance = get_cw20_balance(
                &mut app,
                cw20_contract_addr.clone(),
                escrow_contract_addr.to_string(),
            );
            assert_eq!(escrow_contract_cw20_balance, Uint128::new(2_000_000));

            // withdrawer should have a balance
            let withdrawer_balance =
                get_cw20_balance(&mut app, cw20_contract_addr, withdraw_address);
            assert_eq!(withdrawer_balance, Uint128::new(3_000_000));

            // messing around with cw20 should not have affected native
            let contract_balance = get_balance(&mut app, &escrow_contract_addr);
            assert_eq!(contract_balance, coins(3_000_000, NATIVE_DENOM));

            // withdrawer should have no balance
            let withdrawer_balance = get_balance(&mut app, &validated_addr);
            assert_eq!(withdrawer_balance, &[]);
        }

        #[test]
        fn start_claim_no_withdraw() {
            let (mut app, cw_template_contract, _, _, _) = mock_instantiate(1, true);

            let withdraw_address = String::from(WITHDRAW_ADDRESS);
            let validated_addr = Addr::unchecked(&withdraw_address);

            let claim_msg = ExecuteMsg::ExecuteNativeWithdraw {
                amount: Uint128::new(2_000_000),
                denom: NATIVE_DENOM.to_string(),
            };
            let claim_msg_res = cw_template_contract.call(claim_msg).unwrap();
            app.execute(validated_addr, claim_msg_res).unwrap_err();
        }

        #[test]
        fn start_withdraw_fails_with_wrong_address() {
            let (mut app, _cw_template_contract, contract_addr, _, _) = mock_instantiate(28, true);

            // have a go at withdrawing as an address
            // that isn't the withdraw addr
            start_native_withdraw(
                &mut app,
                "some-random-address".to_string(),
                contract_addr,
                Uint128::new(2_000_000),
                NATIVE_DENOM.to_string(),
            )
            .unwrap_err(); // this should fail
        }

        #[test]
        fn change_override_address() {
            let (mut app, _cw_template_contract, contract_addr, _, _) = mock_instantiate(28, true);
            let new_address = "some-random-address";

            // if a random tries to change it, then failtown
            exec_update_override_address(
                &mut app,
                new_address.to_string(),
                contract_addr.clone(),
                new_address.to_string(),
            )
            .unwrap_err();

            let config = get_config(&mut app, contract_addr.clone()).unwrap();

            assert_eq!(config.override_address, OVERRIDE_ADDRESS);

            // but this is legit
            exec_update_override_address(
                &mut app,
                OVERRIDE_ADDRESS.to_string(),
                contract_addr.clone(),
                new_address.to_string(),
            )
            .unwrap();

            let config_two = get_config(&mut app, contract_addr).unwrap();
            assert_eq!(config_two.override_address, Addr::unchecked(new_address));
        }

        #[test]
        fn change_withdraw_address() {
            let (mut app, _cw_template_contract, contract_addr, _, _) = mock_instantiate(28, false);
            let new_address = "some-random-address";

            // should still fail if some random calls it
            exec_update_withdraw_address(
                &mut app,
                new_address.to_string(),
                contract_addr.clone(),
                new_address.to_string(),
            )
            .unwrap_err();

            let config = get_config(&mut app, contract_addr.clone()).unwrap();
            assert_eq!(config.withdraw_address, WITHDRAW_ADDRESS);

            exec_update_withdraw_address(
                &mut app,
                OVERRIDE_ADDRESS.to_string(),
                contract_addr.clone(),
                new_address.to_string(),
            )
            .unwrap();

            let config_two = get_config(&mut app, contract_addr).unwrap();
            assert_eq!(config_two.withdraw_address, Addr::unchecked(new_address));
        }

        #[test]
        fn change_withdraw_address_fails_if_immutable() {
            let (mut app, _cw_template_contract, contract_addr, _, _) = mock_instantiate(28, true);

            // this will error
            exec_update_withdraw_address(
                &mut app,
                OVERRIDE_ADDRESS.to_string(),
                contract_addr.clone(),
                "some-random-address".to_string(),
            )
            .unwrap_err();

            // check it's unchanged
            let config = get_config(&mut app, contract_addr).unwrap();
            assert_eq!(config.withdraw_address, WITHDRAW_ADDRESS);
        }
    }
}
