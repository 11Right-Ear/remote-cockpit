//! JWT verification (gateway side). Signing lives in `phone-sim`.

use anyhow::Context;
use jsonwebtoken::{decode, DecodingKey, Validation};
use protocol::auth::Claims;

/// Verify a JWT against the shared secret and return its claims.
pub fn verify_token(secret: &str, token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .context("jwt verify failed")?;
    Ok(data.claims)
}

/// Map the role string in a token to a `ClientRole`.
pub fn role_from_claims(claims: &Claims) -> anyhow::Result<protocol::ClientRole> {
    match claims.role.as_str() {
        "desktop" => Ok(protocol::ClientRole::Desktop),
        "phone" => Ok(protocol::ClientRole::Phone),
        other => anyhow::bail!("unknown role in token: {other}"),
    }
}
