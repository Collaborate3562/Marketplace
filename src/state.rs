#![cfg_attr(not(feature = "std"), no_std)]

use concordium_cis2::*;
use concordium_std::*;

pub type ContractTokenId = TokenIdU32;

#[derive(SchemaType, Clone, Serialize, PartialEq, Eq, Debug)]
pub struct TokenInfo {
    pub id: ContractTokenId,
    pub address: ContractAddress,
}

impl TokenInfo {
    fn new(id: ContractTokenId, address: ContractAddress) -> Self {
        TokenInfo { id, address }
    }
}

#[derive(SchemaType, Clone, Serialize, Copy, PartialEq, Eq, Debug)]
pub enum TokenListState {
    UnListed,
    Listed(Amount),
}

#[derive(SchemaType, Clone, Serialize, Copy, PartialEq, Eq, Debug)]
pub enum TokenSaleTypeState {
    Fixed,
    Auction,
}

#[derive(SchemaType, Clone, Serialize, Copy, PartialEq, Eq, Debug)]
pub struct TokenSaleState {
    pub sale_type: TokenSaleTypeState,
    pub expiry: u64,
    pub highest_bidder: AccountAddress,
}

impl TokenSaleState {
    fn new (sale_type: TokenSaleTypeState, expiry: u64, highest_bidder: AccountAddress) -> Self {
        TokenSaleState { sale_type, expiry, highest_bidder }
    }
}

#[derive(SchemaType, Clone, Serialize, Debug, PartialEq, Eq)]
pub struct TokenState {
    pub counter: u64,
    pub curr_state: TokenListState,
    pub owner: AccountAddress,
    pub sale_state: TokenSaleState,
}

impl TokenState {
    pub fn get_curr_state(&self) -> TokenListState {
        self.curr_state
    }

    pub(crate) fn get_owner(&self) -> AccountAddress {
        self.owner
    }

    pub(crate) fn is_listed(&self) -> bool {
        match self.curr_state {
            TokenListState::UnListed => false,
            TokenListState::Listed(_) => true,
        }
    }

    pub(crate) fn get_price(&self) -> Option<Amount> {
        match self.get_curr_state() {
            TokenListState::UnListed => Option::None,
            TokenListState::Listed(price) => Option::Some(price),
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Commission {
    /// Commission basis points. equals to percent * 100
    pub(crate) percentage_basis: u8,
}

#[derive(Serial, DeserialWithState, StateClone)]
#[concordium(state_parameter = "S")]
pub(crate) struct State<S>
where
    S: HasStateApi,
{
    pub(crate) commission: Commission,
    pub(crate) tokens: StateMap<TokenInfo, TokenState, S>,
}

impl<S: HasStateApi> State<S> {
    pub(crate) fn new(state_builder: &mut StateBuilder<S>) -> Self {
        State {
            commission: Commission {
                percentage_basis: 250,
            },
            tokens: state_builder.new_map(),
        }
    }

    pub(crate) fn list_token(
        &mut self,
        token_id: ContractTokenId,
        nft_contract_address: ContractAddress,
        owner: AccountAddress,
        price: Amount,
        sale_type: u8,
        expiry: u64,
    ) {
        let info = TokenInfo::new(token_id, nft_contract_address);
        let counter = match self.token(&info) {
            Some(r) => (r.counter + 1),
            None => 0,
        };

        let sale_state;
        if sale_type == 0 {
            sale_state = TokenSaleState::new(TokenSaleTypeState::Fixed, 0, AccountAddress([0u8; 32]));
        } else {
            sale_state = TokenSaleState::new(TokenSaleTypeState::Auction, expiry, AccountAddress([0u8; 32]));
        }

        self.tokens.insert(
            info,
            TokenState {
                owner,
                counter,
                curr_state: TokenListState::Listed(price),
                sale_state,
            },
        );
    }

    pub(crate) fn delist_token(
        &mut self,
        token_id: TokenIdU32,
        nft_contract_address: ContractAddress,
        owner: AccountAddress,
    ) {
        let info = TokenInfo::new(token_id, nft_contract_address);
        let counter = match self.token(&info) {
            Some(r) => r.counter,
            None => 0,
        };

        let sale_state = TokenSaleState::new(TokenSaleTypeState::Fixed, 0, AccountAddress([0u8;32]));

        self.tokens.insert(
            info,
            TokenState {
                owner,
                counter,
                curr_state: TokenListState::UnListed,
                sale_state,
            },
        );
    }

    pub(crate) fn get_token(
        &self,
        token_id: TokenIdU32,
        nft_contract_address: ContractAddress,
    ) -> Option<StateRef<TokenState>> {
        let info = TokenInfo::new(token_id, nft_contract_address);
        self.token(&info)
    }

    fn token(&self, info: &TokenInfo) -> Option<StateRef<TokenState>> {
        self.tokens.get(info)
    }
}
