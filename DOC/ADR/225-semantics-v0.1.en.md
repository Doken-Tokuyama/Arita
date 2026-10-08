Translation of `225-semantics-v0.1.md`; the original is normative. / Traducción de `225-semantics-v0.1.md`; el original es el normativo.

# ADR-225 — Public ARITA v0.1 semantics (normative map)

- **Estado:** **propuesta ampliada** (mapa); RFC **rev. 3** language-first (propuesta); addenda 227–229; evidencia ADR-230 **HOLD**
- **CUT-ID:** `SEMANTICS-V0.1-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins) · <person> (norte producto) · Ingeniero (evidencia posterior)
- **Relacionados:** THREAT_MODEL; ADR-001 emit-Rust; ADR-008 evidence; parks insert/`[]`/Mutex
- **Gobernanza:** Sin idle en mapa; std aislado HOLD; no inventar PASS.
- **Identidad:** lenguaje **AI-native** para software **verificable** (errores/efectos/recursos/límites/concurrencia/evidencia explícitos en tipo/sintaxis) — **no** «Rust amputado» ni policy layer opaca.

## 0. Meta-rule (back doors)

**R0.** Every guarantee the runtime/emit assumes must be **expressible and checkable** in the ARITA surface (or marked explicit OUT).  
If Rust emit uses a trick not nameable in ARITA → **backdoor** → forbidden in v0.1.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| “trust the backend, the type doesn’t say it” | — | DOC review + grep `unsafe` / hidden panics in emit |

---

## 1. Value model and ownership

**R1.1 Values.** Base v0.1 types: `Int` (i64), `Bool`, `String`, `Vec<T>` (T ∈ {Int,Bool,String} in F1.x; nested Option/Result per ADRs). No raw pointers.

**R1.2 Rust-like ownership (subset).** Move by default; explicit `clone`; shared borrow vs exclusive `mut` (E0202). No lifetimes in surface.

**R1.3 Copy.** Only `Int`/`Bool` implicit Copy; `String`/`Vec`/`Option`/`Result` move.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| use-after-move / double-mut | rustc borrow + ARITA E02xx gates | measure neg-e0202*; move examples |
| “shared &mut” | not emitted | TRAPS + measure |

---

## 2. Failures and panic

**R2.1 Typed failure.** Prefer `Option`/`Result` + E0xxx at compile time. Rust panic is **not** a success channel.

**R2.2 Lit domain errors.** Div0, negative lit indices, etc. → E0xxx (E0216, E0288, E029x…) — do not defer to panic.

**R2.3 Runtime OOB / non-lit div0.** **Safe** emit: Option/None, documented clamp, or non-panicking helper (e.g. OOB swap no-op). **Forbidden** to emit `[]` / `insert` / `unwrap` that can panic without an ARITA contract.

**R2.4 Anti-theater.** Swallow Err/None, vacuous assert, default-as-success → E027x/E028x (TRAPS catalog).

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| `unwrap`/`expect`/`[]` panic-as-OK | no whitelist | E0206 + traps; measure neg |
| Err→0 / find miss→0 | E0285/E0289… | measure neg-* |

---

## 3. Mutability

**R3.1** Mutation only via `mut` + exclusive methods (push, append, reserve, …).  
**R3.2** Shared receiver cannot mutate (E0202).  
**R3.3** Interior mutability (`Mutex`, `Cell`) **PARK** until §7 with a contract.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| mutation via shared | E0202 | measure neg-e0202* |

---

## 4. Indexing

**R4.1** `v[i]` / `s[i]` **OUT v0.1** until a contract.  
**R4.2** IN substitutes: `get`→Option, bounds-checked `swap`, `binary_search`→Result.  
**R4.3** When `[]` is unblocked: it must be **either** total (Option) **or** E0xxx-gated + evidence; never silent panic.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| `v[i]` today | E0xxx / parse fail | park + measure |
| neg lit index on IN APIs | E0292/E0295… | neg oracles |

---

## 5. Collections

**R5.1** `Vec`/`String` grow via whitelisted APIs; explicit capacity/reserve/try_reserve.  
**R5.2** `insert` **PARK** until a contract (off-by-one / mid panic).  
**R5.3** Iterators/closures **OUT** v0.1 (no `retain`/`map` fn). Prefer total methods.  
**R5.4** `append` drains; `extend` copies (shared) — distinct documented semantics.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| `insert` / `drain` / closure adaptors | park / E0206 | TRAPS park DOC |
| confuse append vs extend | DOC + oracles | measure |

---

## 6. Resources and effects

**R6.1** Typed I/O (`Io<()>`, print) — effects visible in the signature where applicable.  
**R6.2** No filesystem/net in v0.1 surface except explicit ADRs.  
**R6.3** Fallible alloc: `try_reserve*` → `Result`; infallible alloc documented as best-effort (host).

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| Hidden side-effect in “pure” fn | type/Io gates | examples + measure |
| OOM as success | try_reserve Err | oracles |

---

## 7. Async / concurrency

**R7.1** `async`/`await` **OUT** v0.1.  
**R7.2** `Mutex` **PARK** until an ARITA contract: explicit acquire/release, no deadlock theater, measure evidence.  
**R7.3** No threads in surface until a concurrency ADR.

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| Mutex/async today | park | ROADMAP PARK |
| data race via shared mut | E0202 + park | — |

---

## 8. Modules and dependencies

**R8.1** `module` + fn; crates workspace emit-Rust (`#![forbid(unsafe_code)]`).  
**R8.2** No IDNI deps in product; own logic island (F3) is orthogonal.  
**R8.3** Rust deps only via controlled codegen (stdlib subset whitelist E0206).

