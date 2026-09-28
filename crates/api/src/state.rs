// crates/api/src/state.rs
// Process-wide state types shared between `main` and the route handlers.

use crypto::vault::MasterKey;

/// One user's unlocked vault: the derived key plus the time of its last
/// use. `last_used` drives the idle-timeout auto-lock in
/// `routes::credentials::get_key`. The key is zeroized on drop via
/// `MasterKey`'s `ZeroizeOnDrop` derive.
pub struct VaultSession {
    pub key: MasterKey,
    pub last_used: std::time::Instant,
}
