//! # SMTP backend
//!
//! The SMTP adapter of the shared cross-protocol client, a send-only
//! transport for the storage backends that cannot send themselves.
//!
//! The RFC 5321 envelope is derived from the message headers: `From:`
//! becomes the reverse path, and `To:`, `Cc:` and `Bcc:` the forward
//! paths. The `Bcc:` header is removed before submission, so the blind
//! recipients stay forward paths only (RFC 5322 section 3.6.3).

use io_smtp::client::SmtpClient as _;
use std::borrow::Cow;

use anyhow::{Result, anyhow, bail};
use io_smtp::rfc5321::{
    SmtpDomain, SmtpEhloDomain, SmtpForwardPath, SmtpLocalPart, SmtpMailbox, SmtpReversePath,
};
use mail_parser::{Address as MailParserAddress, MessageParser};

use crate::smtp::client::SmtpClient;

impl SmtpClient {
    /// Runs the RFC 5321 mail transaction (MAIL FROM / RCPT TO / DATA)
    /// for `raw`, deriving the envelope from its headers.
    pub fn send_message(&mut self, raw: Vec<u8>) -> Result<()> {
        let (reverse, forwards) = {
            let parsed = MessageParser::default()
                .parse_headers(&raw)
                .ok_or_else(|| anyhow!("Could not parse raw RFC 5322 message"))?;

            let reverse = parsed
                .from()
                .and_then(first_address)
                .ok_or_else(|| anyhow!("No `From:` header found in raw message"))?;
            let reverse = parse_smtp_mailbox(&reverse)?;

            let mut forwards = Vec::new();
            for group in [parsed.to(), parsed.cc(), parsed.bcc()]
                .into_iter()
                .flatten()
            {
                for address in addresses(group) {
                    forwards.push(parse_smtp_mailbox(&address)?);
                }
            }

            (reverse, forwards)
        };

        if forwards.is_empty() {
            bail!("No `To:` / `Cc:` / `Bcc:` recipients found in raw message");
        }

        let reverse_path = SmtpReversePath::SmtpMailbox(reverse);
        let forward_paths: Vec<SmtpForwardPath<'static>> =
            forwards.into_iter().map(SmtpForwardPath::from).collect();

        self.send(reverse_path, forward_paths, strip_bcc_header(raw))?;
        Ok(())
    }
}

/// Flattens a mail-parser address group into bare `local-part@domain`
/// strings.
fn addresses(group: &MailParserAddress<'_>) -> Vec<String> {
    group
        .clone()
        .into_list()
        .into_iter()
        .filter_map(|address| {
            let email = address.address?.into_owned();
            (!email.is_empty()).then_some(email)
        })
        .collect()
}

/// First address in a group; picks the `From:` envelope sender.
fn first_address(group: &MailParserAddress<'_>) -> Option<String> {
    addresses(group).into_iter().next()
}

/// Parses `local-part@domain` into an owned SMTP mailbox.
fn parse_smtp_mailbox(address: &str) -> Result<SmtpMailbox<'static>> {
    let (local, domain) = address
        .rsplit_once('@')
        .ok_or_else(|| anyhow!("Invalid email address `{address}` in envelope"))?;
    if local.is_empty() || domain.is_empty() {
        bail!("Invalid email address `{address}` in envelope");
    }

    Ok(SmtpMailbox {
        local_part: SmtpLocalPart(Cow::Owned(local.to_string())),
        domain: SmtpEhloDomain::SmtpDomain(SmtpDomain(Cow::Owned(domain.to_string()))),
    })
}

/// Removes every `Bcc:` header from the raw message, folded lines
/// included, leaving the other bytes and the line endings untouched.
///
/// RFC 5322 section 3.6.3 removes the `Bcc:` line when the message is
/// prepared to be sent: every recipient, blind ones included, gets a
/// copy, and the addresses stay envelope forward paths only.
fn strip_bcc_header(raw: Vec<u8>) -> Vec<u8> {
    let Some(headers_end) = headers_end(&raw) else {
        return raw;
    };

    let mut out = Vec::with_capacity(raw.len());
    let mut dropping = false;
    let mut stripped = false;

    for line in raw[..headers_end].split_inclusive(|byte| *byte == b'\n') {
        if !line.starts_with(b" ") && !line.starts_with(b"\t") {
            dropping = is_bcc_header(line);
            stripped |= dropping;
        }
        if !dropping {
            out.extend_from_slice(line);
        }
    }

    if !stripped {
        return raw;
    }

    out.extend_from_slice(&raw[headers_end..]);
    out
}

/// Tells whether a header line starts a `Bcc:` header, the obsolete
/// space-before-colon spelling included.
fn is_bcc_header(line: &[u8]) -> bool {
    if !line
        .get(..3)
        .is_some_and(|name| name.eq_ignore_ascii_case(b"bcc"))
    {
        return false;
    }

    line[3..]
        .iter()
        .copied()
        .find(|byte| *byte != b' ' && *byte != b'\t')
        == Some(b':')
}

/// Returns where the header block ends, meaning where its blank-line
/// terminator starts, or `None` when the message carries no separator.
fn headers_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 2)
        .or_else(|| {
            raw.windows(2)
                .position(|window| window == b"\n\n")
                .map(|index| index + 1)
        })
}

#[cfg(test)]
mod tests {
    use super::strip_bcc_header;

    #[test]
    fn bcc_header_is_stripped() {
        let raw = b"From: me@example.com\r\nTo: you@example.net\r\nBcc: hidden@example.org\r\nSubject: hi\r\n\r\nbody\r\n";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(
            stripped,
            b"From: me@example.com\r\nTo: you@example.net\r\nSubject: hi\r\n\r\nbody\r\n".to_vec()
        );
    }

    #[test]
    fn folded_bcc_header_is_stripped_whole() {
        let raw = b"From: me@example.com\r\nBcc: one@example.org,\r\n two@example.org\r\nTo: you@example.net\r\n\r\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(
            stripped,
            b"From: me@example.com\r\nTo: you@example.net\r\n\r\nbody".to_vec()
        );
    }

    #[test]
    fn every_bcc_header_is_stripped() {
        let raw =
            b"Bcc: one@example.org\r\nFrom: me@example.com\r\nbcc: two@example.org\r\n\r\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(stripped, b"From: me@example.com\r\n\r\nbody".to_vec());
    }

    #[test]
    fn obsolete_space_before_colon_is_stripped() {
        let raw = b"From: me@example.com\r\nBcc : hidden@example.org\r\n\r\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(stripped, b"From: me@example.com\r\n\r\nbody".to_vec());
    }

    #[test]
    fn lf_only_line_endings_are_preserved() {
        let raw = b"From: me@example.com\nBcc: hidden@example.org\nTo: you@example.net\n\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(
            stripped,
            b"From: me@example.com\nTo: you@example.net\n\nbody".to_vec()
        );
    }

    #[test]
    fn lookalike_headers_are_kept() {
        let raw = b"From: me@example.com\r\nX-Bcc: not-a-bcc@example.org\r\nBccx: nope@example.org\r\n\r\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(stripped, raw.to_vec());
    }

    #[test]
    fn message_without_bcc_is_returned_unchanged() {
        let raw = b"From: me@example.com\r\nTo: you@example.net\r\n\r\nbody\r\n";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(stripped, raw.to_vec());
    }

    #[test]
    fn message_without_header_separator_is_returned_unchanged() {
        let raw = b"From: me@example.com\r\nBcc: hidden@example.org\r\nbody";
        let stripped = strip_bcc_header(raw.to_vec());
        assert_eq!(stripped, raw.to_vec());
    }
}
