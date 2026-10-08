# ADR-225 — Semántica pública ARITA v0.1 (mapa normativo)

- **Estado:** **propuesta ampliada** (mapa); RFC **rev. 3** language-first (propuesta); addenda 227–229; evidencia ADR-230 **HOLD**
- **CUT-ID:** `SEMANTICS-V0.1-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins) · <person> (norte producto) · Ingeniero (evidencia posterior)
- **Relacionados:** THREAT_MODEL; ADR-001 emit-Rust; ADR-008 evidence; parks insert/`[]`/Mutex
- **Gobernanza:** Sin idle en mapa; std aislado HOLD; no inventar PASS.
- **Identidad:** lenguaje **AI-native** para software **verificable** (errores/efectos/recursos/límites/concurrencia/evidencia explícitos en tipo/sintaxis) — **no** «Rust amputado» ni policy layer opaca.

## 0. Meta-regla (puertas traseras)

**R0.** Toda garantía que el runtime/emit asuma debe ser **expresable y comprobable** en el surface ARITA (o marcada OUT explícito).  
Si emit Rust usa un truco no nombrable en ARITA → **backdoor** → prohibido en v0.1.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| «confía en el backend, el tipo no lo dice» | — | review DOC + grep `unsafe` / hidden panics en emit |

---

## 1. Modelo de valores y ownership

**R1.1 Valores.** Tipos base v0.1: `Int` (i64), `Bool`, `String`, `Vec<T>` (T ∈ {Int,Bool,String} en F1.x; anidados Option/Result según ADRs). Sin raw pointers.

**R1.2 Ownership Rust-like (subset).** Move por defecto; `clone` explícito; shared borrow vs exclusive `mut` (E0202). Sin lifetimes en surface.

**R1.3 Copy.** Solo `Int`/`Bool` Copy implícito; `String`/`Vec`/`Option`/`Result` move.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| use-after-move / double-mut | rustc borrow + gates ARITA E02xx | measure neg-e0202*; ejemplos move |
| «&mut compartido» | no se emite | TRAPS + measure |

---

## 2. Fallos y panic

**R2.1 Fallo tipado.** Preferir `Option`/`Result` + E0xxx en compile. Panic de Rust **no** es canal de éxito.

**R2.2 Lit domain errors.** Div0, índices neg literales, etc. → E0xxx (E0216, E0288, E029x…) — no defer a panic.

**R2.3 Runtime OOB / div0 no-lit.** Emit **seguro**: Option/None, clamp documentado, o helper que no paniquea (p.ej. swap OOB no-op). **Prohibido** emitir `[]` / `insert` / `unwrap` que puedan panic sin contrato ARITA.

**R2.4 Anti-theater.** Swallow Err/None, vacuous assert, default-as-success → E027x/E028x (catálogo TRAPS).

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| `unwrap`/`expect`/`[]` panic-as-OK | no whitelist | E0206 + traps; measure neg |
| Err→0 / find miss→0 | E0285/E0289… | measure neg-* |

---

## 3. Mutabilidad

**R3.1** Mutación solo vía `mut` + métodos exclusive (push, append, reserve, …).  
**R3.2** Shared receiver no puede mutar (E0202).  
**R3.3** Interior mutability (`Mutex`, `Cell`) **PARK** hasta §7 con contrato.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| mutación vía shared | E0202 | measure neg-e0202* |

---

## 4. Indexación

**R4.1** `v[i]` / `s[i]` **OUT v0.1** hasta contrato.  
**R4.2** Sustitutos IN: `get`→Option, `swap` bounds-checked, `binary_search`→Result.  
**R4.3** Cuando se desbloquee `[]`: debe ser **o** total (Option) **o** gated E0xxx + evidencia; nunca panic silencioso.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| `v[i]` hoy | E0xxx / parse fail | park + measure |
| índice neg lit en APIs IN | E0292/E0295… | neg oracles |

---

## 5. Colecciones

**R5.1** `Vec`/`String` crecen por APIs whitelisted; capacity/reserve/try_reserve explícitos.  
**R5.2** `insert` **PARK** hasta contrato (off-by-one / panic mid).  
**R5.3** Iteradores/closures **OUT** v0.1 (no `retain`/`map` fn). Methods totales preferidos.  
**R5.4** `append` drena; `extend` copia (shared) — semántica distinta, documentada.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| `insert` / `drain` / closure adaptors | park / E0206 | TRAPS park DOC |
| confundir append vs extend | DOC + oracles | measure |

---

## 6. Recursos y efectos

**R6.1** I/O tipado (`Io<()>`, print) — efectos visibles en firma donde aplique.  
**R6.2** Sin filesystem/net en surface v0.1 salvo ADRs explícitos.  
**R6.3** Alloc fallible: `try_reserve*` → `Result`; alloc infalible documentado como best-effort (host).

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| side-effect oculto en «pure» fn | type/Io gates | ejemplos + measure |
| OOM como éxito | try_reserve Err | oracles |

---

## 7. Async / concurrencia

**R7.1** `async`/`await` **OUT** v0.1.  
**R7.2** `Mutex` **PARK** hasta contrato ARITA: acquire/release explícitos, sin deadlock theater, evidencia measure.  
**R7.3** No threads en surface hasta ADR concurrencia.

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| Mutex/async hoy | park | ROADMAP PARK |
| data race via shared mut | E0202 + park | — |

---

## 8. Módulos y dependencias

**R8.1** `module` + fn; crates workspace emit-Rust (`#![forbid(unsafe_code)]`).  
**R8.2** Sin deps IDNI en producto; isla lógica propia (F3) ortogonal.  
**R8.3** Dependencias Rust solo vía codegen controlado (stdlib subset whitelist E0206).

