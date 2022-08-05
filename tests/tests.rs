use cosmwasm_std::{StdError, Uint128, coin};
use cw_coins::{helpers::parse_coin_str, Coins};
use std::str::FromStr;

#[test]
fn casting_vec() {
    let mut vec = helpers::mock_vec();
    let coins = helpers::mock_coins();

    // &[Coin] --> Coins
    assert_eq!(Coins::try_from(vec.as_slice()).unwrap(), coins);
    // Vec<Coin> --> Coins
    assert_eq!(Coins::try_from(vec.clone()).unwrap(), coins);

    helpers::sort_by_denom(&mut vec);

    // &Coins --> Vec<Coins>
    // NOTE: the returned vec should be sorted
    assert_eq!((&coins).to_vec(), vec);
    // Coins --> Vec<Coins>
    // NOTE: the returned vec should be sorted
    assert_eq!(coins.into_vec(), vec);
}

#[test]
fn casting_str() {
    // not in order
    let s1 = "88888factory/osmo1234abcd/subdenom,12345uatom,69420ibc/1234ABCD";
    // in order
    let s2 = "88888factory/osmo1234abcd/subdenom,69420ibc/1234ABCD,12345uatom";

    let coins = helpers::mock_coins();

    // &str --> Coins
    // NOTE: should generate the same Coins, regardless of input order
    assert_eq!(Coins::from_str(s1).unwrap(), coins);
    assert_eq!(Coins::from_str(s2).unwrap(), coins);

    // Coins --> String
    // NOTE: the generated string should be sorted
    assert_eq!(coins.to_string(), s2);
}

#[test]
fn serde() {
    // not in order, with indentation
    let s1 = r#"{
        "uatom": "12345",
        "factory/osmo1234abcd/subdenom": "88888",
        "ibc/1234ABCD": "69420"
    }"#;
    // in order, no indentation
    let s2 = r#"{"factory/osmo1234abcd/subdenom":"88888","ibc/1234ABCD":"69420","uatom":"12345"}"#;

    let coins = helpers::mock_coins();

    // &str --> Coins
    // NOTE: should generate the same Coins, regardless of input order or indentation
    assert_eq!(serde_json::from_str::<Coins>(s1).unwrap(), coins);
    assert_eq!(serde_json::from_str::<Coins>(s2).unwrap(), coins);

    // Coins --> String
    // NOTE: the generated string should be sorted
    assert_eq!(serde_json::to_string(&coins).unwrap(), s2);
}

#[test]
fn handling_duplicates() {
    // a JSON string that contains a duplicate coin denom; should fail
    let s = r#"{
        "uatom": "67890",
        "factory/osmo1234abcd/subdenom": "88888",
        "uatom": "12345",
        "ibc/1234ABCD": "69420"
    }"#;

    let err = serde_json::from_str::<Coins>(s).unwrap_err();
    assert!(err.to_string().contains("failed to parse into Coins! duplicate denom: uatom"));

    // same with Vec<Coin>
    let mut vec = helpers::mock_vec();
    vec.push(coin(67890, "uatom"));

    let err = Coins::try_from(vec).unwrap_err();
    assert_eq!(err.to_string(), "Error parsing into type cw_coins::Coins: duplicate denoms");
}

#[test]
fn handling_invalid_amount() {
    // a JSON string that contains an invalid coin amount; should fail
    let s = r#"{
        "uatom": "67890",
        "factory/osmo1234abcd/subdenom": "ngmi",
        "ibc/1234ABCD": "69420"
    }"#;

    let err = serde_json::from_str::<Coins>(s).unwrap_err();
    assert!(err.to_string().contains("failed to parse into Coins! invalid amount: ngmi"));
}

#[test]
fn length() {
    let coins = Coins::new();
    assert_eq!(coins.len(), 0);
    assert_eq!(coins.is_empty(), true);

    let coins = helpers::mock_coins();
    assert_eq!(coins.len(), 3);
    assert_eq!(coins.is_empty(), false);
}

#[test]
fn parsing_coin() {
    let (denom, amount) = parse_coin_str("12345uatom").unwrap();
    assert_eq!(denom, "uatom".to_string());
    assert_eq!(amount, Uint128::new(12345));

    let (denom, amount) = parse_coin_str("69420ibc/1234ABCD").unwrap();
    assert_eq!(denom, "ibc/1234ABCD".to_string());
    assert_eq!(amount, Uint128::new(69420));

    let (denom, amount) = parse_coin_str("88888factory/osmo1234abcd/subdenom").unwrap();
    assert_eq!(denom, "factory/osmo1234abcd/subdenom");
    assert_eq!(amount, Uint128::new(88888));

    let err = parse_coin_str("ngmi").unwrap_err();
    assert_eq!(err, StdError::generic_err("Parsing u128: cannot parse integer from empty string"));

    let err = parse_coin_str("69420").unwrap_err();
    assert_eq!(
        err,
        StdError::parse_err("cosmwasm_std::coins::Coin", "Invalid coin string (69420)")
    );
}

mod helpers {
    use cosmwasm_std::{coin, Coin, Uint128};
    use cw_coins::Coins;
    use std::collections::BTreeMap;

    /// Sort a Vec<Coin> by denom alphabetically
    pub(super) fn sort_by_denom(vec: &mut Vec<Coin>) {
        vec.sort_by(|a, b| a.denom.cmp(&b.denom));
    }

    /// Returns a mockup Vec<Coin>. In this example, the coins are not in order
    pub(super) fn mock_vec() -> Vec<Coin> {
        vec![
            coin(12345, "uatom"),
            coin(69420, "ibc/1234ABCD"),
            coin(88888, "factory/osmo1234abcd/subdenom"),
        ]
    }

    /// Return a mockup Coins that contains the same coins as in `mock_vec`
    pub(super) fn mock_coins() -> Coins {
        let mut map = BTreeMap::new();

        map.insert("uatom".to_string(), Uint128::new(12345));
        map.insert("ibc/1234ABCD".to_string(), Uint128::new(69420));
        map.insert("factory/osmo1234abcd/subdenom".to_string(), Uint128::new(88888));

        Coins(map)
    }
}
