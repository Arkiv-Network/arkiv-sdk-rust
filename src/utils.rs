use alloy::primitives::U256;
use bigdecimal::{BigDecimal, ToPrimitive};
use std::str::FromStr;

/// Converts an ETH amount to wei as a `U256`.
/// Accepts a `BigDecimal` ETH value and returns the equivalent amount in wei as a `U256`.
/// This is useful for preparing values for smart contract calls or transactions.
/// Returns an error if the value is too large to fit in a `u128`.
pub fn eth_to_wei(eth: BigDecimal) -> anyhow::Result<U256> {
    let wei = (eth * BigDecimal::from(1_000_000_000_000_000_000u128))
        .to_u128()
        .ok_or_else(|| anyhow::anyhow!("Value too large"))?;
    Ok(U256::from(wei))
}

/// Converts a wei amount (`U256`) to ETH as a `BigDecimal`.
/// Useful for displaying human-readable ETH values from raw wei amounts, such as for UI or logs.
/// Panics if the `U256` value cannot be parsed as a string (should not happen for valid values).
pub fn wei_to_eth(wei: U256) -> BigDecimal {
    BigDecimal::from_str(&wei.to_string()).unwrap()
        / BigDecimal::from(1_000_000_000_000_000_000u128)
}
