use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use cosmwasm_std::{Coin, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Coins(pub BTreeMap<String, Uint128>);

impl From<Vec<Coin>> for Coins {
    fn from(coins: Vec<Coin>) -> Self {
        let map = coins.into_iter().map(|coin| (coin.denom, coin.amount)).collect();
        Self(map)
    }
}

impl From<&[Coin]> for Coins {
    fn from(coins: &[Coin]) -> Self {
        coins.to_vec().into()
    }
}

impl FromStr for Coins {
    type Err = StdError;

    fn from_str(s: &str) -> StdResult<Self> {
        let map = s
            .split(",")
            .into_iter()
            .map(|split| helpers::parse_coin(split))
            .collect::<StdResult<_>>()?;
        Ok(Self(map))
    }
}

impl fmt::Display for Coins {
    // TODO: For empty coins, this stringifies to am empty string, which may cause confusions.
    // Should it stringify to a more informative string, such as `[]`?
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // NOTE: The `iter` method for BTreeMap returns an Iterator where entries are already sorted by key,
        // so we don't need sort the coins manually
        let s = self
            .0
            .iter()
            .map(|(denom, amount)| format!("{}{}", amount, denom))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", s)
    }
}

impl Coins {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn to_vec(&self) -> Vec<Coin> {
        self.0
            .iter()
            .map(|(denom, amount)| Coin {
                denom: denom.clone(),
                amount: *amount,
            })
            .collect()
    }

    pub fn into_vec(self) -> Vec<Coin> {
        self.0
            .into_iter()
            .map(|(denom, amount)| Coin {
                denom,
                amount,
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub mod helpers {
    use std::any::type_name;
    use std::str::FromStr;

    use cosmwasm_std::{Coin, StdError, StdResult, Uint128};

    /// `cosmwasm_std::Coin` does not implement `FromStr`, so we have do it ourselves
    ///
    /// Parsing the string with regex doesn't work, because the resulting wasm binary would be too big
    /// from including the `regex` library.
    ///
    /// If the binary size is not a concern, here's an example:
    /// https://github.com/PFC-Validator/terra-rust/blob/v1.1.8/terra-rust-api/src/client/core_types.rs#L34-L55
    ///
    /// We opt for a dirtier solution. Enumerate characters in the string, and break before the first
    /// non-number character. Split the string at that index.
    ///
    /// This assumes the denom never starts with a number, which is the case:
    /// https://github.com/cosmos/cosmos-sdk/blob/v0.46.0/types/coin.go#L854-L856
    pub fn parse_coin(s: &str) -> StdResult<(String, Uint128)> {
        for (i, c) in s.chars().enumerate() {
            if c.is_alphabetic() {
                let amount = Uint128::from_str(&s[..i])?;
                let denom = String::from(&s[i..]);
                return Ok((denom, amount));
            }
        }

        Err(StdError::parse_err(type_name::<Coin>(), format!("Invalid coin string ({})", s)))
    }
}
