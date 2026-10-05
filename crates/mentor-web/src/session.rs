//! Signed session cookie: who is signed in, on which tenant, until when.
//!
//! The cookie holds `tenant.learner.expiry.signature`. It is not encrypted (it contains no secret), it is
//! authenticated: the signature is an HMAC-SHA-256 over the three first fields with the server's secret. A
//! cookie issued for one tenant is refused on another.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub const COOKIE: &str = "mentor_session";
/// Lifetime of a session.
pub const LIFETIME_SECONDS: i64 = 7 * 24 * 3600;

fn signature(secret: &[u8], payload: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts a key of any length");
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}

/// Value of the cookie for a learner signed in at `now` (seconds since the epoch).
pub fn issue(secret: &[u8], tenant: Uuid, learner: Uuid, now: i64) -> String {
    let payload = format!("{tenant}.{learner}.{}", now + LIFETIME_SECONDS);
    format!("{payload}.{}", hex(&signature(secret, &payload)))
}

/// The learner a cookie value designates, if it is authentic, issued for `tenant` and not expired.
pub fn verify(secret: &[u8], value: &str, tenant: Uuid, now: i64) -> Option<Uuid> {
    let (payload, signed) = value.rsplit_once('.')?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).ok()?;
    mac.update(payload.as_bytes());
    // Constant-time comparison.
    mac.verify_slice(&unhex(signed)?).ok()?;
    let mut fields = payload.split('.');
    let (cookie_tenant, learner, expiry) = (fields.next()?, fields.next()?, fields.next()?);
    if fields.next().is_some() || cookie_tenant.parse::<Uuid>().ok()? != tenant || expiry.parse::<i64>().ok()? <= now {
        return None;
    }
    learner.parse().ok()
}

/// `Set-Cookie` value that opens a session. Not readable by scripts, and not sent by other sites' forms.
pub fn set_cookie(value: &str, secure: bool) -> String {
    format!("{COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={LIFETIME_SECONDS}{}", if secure { "; Secure" } else { "" })
}

/// `Set-Cookie` value that closes the session.
pub fn clear_cookie() -> String {
    format!("{COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// Value of the session cookie in a `Cookie` header.
pub fn from_header(header: &str) -> Option<&str> {
    header.split(';').filter_map(|pair| pair.trim().split_once('=')).find(|(name, _)| *name == COOKIE).map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test secret";

    #[test]
    fn a_fresh_cookie_designates_its_learner() {
        let (tenant, learner) = (Uuid::new_v4(), Uuid::new_v4());
        let value = issue(SECRET, tenant, learner, 1000);
        assert_eq!(verify(SECRET, &value, tenant, 1001), Some(learner));
    }

    #[test]
    fn expired_forged_or_foreign_cookies_are_refused() {
        let (tenant, learner) = (Uuid::new_v4(), Uuid::new_v4());
        let value = issue(SECRET, tenant, learner, 1000);
        assert_eq!(verify(SECRET, &value, tenant, 1000 + LIFETIME_SECONDS), None);
        assert_eq!(verify(b"another secret", &value, tenant, 1001), None);
        assert_eq!(verify(SECRET, &value, Uuid::new_v4(), 1001), None);
        // Changing the learner without the secret breaks the signature.
        let forged = value.replace(&learner.to_string(), &Uuid::new_v4().to_string());
        assert_eq!(verify(SECRET, &forged, tenant, 1001), None);
        assert_eq!(verify(SECRET, "garbage", tenant, 1001), None);
    }

    #[test]
    fn the_cookie_is_found_among_others() {
        assert_eq!(from_header("theme=dark; mentor_session=abc.def; other=1"), Some("abc.def"));
        assert_eq!(from_header("theme=dark"), None);
    }
}
