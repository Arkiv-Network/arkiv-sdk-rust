/// Tracks and assigns sequential Ethereum nonces for concurrent transactions.
pub struct NonceManager {
    /// Last known on-chain nonce.
    pub base_nonce: u64,
    /// Number of in-flight (pending) transactions.
    pub in_flight: u64,
}

impl NonceManager {
    /// Returns the next available nonce and increments the in-flight counter.
    pub async fn next_nonce(&mut self) -> u64 {
        let nonce = self.base_nonce + self.in_flight;
        self.in_flight += 1;
        nonce
    }

    /// Marks a transaction as completed by decrementing the in-flight counter.
    pub async fn complete(&mut self) {
        if self.in_flight > 0 {
            self.in_flight -= 1;
        }
    }
}