| AI impossible | Emit | Evidence |
|--------------|------|-----------|
| `unsafe` / phantom deps | forbid + whitelist | clippy/measure; E0206 |
| copy IDNI | ADR-002 policy | review |

---

## 9. Automated evidence (chain)

Every R* rule must bind to ≥1 of:

1. **arita measure** (compile+run / neg E0xxx oracle)  
2. **Rust twin** (bootstrap GOLDEN_*)  
3. **cargo test** crates  
4. **DOC/TRAPS + STABLE_VERIFY**  

`skip ≠ PASS`; `inconclusive` ≠ accepted.

---

## 10. Post-map unlock order

1. Accept this ADR (<person> + Architect).  
2. Contract addenda: **IndexGet** (unified `[]` or `get`), **Insert**, **Mutex**.  
3. Remote CI certifies contracts.  
4. Point GO IMPL per CUT (Engineer sole).

## 11. Checklist

- [x] R0–R8 map draft + evidence  
- [x] ACK <person> vía RFC rev. 2  
- [x] IndexGet / Insert / Mutex addenda (ADR-227/228/229)  
- [ ] Docs signatures (THREAT_MODEL + ROADMAP tick)  
- [ ] ADR-230 evidence chain  

## Immediate queue

**HOLD** ADR-230 / R4 restart until <person> notice+GO (rev. 3 thesis). Docs: ROADMAP Core 0.1. HOLD insert/`[]`/Mutex.

---

## 12. Evidence gaps (Engineer, 2026-09-19 — read-only; zero IMPL)

Source: Lex measure + DOC on box `/workspace/arita/DOC/` (Lex Documents sync pending).

| Rule | Measure status | Gap |
|-------|----------------|-----|
| **R0** backdoor | review/grep | Missing job/oracle that **fails** if emit injects a helper/panic not nameable in surface |
| **R1** ownership | strong e0201/e0202/move | Incomplete Copy vs move matrix for nested Option/Result |
| **R2** failures/panic | strong e0216/e027–e029 | No global “emit without unwrap/expect/panic!” sweep nor unified non-lit OOB suite |
| **R3** mut | OK neg-e0202* | None material (Mutex → R7) |
| **R4** index | park DOC | Missing stable neg oracle `v[i]`/`s[i]` → E0xxx (parse fail inconsistent today) |
| **R5** collections | append/extend OK | `insert`/`drain`/closures: park DOC without dedicated neg-E0206 |
| **R6** effects | Io + try_reserve OK | No “fn without Io cannot print” oracle; fs/net OUT = absence not a measured park |
| **R7** async | e0240/41/42 OK | Mutex park DOC-only (no surface `Mutex` neg) |
| **R8** modules | forbid(unsafe)+E0206 OK | Remote Origin CI not yet certifying; anti-IDNI = policy not measure |

**Post-ACK <person>:** IndexGet / Insert / Mutex contract addenda + R0/R2/R4/R5/R7 gap queue as evidence CUTs (not std features).

HOLD insert/`[]`/Mutex + isolated std CUTs until ACK + addenda.


Extended by RFC-AINATIVE-VERIFIED-MODEL.md.


---

**Rev. 2 note (2026-09-19):** Profiles and meaning of “verifiable” → RFC rev. 2 (`safe`/`service`/`sandboxed`/`high-assurance`/`ffi`). Optional formal contracts; Core 1 = GP expressiveness with fallible/total APIs.

**Addenda 2026-09-19:** ADR-227 IndexGet · ADR-228 Insert · ADR-229 Mutex · ADR-230 evidence.

---

**Rev. 3 note (2026-09-19):** Product identity = **AI-native language → real Rust** (RFC rev. 3 / PRODUCT-VISION). Rev. 2 (proportional profiles) remains a *policy layer*, not the product definition. R4/evidence HOLD; no GO IMPL until <person> restart.
