#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{
        from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, MessageInfo, Timestamp,
        Uint128, Uint256, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use cw_utils::PaymentError;

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryConfigResponse, QueryMsg, QueryPositionResponse,
        QueryStateResponse,
    };
    use crate::state::Batch;
    use crate::ContractError;

    fn default_init_msg() -> InstantiateMsg {
        let first_batch_release_time = Timestamp::from_nanos(100000000000);
        InstantiateMsg {
            admin: None,
            batch_duration: Uint128::from(100u128),
            batch_amount: Uint128::from(10u128),
            revenue_collector: "revenue_collector".to_string(),
            price: Decimal::from_str("0.1").unwrap(),
            buy_denom: "ujuno".to_string(),
            sell_denom: "token".to_string(),
            first_batch_release_time: first_batch_release_time,
        }
    }
    #[test]
    pub fn test_proper_init() {
        let mut deps = mock_dependencies();
        let init_msg = default_init_msg();
        let info = mock_info("creator", &[]);
        // instantiate without admin
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!("action".to_string(), res.attributes[0].key);
        assert_eq!("instantiate".to_string(), res.attributes[0].value);
        assert_eq!("admin".to_string(), res.attributes[1].key);
        assert_eq!("creator".to_string(), res.attributes[1].value);
        assert_eq!("batch_duration".to_string(), res.attributes[2].key);
        assert_eq!("100".to_string(), res.attributes[2].value);
        assert_eq!("batch_amount".to_string(), res.attributes[3].key);
        assert_eq!("10".to_string(), res.attributes[3].value);
        assert_eq!("revenue_collector".to_string(), res.attributes[4].key);
        assert_eq!("revenue_collector".to_string(), res.attributes[4].value);
        assert_eq!("price".to_string(), res.attributes[5].key);
        assert_eq!("0.1".to_string(), res.attributes[5].value);
        assert_eq!("buy_denom".to_string(), res.attributes[6].key);
        assert_eq!("ujuno".to_string(), res.attributes[6].value);
        assert_eq!("sell_denom".to_string(), res.attributes[7].key);
        assert_eq!("token".to_string(), res.attributes[7].value);
        assert_eq!(
            "first_batch_release_time".to_string(),
            res.attributes[8].key
        );
        assert_eq!(
            Timestamp::from_seconds(100).to_string(),
            res.attributes[8].value
        );
        // instantiate with admin
        let mut init_msg = default_init_msg();
        init_msg.admin = Some("admin".to_string());
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!("admin".to_string(), res.attributes[1].key);
        assert_eq!("admin".to_string(), res.attributes[1].value);
    }

    #[test]
    pub fn test_buy() {
        // instantiate
        let mut deps = mock_dependencies();
        let init_msg = default_init_msg();
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

        // Try buying before sale start
        let info = mock_info("buyer", &[]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(ContractError::SaleNotActive {}, res);
        // acticate sale
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::StartSale {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try buying with without funds
        let info = mock_info("buyer", &[]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(ContractError::Payment(PaymentError::NoFunds {}), res);

        // Try buying with wrong denom
        let info = mock_info("buyer", &[Coin::new(10, "wrong_denom")]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            ContractError::Payment(PaymentError::MissingDenom(("ujuno".to_string()))),
            res
        );

        // Try buying with multiple coins
        let info = mock_info("buyer", &[Coin::new(10, "ujuno"), Coin::new(10, "ujuno2")]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(ContractError::Payment(PaymentError::MultipleDenoms {}), res);

        // Happy path
        let info = mock_info("buyer", &[Coin::new(10, "ujuno")]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        // check if the payment was sent to revenue collector
        assert_eq!(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "revenue_collector".to_string(),
                amount: vec![Coin::new(10, "ujuno")],
            }),
            res.messages[0].msg
        );
        // check state
        let state: QueryStateResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::QueryState {}).unwrap())
                .unwrap();
        assert_eq!(state.total_revenue, Uint128::from(10u128));
        assert_eq!(state.total_sold, Uint128::from(100u128));

        // check position
        let position: QueryPositionResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::QueryPosition {
                    address: "buyer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(position.total_bought, Uint128::from(100u128));
        assert_eq!(position.total_paid, Uint128::from(10u128));
        assert_eq!(position.address, "buyer".to_string());
        let expected_batches: Vec<Batch> = [
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000000),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000100),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000200),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000300),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000400),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000500),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000600),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000700),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000800),
                released: false,
            },
            Batch {
                amount: 10u128.into(),
                release_time: Timestamp::from_nanos(100000000900),
                released: false,
            },
        ]
        .to_vec();
        assert_eq!(position.batches, expected_batches);
        // Now User has 10 batches of 10 cw20 tokens each
        // First batch will be released at plus 100 seconds(100000000000 nanoseconds)
        // Second batch will be released at plus 100.0000001 seconds(100000000100 nanoseconds)

        // Try buying again
        let info = mock_info("buyer", &[Coin::new(877, "ujuno")]);
        let msg = ExecuteMsg::Buy {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        // check if the payment was sent to revenue collector
        assert_eq!(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "revenue_collector".to_string(),
                amount: vec![Coin::new(877, "ujuno")],
            }),
            res.messages[0].msg
        );
        // check state
        let state: QueryStateResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::QueryState {}).unwrap())
                .unwrap();
        assert_eq!(state.total_revenue, Uint128::from(887u128));
        assert_eq!(state.total_sold, Uint128::from(100u128 + 8770u128));

        // check position
        let position: QueryPositionResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::QueryPosition {
                    address: "buyer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(position.total_bought, Uint128::from(100u128 + 8770u128));
        assert_eq!(position.total_paid, Uint128::from(887u128));
        assert_eq!(position.address, "buyer".to_string());
        // Now User has still 10 batches of 10 cw20 tokens each
        assert_eq!(position.batches.len(), 10);
        assert_eq!(position.batches[0].amount, Uint128::from(887u128));

        // start distribution
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::StartDistribution {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // try claiming
        let mut env = mock_env();
        // First batch should be released
        env.block.time = Timestamp::from_nanos(100000000000 + 1);
        let info = mock_info("buyer", &[]);
        let msg = ExecuteMsg::Claim {};
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        // check if the cw20 tokens were sent to user
        assert_eq!(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "buyer".to_string(),
                    amount: Uint128::from(887u128),
                })
                .unwrap(),
                funds: vec![],
            }),
            res.messages[0].msg
        );
        // check batches
        let position: QueryPositionResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::QueryPosition {
                    address: "buyer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        // Now User has 9 batches of cw20 tokens
        assert_eq!(position.batches.len(), 9);
        assert_eq!(position.total_claimed, Uint128::from(887u128));
        assert_eq!(position.batches[0].amount, Uint128::from(887u128));
        assert_eq!(
            position.batches[0].release_time,
            Timestamp::from_nanos(100000000100)
        );

        // now try claiming everything left
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(100000000900 + 1);
        let info = mock_info("buyer", &[]);
        let msg = ExecuteMsg::Claim {};
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        // check if the cw20 tokens were sent to user
        assert_eq!(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "buyer".to_string(),
                    amount: Uint128::from(8870u128 - 887u128),
                })
                .unwrap(),
                funds: vec![],
            }),
            res.messages[0].msg
        );
        // check batches
        let position: QueryPositionResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::QueryPosition {
                    address: "buyer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        // Now User has 0 batches of cw20 tokens
        assert_eq!(position.batches.len(), 0);
        assert_eq!(position.total_claimed, Uint128::from(8870u128));
    }

    #[test]
    pub fn test_update_config() {
        // init
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let msg = default_init_msg();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // random update config
        let info = mock_info("random", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            admin: None,
            batch_duration: Some(Uint128::from(12u128)),
            revenue_collector: None,
            buy_denom: None,
            sell_denom: None,
            batch_amount: None,
            price: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // update config
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            admin: None,
            batch_duration: Some(Uint128::from(12u128)),
            revenue_collector: None,
            buy_denom: None,
            sell_denom: None,
            batch_amount: None,
            price: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        // check config
        let config: QueryConfigResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::QueryConfig {}).unwrap())
                .unwrap();
        assert_eq!(config.batch_duration, Uint128::from(12u128));

        // start sale
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::StartSale {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // update config
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            admin: None,
            batch_duration: Some(Uint128::from(12u128)),
            revenue_collector: None,
            buy_denom: None,
            sell_denom: None,
            batch_amount: None,
            price: None,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::SaleNotPending {});
    }
}
