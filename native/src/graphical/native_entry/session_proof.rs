//! Cancel only this login's challenge if the credential provider declines/fails.
use qualia_core_db::poet_host::native_authority::{
    NativeAuthority, SessionChallenge, SessionHandle,
};
pub(crate) fn authenticate(
    authority: &NativeAuthority,
    subject: &str,
    lifetime: u64,
    prove: impl FnOnce(&SessionChallenge) -> Result<[u8; 64], String>,
) -> Result<SessionHandle, String> {
    let challenge = authority.begin_session(subject, lifetime)?;
    let proof = match prove(&challenge) {
        Ok(proof) => proof,
        Err(error) => {
            return match authority.cancel_session_challenge(&challenge) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!(
                    "{error}; pending native challenge cleanup failed: {cleanup}"
                )),
            }
        }
    };
    authority.complete_session(challenge, &proof)
}
