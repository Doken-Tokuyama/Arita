[Español](EVIDENCE-GAPS-V0.1.md) | English

# Evidence gaps vs Semantics v0.1

Canonical: [ADR-225 §12](ADR/225-semantics-v0.1.md) · pins: [ADR-230](ADR/230-evidence-cuts-v0.1.md)

## HOLD (RFC rev. 3)

**Do not restart** CUT/measure until notice to <person> + their GO.

Partial R4 in tree (E0310+neg+emit-ban); measure **not** signed → **not CLOSED**.

## Order (when restarting)

1. R4 — E0310 `neg-e0310-index-*`
2. R0/R2 — emit ban unwrap/expect/panic
3. R7 — E0312 Mutex neg
4. R5 — neg-E0206 insert
5. R8 — remote CI

Codes: **E0310** IndexGet · **E0311** insert lit · **E0312** Mutex · **E0313** Mutex×await (future).

HOLD IMPL: insert / sugar `[]` / Mutex.
