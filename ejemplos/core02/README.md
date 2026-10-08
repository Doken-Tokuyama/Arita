# Core 0.2 — ejemplos (`service`) — **CLOSED** Lex **632/632**

**Pins:** [`DOC/ADR/233-core-0.2-pins.md`](../../DOC/ADR/233-core-0.2-pins.md) · ref [`ADR-248`](../../DOC/ADR/248-core-ref-http-daemon-v0.md) · SoT [`ROADMAP.md`](../../ROADMAP.md).  
**HOLD (gates):** sugar `[]` · insert panic · Mutex · repair IMPL · idle-kill DEFER — no reabrir.

## Para humanos

Vertical **0.2 CLOSED** (perfil `service`). No es el techo del lenguaje GP.

| Área | Barra |
|------|-------|
| Async / spawn / timeout | 605 / 607 / 611 |
| HTTP + policy + scenarios | 614 / 622 / 627 |
| Ref daemon + evidence | **632/632** ADR-248 |

Gate final: `GATE-CORE02-REF-HTTP-DAEMON` (not included in the public tree / no incluido en el árbol público).

```bash
./target/release/arita measure   # SoT; skip ≠ PASS
```

## Para IAs

- Surface 0.2 = APIs tipadas HTTP/policy/client/scenarios (ADRs 245–248). Free `http_listen` / `reqwest` → reject.
- **No** Mutex, `[]`, sockets raw, `tokio::` en `.arita`.
- No inventar PASS; norte = Core 0.3 (pins), no unpark HOLDs.

## Estado (2026-09-20)

- **Core 0.2 CLOSED** Lex **632/632**.
