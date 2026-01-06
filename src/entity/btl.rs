use std::time;

use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

/// The blocks-to-live (BTL) for the entity.
///
/// # Example
///
/// > NOTE: Each block is roughly 2 seconds of life.
///
/// ```rs
/// use std::time;
/// use arkiv_sdk::entity::BlocksToLive;
///
/// // As a const value
/// const THIRTY_SECONDS: BlocksToLive = BlocksToLive::new(15u64);
/// // From `std::time::Duration`
/// let thirty_secs = BlocksToLive::from(time::Duration::from_secs(30));
/// ```
///
/// # Panics
///
/// Panics if the value is `u64::MIN`, i.e. it must be non-zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct BlocksToLive(u64);
impl BlocksToLive {
    pub const fn new(btl: u64) -> Self {
        if btl == u64::MIN {
            panic!("`BlocksToLive` must be non-zero");
        }
        Self(btl)
    }
}
impl Default for BlocksToLive {
    // We set this to 30s of life by default since we cannot have
    // a zero value, and anything less would be too short to be sane.
    fn default() -> Self {
        Self::new(15u64)
    }
}
impl From<u64> for BlocksToLive {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl From<time::Duration> for BlocksToLive {
    fn from(value: time::Duration) -> Self {
        Self::new(value.as_secs() / 2)
    }
}

#[test]
fn btl_const_compiles() {
    const THIRTY_SECONDS: BlocksToLive = BlocksToLive::new(15u64);
}

#[test]
fn btl_from_duration() {
    let thirty_secs = BlocksToLive::from(time::Duration::from_secs(30));
    assert_eq!(thirty_secs.0, 15);
}