| AI imposible | Emit | Evidencia |
|--------------|------|-----------|
| `unsafe` / deps fantasma | forbid + whitelist | clippy/measure; E0206 |
| copiar IDNI | política ADR-002 | review |

---

## 9. Evidencia automatizada (cadena)

Toda regla R* debe amarrarse a ≥1 de:

1. **arita measure** (oráculo compile+run / neg E0xxx)  
2. **Rust twin** (bootstrap GOLDEN_*)  
3. **cargo test** crates  
4. **DOC/TRAPS + STABLE_VERIFY**  

`skip ≠ PASS`; `inconclusive` ≠ accepted.

---

## 10. Orden de desbloqueo post-mapa

1. Aceptar este ADR (<person> + Arquitecto).  
2. Addenda contratos: **IndexGet** (`[]` o `get` unificado), **Insert**, **Mutex**.  
3. CI remoto certifica contratos.  
4. GO IMPL puntual por CUT (Ingeniero sole).

## 11. Checklist

- [x] Borrador mapa R0–R8 + evidencia  
- [x] ACK <person> vía RFC rev. 2  
- [x] Addenda IndexGet / Insert / Mutex (ADR-227/228/229)  
- [ ] Firmas Docs (THREAT_MODEL + ROADMAP tick)  
- [ ] Cadena evidencia ADR-230  

## Cola inmediata

**HOLD** ADR-230 / R4 reinicio hasta aviso+GO <person> (tesis rev. 3). Docs: ROADMAP Core 0.1. HOLD insert/`[]`/Mutex.

---

## 12. Gaps evidencia (Ingeniero, 2026-09-19 — lectura; cero IMPL)

Fuente: Lex measure + DOC en box `/workspace/arita/DOC/` (sync Lex Documents pendiente).

| Regla | Estado measure | Gap |
|-------|----------------|-----|
| **R0** backdoor | review/grep | Falta job/oráculo que **falle** si emit mete helper/panic no nombrable en surface |
| **R1** ownership | fuerte e0201/e0202/move | Matriz Copy vs move en Option/Result anidados incompleta |
| **R2** fallos/panic | fuerte e0216/e027–e029 | Sin barrido global «emit sin unwrap/expect/panic!» ni suite unificada OOB no-lit |
| **R3** mut | OK neg-e0202* | Ninguno material (Mutex → R7) |
| **R4** index | park DOC | Falta neg oracle estable `v[i]`/`s[i]` → E0xxx (parse fail inconsistente hoy) |
| **R5** colecciones | append/extend OK | `insert`/`drain`/closures: park DOC sin neg-E0206 dedicado |
| **R6** efectos | Io + try_reserve OK | Sin oracle «fn sin Io no puede print»; fs/net OUT = ausencia no park medido |
| **R7** async | e0240/41/42 OK | Mutex park DOC-only (sin neg surface `Mutex`) |
| **R8** módulos | forbid(unsafe)+E0206 OK | CI Origin remoto aún no certifica; anti-IDNI = política no measure |

**Post-ACK <person>:** addenda contratos IndexGet / Insert / Mutex + cola gaps R0/R2/R4/R5/R7 como CUT evidencia (no std feature).

HOLD insert/`[]`/Mutex + std CUT aislados hasta ACK + addenda.


Ampliado por RFC-AINATIVE-VERIFIED-MODEL.md.


---

**Nota rev. 2 (2026-09-19):** Perfiles y significado de «verificable» → RFC rev. 2 (`safe`/`service`/`sandboxed`/`high-assurance`/`ffi`). Contratos formales opcionales; Núcleo 1 = expresividad GP con APIs fallible/total.

**Addenda 2026-09-19:** ADR-227 IndexGet · ADR-228 Insert · ADR-229 Mutex · ADR-230 evidencia.

---

**Nota rev. 3 (2026-09-19):** Identidad de producto = **lenguaje AI-native → Rust real** (RFC rev. 3 / PRODUCT-VISION). Rev. 2 (perfiles proporcionales) queda como *policy layer*, no como definición del producto. R4/evidence HOLD; no GO IMPL hasta reinicio <person>.
