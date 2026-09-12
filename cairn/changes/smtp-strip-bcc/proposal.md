---
cairn: change
id: smtp-strip-bcc
status: landed
created: 2026-09-12
---

# Strip the Bcc header before the SMTP submission

## Why

The SMTP adapter derives the envelope from the message headers, so `Bcc:` is already an input to the send path: the addresses become RCPT TO forward paths, and every envelope recipient then receives the raw bytes with the `Bcc:` line still in them. One `message send` of a message carrying five `Bcc:` addresses shows all five to each recipient, the failure [#747](https://github.com/pimalaya/himalaya/issues/747) reported.

RFC 5322 section 3.6.3 settles what should happen: in its first method the `Bcc:` line is removed when the message is prepared to be sent, and every recipient including the blind ones receives a copy. The Security Considerations section of the same RFC names the opposite behavior as mishandling that discloses confidential information.

## What

`SmtpClient::send_message` removes every `Bcc:` header from the bytes it submits, after the envelope is derived, so the blind recipients stay forward paths only.

The strip keeps the rest of the message byte-for-byte and the line endings it came with, removes folded continuations with their header, matches the name case-insensitively and accepts the obsolete `Bcc :` spelling (RFC 5322 obs-bcc). A message carrying no `Bcc:` header is returned unchanged.

## What this is not

Not a composition change: nothing is added to the bytes the caller hands over, the send path only drops a header it has already consumed for RCPT TO. That is the difference with [#733](https://github.com/pimalaya/himalaya/pull/733), where the missing piece was a header the composer owes and the send path was right to stay opaque.

`Resent-Bcc:` stays, since the adapter never consumes it and RFC 5322 keeps it distinct. The saved copy keeps its `Bcc:` header, a sender's own copy showing what they blind-copied, as before.
