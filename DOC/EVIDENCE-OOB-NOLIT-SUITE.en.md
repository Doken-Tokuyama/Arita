[Español](EVIDENCE-OOB-NOLIT-SUITE.md) | English

# OOB / non-lit failure suite (R2) — inventory

SoT measure Lex. skip ≠ PASS.

| Family | Emit behavior | Oracles |
|---------|---------------------|----------|
| Vec/String `get` OOB / neg | `None` (no panic index) | `68/69`, neg E0287 |
| `swap` OOB | no-op | `186` |
| Index `[]` | **E0310** (R4) | neg-e0310-* |
| `pow` overflow no-lit | `checked_pow` + `unwrap_or(MAX)` (R0/R2) | std-pow* + emit-ban-r0r2 |
| next/prev multiple overflow | helper `unwrap_or(MAX/MIN)` | std next/prev multiple |
| insert / Mutex | HOLD neg (R5/R7) | pending |

Global emit gate: `emit-ban-r0r2` (ADR-241).
