---
cairn: tasks
change: smtp-strip-bcc
---

# Tasks

- [x] src/smtp/backend.rs: `strip_bcc_header` removing every `Bcc:` header before `SmtpClient::send`, folded lines included, plus its unit tests.
- [x] CHANGELOG entry under Unreleased, referencing [#747].
- [x] Fold the delta into [cairn/spec/backends.md](../../spec/backends.md) and write [cairn/log/2026-09-12-smtp-strip-bcc.md](../../log/2026-09-12-smtp-strip-bcc.md).
