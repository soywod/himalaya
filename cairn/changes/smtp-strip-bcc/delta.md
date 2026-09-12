---
cairn: delta
change: smtp-strip-bcc
---

# Delta

## MODIFIED Requirements

### Requirement: Sending transport
Backends that self-send (JMAP, Gmail, Graph) SHALL route `send_message` through their own API. Storage backends that cannot send (IMAP, Maildir, m2dir) SHALL send through the account's SMTP transport, adapted in `src/smtp/backend.rs` over io-smtp, which parses the RFC 5321 envelope from the raw message headers. The adapter SHALL remove the `Bcc:` header from the submitted bytes, so the blind recipients stay envelope forward paths only (RFC 5322 section 3.6.3).
