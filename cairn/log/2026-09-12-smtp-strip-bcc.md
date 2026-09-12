---
cairn: log
change: smtp-strip-bcc
landed: 2026-09-12
---

# The Bcc line no longer rides along the SMTP submission

[#747](https://github.com/pimalaya/himalaya/issues/747) reported a `message send` of a message carrying a `Bcc:` header delivering that header to every recipient, the whole blind list visible to each of the five. The adapter derived the envelope from `To:`, `Cc:` and `Bcc:` and then handed io-smtp the raw bytes untouched, so the header it had just consumed for RCPT TO went out in the DATA.

## What landed

[`strip_bcc_header`](../../src/smtp/backend.rs) in src/smtp/backend.rs, applied between envelope derivation and `SmtpClient::send`. It removes every `Bcc:` header, folded continuations included, keeps the rest of the message byte-for-byte, matches the name case-insensitively and accepts the obsolete `Bcc :` spelling. A message carrying no `Bcc:` header returns as it came.

The choice follows RFC 5322 section 3.6.3, whose first method removes the line while every recipient, blind ones included, receives a copy: the blind addresses stay envelope forward paths. The header is also not composition the CLI owes the caller, it is a header the adapter already parses for RCPT TO, and leaving it in the payload is what the Security Considerations section of the same RFC names as mishandling.

## Left out

`Resent-Bcc:`, which the adapter never consumes and RFC 5322 keeps distinct. The copy a `--save` writes keeps its `Bcc:` header, the sender's own copy showing what they blind-copied, as before.

## Capabilities moved

- **backends**: the sending transport requirement now says the SMTP adapter removes the `Bcc:` header from the submitted bytes.
