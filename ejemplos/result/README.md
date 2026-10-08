# Result v0 (ADR-047) + E0272 swallow (ADR-048)

- `01-ok.arita` — Ok path → stdout `42`
- `02-err.arita` — Err path → stdout `fail`
- `03-swallow-ok.arita` — Err(e)+print(e) → stdout `1` (no E0272)
- `neg/e0270-nonexhaustive.arita` — missing Err arm → E0270
- `neg/e0272-err-swallow.arita` — Err(e)=>`{ 0 }` → **E0272** `result error swallowed`
- `neg/e0272-err-underscore.arita` — Err(_)=>`{ 0 }` → **E0272**
