use std::fmt;
use std::str::FromStr;

const SCHEME: &str = "larder://";

/// What a client needs to pair with a host, packed into one string you can
/// paste: `larder://home.example.com:7878?token=...&fp=...`
///
/// `fp` is the sha256 of the host's self-signed certificate. It's left out
/// when the host has a real certificate for a domain, since normal TLS
/// checks already cover that case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
    pub address: String,
    pub token: String,
    pub fingerprint: Option<String>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InviteError {
    #[error("invites start with larder://")]
    Scheme,
    #[error("the invite has no host address")]
    Address,
    #[error("the invite is missing its token")]
    Token,
}

impl FromStr for Invite {
    type Err = InviteError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        // People paste these out of chat apps, so be forgiving about
        // whitespace and a trailing slash.
        let rest = raw.trim().strip_prefix(SCHEME).ok_or(InviteError::Scheme)?;
        let (address, query) = rest.split_once('?').unwrap_or((rest, ""));
        let address = address.trim_end_matches('/');
        if address.is_empty() || address.contains(char::is_whitespace) {
            return Err(InviteError::Address);
        }
        let mut token = None;
        let mut fingerprint = None;
        for pair in query.split('&') {
            match pair.split_once('=') {
                Some(("token", value)) if !value.is_empty() => token = Some(value.to_string()),
                Some(("fp", value)) if !value.is_empty() => fingerprint = Some(value.to_lowercase()),
                _ => {}
            }
        }
        Ok(Invite {
            address: address.to_string(),
            token: token.ok_or(InviteError::Token)?,
            fingerprint,
        })
    }
}

impl fmt::Display for Invite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SCHEME}{}?token={}", self.address, self.token)?;
        if let Some(fp) = &self.fingerprint {
            write!(f, "&fp={fp}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let invite = Invite {
            address: "192.168.1.20:7878".into(),
            token: "abc123".into(),
            fingerprint: Some("deadbeef".into()),
        };
        assert_eq!(invite.to_string().parse::<Invite>().unwrap(), invite);
    }

    #[test]
    fn domain_invites_have_no_fingerprint() {
        let invite: Invite = "larder://files.example.com?token=t".parse().unwrap();
        assert_eq!(invite.address, "files.example.com");
        assert_eq!(invite.fingerprint, None);
    }

    #[test]
    fn tolerates_pasting_mess() {
        let invite: Invite = "  larder://pc:7878/?token=t&fp=ABCD \n".parse().unwrap();
        assert_eq!(invite.address, "pc:7878");
        assert_eq!(invite.fingerprint.as_deref(), Some("abcd"));
    }

    #[test]
    fn rejects_broken_invites() {
        assert_eq!("http://pc?token=t".parse::<Invite>(), Err(InviteError::Scheme));
        assert_eq!("larder://?token=t".parse::<Invite>(), Err(InviteError::Address));
        assert_eq!("larder://pc:7878".parse::<Invite>(), Err(InviteError::Token));
    }
}
