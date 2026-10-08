# ADR-198 — Result `flatten()` v0

- **Estado:** **cerrada** Lex **514/514**
- **CUT-ID:** `STD-RESULT-FLATTEN-20260919`
- IN: `r.flatten() → Result` on `Result<Result<T,E>,E>`; emit `.flatten()`.
- Minimal nest surface (parity ADR-184 Option); non-nested / Int → E0206.
- Oracles: Ok(Ok), Ok(Err), Err outer, neg-e0206. Baseline 510 → ~514.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD (minimal pest nest exception like ADR-184).
