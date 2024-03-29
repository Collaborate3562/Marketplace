use std::vec;

use concordium_cis2::*;
use concordium_std::*;

use crate::{
    errors::Cis2ClientError,
    state::{ContractTokenId, State},
};

pub const SUPPORTS_ENTRYPOINT_NAME: &str = "supports";
pub const OPERATOR_OF_ENTRYPOINT_NAME: &str = "operatorOf";
pub const BALANCE_OF_ENTRYPOINT_NAME: &str = "balanceOf";
pub const TRANSFER_ENTRYPOINT_NAME: &str = "transfer";

pub type ContractTokenAmount = TokenAmountU8;
type ContractBalanceOfQueryParams = BalanceOfQueryParams<ContractTokenId>;
type ContractBalanceOfQueryResponse = BalanceOfQueryResponse<ContractTokenAmount>;
type TransferParameter = TransferParams<ContractTokenId, ContractTokenAmount>;

pub struct Cis2Client;

impl Cis2Client {
    pub(crate) fn supports_cis2<S: HasStateApi>(
        host: &mut impl HasHost<State<S>, StateApiType = S>,
        nft_contract_address: &ContractAddress,
    ) -> Result<bool, Cis2ClientError> {
        let params = SupportsQueryParams {
            queries: vec![StandardIdentifierOwned::new_unchecked("CIS-2".to_string())],
        };
        let parsed_res: SupportsQueryResponse = Cis2Client::invoke_contract_read_only(
            host,
            nft_contract_address,
            SUPPORTS_ENTRYPOINT_NAME,
            &params,
        )?;
        let supports_cis2: bool = {
            let f = parsed_res
                .results
                .first()
                .ok_or(Cis2ClientError::InvokeContractError)?;
            match f {
                SupportResult::NoSupport => false,
                SupportResult::Support => true,
                SupportResult::SupportBy(_) => false,
            }
        };

        Ok(supports_cis2)
    }

    pub(crate) fn is_operator_of<S: HasStateApi>(
        host: &mut impl HasHost<State<S>, StateApiType = S>,
        owner: Address,
        current_contract_address: ContractAddress,
        nft_contract_address: &ContractAddress,
    ) -> Result<bool, Cis2ClientError> {
        let params = &OperatorOfQueryParams {
            queries: vec![OperatorOfQuery {
                owner,
                address: Address::Contract(current_contract_address),
            }],
        };

        let parsed_res: OperatorOfQueryResponse = Cis2Client::invoke_contract_read_only(
            host,
            nft_contract_address,
            OPERATOR_OF_ENTRYPOINT_NAME,
            params,
        )?;

        let is_operator = parsed_res
            .0
            .first()
            .ok_or(Cis2ClientError::InvokeContractError)?
            .to_owned();

        Ok(is_operator)
    }

    pub(crate) fn has_balance<S: HasStateApi>(
        host: &mut impl HasHost<State<S>, StateApiType = S>,
        token_id: ContractTokenId,
        nft_contract_address: &ContractAddress,
        owner: Address,
    ) -> Result<bool, Cis2ClientError> {
        let params = ContractBalanceOfQueryParams {
            queries: vec![BalanceOfQuery {
                token_id,
                address: owner,
            }],
        };

        let parsed_res: ContractBalanceOfQueryResponse = Cis2Client::invoke_contract_read_only(
            host,
            nft_contract_address,
            BALANCE_OF_ENTRYPOINT_NAME,
            &params,
        )?;

        let is_operator = parsed_res
            .0
            .first()
            .ok_or(Cis2ClientError::InvokeContractError)?
            .to_owned();

        Result::Ok(is_operator.cmp(&TokenAmountU8(1)).is_ge())
    }

    pub(crate) fn transfer<S: HasStateApi>(
        host: &mut impl HasHost<State<S>, StateApiType = S>,
        token_id: TokenIdU32,
        nft_contract_address: ContractAddress,
        amount: ContractTokenAmount,
        from: AccountAddress,
        to: Receiver,
    ) -> Result<bool, Cis2ClientError> {
        let params: TransferParameter = TransferParams(vec![Transfer {
            token_id,
            amount,
            from: concordium_std::Address::Account(from),
            data: AdditionalData::empty(),
            to,
        }]);

        Cis2Client::invoke_contract_read_only(
            host,
            &nft_contract_address,
            TRANSFER_ENTRYPOINT_NAME,
            &params,
        )?;

        Result::Ok(true)
    }

    fn invoke_contract_read_only<S: HasStateApi, R: Deserial, P: Serial>(
        host: &mut impl HasHost<State<S>, StateApiType = S>,
        contract_address: &ContractAddress,
        entrypoint_name: &str,
        params: &P,
    ) -> Result<R, Cis2ClientError> {
        let invoke_contract_result = host
            .invoke_contract_read_only(
                contract_address,
                params,
                EntrypointName::new(entrypoint_name).unwrap_abort(),
                Amount::from_ccd(0),
            )
            .map_err(|_e| Cis2ClientError::InvokeContractError)?;
        let mut invoke_contract_res = match invoke_contract_result {
            Some(s) => s,
            None => return Result::Err(Cis2ClientError::InvokeContractError),
        };
        let parsed_res =
            R::deserial(&mut invoke_contract_res).map_err(|_e| Cis2ClientError::ParseResult)?;

        Ok(parsed_res)
    }
}
