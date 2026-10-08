[Español](README.md) | English

# Core 0.2 — examples (`service`) — **CLOSED** Lex **632/632**

**Pins:** [`DOC/ADR/233-core-0.2-pins.md`](../../DOC/ADR/233-core-0.2-pins.md) · ref [`ADR-248`](../../DOC/ADR/248-core-ref-http-daemon-v0.md) · SoT [`ROADMAP.md`](../../ROADMAP.md).  
**HOLD (gates):** sugar `[]` · insert panic · Mutex · repair IMPL · idle-kill DEFER — do not reopen.

## For humans

Vertical **0.2 CLOSED** (`service` profile). Not the ceiling of the GP language.

| Area | Bar |
|------|-------|
| Async / spawn / timeout | 605 / 607 / 611 |
| HTTP + policy + scenarios | 614 / 622 / 627 |
| Ref daemon + evidence | **632/632** ADR-248 |

Final gate: `GATE-CORE02-REF-HTTP-DAEMON` (not included in the public tree / no incluido en el árbol público).

```bash
./target/release/arita measure   # SoT; skip ≠ PASS
```

## For AIs

- Surface 0.2 = typed HTTP/policy/client/scenarios APIs (ADRs 245–248). Free `http_listen` / `reqwest` → reject.
- **No** Mutex, `[]`, raw sockets, `tokio::` in `.arita`.
- Do not invent PASS; north star = Core 0.3 (pins), do not unpark HOLDs.

## Estado (2026-09-20)

- **Core 0.2 CLOSED** Lex **632/632**.
