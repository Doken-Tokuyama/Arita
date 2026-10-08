//! ARITA emit-Rust. F1/F2/async-v0 surface → Rust (functions, calls, Let/Lit*/print/Vec, tests).
//! ADR-020: `emit_rust_with_entry` rewrites Rust `main` to call contract `target` when ≠ main.
//! ADR-027: `async fn` → Rust async; `async fn main` → `#[tokio::main(flavor = "current_thread")]`.
//! ADR-028: `BuildProfile` + `emit_cargo_toml` (`opt-level = 3` on release; LTO off).
//! ADR-292: release also carries `overflow-checks = true` (uniform panic on Int overflow).
//! ADR-029: deps whitelist v0 (tokio-only bridge); emit pins from whitelist when declared.
//! ADR-035 / 033b: `[host-bridges]` path deps + `host::<fn>` emit rewrite to host crate.
//! ADR-047: Result; ADR-050: Option/Some/None.

use arita_syntax::{BinOp, Call, Expr, Function, Module, Pat, Path, Stmt, Type}; // Match via Expr::Match

/// ADR-160: injected when `truncate` appears (String floor_char_boundary; Vec len clamp).
const ARITA_TRUNCATE_HELPER: &str = r#"
trait __AritaTruncate {
    fn __arita_truncate(&mut self, n: i64);
}
impl __AritaTruncate for String {
    fn __arita_truncate(&mut self, n: i64) {
        let n = n.max(0) as usize;
        let b = self.floor_char_boundary(n);
        self.truncate(b);
    }
}
impl<T> __AritaTruncate for Vec<T> {
    fn __arita_truncate(&mut self, n: i64) {
        self.truncate(n.max(0) as usize);
    }
}
"#;

fn module_uses_truncate(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "truncate" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

/// ADR-162: injected when `split_off` appears (String floor; Vec clamp; at>len → empty).
const ARITA_SPLIT_OFF_HELPER: &str = r#"
trait __AritaSplitOff {
    type Out;
    fn __arita_split_off(&mut self, at: i64) -> Self::Out;
}
impl __AritaSplitOff for String {
    type Out = String;
    fn __arita_split_off(&mut self, at: i64) -> String {
        let at = at.max(0) as usize;
        let b = self.floor_char_boundary(at);
        self.split_off(b)
    }
}
impl<T> __AritaSplitOff for Vec<T> {
    type Out = Vec<T>;
    fn __arita_split_off(&mut self, at: i64) -> Vec<T> {
        let at = (at.max(0) as usize).min(self.len());
        self.split_off(at)
    }
}
"#;

fn module_uses_split_off(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "split_off" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

/// ADR-260: fallible Vec insert — never panics (i<0 or i>len → Err(0)).
/// ADR-283 R1 (CORE-0.9-VEC-ASSIGN-20260926): `usize::try_from(i)` instead of `as usize`;
/// conversion failure (i<0) → Err(0). Behaviour identical (oracle core09-vec-helpers-semantics-unchanged).
/// In-fn `use std::convert::TryFrom as _;`: `arita build` single-file rustc = edition 2015 (no
/// TryFrom in prelude). The underscore import is not flagged by `unused_imports` nor by
/// `redundant_imports` on 2021+ (Ingeniero FASE 2 cond. 1; oracle adr283_tryfrom_import_clippy_2015_2021).
const ARITA_VEC_INSERT_HELPER: &str = r#"
fn __arita_vec_insert<T>(v: &mut Vec<T>, i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom as _;
    let Ok(u) = usize::try_from(i) else {
        return Err(0);
    };
    if u > v.len() {
        return Err(0);
    }
    v.insert(u, x);
    Ok(())
}
"#;

/// ADR-265 / CORE-0.6-SET-FALLIBLE-20260926: fallible Vec set — never panics
/// (i<0 or i>=len → Err(0)); replace in place; does NOT grow len.
/// ADR-283 R1 (CORE-0.9-VEC-ASSIGN-20260926): `usize::try_from(i)` + `get_mut` (no `as usize`);
/// conversion failure (i<0) or i>=len → Err(0). Behaviour identical to the ADR-265 helper.
/// `&mut [T]` (call sites pass `&mut v`, v: Vec → deref coercion): clears clippy `ptr_arg`
/// that the ADR-265 `&mut Vec<T>` signature triggered on emitted code.
const ARITA_VEC_SET_HELPER: &str = r#"
fn __arita_vec_set<T>(v: &mut [T], i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom as _;
    let Ok(u) = usize::try_from(i) else {
        return Err(0);
    };
    match v.get_mut(u) {
        Some(slot) => {
            *slot = x;
            Ok(())
        }
        None => Err(0),
    }
}
"#;

/// ADR-290 slice A (CORE-0.10-INDEX-MUT-20261002): fallible Vec<Int> compound update
/// `v[i] op= x` for `op` in {`+`, `-`, `*`} — never panics. One lookup (`get_mut`): `i` not
/// convertible to `usize` (i < 0), `i >= len`, or `op(cur, x) == None` (checked overflow) →
/// `Err(0)` (single code, ADR-290 D3/D5); the slot is written ONLY on success, so `v` is intact
/// on every `Err`. No `+`/`-`/`*` in the emitted Rust → identical in debug and release.
/// R1 (ADR-283): `usize::try_from`, no `as usize`. `op` is `i64::checked_add|sub|mul` passed as
/// `fn(i64, i64) -> Option<i64>`; `&mut [i64]` (call sites pass `&mut v`, deref coercion) as
/// `__arita_vec_set`. Measured clippy `-D warnings` + pedantic-clean, rustc-warning-free on
/// 2015 and 2021 (DOC/reviews PREP_ADR290_CODEGEN §3 + /tmp/arita_adr290_p7, form A with `n`→`new`).
const ARITA_VEC_UPDATE_HELPER: &str = r#"
fn __arita_vec_update(v: &mut [i64], i: i64, x: i64, op: fn(i64, i64) -> Option<i64>) -> Result<(), i64> {
    use std::convert::TryFrom as _;
    let Ok(u) = usize::try_from(i) else {
        return Err(0);
    };
    let Some(slot) = v.get_mut(u) else {
        return Err(0);
    };
    let Some(new) = op(*slot, x) else {
        return Err(0);
    };
    *slot = new;
    Ok(())
}
"#;

fn module_uses_vec_insert(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "insert" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

fn module_uses_vec_set(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "set" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
        // ADR-283: `v[i] = x` on a Vec local emits `__arita_vec_set` (even as its only use).
        // ADR-290: plain `=` only — a compound `v[i] op= x` uses `__arita_vec_update` instead.
        || module_has_index_assign_on(module, |t| matches!(t, Type::Vec(_)), false)
}

/// ADR-290: true when some fn/test body has a COMPOUND index-assign (`v[i] op= x`) whose target is
/// a `Vec` local/param. The only way to need `__arita_vec_update` (no method name to scan for).
fn module_uses_vec_update(module: &Module) -> bool {
    module_has_index_assign_on(module, |t| matches!(t, Type::Vec(_)), true)
}

/// ADR-192: next_multiple_of / checked_next_multiple_of (stable; signed int_roundings unstable).
const ARITA_NEXT_MULTIPLE_HELPER: &str = r#"
trait __AritaNextMultiple {
    fn __arita_next_multiple_of(self, m: i64) -> i64;
    fn __arita_checked_next_multiple_of(self, m: i64) -> Option<i64>;
    fn __arita_previous_multiple_of(self, m: i64) -> i64;
    fn __arita_checked_previous_multiple_of(self, m: i64) -> Option<i64>;
}
impl __AritaNextMultiple for i64 {
    fn __arita_next_multiple_of(self, m: i64) -> i64 {
        self.__arita_checked_next_multiple_of(m).unwrap_or(i64::MAX)
    }
    fn __arita_checked_next_multiple_of(self, m: i64) -> Option<i64> {
        if m == 0 {
            return None;
        }
        let r = self.checked_rem(m)?;
        if r == 0 {
            return Some(self);
        }
        if (self > 0) == (m > 0) {
            self.checked_add(m - r)
        } else {
            Some(self - r)
        }
    }
    fn __arita_previous_multiple_of(self, m: i64) -> i64 {
        self.__arita_checked_previous_multiple_of(m).unwrap_or(i64::MIN)
    }
    fn __arita_checked_previous_multiple_of(self, m: i64) -> Option<i64> {
        if m == 0 {
            return None;
        }
        let r = self.checked_rem(m)?;
        if r == 0 {
            return Some(self);
        }
        if (self > 0) == (m > 0) {
            self.checked_sub(r)
        } else {
            self.checked_sub(m.checked_add(r)?)
        }
    }
}
"#;

fn module_uses_next_multiple(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => {
                method == "next_multiple_of"
                    || method == "checked_next_multiple_of"
                    || method == "previous_multiple_of"
                    || method == "checked_previous_multiple_of"
                    || expr_has(receiver)
                    || args.iter().any(expr_has)
            }
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

/// ADR-188: signed div_ceil/div_floor (stable helper; rustc 1.97 i64 int_roundings still unstable).
const ARITA_DIV_ROUND_HELPER: &str = r#"
trait __AritaDivRound {
    fn __arita_div_ceil(self, b: i64) -> i64;
    fn __arita_div_floor(self, b: i64) -> i64;
}
impl __AritaDivRound for i64 {
    fn __arita_div_ceil(self, b: i64) -> i64 {
        let (d, r) = (self / b, self % b);
        if (r > 0 && b > 0) || (r < 0 && b < 0) {
            d + 1
        } else {
            d
        }
    }
    fn __arita_div_floor(self, b: i64) -> i64 {
        let (d, r) = (self / b, self % b);
        if (r > 0 && b < 0) || (r < 0 && b > 0) {
            d - 1
        } else {
            d
        }
    }
}
"#;

/// ADR-214: saturating_rem (stable helper; i64 has no saturating_rem on rustc 1.97).
/// Semantics: rem; overflow case MIN % -1 → 0 (parity wrapping_rem).
const ARITA_SATURATING_REM_HELPER: &str = r#"
trait __AritaSaturatingRem {
    fn __arita_saturating_rem(self, b: i64) -> i64;
}
impl __AritaSaturatingRem for i64 {
    fn __arita_saturating_rem(self, b: i64) -> i64 {
        self.wrapping_rem(b)
    }
}
"#;

/// ADR-218/220: is_power_of_two + checked_next_power_of_two (stable helpers; i64 lacks both on rustc 1.97).
const ARITA_IS_POWER_OF_TWO_HELPER: &str = r#"
trait __AritaIsPowerOfTwo {
    fn __arita_is_power_of_two(self) -> bool;
    fn __arita_checked_next_power_of_two(self) -> Option<i64>;
}
impl __AritaIsPowerOfTwo for i64 {
    fn __arita_is_power_of_two(self) -> bool {
        self > 0 && (self as u64).is_power_of_two()
    }
    fn __arita_checked_next_power_of_two(self) -> Option<i64> {
        // ADR-220: n≤0 or next power > i64::MAX → None
        if self <= 0 {
            return None;
        }
        match (self as u64).checked_next_power_of_two() {
            Some(u) if u <= i64::MAX as u64 => Some(u as i64),
            _ => None,
        }
    }
}
"#;

fn module_uses_is_power_of_two(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => {
                method == "is_power_of_two"
                    || method == "checked_next_power_of_two"
                    || expr_has(receiver)
                    || args.iter().any(expr_has)
            }
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

fn module_uses_saturating_rem(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "saturating_rem" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

fn module_uses_div_round(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => {
                method == "div_ceil"
                    || method == "div_floor"
                    || expr_has(receiver)
                    || args.iter().any(expr_has)
            }
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

/// ADR-170: Vec rotate (len0/neg → no-op; n % len). ADR-131 Int bit-rotate via same method name.
const ARITA_ROTATE_HELPER: &str = r#"
trait __AritaIntRotate {
    fn __arita_rotate_left(self, n: i64) -> i64;
    fn __arita_rotate_right(self, n: i64) -> i64;
}
impl __AritaIntRotate for i64 {
    fn __arita_rotate_left(self, n: i64) -> i64 {
        self.rotate_left(n.max(0) as u32)
    }
    fn __arita_rotate_right(self, n: i64) -> i64 {
        self.rotate_right(n.max(0) as u32)
    }
}
trait __AritaVecRotate {
    fn __arita_rotate_left(&mut self, n: i64);
    fn __arita_rotate_right(&mut self, n: i64);
}
impl<T> __AritaVecRotate for Vec<T> {
    fn __arita_rotate_left(&mut self, n: i64) {
        let len = self.len();
        if len == 0 || n < 0 {
            return;
        }
        self.rotate_left((n as usize) % len);
    }
    fn __arita_rotate_right(&mut self, n: i64) {
        let len = self.len();
        if len == 0 || n < 0 {
            return;
        }
        self.rotate_right((n as usize) % len);
    }
}
"#;

fn module_uses_rotate(module: &Module) -> bool {
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => {
                method == "rotate_left"
                    || method == "rotate_right"
                    || expr_has(receiver)
                    || args.iter().any(expr_has)
            }
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            _ => false,
        }
    }
    fn stmts_have(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Let { init, .. } => expr_has(init),
            Stmt::Expr(e) | Stmt::Assign { value: e, .. } => expr_has(e),
            // ADR-282: `m[k] = v` — visit key/value exprs.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::Assert { lhs, rhs } => expr_has(lhs) || expr_has(rhs),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || stmts_have(then_body)
                    || else_body.as_ref().map(|b| stmts_have(b)).unwrap_or(false)
            }
            Stmt::While { cond, body } => expr_has(cond) || stmts_have(body),
            Stmt::WhileLet {
                scrutinee, body, ..
            } => expr_has(scrutinee) || stmts_have(body),
            _ => false,
        })
    }
    module.functions.iter().any(|f| stmts_have(&f.body))
        || module.tests.iter().any(|t| stmts_have(&t.body))
}

/// Emit Rust with entry `main` (default).
pub fn module_uses_map(module: &Module) -> bool {
    fn ty_has(t: &Type) -> bool {
        match t {
            Type::Map(_, _) => true,
            Type::Vec(i) | Type::Option(i) => ty_has(i),
            Type::Result(a, b) => ty_has(a) || ty_has(b),
            _ => false,
        }
    }
    fn expr_has(e: &Expr) -> bool {
        match e {
            Expr::Call(c) if c.callee == "Map::new" => true,
            Expr::MethodCall {
                method,
                receiver,
                args,
            } => method == "put" || expr_has(receiver) || args.iter().any(expr_has),
            Expr::Binary { lhs, rhs, .. } => expr_has(lhs) || expr_has(rhs),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_has(inner),
            Expr::Call(c) => c.args.iter().any(expr_has),
            Expr::Match { scrutinee, arms } => {
                expr_has(scrutinee) || arms.iter().any(|a| expr_has(&a.value))
            }
            Expr::RecordLit { fields, .. } => fields.iter().any(|(_, e)| expr_has(e)),
            Expr::FieldAccess { base, .. } => expr_has(base),
            _ => false,
        }
    }
    fn stmt_has(s: &Stmt) -> bool {
        match s {
            Stmt::Let { ty, init, .. } => ty_has(ty) || expr_has(init),
            Stmt::Expr(e) | Stmt::Assert { lhs: e, .. } => expr_has(e),
            // ADR-282/283: `t[k] = v` needs HashMap only when `t` is a Map local (checked at
            // module level by `module_has_index_assign_on`); a Vec target must not import it.
            Stmt::IndexAssign { key, value, .. } => expr_has(key) || expr_has(value),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                expr_has(cond)
                    || then_body.iter().any(stmt_has)
                    || else_body
                        .as_ref()
                        .map(|b| b.iter().any(stmt_has))
                        .unwrap_or(false)
            }
            _ => false,
        }
    }
    module
        .functions
        .iter()
        .any(|f| ty_has(&f.ret_ty) || f.body.iter().any(stmt_has))
        || module
            .records
            .iter()
            .any(|r| r.fields.iter().any(|f| ty_has(&f.ty)))
        || module_has_index_assign_on(module, |t| matches!(t, Type::Map(_, _)), false)
}

/// ADR-282/283: true when some fn/test body contains `t[k] = v` (at any nesting depth) whose
/// target `t` is declared in that same body (let, any depth) or as a param with a type
/// satisfying `pred`. Structural walk only — used to decide which runtime items to emit.
/// ADR-290: `compound` selects the statement kind — `false` = plain `=` (`op` is `None`),
/// `true` = compound `op=`.
fn module_has_index_assign_on(module: &Module, pred: fn(&Type) -> bool, compound: bool) -> bool {
    fn walk<'a>(
        stmts: &'a [Stmt],
        lets: &mut Vec<(&'a str, &'a Type)>,
        targets: &mut Vec<(&'a str, bool)>,
    ) {
        for s in stmts {
            match s {
                Stmt::Let { name, ty, .. } => lets.push((name.as_str(), ty)),
                // ADR-290: `op: Option<BinOp>` on `Stmt::IndexAssign` (`None` = plain `=`).
                Stmt::IndexAssign { target, op, .. } => {
                    targets.push((target.as_str(), op.is_some()))
                }
                Stmt::If {
                    then_body,
                    else_body,
                    ..
                }
                | Stmt::IfLet {
                    then_body,
                    else_body,
                    ..
                } => {
                    walk(then_body, lets, targets);
                    if let Some(b) = else_body {
                        walk(b, lets, targets);
                    }
                }
                Stmt::While { body, .. } | Stmt::WhileLet { body, .. } => walk(body, lets, targets),
                Stmt::Match { arms, .. } => {
                    for a in arms {
                        walk(&a.body, lets, targets);
                    }
                }
                _ => {}
            }
        }
    }
    let body_hits = |params: &[(&str, &Type)], body: &[Stmt]| {
        let mut lets: Vec<(&str, &Type)> = params.to_vec();
        let mut targets = Vec::new();
        walk(body, &mut lets, &mut targets);
        targets
            .iter()
            .filter(|(_, is_compound)| *is_compound == compound)
            .any(|(t, _)| lets.iter().any(|(n, ty)| n == t && pred(ty)))
    };
    module.functions.iter().any(|f| {
        let params: Vec<(&str, &Type)> =
            f.params.iter().map(|p| (p.name.as_str(), &p.ty)).collect();
        body_hits(&params, &f.body)
    }) || module.tests.iter().any(|t| body_hits(&[], &t.body))
}

/// ADR-283: the parser's synthetic `<arita:deferred-shape>` call (HIR always rejects it) must
/// never reach emitted Rust. Returns the deferred diagnostic as a codegen error if present.
fn module_deferred_shape_error(module: &Module) -> Result<(), String> {
    fn expr_find(e: &Expr) -> Option<String> {
        match e {
            Expr::Call(c) if c.callee == arita_syntax::DEFERRED_SHAPE_MARKER => {
                let code = match c.args.first() {
                    Some(Expr::LitStr(s)) => s.clone(),
                    _ => "E0006".to_string(),
                };
                let msg = match c.args.get(1) {
                    Some(Expr::LitStr(s)) => s.clone(),
                    _ => String::new(),
                };
                Some(format!(
                    "{code}: {msg} (ADR-283: deferred-shape marker reached codegen; HIR must reject it)"
                ))
            }
            Expr::Call(c) => c.args.iter().find_map(expr_find),
            Expr::MethodCall { receiver, args, .. } => {
                expr_find(receiver).or_else(|| args.iter().find_map(expr_find))
            }
            Expr::Binary { lhs, rhs, .. } => expr_find(lhs).or_else(|| expr_find(rhs)),
            Expr::Borrow { inner, .. }
            | Expr::Await { inner }
            | Expr::Try { inner }
            | Expr::Ok(inner)
            | Expr::Err(inner)
            | Expr::Some(inner) => expr_find(inner),
            Expr::Match { scrutinee, arms } => {
                expr_find(scrutinee).or_else(|| arms.iter().find_map(|a| expr_find(&a.value)))
            }
            Expr::RecordLit { fields, .. } => fields.iter().find_map(|(_, e)| expr_find(e)),
            Expr::FieldAccess { base, .. } => expr_find(base),
            _ => None,
        }
    }
    fn stmts_find(stmts: &[Stmt]) -> Option<String> {
        stmts.iter().find_map(|s| match s {
            Stmt::Let { init: e, .. } | Stmt::Expr(e) | Stmt::Assign { value: e, .. } => {
                expr_find(e)
            }
            Stmt::IndexAssign { key, value, .. } => expr_find(key).or_else(|| expr_find(value)),
            Stmt::Assert { lhs, rhs } => expr_find(lhs).or_else(|| expr_find(rhs)),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => expr_find(cond)
                .or_else(|| stmts_find(then_body))
                .or_else(|| else_body.as_deref().and_then(stmts_find)),
            Stmt::IfLet {
                scrutinee,
                then_body,
                else_body,
                ..
            } => expr_find(scrutinee)
                .or_else(|| stmts_find(then_body))
                .or_else(|| else_body.as_deref().and_then(stmts_find)),
            Stmt::While { cond: e, body }
            | Stmt::WhileLet {
                scrutinee: e, body, ..
            } => expr_find(e).or_else(|| stmts_find(body)),
            Stmt::Match { scrutinee, arms } => {
                expr_find(scrutinee).or_else(|| arms.iter().find_map(|a| stmts_find(&a.body)))
            }
            _ => None,
        })
    }
    match module
        .functions
        .iter()
        .find_map(|f| stmts_find(&f.body))
        .or_else(|| module.tests.iter().find_map(|t| stmts_find(&t.body)))
    {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

thread_local! {
    /// ADR-237: let-binding types for Map vs Vec `get` emit dispatch.
    static EMIT_LOCAL_TYPES: std::cell::RefCell<std::collections::HashMap<String, Type>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    /// ADR-286 S1b: user fns visible to the current emit (module + deps): name → (is_async, ret_ty).
    /// Used by the `join(spawn(g()))` guard (P6) and by shadowed-builtin E0006 messages.
    static EMIT_FN_SIGS: std::cell::RefCell<std::collections::HashMap<String, (bool, Type)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    /// ADR-286 S1b: most specific `E0006` reason recorded by a nested emit site that returned
    /// `None`; consumed when the enclosing fn/test turns the `None` into an `Err`.
    static EMIT_DIAG: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// ADR-286 S1b: restores the previous fn-signature table when an emit entry point returns.
struct EmitFnSigsGuard {
    prev: std::collections::HashMap<String, (bool, Type)>,
}

impl Drop for EmitFnSigsGuard {
    fn drop(&mut self) {
        let prev = std::mem::take(&mut self.prev);
        EMIT_FN_SIGS.with(|s| *s.borrow_mut() = prev);
        // A failed emit must not leave a stale reason for the next one either.
        clear_emit_diag();
    }
}

/// ADR-286 S1b: signature of a fn that lives outside the emitted module (e.g. a `pub fn` of the
/// workspace lib that `package.rs` strips from the bin module before emit). Lets the
/// `join(spawn(g()))` guard (P6) know `g`'s `(async, ret)` without `g` being emitted here.
#[derive(Clone, Debug, PartialEq)]
pub struct ExternFnSig {
    pub name: String,
    pub is_async: bool,
    pub ret_ty: Type,
}

impl ExternFnSig {
    pub fn from_function(f: &Function) -> Self {
        Self {
            name: f.name.clone(),
            is_async: f.is_async,
            ret_ty: f.ret_ty.clone(),
        }
    }
}

/// ADR-286 S1b: register `externs` and the fns of `modules` for the current emit. Own-module
/// fns win over externs of the same name. EVERY emit entry point holds the returned guard for
/// its whole body; `Drop` restores the previous table on Ok, Err, `?` and unwinding, so
/// signatures never leak into the next emit.
fn enter_fn_sigs<'a>(
    externs: &[ExternFnSig],
    modules: impl IntoIterator<Item = &'a Module>,
) -> EmitFnSigsGuard {
    let prev = EMIT_FN_SIGS.with(|s| s.borrow().clone());
    EMIT_FN_SIGS.with(|s| {
        let mut s = s.borrow_mut();
        for e in externs {
            s.insert(e.name.clone(), (e.is_async, e.ret_ty.clone()));
        }
        for m in modules {
            for f in &m.functions {
                s.insert(f.name.clone(), (f.is_async, f.ret_ty.clone()));
            }
        }
    });
    EmitFnSigsGuard { prev }
}

fn lookup_fn_sig(name: &str) -> Option<(bool, Type)> {
    EMIT_FN_SIGS.with(|s| s.borrow().get(name).cloned())
}

/// ADR-286 S1b: record a precise `E0006` reason (first one wins until consumed).
fn set_emit_diag(msg: String) {
    EMIT_DIAG.with(|d| {
        let mut d = d.borrow_mut();
        if d.is_none() {
            *d = Some(msg);
        }
    });
}

fn take_emit_diag() -> Option<String> {
    EMIT_DIAG.with(|d| d.borrow_mut().take())
}

fn clear_emit_diag() {
    EMIT_DIAG.with(|d| *d.borrow_mut() = None);
}

/// Builtin callees with a dedicated emit arm (a user fn with one of these names never
/// reaches `emit_user_call`).
const EMIT_BUILTIN_CALLEES: &[&str] = &[
    "print",
    "Vec::new",
    "List::new",
    "Map::new",
    "spawn",
    "join",
    "timeout",
    "cancel_token",
    "cancel",
    "delay",
    "until_cancelled",
];

fn stmt_kind_name(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::Let { .. } => "`let`",
        Stmt::Expr(_) => "expression",
        Stmt::Assert { .. } => "`assert`",
        Stmt::If { .. } => "`if`",
        Stmt::IfLet { .. } => "`if let`",
        Stmt::WhileLet { .. } => "`while let`",
        Stmt::While { .. } => "`while`",
        Stmt::Assign { .. } => "assignment",
        Stmt::IndexAssign { .. } => "index assignment",
        Stmt::Match { .. } => "`match`",
        Stmt::Break => "`break`",
        Stmt::Continue => "`continue`",
    }
}

/// ADR-286 S1b (§0.1d P3/P4): actionable `E0006` for a statement codegen cannot emit.
/// `ctx` names the enclosing item, e.g. "fn `f`" or "test `t`". Never a silent drop.
fn unemittable_stmt_error(stmt: &Stmt, ctx: &str) -> String {
    if let Some(d) = take_emit_diag() {
        return format!("{d} (in {ctx})");
    }
    let Stmt::Expr(expr) = stmt else {
        return format!(
            "E0006: cannot emit {} statement in {ctx}: a nested statement or expression is not supported by codegen (the statement would be dropped)",
            stmt_kind_name(stmt)
        );
    };
    let call = match expr {
        Expr::Call(c) => Some(c),
        Expr::Await { inner } => match inner.as_ref() {
            Expr::Call(c) => Some(c),
            _ => None,
        },
        _ => None,
    };
    if let Some(c) = call {
        let callee = c.callee.as_str();
        if EMIT_BUILTIN_CALLEES.contains(&callee) {
            if lookup_fn_sig(callee).is_some() {
                return format!(
                    "E0006: cannot emit call `{callee}(..)` in {ctx}: user fn `{callee}` shadows the builtin `{callee}` and the call does not match the builtin's shape (the call would be dropped); rename the fn"
                );
            }
            return format!(
                "E0006: cannot emit builtin call `{callee}(..)` in {ctx}: unsupported arity or argument shape (the statement would be dropped)"
            );
        }
        return format!(
            "E0006: cannot emit call statement `{callee}(..)` in {ctx} (the statement would be dropped)"
        );
    }
    match expr {
        Expr::MethodCall { method, .. } => format!(
            "E0006: cannot emit method call statement `.{method}(..)` in {ctx}: method not supported by codegen or wrong arity (the statement would be dropped)"
        ),
        Expr::Await { .. } | Expr::Try { .. } => format!(
            "E0006: cannot emit `await`/`?` statement in {ctx}: operand not supported by codegen (the statement would be dropped)"
        ),
        _ => format!(
            "E0006: cannot emit expression statement whose value is discarded in {ctx}: a value expression used as a statement would be dropped; bind it with `let` or remove it"
        ),
    }
}

/// Emit `module` with the default `main` entry (the `arita test` path).
/// ADR-286 S1b (§0.1d P1): an emit error is propagated as `E0006`, never replaced by a
/// runnable `fn main() {}` fallback.
pub fn emit_rust(module: &Module) -> Result<String, String> {
    emit_rust_with_entry(module, "main").map_err(|e| {
        if e.starts_with("E0006:") {
            e
        } else {
            format!("E0006: cannot emit module `{}`: {e}", module.name)
        }
    })
}

/// ADR-027: true when any `async fn` is present (needs tokio Cargo bridge).
pub fn needs_async_runtime(module: &Module) -> bool {
    module.functions.iter().any(|f| f.is_async)
}

/// ADR-028: build profile for emit / cargo (`arita build --profile`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildProfile {
    /// Default: unoptimized / Cargo `dev` (no forced opt-level).
    Debug,
    /// Release: Cargo `--release` with `[profile.release] opt-level = 3` (LTO off).
    Release,
}

impl BuildProfile {
    /// Parse CLI profile name. Unknown → **E0250** `unknown build profile`.
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "debug" => Ok(Self::Debug),
            "release" => Ok(Self::Release),
            other => Err(format!("E0250: unknown build profile `{other}`")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

/// ADR-027 pinned tokio Cargo.toml fragment for generated crates.
pub const TOKIO_DEP: &str = r#"tokio = { version = "=1.53.1", default-features = false, features = ["rt", "macros", "time", "sync"] }"#;

/// ADR-029: one declared dependency from `arita.toml` `[deps]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredDep {
    pub name: String,
    pub bridge: String,
}

/// ADR-035: one declared host bridge from `arita.toml` `[host-bridges]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredHostBridge {
    /// Manifest key (e.g. `demo`).
    pub id: String,
    /// Cargo package name (e.g. `arita-host-demo`).
    pub crate_name: String,
    /// Path relative to workspace root (e.g. `crates/arita-host-demo`).
    pub path: String,
}

impl DeclaredHostBridge {
    /// Rust crate ident for `use` / path (`-` → `_`).
    pub fn rust_crate_ident(&self) -> String {
        self.crate_name.replace('-', "_")
    }
}

/// Validate `[host-bridges]` entries (layout pin: `crates/arita-host-*`).
pub fn validate_host_bridges(bridges: &[DeclaredHostBridge]) -> Result<(), String> {
    for b in bridges {
        let lower = b.id.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "secret" | "api_key" | "apikey" | "token" | "password" | "demo_key" | "demokey"
        ) {
            return Err(
                "E0260: dependency not in ARITA whitelist (demo keys / secrets not allowed)".into(),
            );
        }
        if !b.crate_name.starts_with("arita-host-") {
            return Err(format!(
                "host bridge crate must be `arita-host-*` (got `{}`)",
                b.crate_name
            ));
        }
        if !b.path.starts_with("crates/arita-host-") {
            return Err(format!(
                "host bridge path must be under `crates/arita-host-*` (got `{}`)",
                b.path
            ));
        }
    }
    Ok(())
}

/// Rewrite intermediate `host::fn` callees in emitted Rust to `<host_crate>::fn`.
pub fn rewrite_host_calls(
    rust_src: &str,
    bridges: &[DeclaredHostBridge],
) -> Result<String, String> {
    if !rust_src.contains("host::") {
        return Ok(rust_src.to_string());
    }
    let Some(b) = bridges.first() else {
        return Err("host bridge call requires [host-bridges] in arita.toml (ADR-035)".into());
    };
    let ident = b.rust_crate_ident();
    Ok(rust_src.replace("host::", &format!("{ident}::")))
}

/// ADR-029 whitelist entry (SoT pins live here + ADR-029 DOC).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhitelistEntry {
    pub crate_name: &'static str,
    pub bridge_id: &'static str,
    pub cargo_dep: &'static str,
}

/// Whitelist v0 = tokio-only (formalizes ADR-027). Ampliar = CUT nuevo.
pub const WHITELIST_V0: &[WhitelistEntry] = &[WhitelistEntry {
    crate_name: "tokio",
    bridge_id: "async-runtime",
    cargo_dep: TOKIO_DEP,
}];

/// Look up a crate in the ADR-029 whitelist.
pub fn whitelist_lookup(name: &str) -> Option<&'static WhitelistEntry> {
    WHITELIST_V0.iter().find(|e| e.crate_name == name)
}

/// Validate declared deps against whitelist. Unknown / wrong bridge → **E0260**.
pub fn validate_declared_deps(deps: &[DeclaredDep]) -> Result<(), String> {
    for d in deps {
        let Some(entry) = whitelist_lookup(&d.name) else {
            return Err(format!(
                "E0260: dependency not in ARITA whitelist (`{}`)",
                d.name
            ));
        };
        if d.bridge != entry.bridge_id {
            return Err(format!(
                "E0260: dependency not in ARITA whitelist (`{}` bridge `{}`; expected `{}`)",
                d.name, d.bridge, entry.bridge_id
            ));
        }
    }
    Ok(())
}

/// Resolve whether the generated Cargo.toml must include the tokio pin.
///
/// - Declared `[deps] tokio` → true (after validate).
/// - `needs_async` without manifest → true (ADR-027 path continues to work).
/// - Declared deps validated first.
pub fn resolve_with_tokio(declared: &[DeclaredDep], needs_async: bool) -> Result<bool, String> {
    validate_declared_deps(declared)?;
    if declared.iter().any(|d| d.name == "tokio") {
        return Ok(true);
    }
    Ok(needs_async)
}

/// Emit Cargo.toml for a generated binary crate.
///
/// - `with_tokio`: ADR-027/029 tokio whitelist pin (`=1.53.1`, `rt`+`macros`).
/// - `profile == Release`: appends `[profile.release]` with `opt-level = 3` (LTO off / omitted)
///   and, ADR-292, `overflow-checks = true` (same panic on `+ - *` overflow as debug).
/// - `profile == Debug`: no forced opt-level (Cargo `dev` baseline; `dev` already checks overflow).
pub fn emit_cargo_toml(package_name: &str, profile: BuildProfile, with_tokio: bool) -> String {
    emit_cargo_toml_with_hosts(package_name, profile, with_tokio, &[])
}

/// ADR-292: the `[profile.release]` line that turns Rust's overflow panic on in release builds.
/// Shared by the single-crate `Cargo.toml` (this module) and the package workspace root
/// (`arita-cli` `package.rs`), so both routes carry exactly the same text.
pub const RELEASE_OVERFLOW_CHECKS_LINE: &str = "overflow-checks = true";

/// Emit Cargo.toml; `host_path_deps` is `(crate_name, absolute_path)` for ADR-035 host-bridges.
pub fn emit_cargo_toml_with_hosts(
    package_name: &str,
    profile: BuildProfile,
    with_tokio: bool,
    host_path_deps: &[(String, String)],
) -> String {
    let mut out = format!(
        r#"[package]
name = "{package_name}"
version = "0.0.0"
edition = "2021"

# Keep generated crate out of the ARITA workspace.
[workspace]
"#
    );
    if with_tokio || !host_path_deps.is_empty() {
        out.push_str("\n[dependencies]\n");
        if with_tokio {
            out.push_str(TOKIO_DEP);
            out.push('\n');
        }
        for (crate_name, abs_path) in host_path_deps {
            let path_esc = abs_path.replace('\\', "/").replace('"', "");
            out.push_str(&format!("{crate_name} = {{ path = \"{path_esc}\" }}\n"));
        }
    }
    if profile == BuildProfile::Release {
        out.push_str("\n[profile.release]\nopt-level = 3\n");
        out.push_str(RELEASE_OVERFLOW_CHECKS_LINE);
        out.push('\n');
    }
    out
}

/// ADR-027 helper: async Cargo.toml (debug baseline; caller chooses cargo --release).
pub fn emit_async_cargo_toml(package_name: &str) -> String {
    emit_cargo_toml(package_name, BuildProfile::Debug, true)
}

/// ADR-020: emit with contract entry rewrite.
///
/// - `entry == "main"`: existing behavior (user `main` is Rust entry).
/// - `entry != "main"`: emit user `main` as `__arita_user_main`, emit helpers as usual,
///   then thin `fn main() { entry(); }`. Target must exist and be `fn() -> Io<()>`.
///
/// Returns `Err` with stable **E0230** when target is missing or not MVP-shaped.
///
/// ADR-254 / CORE-0.4-MULTI-MODULE: emit entry + local dependency modules as Rust `mod` + `use`.
/// `deps` are library modules (no `main`); their public fns are `pub` inside `mod <name> { … }`.
pub fn emit_rust_program(
    entry: &Module,
    deps: &[Module],
    entry_name: &str,
) -> Result<String, String> {
    module_deferred_shape_error(entry)?;
    for dep in deps {
        module_deferred_shape_error(dep)?;
    }
    let _sigs = enter_fn_sigs(&[], std::iter::once(entry).chain(deps.iter()));
    clear_emit_diag();
    let mut out = String::new();
    out.push_str(&format!(
        "// Generated by ARITA multi-module program (entry `{}`)\n",
        entry.name
    ));
    out.push_str("#![forbid(unsafe_code)]\n");

    for dep in deps {
        out.push_str(&format!("mod {} {{\n", dep.name));
        for func in &dep.functions {
            let body = emit_function(func).map_err(|e| {
                format!(
                    "E0006: cannot emit dep fn `{}::{}`: {}",
                    dep.name,
                    func.name,
                    e.trim_start_matches("E0006: ")
                )
            })?;
            let pub_body = if let Some(rest) = body.strip_prefix("fn ") {
                format!("pub fn {rest}")
            } else if let Some(rest) = body.strip_prefix("async fn ") {
                format!("pub async fn {rest}")
            } else if body.contains("\nfn ") {
                body.replacen("\nfn ", "\npub fn ", 1)
            } else {
                format!("pub {body}")
            };
            for line in pub_body.lines() {
                out.push_str("    ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str("}\n\n");
    }

    for u in &entry.uses {
        out.push_str(&format!("use {}::{};\n", u.module, u.item));
    }
    if !entry.uses.is_empty() {
        out.push('\n');
    }

    let entry_rs = emit_rust_with_entry(entry, entry_name)?;
    for line in entry_rs.lines() {
        if line.starts_with("// Generated by ARITA") {
            continue;
        }
        if line == "#![forbid(unsafe_code)]" {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    Ok(out)
}

/// ADR-255/256: emit a library module as `lib.rs`.
/// ADR-256: only `pub fn` / `pub record` are `pub` in Rust; private stay crate-private.
pub fn emit_rust_lib(module: &Module) -> Result<String, String> {
    let _sigs = enter_fn_sigs(&[], std::iter::once(module));
    clear_emit_diag();
    if module.functions.iter().any(|f| f.name == "main") {
        return Err("E0331: library module must not define main".into());
    }
    if module.functions.is_empty() && module.records.is_empty() {
        return Err("E0331: library module has no functions".into());
    }
    module_deferred_shape_error(module)?;
    let mut out = format!("// Generated by ARITA lib `{}`\n", module.name);
    out.push_str("#![forbid(unsafe_code)]\n");
    // ADR-263: lib may host fallible insert / other helpers — emit same runtime helpers as bins.
    if module_uses_map(module) {
        out.push_str("use std::collections::HashMap;\n");
    }
    if module_uses_truncate(module) {
        out.push_str(ARITA_TRUNCATE_HELPER);
    }
    if module_uses_split_off(module) {
        out.push_str(ARITA_SPLIT_OFF_HELPER);
    }
    if module_uses_vec_insert(module) {
        out.push_str(ARITA_VEC_INSERT_HELPER);
    }
    if module_uses_vec_set(module) {
        out.push_str(ARITA_VEC_SET_HELPER);
    }
    // ADR-290: `v[i] op= x` on a Vec<Int> local.
    if module_uses_vec_update(module) {
        out.push_str(ARITA_VEC_UPDATE_HELPER);
    }
    if module_uses_div_round(module) {
        out.push_str(ARITA_DIV_ROUND_HELPER);
    }
    if module_uses_saturating_rem(module) {
        out.push_str(ARITA_SATURATING_REM_HELPER);
    }
    if module_uses_is_power_of_two(module) {
        out.push_str(ARITA_IS_POWER_OF_TWO_HELPER);
    }
    if module_uses_next_multiple(module) {
        out.push_str(ARITA_NEXT_MULTIPLE_HELPER);
    }
    if module_uses_rotate(module) {
        out.push_str(ARITA_ROTATE_HELPER);
    }

    for rec in &module.records {
        let mut fields_rs = String::new();
        let mut all_copy = true;
        for f in &rec.fields {
            let ty = emit_type(&f.ty).ok_or_else(|| "E0006: record field type".to_string())?;
            if matches!(f.ty, Type::String) {
                all_copy = false;
            }
            fields_rs.push_str(&format!("    pub {}: {},\n", f.name, ty));
        }
        let vis = if rec.is_pub { "pub " } else { "" };
        let derives = if all_copy {
            "#[derive(Copy, Clone, Debug)]\n"
        } else {
            "#[derive(Clone, Debug)]\n"
        };
        out.push_str(derives);
        out.push_str(&format!("{vis}struct {} {{\n{}}}\n\n", rec.name, fields_rs));
    }

    for func in &module.functions {
        let body = emit_function(func).map_err(|e| {
            format!(
                "E0006: cannot emit lib fn `{}`: {}",
                func.name,
                e.trim_start_matches("E0006: ")
            )
        })?;
        let with_vis = if func.is_pub {
            if let Some(rest) = body.strip_prefix("fn ") {
                format!("pub fn {rest}")
            } else if let Some(rest) = body.strip_prefix("async fn ") {
                format!("pub async fn {rest}")
            } else if body.contains("\nfn ") {
                body.replacen("\nfn ", "\npub fn ", 1)
            } else {
                format!("pub {body}")
            }
        } else {
            body
        };
        out.push_str(&with_vis);
        if !with_vis.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

pub fn emit_rust_with_entry(module: &Module, entry: &str) -> Result<String, String> {
    emit_rust_with_entry_and_sigs(module, entry, &[])
}

/// Like [`emit_rust_with_entry`], plus the signatures of fns defined outside `module`
/// (ADR-286 S1b: the bin of a package whose lib fns were stripped before emit).
pub fn emit_rust_with_entry_and_sigs(
    module: &Module,
    entry: &str,
    externs: &[ExternFnSig],
) -> Result<String, String> {
    let _sigs = enter_fn_sigs(externs, std::iter::once(module));
    clear_emit_diag();
    let entry_fn = module
        .functions
        .iter()
        .find(|f| f.name == entry)
        .ok_or_else(|| format!("E0230: contract target fn `{entry}` not found"))?;
    if !matches!(entry_fn.ret_ty, Type::IoUnit) || !entry_fn.params.is_empty() {
        return Err(format!(
            "E0230: contract target fn `{entry}` must be fn() -> Io<()>"
        ));
    }

    // ADR-283: the parser's deferred-shape marker is a codegen ERROR, never Rust text / panic.
    module_deferred_shape_error(module)?;
    let mut out = format!("// Generated by ARITA from module `{}`\n", module.name);
    // ADR-022: safe-only — generated user programs forbid unsafe Rust.
    out.push_str("#![forbid(unsafe_code)]\n");
    if module_uses_map(module) {
        out.push_str("use std::collections::HashMap;\n");
    }
    if module_uses_truncate(module) {
        out.push_str(ARITA_TRUNCATE_HELPER);
    }
    if module_uses_split_off(module) {
        out.push_str(ARITA_SPLIT_OFF_HELPER);
    }
    if module_uses_vec_insert(module) {
        out.push_str(ARITA_VEC_INSERT_HELPER);
    }
    if module_uses_vec_set(module) {
        out.push_str(ARITA_VEC_SET_HELPER);
    }
    // ADR-290: `v[i] op= x` on a Vec<Int> local.
    if module_uses_vec_update(module) {
        out.push_str(ARITA_VEC_UPDATE_HELPER);
    }
    if module_uses_div_round(module) {
        out.push_str(ARITA_DIV_ROUND_HELPER);
    }
    if module_uses_saturating_rem(module) {
        out.push_str(ARITA_SATURATING_REM_HELPER);
    }
    if module_uses_is_power_of_two(module) {
        out.push_str(ARITA_IS_POWER_OF_TWO_HELPER);
    }
    if module_uses_next_multiple(module) {
        out.push_str(ARITA_NEXT_MULTIPLE_HELPER);
    }
    if module_uses_rotate(module) {
        out.push_str(ARITA_ROTATE_HELPER);
    }
    if entry != "main" {
        out.push_str(&format!(
            "// CONTRACT-TARGET-FN: entry rewrite → Rust main calls `{entry}()`\n"
        ));
    }

    // ADR-233: emit user records as Rust structs (forbid(unsafe) already set).
    for rec in &module.records {
        let mut fields_rs = String::new();
        let mut all_copy = true;
        for f in &rec.fields {
            let ty = emit_type(&f.ty).ok_or_else(|| "E0006: record field type".to_string())?;
            if matches!(f.ty, Type::String) {
                all_copy = false;
            }
            fields_rs.push_str(&format!(
                "    pub {}: {},
",
                f.name, ty
            ));
        }
        let derives = if all_copy {
            "#[derive(Copy, Clone, Debug)]
"
        } else {
            "#[derive(Clone, Debug)]
"
        };
        out.push_str(derives);
        out.push_str(&format!(
            "struct {} {{
{}}}

",
            rec.name, fields_rs
        ));
    }
    // ADR-234: emit user enums as Rust enums.
    for en in &module.enums {
        let mut vars_rs = String::new();
        for (i, v) in en.variants.iter().enumerate() {
            if i > 0 {
                vars_rs.push_str(
                    ",
",
                );
            }
            vars_rs.push_str(&format!("    {}", v));
        }
        out.push_str(
            "#[derive(Copy, Clone, Debug, PartialEq, Eq)]
",
        );
        out.push_str(&format!(
            "enum {} {{
{}
}}

",
            en.name, vars_rs
        ));
    }

    // Helpers first (all non-main), then user main (possibly renamed), then thin wrapper.
    let mut rest: Vec<&Function> = Vec::new();
    let mut main_fn: Option<&Function> = None;
    for func in &module.functions {
        if func.name == "main" {
            main_fn = Some(func);
        } else {
            rest.push(func);
        }
    }
    // ADR-286 S1b (§0.1d P2): an unemittable fn fails the module emit with E0006;
    // it is never omitted from the Rust output.
    for func in rest {
        let chunk = emit_function(func)?;
        out.push_str(&chunk);
        out.push('\n');
    }
    if let Some(func) = main_fn {
        if entry == "main" {
            let chunk = emit_function(func)?;
            out.push_str(&chunk);
        } else {
            let chunk = emit_function_as(func, "__arita_user_main")?;
            out.push_str(&chunk);
            out.push('\n');
            out.push_str("fn main() {\n");
            out.push_str(&format!("    {entry}();\n"));
            out.push_str("}\n");
        }
    } else if entry != "main" {
        // No user main in module (should not happen post-parse); still emit wrapper.
        out.push_str("fn main() {\n");
        out.push_str(&format!("    {entry}();\n"));
        out.push_str("}\n");
    }

    if let Some(tests) = emit_tests_mod(&module.tests)? {
        out.push('\n');
        out.push_str(&tests);
    }
    Ok(out)
}

fn emit_function(func: &Function) -> Result<String, String> {
    emit_function_as(func, &func.name)
}

/// Emit a function under `emit_name` (ADR-020 may rename user `main` → `__arita_user_main`).
fn emit_function_as(func: &Function, emit_name: &str) -> Result<String, String> {
    let mut params = Vec::new();
    for p in &func.params {
        let ty = emit_type(&p.ty).ok_or_else(|| {
            format!(
                "E0006: cannot emit type of param `{}` in fn `{}`",
                p.name, func.name
            )
        })?;
        params.push(format!("{}: {}", p.name, ty));
    }
    let params = params.join(", ");

    // Only the true Rust entry uses the bare `fn main()` form (Io<()>).
    // ADR-027: async main → #[tokio::main(flavor = "current_thread")] async fn main
    let is_rust_main = emit_name == "main" && matches!(func.ret_ty, Type::IoUnit);
    let async_kw = if func.is_async { "async " } else { "" };
    let sig = if is_rust_main {
        format!("{async_kw}fn main({params})")
    } else if matches!(func.ret_ty, Type::IoUnit) {
        // Io<()> helpers / demoted main: unit fn without `-> ()` noise.
        format!("{async_kw}fn {emit_name}({params})")
    } else {
        let ret = emit_type(&func.ret_ty)
            .ok_or_else(|| format!("E0006: cannot emit return type of fn `{}`", func.name))?;
        format!("{async_kw}fn {emit_name}({params}) -> {ret}")
    };
    let tokio_attr = if is_rust_main && func.is_async {
        "#[tokio::main(flavor = \"current_thread\")]\n"
    } else {
        ""
    };

    let unit_ret = matches!(func.ret_ty, Type::IoUnit);
    let mut body = String::new();
    let n = func.body.len();
    EMIT_LOCAL_TYPES.with(|e| {
        e.borrow_mut().clear();
        for p in &func.params {
            e.borrow_mut().insert(p.name.clone(), p.ty.clone());
        }
    });
    let ctx = format!("fn `{}`", func.name);
    for (i, stmt) in func.body.iter().enumerate() {
        clear_emit_diag();
        if let Stmt::Let { name, ty, .. } = stmt {
            EMIT_LOCAL_TYPES.with(|e| {
                e.borrow_mut().insert(name.clone(), ty.clone());
            });
        }
        let last = i + 1 == n;
        if last && !unit_ret {
            if let Stmt::Expr(expr) = stmt {
                if let Some(v) = emit_expr_ctx(expr, ExprCtx::Bare) {
                    body.push_str(&format!("    {v}\n"));
                    continue;
                }
            }
            // ADR-245: trailing Match stmt in valued fn (e.g. HTTP handler) → expression match
            if let Stmt::Match { scrutinee, arms } = stmt {
                if let Some(v) = emit_match_stmt_as_expr(scrutinee, arms) {
                    body.push_str(&format!("    {v}\n"));
                    continue;
                }
            }
        }
        match emit_stmt(stmt) {
            Some(line) => body.push_str(&line),
            // ADR-282: never silently drop `m[k] = v` (also nested in if/while/match bodies;
            // non-Map / unemittable) — fail the fn so the build breaks loudly.
            None if stmt_contains_index_assign(stmt) => {
                EMIT_LOCAL_TYPES.with(|e| e.borrow_mut().clear());
                let detail = take_emit_diag()
                    .map(|d| format!(" [{d}]"))
                    .unwrap_or_default();
                return Err(format!(
                    "E0006: cannot emit index assignment `x[..] = ..` in {ctx}: target is neither a local Map nor a Vec (the statement would be dropped){detail}"
                ));
            }
            // ADR-286 S1b (§0.1d P3): never drop a statement silently.
            None => {
                EMIT_LOCAL_TYPES.with(|e| e.borrow_mut().clear());
                return Err(unemittable_stmt_error(stmt, &ctx));
            }
        }
    }
    EMIT_LOCAL_TYPES.with(|e| e.borrow_mut().clear());
    Ok(format!("{tokio_attr}{sig} {{\n{body}}}\n"))
}

/// Emit `Stmt::Match` arm blocks as a Rust expression `match` (last stmt of valued fn).
fn emit_match_stmt_as_expr(scrutinee: &Expr, arms: &[arita_syntax::MatchArm]) -> Option<String> {
    let s = emit_expr_ctx(scrutinee, ExprCtx::Head)?;
    let mut arms_rs = String::new();
    for arm in arms {
        let p = emit_pat(&arm.pat)?;
        let n = arm.body.len();
        if n == 0 {
            return None;
        }
        // Single trailing Expr → `pat => expr,`
        if n == 1 {
            if let Stmt::Expr(e) = &arm.body[0] {
                let v = emit_expr_ctx(e, ExprCtx::Bare)?;
                arms_rs.push_str(&format!("        {p} => {v},\n"));
                continue;
            }
            if let Stmt::Match {
                scrutinee: nested_s,
                arms: nested_a,
            } = &arm.body[0]
            {
                let v = emit_match_stmt_as_expr(nested_s, nested_a)?;
                arms_rs.push_str(&format!("        {p} => {v},\n"));
                continue;
            }
        }
        // Multi-stmt: block; last Expr/Match is return value without `;`
        let mut block = String::new();
        for (i, st) in arm.body.iter().enumerate() {
            let is_last = i + 1 == n;
            if is_last {
                if let Stmt::Expr(e) = st {
                    let e_rs = emit_expr_ctx(e, ExprCtx::Bare)?;
                    block.push_str(&format!("            {e_rs}\n"));
                    continue;
                }
                if let Stmt::Match {
                    scrutinee: nested_s,
                    arms: nested_a,
                } = st
                {
                    let e_rs = emit_match_stmt_as_expr(nested_s, nested_a)?;
                    block.push_str(&format!("            {e_rs}\n"));
                    continue;
                }
            }
            let line = emit_stmt(st)?;
            for l in line.lines() {
                if l.is_empty() {
                    block.push('\n');
                } else {
                    block.push_str("        ");
                    block.push_str(l);
                    block.push('\n');
                }
            }
        }
        arms_rs.push_str(&format!("        {p} => {{\n{block}        }},\n"));
    }
    Some(format!("match {s} {{\n{arms_rs}    }}"))
}

fn emit_stmt(stmt: &Stmt) -> Option<String> {
    match stmt {
        Stmt::Let {
            mutable,
            name,
            ty,
            init,
        } => {
            let kw = if *mutable { "let mut" } else { "let" };
            // ADR-244: cancel_token is opaque AtomicBool (surface types as Int)
            if let Expr::Call(c) = init {
                if c.callee == "cancel_token" {
                    return Some(format!(
                        "    {kw} {name} = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));\n"
                    ));
                }
            }
            let ty_rs = emit_type(ty)?;
            // ADR-184: None turbofish uses Option inner (nested Option ok)
            let init_rs = match init {
                Expr::None => {
                    if let Type::Option(inner) = ty {
                        format!("None::<{}>", emit_type(inner)?)
                    } else {
                        "None".into()
                    }
                }
                // B-282-2 (ADR-283 FASE 2): `let n: Int = v.len()` — the len emit is
                // `(recv.len() as i64)`; drop the outer pair here (rustc `unused_parens`).
                other @ Expr::MethodCall { method, .. } if method == "len" => {
                    strip_one_outer_parens(&emit_expr(other)?).to_string()
                }
                other => emit_expr_ctx(other, ExprCtx::Bare)?,
            };
            Some(format!("    {kw} {name}: {ty_rs} = {init_rs};\n"))
        }
        Stmt::Expr(expr) => emit_expr_as_stmt(expr),
        Stmt::Assert { lhs, rhs } => {
            let l = emit_expr(lhs)?;
            let r = emit_expr(rhs)?;
            Some(format!("    assert_eq!({l}, {r});\n"))
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let c = emit_expr_ctx(cond, ExprCtx::Head)?;
            let then_rs = emit_block_body(then_body)?;
            if let Some(eb) = else_body {
                let else_rs = emit_block_body(eb)?;
                Some(format!(
                    "    if {c} {{\n{then_rs}    }} else {{\n{else_rs}    }}\n"
                ))
            } else {
                Some(format!("    if {c} {{\n{then_rs}    }}\n"))
            }
        }
        Stmt::WhileLet {
            pat,
            scrutinee,
            body,
        } => {
            let s = emit_expr_ctx(scrutinee, ExprCtx::Head)?;
            let p = emit_pat(pat)?;
            let body_rs = emit_block_body(body)?;
            Some(format!(
                "    while let {p} = {s} {{
{body_rs}    }}
"
            ))
        }
        Stmt::While { cond, body } => {
            let c = emit_expr_ctx(cond, ExprCtx::Head)?;
            let body_rs = emit_block_body(body)?;
            Some(format!(
                "    while {c} {{
{body_rs}    }}
"
            ))
        }
        Stmt::Assign { name, value } => {
            let v = emit_expr_ctx(value, ExprCtx::Bare)?;
            Some(format!("    {name} = {v};\n"))
        }
        // ADR-282 (CORE-0.9-MAP-ASSIGN-20260926) slice 1: `m[k] = v` on a local Map.
        // R2: k → v into block-scoped temps BEFORE the `&mut m` of insert (self-ref `m[k] = m.len()`).
        // R3: bare `insert(..);` statement (old value discarded; no `let _ =`). Emit-ban 241: no
        // Rust `m[k] =` / IndexMut / entry() / unwrap.
        // ADR-283 (CORE-0.9-VEC-ASSIGN-20260926): Vec target → same emit as `v.set(i, x)?`
        // (helper `__arita_vec_set` + `?`), with i → x into temps BEFORE `&mut v` (R2).
        // Any other target → None (the fn then fails to emit; never a silent drop).
        // ADR-290 (CORE-0.10-INDEX-MUT-20261002) slice A: compound `v[i] op= x` (op in +,-,*) on a
        // `Vec<Int>` local → `__arita_vec_update(&mut v, i, x, i64::checked_<op>)?` with i → x into
        // temps BEFORE the `&mut v` (order i → rhs → read → op → write; R2). Any other compound
        // (non-Vec target, element ≠ Int, op ∉ {+,-,*}) → None: never a silent drop, never a
        // `v[i] = v[i] op x` / `+=` in Rust.
        Stmt::IndexAssign {
            target,
            key,
            value,
            op, // ADR-290: `Option<BinOp>`; `None` = plain `=`
            ..
        } => {
            let target_ty = EMIT_LOCAL_TYPES.with(|e| e.borrow().get(target).cloned());
            if let Some(Type::Vec(elem)) = &target_ty {
                if let Some(op) = op {
                    if **elem != Type::Int {
                        return None;
                    }
                    let checked = vec_update_checked_fn(op)?;
                    let i = emit_expr(key)?;
                    let x = emit_expr(value)?;
                    let i = strip_one_outer_parens(&i);
                    let x = strip_one_outer_parens(&x);
                    return Some(format!(
                        "    {{ let __arita_vi = {i}; let __arita_vx = {x}; __arita_vec_update(&mut {target}, __arita_vi, __arita_vx, {checked})?; }}\n"
                    ));
                }
                let i = emit_expr(key)?;
                let x = emit_expr(value)?;
                let i = strip_one_outer_parens(&i);
                let x = strip_one_outer_parens(&x);
                return Some(format!(
                    "    {{ let __arita_vi = {i}; let __arita_vx = {x}; __arita_vec_set(&mut {target}, __arita_vi, __arita_vx)?; }}\n"
                ));
            }
            // Map (ADR-282) is plain-only; a compound Map target is a HIR error (E0006) — if one
            // ever reaches codegen it must fail the fn, not degrade to `insert`.
            if op.is_some() || !matches!(target_ty, Some(Type::Map(_, _))) {
                return None;
            }
            let k = emit_expr(key)?;
            let v = emit_expr(value)?;
            let k = strip_one_outer_parens(&k);
            let v = strip_one_outer_parens(&v);
            Some(format!(
                "    {{ let __arita_mk = {k}; let __arita_mv = {v}; {target}.insert(__arita_mk, __arita_mv); }}\n"
            ))
        }
        Stmt::Match { scrutinee, arms } => {
            let s = emit_expr_ctx(scrutinee, ExprCtx::Head)?;
            let mut arms_rs = String::new();
            for arm in arms {
                let p = emit_pat(&arm.pat)?;
                let body_rs = emit_block_body(&arm.body)?;
                // One more indent level inside `pat => { … }` relative to if/while blocks.
                let mut deeper = String::new();
                for line in body_rs.lines() {
                    if line.is_empty() {
                        deeper.push('\n');
                    } else {
                        deeper.push_str("    ");
                        deeper.push_str(line);
                        deeper.push('\n');
                    }
                }
                arms_rs.push_str(&format!("        {p} => {{\n{deeper}        }}\n"));
            }
            Some(format!("    match {s} {{\n{arms_rs}    }}\n"))
        }
        Stmt::IfLet {
            pat,
            scrutinee,
            then_body,
            else_body,
        } => {
            let s = emit_expr_ctx(scrutinee, ExprCtx::Head)?;
            let p = emit_pat(pat)?;
            let then_rs = emit_block_body(then_body)?;
            let eb = else_body.as_ref()?;
            let else_rs = emit_block_body(eb)?;
            Some(format!(
                "    if let {p} = {s} {{
{then_rs}    }} else {{
{else_rs}    }}
"
            ))
        }
        Stmt::Break => Some("    break;\n".into()),
        Stmt::Continue => Some("    continue;\n".into()),
    }
}

fn emit_pat(pat: &Pat) -> Option<String> {
    Some(match pat {
        Pat::LitBool(b) => b.to_string(),
        Pat::LitInt(n) => n.to_string(),
        Pat::Wildcard => "_".into(),
        Pat::Ok(name) => format!("Ok({name})"),
        Pat::Err(name) => format!("Err({name})"),
        Pat::Some(name) => format!("Some({name})"),
        Pat::None => "None".into(),
        Pat::Variant { enum_name, variant } => {
            if let Some(en) = enum_name {
                format!("{en}::{variant}")
            } else {
                // Bare variant: emit as-is; prefer Enum::V in source (rustc E0170).
                variant.clone()
            }
        }
    })
}

/// ADR-290: the `i64::checked_*` path passed as the `op` fn pointer of `__arita_vec_update`.
/// `None` for any operator outside slice A's {`+`, `-`, `*`} (the Parser rejects them with E0006;
/// codegen refuses defensively).
fn vec_update_checked_fn(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Add => Some("i64::checked_add"),
        BinOp::Sub => Some("i64::checked_sub"),
        BinOp::Mul => Some("i64::checked_mul"),
        _ => None,
    }
}

/// ADR-282: drop ONE balanced outer `( … )` pair (e.g. `(m.len() as i64)`) so the
/// `let __arita_mv = …;` slot does not trip rustc `unused_parens`. Returns the input unchanged
/// when the first `(` does not close at the very end (`(a) + (b)`), or when not parenthesized.
fn strip_one_outer_parens(s: &str) -> &str {
    let t = s.trim();
    let bytes = t.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'(' || bytes[bytes.len() - 1] != b')' {
        return s;
    }
    let mut depth: usize = 0;
    let mut in_str = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'(' => depth += 1,
            b')' => {
                depth = match depth.checked_sub(1) {
                    Some(d) => d,
                    None => return s,
                };
                if depth == 0 && i != bytes.len() - 1 {
                    return s;
                }
            }
            _ => {}
        }
    }
    if depth == 0 {
        t[1..t.len() - 1].trim()
    } else {
        s
    }
}

/// ADR-282: true when `stmt` is, or (recursively) contains in a nested body, an `IndexAssign`.
fn stmt_contains_index_assign(stmt: &Stmt) -> bool {
    fn any(stmts: &[Stmt]) -> bool {
        stmts.iter().any(stmt_contains_index_assign)
    }
    match stmt {
        Stmt::IndexAssign { .. } => true,
        Stmt::If {
            then_body,
            else_body,
            ..
        }
        | Stmt::IfLet {
            then_body,
            else_body,
            ..
        } => any(then_body) || else_body.as_deref().is_some_and(any),
        Stmt::While { body, .. } | Stmt::WhileLet { body, .. } => any(body),
        Stmt::Match { arms, .. } => arms.iter().any(|a| any(&a.body)),
        _ => false,
    }
}

fn emit_block_body(stmts: &[Stmt]) -> Option<String> {
    // ADR-282: block-scoped let types (restored on exit) so a Map declared inside this block
    // is known to the `m[k] = v` gate; outer bindings shadowed here are restored afterwards.
    let saved = EMIT_LOCAL_TYPES.with(|e| e.borrow().clone());
    let mut body = String::new();
    let mut failed = false;
    for stmt in stmts {
        if let Stmt::Let { name, ty, .. } = stmt {
            EMIT_LOCAL_TYPES.with(|e| {
                e.borrow_mut().insert(name.clone(), ty.clone());
            });
        }
        match emit_stmt(stmt) {
            Some(line) => body.push_str(&line),
            None => {
                failed = true;
                break;
            }
        }
    }
    EMIT_LOCAL_TYPES.with(|e| *e.borrow_mut() = saved);
    if failed {
        return None;
    }
    // Nested blocks already include 4-space indent from emit_stmt; add 4 more.
    let mut indented = String::new();
    for line in body.lines() {
        if line.is_empty() {
            indented.push('\n');
        } else {
            indented.push_str("    ");
            indented.push_str(line);
            indented.push('\n');
        }
    }
    Some(indented)
}

fn emit_type(ty: &Type) -> Option<String> {
    Some(match ty {
        Type::Int => "i64".into(),
        Type::Bool => "bool".into(),
        Type::String => "String".into(),
        Type::Bytes => "Vec<u8>".into(),
        Type::Task => "tokio::task::JoinHandle<()>".into(),
        Type::Map(k, v) => format!("HashMap<{}, {}>", emit_type(k)?, emit_type(v)?),
        Type::Unit => "()".into(),
        Type::IoUnit => "()".into(),
        Type::Vec(inner) => format!("Vec<{}>", emit_type(inner)?),
        Type::Result(t, e) => format!("Result<{}, {}>", emit_type(t)?, emit_type(e)?),
        Type::Option(t) => format!("Option<{}>", emit_type(t)?),
        // ADR-245 opaque HTTP types → host crate
        // Builtins must never fall through as Rust type names (pest order footgun).
        Type::Named(n) if n == "Int" => "i64".into(),
        Type::Named(n) if n == "Bool" => "bool".into(),
        Type::Named(n) if n == "Text" || n == "String" => "String".into(),
        Type::Named(n) if n == "Bytes" => "Vec<u8>".into(),
        Type::Named(n) if n == "Unit" => "()".into(),
        Type::Named(n) if n == "HttpServer" => "i64".into(),
        Type::Named(n) if n == "HttpRequest" => "arita_host_http::HttpRequest".into(),
        Type::Named(n) if n == "HttpResponse" => "arita_host_http::HttpResponse".into(),
        Type::Named(n) if n == "IoError" => "arita_host_http::IoError".into(),
        Type::Named(n) if n == "ServicePolicy" => "arita_host_http::ServicePolicy".into(),
        Type::Named(n) => n.clone(),
    })
}

fn emit_expr_as_stmt(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Call(call) => emit_call_stmt(call),
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let e = emit_method_call(receiver, method, args)?;
            Some(format!("    {e};\n"))
        }
        Expr::Await { inner } => {
            // ADR-243: await join(t) already emits `.await` inside join
            if let Expr::Call(call) = inner.as_ref() {
                if call.callee == "join" {
                    let e = emit_expr(inner)?;
                    return Some(format!("    {e};\n"));
                }
            }
            let e = emit_expr(inner)?;
            Some(format!("    {e}.await;\n"))
        }
        // ADR-278 bridge: Parser Expr::Try → Rust `?` (Codegen owns full oracles)
        Expr::Try { inner } => {
            let e = emit_expr(inner)?;
            Some(format!("    {e}?;\n"))
        }
        _ => None,
    }
}

fn emit_call_stmt(call: &Call) -> Option<String> {
    match call.callee.as_str() {
        "print" => {
            let arg = call.args.first()?;
            match arg {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("    println!(\"{escaped}\");\n"))
                }
                other => {
                    let e = emit_expr(other)?;
                    Some(format!("    println!(\"{{}}\", {e});\n"))
                }
            }
        }
        "Vec::new" | "List::new" | "Map::new" => None, // value position only
        // ADR-244 curated time/cancel as statements
        "delay" => {
            let ms = emit_expr(call.args.first()?)?;
            Some(format!(
                "    tokio::time::sleep(std::time::Duration::from_millis({ms} as u64)).await;\n"
            ))
        }
        "cancel" => {
            let tok = emit_expr(call.args.first()?)?;
            Some(format!(
                "    {tok}.store(true, std::sync::atomic::Ordering::SeqCst);\n"
            ))
        }
        "until_cancelled" => {
            let tok = emit_expr(call.args.first()?)?;
            Some(format!(
                "    {{ while !{tok}.load(std::sync::atomic::Ordering::SeqCst) {{ tokio::time::sleep(std::time::Duration::from_millis(10)).await; }} }}\n"
            ))
        }
        _ => {
            // Prefer curated emit_call_expr (timeout/http_*) over raw user call.
            let e = emit_call_expr(call).or_else(|| emit_user_call(call))?;
            Some(format!("    {e};\n"))
        }
    }
}

/// ADR-289: where an emitted expression lands. `Operand` (operand of Binary/Cast/receiver, or
/// anything not migrated) keeps the outer parens `emit_expr` produces; `Bare` (value of let/assign,
/// call/method argument, `Ok`/`Err`/`Some`, match arm, block/fn tail) and `Head` (condition or
/// scrutinee followed by a `{` block) drop ONE redundant outer pair (rustc `unused_parens`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExprCtx {
    Operand,
    Bare,
    Head,
}

/// `emit_expr` for a non-operand position. Only `Binary`, `Borrow` and `MethodCall` (the
/// parenthesized `as i64` casts) produce a redundant outer pair; every other expression is
/// returned as is. Children of a `Binary` always go through `emit_expr` (= `Operand`).
fn emit_expr_ctx(expr: &Expr, ctx: ExprCtx) -> Option<String> {
    let s = emit_expr(expr)?;
    if ctx == ExprCtx::Operand
        || !matches!(
            expr,
            Expr::Binary { .. } | Expr::Borrow { .. } | Expr::MethodCall { .. }
        )
    {
        return Some(s);
    }
    let t = strip_redundant_outer_parens(&s);
    if ctx == ExprCtx::Head && has_brace_outside_strings(t) {
        // exterior struct literal / block under `if`/`while`/`match`: parens are needed.
        return Some(s);
    }
    Some(t.to_string())
}

/// `strip_one_outer_parens` that never produces an empty string (`()`) nor a bare tuple body.
fn strip_redundant_outer_parens(s: &str) -> &str {
    let t = strip_one_outer_parens(s);
    if t.len() >= s.trim().len() || t.is_empty() {
        return s;
    }
    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut escaped = false;
    for b in t.bytes() {
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => return s,
            _ => {}
        }
    }
    t
}

fn has_brace_outside_strings(s: &str) -> bool {
    let mut in_str = false;
    let mut escaped = false;
    for b in s.bytes() {
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => return true,
            _ => {}
        }
    }
    false
}

fn emit_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::LitInt(n) => Some(n.to_string()),
        Expr::LitBool(b) => Some(b.to_string()),
        Expr::LitStr(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            Some(format!("\"{escaped}\".to_string()"))
        }
        Expr::Path(p) => emit_path(p),
        Expr::Binary { op, lhs, rhs } => {
            let l = emit_expr(lhs)?;
            let r = emit_expr(rhs)?;
            let o = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Rem => "%",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
            };
            Some(format!("({l} {o} {r})"))
        }
        Expr::Borrow { mutable, inner } => {
            // ADR-235: Copy-out through temporary loan so `let a: Int = borrow x` typechecks in rustc.
            let i = emit_expr(inner)?;
            if *mutable {
                Some(format!("(*&mut {i})"))
            } else {
                Some(format!("(*&{i})"))
            }
        }
        Expr::Call(call) => emit_call_expr(call),
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => emit_method_call(receiver, method, args),
        Expr::Match { scrutinee, arms } => {
            let s = emit_expr_ctx(scrutinee, ExprCtx::Head)?;
            let mut arms_rs = String::new();
            for arm in arms {
                let p = emit_pat(&arm.pat)?;
                let v = emit_expr_ctx(&arm.value, ExprCtx::Bare)?;
                arms_rs.push_str(&format!(
                    "        {p} => {v},
"
                ));
            }
            Some(format!(
                "match {s} {{
{arms_rs}    }}"
            ))
        }
        Expr::Await { inner } => {
            if let Expr::Call(call) = inner.as_ref() {
                if call.callee == "join" {
                    return emit_expr(inner);
                }
            }
            let e = emit_expr(inner)?;
            Some(format!("{e}.await"))
        }
        // ADR-278 bridge: `expr?` → Rust `expr?` (oracles/ejemplos still Codegen)
        Expr::Try { inner } => {
            let e = emit_expr(inner)?;
            Some(format!("{e}?"))
        }
        Expr::LitUnit => Some("()".into()),
        Expr::Ok(inner) => {
            let e = emit_expr_ctx(inner, ExprCtx::Bare)?;
            Some(format!("Ok({e})"))
        }
        Expr::Err(inner) => {
            let e = emit_expr_ctx(inner, ExprCtx::Bare)?;
            Some(format!("Err({e})"))
        }
        Expr::Some(inner) => {
            let e = emit_expr_ctx(inner, ExprCtx::Bare)?;
            Some(format!("Some({e})"))
        }
        Expr::None => Some("None::<i64>".into()), // turbofish: bare None as scrutinee (ADR-055)
        Expr::EnumPath { enum_name, variant } => Some(format!("{}::{}", enum_name, variant)),
        Expr::RecordLit { name, fields } => {
            let mut parts = Vec::new();
            for (fname, fexpr) in fields {
                let e = emit_expr(fexpr)?;
                parts.push(format!("{fname}: {e}"));
            }
            let ty = if name == "ServicePolicy" {
                "arita_host_http::ServicePolicy".to_string()
            } else {
                name.clone()
            };
            Some(format!("{ty} {{ {} }}", parts.join(", ")))
        }
        Expr::FieldAccess { base, field } => {
            let b = emit_expr(base)?;
            // Shared get: Copy fields fine; String fields clone to avoid partial move.
            // MVP: always clone String-typed fields via .clone() when base path — emit plain field;
            // Int/Bool Copy. Use plain `.field` (Rust Copy/move); String field moves out.
            Some(format!("{b}.{field}"))
        }
    }
}
fn emit_call_expr(call: &Call) -> Option<String> {
    match call.callee.as_str() {
        "print" => None, // print is statement-only in emit
        "Vec::new" | "List::new" => {
            if !call.args.is_empty() {
                return None;
            }
            Some("Vec::new()".into())
        }
        "Map::new" => {
            if !call.args.is_empty() {
                return None;
            }
            Some("HashMap::new()".into())
        }
        "spawn" => {
            // ADR-243: tokio::spawn(async_fn())
            if call.args.len() != 1 {
                return None;
            }
            let inner = emit_expr_ctx(&call.args[0], ExprCtx::Bare)?;
            Some(format!("tokio::spawn({inner})"))
        }
        "join" => {
            // only valid under await — emit handle.await
            if call.args.len() != 1 {
                set_emit_diag(format!(
                    "E0006: cannot emit `join` with {} arguments (expected exactly one Task)",
                    call.args.len()
                ));
                return None;
            }
            // ADR-286 S1b (§0.1d P6): `{ let _ = t.await; }` is only emitted over an Io<()> task.
            match &call.args[0] {
                // Task ident: `Type::Task` is emitted as `JoinHandle<()>`, rustc guards it.
                Expr::Path(p) if p.segments.len() == 1 => {}
                Expr::Call(sp) if sp.callee == "spawn" => {
                    let [Expr::Call(g)] = sp.args.as_slice() else {
                        set_emit_diag(
                            "E0006: cannot emit join over `spawn(..)` whose argument is not an async fn call (its result would be discarded)"
                                .to_string(),
                        );
                        return None;
                    };
                    match lookup_fn_sig(&g.callee) {
                        Some((true, Type::IoUnit)) => {}
                        Some((true, _)) => {
                            set_emit_diag(format!(
                                "E0006: cannot emit join over spawn of non-Io<()> async fn '{}' (its result would be discarded)",
                                g.callee
                            ));
                            return None;
                        }
                        Some((false, _)) => {
                            set_emit_diag(format!(
                                "E0006: cannot emit join over spawn of non-async fn '{}' (its result would be discarded)",
                                g.callee
                            ));
                            return None;
                        }
                        None => {
                            set_emit_diag(format!(
                                "E0006: cannot emit join over spawn of '{}': not an async fn of this module or its deps, so its Io<()> return cannot be verified (its result would be discarded)",
                                g.callee
                            ));
                            return None;
                        }
                    }
                }
                _ => {
                    set_emit_diag(
                        "E0006: cannot emit join over an argument that is neither a Task ident nor `spawn(g())` of an Io<()> async fn (its result would be discarded)"
                            .to_string(),
                    );
                    return None;
                }
            }
            let t = emit_expr(&call.args[0])?;
            Some(format!("{{ let _ = {t}.await; }}"))
        }
        // ADR-244: timeout(ms, async_call|delay|until_cancelled) -> Result<(), String>
        "timeout" => {
            if call.args.len() != 2 {
                return None;
            }
            let ms = emit_expr(&call.args[0])?;
            let fut = match &call.args[1] {
                Expr::Call(inner) if inner.callee == "delay" => {
                    let d = emit_expr(inner.args.first()?)?;
                    format!("tokio::time::sleep(std::time::Duration::from_millis({d} as u64))")
                }
                Expr::Call(inner) if inner.callee == "until_cancelled" => {
                    let tok = emit_expr(inner.args.first()?)?;
                    format!(
                        "async {{ while !{tok}.load(std::sync::atomic::Ordering::SeqCst) {{ tokio::time::sleep(std::time::Duration::from_millis(10)).await; }} }}"
                    )
                }
                Expr::Call(inner) => {
                    let c = emit_user_call(inner)?;
                    format!("async {{ {c}.await }}")
                }
                _ => return None,
            };
            Some(format!(
                "match tokio::time::timeout(std::time::Duration::from_millis({ms} as u64), {fut}).await {{ Ok(v) => Ok(v), Err(_) => Err(\"timeout\".to_string()) }}"
            ))
        }
        "cancel_token" => {
            if !call.args.is_empty() {
                return None;
            }
            Some("std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))".into())
        }
        "delay" => {
            let ms = emit_expr(call.args.first()?)?;
            Some(format!(
                "tokio::time::sleep(std::time::Duration::from_millis({ms} as u64)).await"
            ))
        }
        "until_cancelled" => {
            let tok = emit_expr(call.args.first()?)?;
            Some(format!(
                "{{ while !{tok}.load(std::sync::atomic::Ordering::SeqCst) {{ tokio::time::sleep(std::time::Duration::from_millis(10)).await; }} }}"
            ))
        }
        _ => emit_user_call(call),
    }
}

fn emit_user_call(call: &Call) -> Option<String> {
    if call.callee.is_empty() || call.callee.contains(' ') {
        return None;
    }
    let mut args = Vec::new();
    for a in &call.args {
        args.push(emit_expr_ctx(a, ExprCtx::Bare)?);
    }
    Some(format!("{}({})", call.callee, args.join(", ")))
}

fn emit_method_call(receiver: &Expr, method: &str, args: &[Expr]) -> Option<String> {
    // ADR-245 associated: HttpServer.bind / HttpResponse.ok_text|status
    if let Expr::Path(p) = receiver {
        let tname = p.segments.first().map(|s| s.as_str()).unwrap_or("");
        if tname == "HttpServer" && method == "bind" && args.len() == 2 {
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let port = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::bind({a}, {port}).await"));
        }
        if tname == "HttpResponse" && method == "ok_text" && args.len() == 1 {
            let b = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::HttpResponse::ok_text({b})"));
        }
        if tname == "HttpResponse" && method == "status" && args.len() == 2 {
            let c = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let b = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::HttpResponse::status({c}, {b})"));
        }
        if tname == "ServicePolicy" && method == "default" && args.is_empty() {
            return Some("arita_host_http::ServicePolicy::default_policy()".into());
        }
        // ADR-247
        if tname == "HttpClient" && method == "url" && args.len() == 2 {
            let p = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let path = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::client_url({p}, {path})"));
        }
        if tname == "HttpClient" && method == "get" && args.len() == 1 {
            let u = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::client_get({u}).await"));
        }
        if tname == "HttpClient" && method == "post_text" && args.len() == 2 {
            let u = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let b = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            return Some(format!("arita_host_http::client_post_text({u}, {b}).await"));
        }
    }

    // ADR-026 whitelist emit only; HIR already gates E0206/E0203.
    let recv = emit_expr(receiver)?;
    // ADR-245 instance methods
    match (method, args.len()) {
        ("serve", 1) => {
            let h = match &args[0] {
                Expr::Path(p) if p.segments.len() == 1 => p.segments[0].clone(),
                _ => return None,
            };
            return Some(format!(
                "arita_host_http::serve({recv}, |__req| {h}(__req)).await"
            ));
        }
        ("port", 0) => return Some(format!("arita_host_http::port({recv})")),
        ("shutdown", 0) => return Some(format!("arita_host_http::shutdown({recv})")),
        ("set_policy", 1) => {
            let p = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            return Some(format!(
                "arita_host_http::set_policy({recv}, {p}).map(|_| {recv})"
            ));
        }
        ("policy", 0) => {
            return Some(format!("arita_host_http::get_policy({recv})"));
        }
        ("method", 0) => return Some(format!("{recv}.method()")),
        ("path", 0) => return Some(format!("{recv}.path()")),
        ("header", 1) => {
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            return Some(format!("{recv}.header({a})"));
        }
        ("body_text", 0) => return Some(format!("{recv}.body_text()")),
        // ADR-247: instance resp.status() → field (associated status(code,body) is Path emit)
        ("status", 0) => return Some(format!("{recv}.status")),
        ("set_header", 2) => {
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let b = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            return Some(format!("{recv}.set_header({a}, {b})"));
        }
        _ => {}
    }
    match method {
        "push" => {
            let mut arg_s = Vec::new();
            for a in args {
                arg_s.push(emit_expr_ctx(a, ExprCtx::Bare)?);
            }
            Some(format!("{recv}.push({})", arg_s.join(", ")))
        }
        "push_str" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.push_str(\"{escaped}\")"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.push_str(&{a})"))
                }
            }
        }
        "resize" => {
            if args.len() != 2 {
                return None;
            }
            let n = emit_expr(&args[0])?;
            let fill = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            Some(format!("{recv}.resize(({n}).max(0) as usize, {fill})"))
        }
        "truncate" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            // ADR-118 Vec / ADR-160 String: trait helper — String floors to char boundary
            Some(format!("{recv}.__arita_truncate({a})"))
        }
        "clear" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.clear()"))
        }
        "try_reserve_exact" => {
            // ADR-206: try_reserve_exact; TryReserveError → Err(0)
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "{recv}.try_reserve_exact(({a}).max(0) as usize).map_err(|_e| 0i64)"
            ))
        }
        "try_reserve" => {
            // ADR-204: try_reserve; TryReserveError → Err(0)
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "{recv}.try_reserve(({a}).max(0) as usize).map_err(|_e| 0i64)"
            ))
        }
        "reserve" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.reserve(({a}).max(0) as usize)"))
        }
        "shrink_to" => {
            // ADR-208: shrink_to(min_cap as usize)
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.shrink_to(({a}).max(0) as usize)"))
        }
        "shrink_to_fit" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.shrink_to_fit()"))
        }

        "reverse" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.reverse()"))
        }
        "sort_unstable" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.sort_unstable()"))
        }
        "dedup" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.dedup()"))
        }
        "fill" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.fill({a})"))
        }
        "swap" => {
            if args.len() != 2 {
                return None;
            }
            let i = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let j = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            // ADR-158: bounds-checked; OOB or runtime neg → no-op (no panic)
            Some(format!(
                "{{ let __i = {i}; let __j = {j}; if __i < 0 || __j < 0 || (__i as usize) >= {recv}.len() || (__j as usize) >= {recv}.len() {{ }} else {{ {recv}.swap(__i as usize, __j as usize); }} }}"
            ))
        }
        "extend" => {
            // ADR-224: a.extend_from_slice(&b) — shared, b intact
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.extend_from_slice(&{a})"))
        }
        "append" => {
            // ADR-222: a.append(&mut b)
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.append(&mut {a})"))
        }
        "split_off" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            // ADR-162: String floors char boundary; Vec clamps to len (via helper)
            Some(format!("{recv}.__arita_split_off({a})"))
        }
        "make_ascii_lowercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.make_ascii_lowercase()"))
        }
        "make_ascii_uppercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.make_ascii_uppercase()"))
        }
        "pop" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.pop()"))
        }
        "get" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            let recv_is_map = match receiver {
                Expr::Path(p) if p.segments.len() == 1 => EMIT_LOCAL_TYPES
                    .with(|e| matches!(e.borrow().get(&p.segments[0]), Some(Type::Map(_, _)))),
                _ => false,
            };
            if recv_is_map {
                // B-282-1 (ADR-283 FASE 2): a string-literal key borrows the `&str` literal
                // directly (`HashMap<String, _>::get` takes `&str` via `Borrow<str>`) — no
                // `&"k".to_string()` temporary (clippy `unnecessary_to_owned`).
                if let (Expr::LitStr(_), Some(lit)) = (&args[0], a.strip_suffix(".to_string()")) {
                    return Some(format!("{recv}.get({lit}).cloned()"));
                }
                Some(format!("{recv}.get(&{a}).cloned()"))
            } else {
                let a_bare = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
                Some(format!(
                    "{{ let __i = {a_bare}; if __i < 0 {{ None }} else {{ {recv}.get(__i as usize).cloned() }} }}"
                ))
            }
        }
        "insert" => {
            // ADR-260 / CORE-0.5-INSERT: fallible helper — never panics
            if args.len() != 2 {
                return None;
            }
            let i = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let x = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            Some(format!("__arita_vec_insert(&mut {recv}, {i}, {x})"))
        }
        "set" => {
            // ADR-265 / CORE-0.6-SET-FALLIBLE-20260926: fallible helper — never panics / no IndexMut
            if args.len() != 2 {
                return None;
            }
            let i = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let x = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            Some(format!("__arita_vec_set(&mut {recv}, {i}, {x})"))
        }
        "put" => {
            if args.len() != 2 {
                return None;
            }
            let k = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let v = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            Some(format!("{{ let _ = {recv}.insert({k}, {v}); }}"))
        }
        "swap_remove" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!(
                "{{ let __i = {a}; if __i < 0 || (__i as usize) >= {recv}.len() {{ None }} else {{ Some({recv}.swap_remove(__i as usize)) }} }}"
            ))
        }
        "remove" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!(
                "{{ let __i = {a}; if __i < 0 || (__i as usize) >= {recv}.len() {{ None }} else {{ Some({recv}.remove(__i as usize)) }} }}"
            ))
        }
        "first" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.first().cloned()"))
        }
        "last" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.last().cloned()"))
        }
        "clone" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.clone()"))
        }
        "find" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.find(\"{escaped}\").map(|i| i as i64)"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.find(&{a}).map(|i| i as i64)"))
                }
            }
        }
        "rfind" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.rfind(\"{escaped}\").map(|i| i as i64)"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.rfind(&{a}).map(|i| i as i64)"))
                }
            }
        }
        "contains" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.contains(\"{escaped}\")"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.contains(&{a})"))
                }
            }
        }
        "starts_with" => {
            // ADR-196: Vec uses .starts_with(&prefix) via non-LitStr arm
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.starts_with(\"{escaped}\")"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.starts_with(&{a})"))
                }
            }
        }
        "ends_with" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.ends_with(\"{escaped}\")"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.ends_with(&{a})"))
                }
            }
        }
        "eq_ignore_ascii_case" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!("{recv}.eq_ignore_ascii_case(\"{escaped}\")"))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.eq_ignore_ascii_case(&{a})"))
                }
            }
        }
        "trim" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim().to_string()"))
        }
        "trim_start" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim_start().to_string()"))
        }
        "trim_end" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim_end().to_string()"))
        }
        "trim_ascii" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim_ascii().to_string()"))
        }
        "trim_ascii_start" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim_ascii_start().to_string()"))
        }
        "trim_ascii_end" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.trim_ascii_end().to_string()"))
        }
        "strip_prefix" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!(
                        "{recv}.strip_prefix(\"{escaped}\").map(str::to_string)"
                    ))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.strip_prefix(&{a}).map(str::to_string)"))
                }
            }
        }
        "strip_suffix" => {
            if args.len() != 1 {
                return None;
            }
            match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    Some(format!(
                        "{recv}.strip_suffix(\"{escaped}\").map(str::to_string)"
                    ))
                }
                other => {
                    let a = emit_expr(other)?;
                    Some(format!("{recv}.strip_suffix(&{a}).map(str::to_string)"))
                }
            }
        }
        "repeat" => {
            if args.len() != 1 {
                return None;
            }
            let n = emit_expr(&args[0])?;
            Some(format!("{recv}.repeat({n} as usize)"))
        }
        "to_uppercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_uppercase()"))
        }
        "to_lowercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_lowercase()"))
        }
        "to_ascii_lowercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_ascii_lowercase()"))
        }
        "to_ascii_uppercase" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_ascii_uppercase()"))
        }
        "parse_int" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!(
                "{recv}.parse::<i64>().map_err(|_| \"invalid integer\".to_string())"
            ))
        }
        "to_string" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_string()"))
        }

        "wrapping_neg" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.wrapping_neg()"))
        }
        "saturating_neg" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.saturating_neg()"))
        }
        "is_positive" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_positive()"))
        }
        "is_negative" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_negative()"))
        }
        "signum" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.signum()"))
        }
        "abs" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.abs()"))
        }
        "pow" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "{recv}.checked_pow(({a}) as u32).unwrap_or(i64::MAX)"
            ))
        }
        "checked_neg" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.checked_neg()"))
        }
        "checked_add" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_add({a})"))
        }
        "checked_sub" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_sub({a})"))
        }
        "checked_mul" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_mul({a})"))
        }
        "checked_div" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_div({a})"))
        }
        "checked_rem" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_rem({a})"))
        }
        "checked_shl" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.checked_shl(({a}) as u32)"))
        }
        "checked_shr" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.checked_shr(({a}) as u32)"))
        }
        "wrapping_shl" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.wrapping_shl(({a}) as u32)"))
        }
        "wrapping_shr" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!("{recv}.wrapping_shr(({a}) as u32)"))
        }
        "saturating_add" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.saturating_add({a})"))
        }
        "saturating_sub" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.saturating_sub({a})"))
        }
        "saturating_mul" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.saturating_mul({a})"))
        }

        "wrapping_add" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.wrapping_add({a})"))
        }

        "wrapping_sub" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.wrapping_sub({a})"))
        }

        "checked_div_euclid" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_div_euclid({a})"))
        }
        "checked_rem_euclid" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.checked_rem_euclid({a})"))
        }
        "wrapping_div" => {
            // ADR-212: emit .wrapping_div(b); MIN/-1 → MIN
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.wrapping_div({a})"))
        }
        "wrapping_rem" => {
            // ADR-212: emit .wrapping_rem(b); MIN/-1 → 0
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.wrapping_rem({a})"))
        }
        "saturating_rem" => {
            // ADR-214: stable helper (no i64::saturating_rem); MIN/-1 → 0
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_saturating_rem({a})"))
        }
        "saturating_div" => {
            // ADR-210: emit .saturating_div(b); MIN/-1 → MAX
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.saturating_div({a})"))
        }
        "div_ceil" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_div_ceil({a})"))
        }
        "div_floor" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_div_floor({a})"))
        }
        "div_euclid" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.div_euclid({a})"))
        }
        "rem_euclid" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.rem_euclid({a})"))
        }
        "wrapping_mul" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.wrapping_mul({a})"))
        }
        "is_some" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_some()"))
        }
        "is_none" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_none()"))
        }
        "transpose" => {
            // ADR-200
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.transpose()"))
        }
        "flatten" => {
            // ADR-184 Option / ADR-198 Result
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.flatten()"))
        }
        "then_some" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.then_some({a})"))
        }
        "ok_or" => {
            // ADR-202
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.ok_or({a})"))
        }
        "ok" => {
            // ADR-202 Result.ok()
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.ok()"))
        }
        "err" => {
            // ADR-202 Result.err()
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.err()"))
        }
        "is_ok" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_ok()"))
        }
        "is_err" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_err()"))
        }
        "unwrap_or" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.unwrap_or({a})"))
        }
        "count_ones" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.count_ones() as i64)"))
        }
        "count_zeros" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.count_zeros() as i64)"))
        }
        "leading_zeros" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.leading_zeros() as i64)"))
        }
        "trailing_zeros" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.trailing_zeros() as i64)"))
        }
        "leading_ones" => {
            // ADR-216
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.leading_ones() as i64)"))
        }
        "trailing_ones" => {
            // ADR-216
            if !args.is_empty() {
                return None;
            }
            Some(format!("({recv}.trailing_ones() as i64)"))
        }
        "reverse_bits" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.reverse_bits()"))
        }
        "rotate_left" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            // ADR-170 / ADR-131: trait helper resolves Int vs Vec
            Some(format!("{recv}.__arita_rotate_left({a})"))
        }
        "rotate_right" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            // ADR-170 / ADR-131: trait helper resolves Int vs Vec
            Some(format!("{recv}.__arita_rotate_right({a})"))
        }
        "swap_bytes" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.swap_bytes()"))
        }
        "to_be" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_be()"))
        }
        "to_le" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.to_le()"))
        }
        "from_be" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("i64::from_be({recv})"))
        }
        "from_le" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("i64::from_le({recv})"))
        }
        "ilog10" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.checked_ilog10().map(|x| x as i64)"))
        }
        "ilog2" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.checked_ilog2().map(|x| x as i64)"))
        }
        "previous_multiple_of" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_previous_multiple_of({a})"))
        }
        "checked_previous_multiple_of" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_checked_previous_multiple_of({a})"))
        }
        "next_multiple_of" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_next_multiple_of({a})"))
        }
        "checked_next_multiple_of" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.__arita_checked_next_multiple_of({a})"))
        }
        "checked_next_power_of_two" => {
            // ADR-220: stable helper; n≤0/overflow → None
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.__arita_checked_next_power_of_two()"))
        }
        "is_power_of_two" => {
            // ADR-218: stable helper (no i64::is_power_of_two)
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.__arita_is_power_of_two()"))
        }
        "is_multiple_of" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!(
                "{{ let __d = {a}; if __d == 0 {{ false }} else {{ {recv} % __d == 0 }} }}"
            ))
        }
        "abs_diff" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("({recv}.abs_diff({a}) as i64)"))
        }
        "midpoint" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.midpoint({a})"))
        }
        "min" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.min({a})"))
        }
        "max" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.max({a})"))
        }
        "clamp" => {
            if args.len() != 2 {
                return None;
            }
            let lo = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            let hi = emit_expr_ctx(&args[1], ExprCtx::Bare)?;
            Some(format!("{recv}.clamp({lo}, {hi})"))
        }

        "or" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.or({a})"))
        }
        "and" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.and({a})"))
        }
        "xor" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!("{recv}.xor({a})"))
        }
        "take" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.take()"))
        }
        "replace" => {
            if args.len() == 1 {
                // ADR-168 Option.replace(x)
                let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
                return Some(format!("{recv}.replace({a})"));
            }
            if args.len() != 2 {
                return None;
            }

            let from = match &args[0] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    format!("\"{escaped}\"")
                }
                other => {
                    let a = emit_expr(other)?;
                    format!("&{a}")
                }
            };
            let to = match &args[1] {
                Expr::LitStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    format!("\"{escaped}\"")
                }
                other => {
                    let a = emit_expr(other)?;
                    format!("&{a}")
                }
            };
            Some(format!("{recv}.replace({from}, {to})"))
        }
        "as_bytes" => {
            // ADR-236: String → Vec<u8>
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.as_bytes().to_vec()"))
        }
        "len" => {
            if !args.is_empty() {
                return None;
            }
            // Int = i64; Rust len is usize
            Some(format!("({recv}.len() as i64)"))
        }
        "capacity" => {
            if !args.is_empty() {
                return None;
            }
            // Int = i64; Rust capacity is usize
            Some(format!("({recv}.capacity() as i64)"))
        }
        "is_char_boundary" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr_ctx(&args[0], ExprCtx::Bare)?;
            Some(format!(
                "{{ let __i = {a}; if __i < 0 {{ false }} else {{ {recv}.is_char_boundary(__i as usize) }} }}"
            ))
        }
        "floor_char_boundary" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "({recv}.floor_char_boundary(({a}).max(0) as usize) as i64)"
            ))
        }
        "ceil_char_boundary" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "({recv}.ceil_char_boundary(({a}).max(0) as usize) as i64)"
            ))
        }
        "binary_search" => {
            if args.len() != 1 {
                return None;
            }
            let a = emit_expr(&args[0])?;
            Some(format!(
                "{recv}.binary_search(&{a}).map(|i| i as i64).map_err(|i| i as i64)"
            ))
        }
        "is_sorted" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_sorted()"))
        }
        "is_empty" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_empty()"))
        }
        "is_ascii" => {
            if !args.is_empty() {
                return None;
            }
            Some(format!("{recv}.is_ascii()"))
        }
        _ => None,
    }
}

/// `Ok(None)` when there are no tests; `Err(E0006)` when a test cannot be emitted
/// (ADR-286 S1b: the tests module is never dropped silently).
fn emit_tests_mod(tests: &[arita_syntax::Test]) -> Result<Option<String>, String> {
    if tests.is_empty() {
        return Ok(None);
    }
    let mut body = String::new();
    for t in tests {
        body.push_str(&emit_one_test(t)?);
    }
    Ok(Some(format!(
        "#[cfg(test)]\nmod tests {{\n    use super::*;\n{body}}}\n"
    )))
}

fn emit_one_test(test: &arita_syntax::Test) -> Result<String, String> {
    let name = rust_test_ident(&test.name);
    let ctx = format!("test `{}`", test.name);
    let mut body = String::new();
    for stmt in &test.body {
        clear_emit_diag();
        // ADR-286 S1b (§0.1d P4): an unemittable stmt (e.g. an assert side) fails the emit;
        // the test never passes empty.
        let Some(line) = emit_stmt(stmt) else {
            return Err(unemittable_stmt_error(stmt, &ctx));
        };
        // indent one more level inside #[test] fn
        for l in line.lines() {
            if l.is_empty() {
                body.push('\n');
            } else {
                body.push_str("    ");
                body.push_str(l);
                body.push('\n');
            }
        }
    }
    Ok(format!("    #[test]\n    fn {name}() {{\n{body}    }}\n"))
}

fn rust_test_ident(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if s.is_empty() || s.chars().next().unwrap().is_ascii_digit() {
        s.insert_str(0, "t_");
    }
    s
}

fn emit_path(path: &Path) -> Option<String> {
    if path.segments.is_empty() {
        return None;
    }
    Some(path.segments.join("::"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arita_syntax::{FnParam, Function, Type};

    fn main_mod(body: Vec<Stmt>) -> Module {
        Module {
            name: "hello".into(),
            functions: vec![Function {
                name: "main".into(),
                params: vec![],
                ret_ty: Type::IoUnit,
                is_async: false,
                is_pub: false,
                body,
            }],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        }
    }

    #[test]
    fn emits_forbid_unsafe_code_attr() {
        let m = main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitStr("hello".into())],
        }))]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("#![forbid(unsafe_code)]"),
            "ADR-022: emit must forbid unsafe; got:\n{rs}"
        );
    }

    #[test]
    fn emits_println_f1() {
        let m = main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitStr("hello".into())],
        }))]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("println!(\"hello\")"));
        assert!(!rs.contains("println!(\"{}\""));
    }

    #[test]
    fn emits_let_litint_print_path() {
        let m = main_mod(vec![
            Stmt::Let {
                mutable: false,
                name: "n".into(),
                ty: Type::Int,
                init: Expr::LitInt(42),
            },
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::Path(Path {
                    segments: vec!["n".into()],
                })],
            })),
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("let n: i64 = 42;"), "got:\n{rs}");
        assert!(rs.contains("println!(\"{}\", n);"), "got:\n{rs}");
    }

    #[test]
    fn emits_print_litint() {
        let m = main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitInt(7)],
        }))]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("println!(\"{}\", 7);"), "got:\n{rs}");
    }

    #[test]
    fn emits_vec_new_push_len() {
        let v = Expr::Path(Path {
            segments: vec!["v".into()],
        });
        let m = main_mod(vec![
            Stmt::Let {
                mutable: true,
                name: "v".into(),
                ty: Type::Vec(Box::new(Type::Int)),
                init: Expr::Call(Call {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(v.clone()),
                method: "push".into(),
                args: vec![Expr::LitInt(1)],
            }),
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::MethodCall {
                    receiver: Box::new(v),
                    method: "len".into(),
                    args: vec![],
                }],
            })),
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("let mut v: Vec<i64> = Vec::new();"),
            "got:\n{rs}"
        );
        assert!(rs.contains("v.push(1);"), "got:\n{rs}");
        assert!(
            rs.contains("println!(\"{}\", (v.len() as i64));"),
            "got:\n{rs}"
        );
    }

    #[test]
    fn emits_let_string_print_path() {
        let m = main_mod(vec![
            Stmt::Let {
                mutable: false,
                name: "s".into(),
                ty: Type::String,
                init: Expr::LitStr("hola".into()),
            },
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::Path(Path {
                    segments: vec!["s".into()],
                })],
            })),
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("let s: String = \"hola\".to_string();"),
            "got:\n{rs}"
        );
        assert!(rs.contains("println!(\"{}\", s);"), "got:\n{rs}");
    }

    #[test]
    fn emits_litbool_and_binary_int() {
        let m = main_mod(vec![
            Stmt::Let {
                mutable: false,
                name: "ok".into(),
                ty: Type::Bool,
                init: Expr::LitBool(true),
            },
            Stmt::Let {
                mutable: false,
                name: "sum".into(),
                ty: Type::Int,
                init: Expr::Binary {
                    op: BinOp::Add,
                    lhs: Box::new(Expr::LitInt(1)),
                    rhs: Box::new(Expr::LitInt(2)),
                },
            },
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::Path(Path {
                    segments: vec!["ok".into()],
                })],
            })),
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::Path(Path {
                    segments: vec!["sum".into()],
                })],
            })),
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("let ok: bool = true;"), "got:\n{rs}");
        assert!(rs.contains("let sum: i64 = 1 + 2;"), "got:\n{rs}");
        assert!(rs.contains("println!(\"{}\", ok);"), "got:\n{rs}");
        assert!(rs.contains("println!(\"{}\", sum);"), "got:\n{rs}");
    }

    #[test]
    fn emits_test_assert_eq() {
        use arita_syntax::Test;
        let m = Module {
            name: "demo".into(),
            functions: vec![Function {
                name: "main".into(),
                params: vec![],
                ret_ty: Type::IoUnit,
                is_async: false,
                is_pub: false,
                body: vec![Stmt::Expr(Expr::Call(Call {
                    callee: "print".into(),
                    args: vec![Expr::LitStr("hi".into())],
                }))],
            }],
            tests: vec![Test {
                name: "one_equals_one".into(),
                body: vec![
                    Stmt::Let {
                        mutable: false,
                        name: "n".into(),
                        ty: Type::Int,
                        init: Expr::LitInt(1),
                    },
                    Stmt::Assert {
                        lhs: Expr::Path(Path {
                            segments: vec!["n".into()],
                        }),
                        rhs: Expr::LitInt(1),
                    },
                ],
            }],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("#[cfg(test)]"), "got:\n{rs}");
        assert!(rs.contains("fn one_equals_one()"), "got:\n{rs}");
        assert!(rs.contains("assert_eq!(n, 1);"), "got:\n{rs}");
        assert!(rs.contains("println!(\"hi\")"), "got:\n{rs}");
    }

    #[test]
    fn emits_if_while_assign() {
        let m = main_mod(vec![
            Stmt::If {
                cond: Expr::LitBool(true),
                then_body: vec![Stmt::Expr(Expr::Call(Call {
                    callee: "print".into(),
                    args: vec![Expr::LitStr("yes".into())],
                }))],
                else_body: Some(vec![Stmt::Expr(Expr::Call(Call {
                    callee: "print".into(),
                    args: vec![Expr::LitStr("no".into())],
                }))]),
            },
            Stmt::Let {
                mutable: true,
                name: "i".into(),
                ty: Type::Int,
                init: Expr::LitInt(0),
            },
            Stmt::While {
                cond: Expr::Binary {
                    op: BinOp::Lt,
                    lhs: Box::new(Expr::Path(Path {
                        segments: vec!["i".into()],
                    })),
                    rhs: Box::new(Expr::LitInt(2)),
                },
                body: vec![
                    Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::Path(Path {
                            segments: vec!["i".into()],
                        })],
                    })),
                    Stmt::Assign {
                        name: "i".into(),
                        value: Expr::Binary {
                            op: BinOp::Add,
                            lhs: Box::new(Expr::Path(Path {
                                segments: vec!["i".into()],
                            })),
                            rhs: Box::new(Expr::LitInt(1)),
                        },
                    },
                ],
            },
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("if true {"), "got:\n{rs}");
        assert!(rs.contains("} else {"), "got:\n{rs}");
        assert!(rs.contains("while i < 2 {"), "got:\n{rs}");
        assert!(rs.contains("i = i + 1;"), "got:\n{rs}");
    }

    #[test]
    fn emits_match() {
        let m = main_mod(vec![Stmt::Match {
            scrutinee: Expr::Path(Path {
                segments: vec!["n".into()],
            }),
            arms: vec![
                arita_syntax::MatchArm {
                    pat: Pat::LitInt(1),
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("a".into())],
                    }))],
                },
                arita_syntax::MatchArm {
                    pat: Pat::Wildcard,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("b".into())],
                    }))],
                },
            ],
        }]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("match n {"), "got:\n{rs}");
        assert!(rs.contains("1 => {"), "got:\n{rs}");
        assert!(rs.contains("_ => {"), "got:\n{rs}");
        assert!(rs.contains("println!(\"a\")"), "got:\n{rs}");
    }

    #[test]
    fn emits_match_expr() {
        let m = main_mod(vec![Stmt::Let {
            mutable: false,
            name: "x".into(),
            ty: Type::Int,
            init: Expr::Match {
                scrutinee: Box::new(Expr::Path(Path {
                    segments: vec!["flag".into()],
                })),
                arms: vec![
                    arita_syntax::MatchExprArm {
                        pat: Pat::LitBool(true),
                        value: Expr::LitInt(1),
                    },
                    arita_syntax::MatchExprArm {
                        pat: Pat::LitBool(false),
                        value: Expr::LitInt(0),
                    },
                ],
            },
        }]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("let x: i64 = match flag {"), "got:\n{rs}");
        assert!(rs.contains("true => 1,"), "got:\n{rs}");
        assert!(rs.contains("false => 0,"), "got:\n{rs}");
    }

    #[test]
    fn emits_string_len_as_i64_and_is_empty() {
        let s = Expr::Path(Path {
            segments: vec!["s".into()],
        });
        let v = Expr::Path(Path {
            segments: vec!["v".into()],
        });
        let m = main_mod(vec![
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::MethodCall {
                    receiver: Box::new(s.clone()),
                    method: "len".into(),
                    args: vec![],
                }],
            })),
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::MethodCall {
                    receiver: Box::new(v),
                    method: "is_empty".into(),
                    args: vec![],
                }],
            })),
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("println!(\"{}\", (s.len() as i64));"),
            "got:\n{rs}"
        );
        assert!(rs.contains("println!(\"{}\", v.is_empty());"), "got:\n{rs}");
    }

    #[test]
    fn rejects_non_whitelist_method() {
        let m = main_mod(vec![Stmt::Expr(Expr::MethodCall {
            receiver: Box::new(Expr::Path(Path {
                segments: vec!["v".into()],
            })),
            method: "retain".into(),
            args: vec![],
        })]);
        // ADR-286 S1b (§0.1d P3 + P1): the stmt is never dropped and there is no
        // `fn main() {}` fallback — the emit fails with E0006.
        let err = emit_rust(&m).expect_err("non-whitelist method stmt must fail the emit");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit method call statement `.retain(..)` in fn `main`"),
            "got: {err}"
        );
        assert!(!err.contains("fn main() {}"), "got: {err}");
    }

    #[test]
    fn emits_clear() {
        let m = main_mod(vec![Stmt::Expr(Expr::MethodCall {
            receiver: Box::new(Expr::Path(Path {
                segments: vec!["v".into()],
            })),
            method: "clear".into(),
            args: vec![],
        })]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("v.clear();"),
            "got:
{rs}"
        );
    }

    #[test]
    fn emits_user_fn_and_call() {
        let add = Function {
            name: "add".into(),
            params: vec![
                FnParam {
                    name: "a".into(),
                    ty: Type::Int,
                },
                FnParam {
                    name: "b".into(),
                    ty: Type::Int,
                },
            ],
            ret_ty: Type::Int,
            is_async: false,
            is_pub: false,
            body: vec![Stmt::Expr(Expr::Binary {
                op: BinOp::Add,
                lhs: Box::new(Expr::Path(Path {
                    segments: vec!["a".into()],
                })),
                rhs: Box::new(Expr::Path(Path {
                    segments: vec!["b".into()],
                })),
            })],
        };
        let main = Function {
            name: "main".into(),
            params: vec![],
            ret_ty: Type::IoUnit,
            is_async: false,
            is_pub: false,
            body: vec![Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![Expr::Call(Call {
                    callee: "add".into(),
                    args: vec![Expr::LitInt(2), Expr::LitInt(3)],
                })],
            }))],
        };
        let m = Module {
            name: "multi".into(),
            functions: vec![add, main],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("fn add(a: i64, b: i64) -> i64"), "got:\n{rs}");
        assert!(rs.contains("\n    a + b\n}"), "got:\n{rs}");
        assert!(!rs.contains("(a + b)"), "got:\n{rs}");
        assert!(rs.contains("println!(\"{}\", add(2, 3));"), "got:\n{rs}");
        assert!(rs.contains("fn main()"), "got:\n{rs}");
    }

    #[test]
    fn emits_break_continue() {
        let m = main_mod(vec![Stmt::While {
            cond: Expr::LitBool(true),
            body: vec![Stmt::Break, Stmt::Continue],
        }]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("break;"), "got:\n{rs}");
        assert!(rs.contains("continue;"), "got:\n{rs}");
    }

    #[test]
    fn contracts_not_emitted() {
        use arita_syntax::{Contract, ContractExpect};
        let mut m = main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitStr("hello".into())],
        }))]);
        m.contracts.push(Contract {
            name: "hello_c".into(),
            expect: ContractExpect::Stdout(vec!["hello".into()]),
            target: None,
        });
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("println!(\"hello\")"), "got:\n{rs}");
        assert!(
            !rs.contains("contract"),
            "contracts must not appear in emit:\n{rs}"
        );
        assert!(!rs.contains("expect_stdout"), "got:\n{rs}");
    }

    #[test]
    fn entry_rewrite_calls_target() {
        let m = Module {
            name: "tgt".into(),
            functions: vec![
                Function {
                    name: "helper".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: false,
                    is_pub: false,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("from-helper".into())],
                    }))],
                },
                Function {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: false,
                    is_pub: false,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("from-main".into())],
                    }))],
                },
            ],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust_with_entry(&m, "helper").expect("emit");
        assert!(rs.contains("fn helper()"), "got:\n{rs}");
        assert!(rs.contains("__arita_user_main"), "got:\n{rs}");
        assert!(rs.contains("fn main() {"), "got:\n{rs}");
        assert!(rs.contains("helper();"), "got:\n{rs}");
        assert!(rs.contains("from-helper"), "got:\n{rs}");
        assert!(rs.contains("from-main"), "demoted main body kept:\n{rs}");
    }

    #[test]
    fn entry_missing_is_e0230() {
        let m = main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitStr("x".into())],
        }))]);
        let err = emit_rust_with_entry(&m, "nope").expect_err("missing");
        assert!(err.contains("E0230"), "err={err}");
    }

    #[test]
    fn emit_cargo_toml_release_has_opt_level_3() {
        let toml = emit_cargo_toml("demo_pkg", BuildProfile::Release, false);
        assert!(
            toml.contains("[profile.release]"),
            "missing [profile.release]:\n{toml}"
        );
        assert!(
            toml.contains("opt-level = 3"),
            "missing opt-level = 3:\n{toml}"
        );
        assert!(
            !toml.contains("lto"),
            "LTO must be off (omitted) in v0:\n{toml}"
        );
        assert!(
            !toml.contains("[dependencies]"),
            "sync release must not pull deps:\n{toml}"
        );
        // ADR-292: the opt-level line is untouched; overflow-checks is added next to it.
        assert!(
            toml.contains("[profile.release]\nopt-level = 3\noverflow-checks = true\n"),
            "ADR-292: overflow-checks must follow opt-level in [profile.release]:\n{toml}"
        );
    }

    /// Lines of the `[profile.release]` section (up to the next `[` header).
    fn release_section_lines(toml: &str) -> Vec<String> {
        let mut inside = false;
        let mut out = Vec::new();
        for line in toml.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                inside = t == "[profile.release]";
                continue;
            }
            if inside && !t.is_empty() {
                out.push(t.to_string());
            }
        }
        out
    }

    #[test]
    fn adr292_release_profile_has_overflow_checks_once_inside_section() {
        for (name, tokio, hosts) in [
            ("p_sync", false, vec![]),
            ("p_async", true, vec![]),
            (
                "p_host",
                false,
                vec![("arita_host_x".to_string(), "/tmp/arita_host_x".to_string())],
            ),
        ] {
            let toml = emit_cargo_toml_with_hosts(name, BuildProfile::Release, tokio, &hosts);
            assert_eq!(
                toml.matches("overflow-checks").count(),
                1,
                "exactly one overflow-checks line ({name}):\n{toml}"
            );
            assert_eq!(toml.matches("[profile.release]").count(), 1, "{toml}");
            assert_eq!(
                release_section_lines(&toml),
                vec![
                    "opt-level = 3".to_string(),
                    RELEASE_OVERFLOW_CHECKS_LINE.to_string()
                ],
                "[profile.release] holds exactly opt-level + overflow-checks ({name}):\n{toml}"
            );
            assert!(!toml.contains("lto"), "LTO stays off:\n{toml}");
            assert!(
                !toml.contains("overflow-checks = false") && !toml.contains("[profile.dev]"),
                "{toml}"
            );
        }
        assert_eq!(RELEASE_OVERFLOW_CHECKS_LINE, "overflow-checks = true");
    }

    #[test]
    fn adr292_debug_cargo_toml_is_unchanged_no_profile_section() {
        // `dev` already checks overflow: debug output must stay byte-identical (no profile at all).
        for tokio in [false, true] {
            let toml = emit_cargo_toml_with_hosts("p", BuildProfile::Debug, tokio, &[]);
            assert!(!toml.contains("overflow-checks"), "{toml}");
            assert!(!toml.contains("[profile"), "{toml}");
        }
        let plain = emit_cargo_toml("demo_pkg", BuildProfile::Debug, false);
        assert_eq!(
            plain,
            "[package]\nname = \"demo_pkg\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n# Keep generated crate out of the ARITA workspace.\n[workspace]\n"
        );
    }

    #[test]
    fn adr292_perf02_shape_still_detectable_with_new_line() {
        // Mirrors the line-scan of measure.rs `cargo_toml_has_release_opt_level_3` (ADR-028):
        // `opt-level=3` is found inside [profile.release] even with the extra ADR-292 line.
        let toml = emit_cargo_toml("demo_pkg", BuildProfile::Release, false);
        let mut in_release = false;
        let mut found = false;
        for line in toml.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                in_release = t == "[profile.release]";
                continue;
            }
            if !in_release || t.is_empty() || t.starts_with('#') {
                continue;
            }
            let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
            if compact == "opt-level=3" {
                found = true;
            }
        }
        assert!(found, "perf-02 scan must still see opt-level = 3:\n{toml}");
    }

    #[test]
    fn emit_cargo_toml_debug_has_no_forced_opt() {
        let toml = emit_cargo_toml("demo_pkg", BuildProfile::Debug, false);
        assert!(
            !toml.contains("opt-level"),
            "debug must not force opt-level:\n{toml}"
        );
        assert!(!toml.contains("[profile.release]"), "got:\n{toml}");
    }

    #[test]
    fn emit_cargo_toml_async_release_has_tokio_and_opt() {
        let toml = emit_cargo_toml("async_pkg", BuildProfile::Release, true);
        assert!(toml.contains("tokio"), "got:\n{toml}");
        assert!(toml.contains("opt-level = 3"), "got:\n{toml}");
        assert!(toml.contains("overflow-checks = true"), "got:\n{toml}");
    }

    #[test]
    fn build_profile_parse_e0250() {
        let err = BuildProfile::parse("fantasma").expect_err("bad profile");
        assert!(err.contains("E0250"), "err={err}");
        assert!(err.contains("unknown build profile"), "err={err}");
        assert_eq!(BuildProfile::parse("debug").unwrap(), BuildProfile::Debug);
        assert_eq!(
            BuildProfile::parse("release").unwrap(),
            BuildProfile::Release
        );
    }

    #[test]
    fn host_bridges_validate_and_rewrite() {
        validate_host_bridges(&[DeclaredHostBridge {
            id: "demo".into(),
            crate_name: "arita-host-demo".into(),
            path: "crates/arita-host-demo".into(),
        }])
        .expect("demo host ok");
        let err = validate_host_bridges(&[DeclaredHostBridge {
            id: "x".into(),
            crate_name: "other".into(),
            path: "crates/other".into(),
        }])
        .unwrap_err();
        assert!(err.contains("arita-host-"), "err={err}");
        let rs = rewrite_host_calls(
            "fn main() { host::mark(); }\n",
            &[DeclaredHostBridge {
                id: "demo".into(),
                crate_name: "arita-host-demo".into(),
                path: "crates/arita-host-demo".into(),
            }],
        )
        .unwrap();
        assert!(rs.contains("arita_host_demo::mark()"), "rs={rs}");
        let err = rewrite_host_calls("host::mark()", &[]).unwrap_err();
        assert!(err.contains("host-bridges"), "err={err}");
    }

    #[test]
    fn deps_whitelist_tokio_ok_and_unknown_e0260() {
        validate_declared_deps(&[DeclaredDep {
            name: "tokio".into(),
            bridge: "async-runtime".into(),
        }])
        .expect("tokio whitelist");
        let err = validate_declared_deps(&[DeclaredDep {
            name: "serde".into(),
            bridge: "json".into(),
        }])
        .expect_err("serde out");
        assert!(err.contains("E0260"), "err={err}");
        assert!(
            err.contains("dependency not in ARITA whitelist"),
            "err={err}"
        );
        let err2 = validate_declared_deps(&[DeclaredDep {
            name: "tokio".into(),
            bridge: "wrong".into(),
        }])
        .expect_err("bad bridge");
        assert!(err2.contains("E0260"), "err={err2}");
    }

    #[test]
    fn resolve_with_tokio_async_without_manifest() {
        assert!(resolve_with_tokio(&[], true).unwrap());
        assert!(!resolve_with_tokio(&[], false).unwrap());
        assert!(resolve_with_tokio(
            &[DeclaredDep {
                name: "tokio".into(),
                bridge: "async-runtime".into(),
            }],
            false
        )
        .unwrap());
    }

    #[test]
    fn emit_result_ok_match() {
        let m = main_mod(vec![
            Stmt::Let {
                mutable: false,
                name: "r".into(),
                ty: Type::Result(Box::new(Type::Int), Box::new(Type::String)),
                init: Expr::Ok(Box::new(Expr::LitInt(42))),
            },
            Stmt::Match {
                scrutinee: Expr::Path(Path {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    arita_syntax::MatchArm {
                        pat: Pat::Ok("x".into()),
                        body: vec![Stmt::Expr(Expr::Call(Call {
                            callee: "print".into(),
                            args: vec![Expr::Path(Path {
                                segments: vec!["x".into()],
                            })],
                        }))],
                    },
                    arita_syntax::MatchArm {
                        pat: Pat::Err("e".into()),
                        body: vec![Stmt::Expr(Expr::Call(Call {
                            callee: "print".into(),
                            args: vec![Expr::Path(Path {
                                segments: vec!["e".into()],
                            })],
                        }))],
                    },
                ],
            },
        ]);
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("let r: Result<i64, String> = Ok(42);"),
            "got:\n{rs}"
        );
        assert!(rs.contains("Ok(x) =>"), "got:\n{rs}");
        assert!(rs.contains("Err(e) =>"), "got:\n{rs}");
    }
}

#[cfg(test)]
mod adr277_fn_result_emit {
    use super::*;
    use arita_syntax::FnParam;

    /// ADR-277 — valued helper `fn → Result` emits Rust Result sig; trailing match as expr; no unwrap.
    #[test]
    fn emit_fn_result_helper_no_unwrap() {
        let m = Module {
            name: "core08_fn_result_ok".into(),
            functions: vec![
                Function {
                    name: "load".into(),
                    params: vec![FnParam {
                        name: "path".into(),
                        ty: Type::String,
                    }],
                    ret_ty: Type::Result(Box::new(Type::String), Box::new(Type::Int)),
                    is_async: false,
                    is_pub: false,
                    body: vec![Stmt::Match {
                        scrutinee: Expr::Path(Path {
                            segments: vec!["r".into()],
                        }),
                        arms: vec![
                            arita_syntax::MatchArm {
                                pat: Pat::Ok("t".into()),
                                body: vec![Stmt::Expr(Expr::Ok(Box::new(Expr::Path(Path {
                                    segments: vec!["t".into()],
                                }))))],
                            },
                            arita_syntax::MatchArm {
                                pat: Pat::Err("e".into()),
                                body: vec![Stmt::Expr(Expr::Err(Box::new(Expr::Path(Path {
                                    segments: vec!["e".into()],
                                }))))],
                            },
                        ],
                    }],
                },
                Function {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: false,
                    is_pub: false,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("ok".into())],
                    }))],
                },
            ],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("fn load(path: String) -> Result<String, i64>"),
            "missing Result sig:\n{rs}"
        );
        assert!(rs.contains("Ok(t) =>"), "got:\n{rs}");
        assert!(rs.contains("Err(e) =>"), "got:\n{rs}");
        assert!(!rs.contains(".unwrap()"), "no unwrap:\n{rs}");
        assert!(!rs.contains(".expect("), "no expect:\n{rs}");
        assert!(!rs.contains(concat!("panic!", "(")), "no panic:\n{rs}");
    }
}

#[cfg(test)]
mod adr278_qmark_emit {
    use super::*;
    use arita_syntax::FnParam;

    /// ADR-278 — Expr::Try / HirExpr::Try emit Rust `?`; zero unwrap/expect/panic (241).
    #[test]
    fn emit_try_qmark_no_unwrap() {
        let m = Module {
            name: "core08_qmark_chain".into(),
            functions: vec![
                Function {
                    name: "load".into(),
                    params: vec![FnParam {
                        name: "path".into(),
                        ty: Type::String,
                    }],
                    ret_ty: Type::Result(Box::new(Type::String), Box::new(Type::Int)),
                    is_async: false,
                    is_pub: false,
                    body: vec![
                        Stmt::Let {
                            mutable: false,
                            name: "t".into(),
                            ty: Type::String,
                            init: Expr::Try {
                                inner: Box::new(Expr::Call(Call {
                                    callee: "host.read_text".into(),
                                    args: vec![Expr::Path(Path {
                                        segments: vec!["path".into()],
                                    })],
                                })),
                            },
                        },
                        Stmt::Expr(Expr::Ok(Box::new(Expr::Path(Path {
                            segments: vec!["t".into()],
                        })))),
                    ],
                },
                Function {
                    name: "pipeline".into(),
                    params: vec![FnParam {
                        name: "path".into(),
                        ty: Type::String,
                    }],
                    ret_ty: Type::Result(Box::new(Type::String), Box::new(Type::Int)),
                    is_async: false,
                    is_pub: false,
                    body: vec![
                        Stmt::Let {
                            mutable: false,
                            name: "a".into(),
                            ty: Type::String,
                            init: Expr::Try {
                                inner: Box::new(Expr::Call(Call {
                                    callee: "load".into(),
                                    args: vec![Expr::Path(Path {
                                        segments: vec!["path".into()],
                                    })],
                                })),
                            },
                        },
                        Stmt::Expr(Expr::Ok(Box::new(Expr::Path(Path {
                            segments: vec!["a".into()],
                        })))),
                    ],
                },
                Function {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: false,
                    is_pub: false,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("ok".into())],
                    }))],
                },
            ],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(
            rs.contains("fn load(path: String) -> Result<String, i64>"),
            "missing Result sig:\n{rs}"
        );
        assert!(
            rs.contains("?;") || rs.contains(")?"),
            "missing try/? emit:\n{rs}"
        );
        assert!(!rs.contains(".unwrap()"), "no unwrap:\n{rs}");
        assert!(!rs.contains(".expect("), "no expect:\n{rs}");
        assert!(!rs.contains(concat!("panic!", "(")), "no panic:\n{rs}");
    }
}

#[cfg(test)]
mod evidence_r4_emit_ban {
    /// ADR-227 / EVIDENCE-R4: codegen must not emit Rust Index panic (`v[i]`).
    #[test]
    fn emit_has_no_panic_index_patterns() {
        let src = include_str!("lib.rs");
        let production = src.split("mod evidence_r4_emit_ban").next().expect("split");
        // Construct needles so this test file does not contain the literal haystack.
        let panic_index = concat!("{", "recv", "}[");
        assert!(
            !production.contains(panic_index),
            "codegen must not format panic index recv[i]"
        );
        assert!(
            !production.contains(".index("),
            "codegen must not call .index("
        );
        assert!(
            !production.contains("get_unchecked"),
            "codegen must not use get_unchecked"
        );
    }
}

#[cfg(test)]
mod evidence_r0r2_emit_ban {
    /// ADR-230 / EVIDENCE-R0R2: injected user emit must not contain panic channels.
    #[test]
    fn emit_helpers_and_templates_have_no_panic_channels() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("production before cfg test");
        // Ban bare unwrap/expect/panic in production codegen (helpers + format templates).
        // Allow: unwrap_or, unwrap_err (tests only below split), unwrap_or_else.
        let mut bad = Vec::new();
        for (i, line) in production.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            // `.unwrap()` but not unwrap_or / unwrap_err
            if line.contains(".unwrap()") {
                bad.push(format!("L{}: unwrap()", i + 1));
            }
            if line.contains(".expect(") {
                bad.push(format!("L{}: expect(", i + 1));
            }
            if line.contains(concat!("panic!", "(")) {
                bad.push(format!("L{}: panic!", i + 1));
            }
        }
        assert!(
            bad.is_empty(),
            "R0/R2 emit-ban: panic channels in production codegen:\n{}",
            bad.join("\n")
        );
    }

    #[test]
    fn pow_template_uses_unwrap_or_not_bare_unwrap() {
        let src = include_str!("lib.rs");
        let production = src.split("#[cfg(test)]").next().expect("prod");
        assert!(
            production.contains("checked_pow") && production.contains("unwrap_or(i64::MAX)"),
            "pow emit must use checked_pow + unwrap_or(i64::MAX)"
        );
        assert!(
            !production.contains("checked_pow(({a}) as u32).unwrap()"),
            "pow must not bare-unwrap checked_pow"
        );
    }
}

#[cfg(test)]
mod timeout_cancel_emit {
    use super::*;

    #[test]
    fn emit_timeout_maps_elapsed() {
        let m = Module {
            name: "t".into(),
            functions: vec![
                Function {
                    name: "quick".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: true,
                    is_pub: false,
                    body: vec![Stmt::Expr(Expr::Call(Call {
                        callee: "print".into(),
                        args: vec![Expr::LitStr("fast".into())],
                    }))],
                },
                Function {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: Type::IoUnit,
                    is_async: true,
                    is_pub: false,
                    body: vec![Stmt::Let {
                        mutable: false,
                        name: "r".into(),
                        ty: Type::Result(Box::new(Type::Unit), Box::new(Type::String)),
                        init: Expr::Call(Call {
                            callee: "timeout".into(),
                            args: vec![
                                Expr::LitInt(1000),
                                Expr::Call(Call {
                                    callee: "quick".into(),
                                    args: vec![],
                                }),
                            ],
                        }),
                    }],
                },
            ],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let rs = emit_rust(&m).expect("emit_rust");
        assert!(rs.contains("tokio::time::timeout"), "got:\n{rs}");
        assert!(rs.contains("Err(\"timeout\".to_string())"), "got:\n{rs}");
        assert!(!rs.contains("unwrap("), "no unwrap in emit:\n{rs}");
    }

    #[test]
    fn emit_cargo_tokio_has_time_feature() {
        let toml = emit_cargo_toml("p", BuildProfile::Debug, true);
        assert!(toml.contains("time"), "got:\n{toml}");
    }
}

#[cfg(test)]
mod adr282_map_assign_emit {
    //! ADR-282 (CORE-0.9-MAP-ASSIGN-20260926) slice 1 — `m[k] = v` emit shape (R2/R3, ban 241).
    use super::*;

    fn emit_main(body: &str) -> String {
        let src = format!("module t\nfn main() -> Io<()> {{\n{body}\n}}\n");
        let m = arita_syntax::parse(&src).expect("ADR-282 source must parse");
        emit_rust_with_entry(&m, "main").expect("ADR-282 emit must succeed")
    }

    fn idx_lines(rs: &str) -> Vec<&str> {
        rs.lines().filter(|l| l.contains("__arita_mk")).collect()
    }

    #[test]
    fn adr282_emit_temporaries_then_bare_insert() {
        let rs = emit_main(
            "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 7\n  m[\"a\"] = 9\n  print(m.len())",
        );
        assert!(rs.contains("use std::collections::HashMap;"), "{rs}");
        let lines = idx_lines(&rs);
        assert_eq!(lines.len(), 2, "one block per site:\n{rs}");
        for l in &lines {
            let t = l.trim();
            // R2: k then v into temps, both before the &mut borrow of insert.
            let mk = t.find("let __arita_mk = ").expect("mk temp");
            let mv = t.find("let __arita_mv = ").expect("mv temp");
            let ins = t
                .find("m.insert(__arita_mk, __arita_mv);")
                .expect("bare insert");
            assert!(mk < mv && mv < ins, "order k → v → insert: {t}");
            // Own block per site.
            assert!(t.starts_with("{ let __arita_mk") && t.ends_with('}'), "{t}");
            // R3: no `let _ =`; ban 241: no Rust index assign / entry / unwrap.
            assert!(!t.contains("let _ ="), "{t}");
            assert!(!t.contains("] ="), "{t}");
            assert!(!t.contains("entry("), "{t}");
            assert!(!t.contains("HashMap::insert"), "{t}");
            assert!(!t.contains(".unwrap()") && !t.contains(".expect("), "{t}");
            assert!(!t.contains("as usize"), "{t}");
        }
        assert!(
            lines[0].contains("let __arita_mk = \"a\".to_string();"),
            "{rs}"
        );
        assert!(lines[0].contains("let __arita_mv = 7;"), "{rs}");
        assert!(lines[1].contains("let __arita_mv = 9;"), "{rs}");
    }

    #[test]
    fn adr282_self_ref_value_strips_outer_parens() {
        let rs = emit_main(
            "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n  m[\"b\"] = m.len()\n  print(m.len())",
        );
        let lines = idx_lines(&rs);
        assert_eq!(lines.len(), 2, "{rs}");
        // unused_parens: `(m.len() as i64)` must be emitted without the outer pair.
        assert!(
            lines[1].contains("let __arita_mv = m.len() as i64;"),
            "{rs}"
        );
        assert!(!lines[1].contains("= (m.len() as i64);"), "{rs}");
        assert!(!lines[1].contains("as usize"), "{rs}");
    }

    #[test]
    fn adr282_non_map_target_is_never_silently_dropped() {
        // ADR-283 FASE 2: Vec targets now emit (`__arita_vec_set`); the no-silent-drop property
        // is kept for any target that is neither Map nor Vec (HIR → E0314): the fn must fail
        // to emit (loud rustc error), never compile without the statement.
        let src =
            "module t\nfn main() -> Io<()> {\n  let mut v: Int = 1\n  v[0] = 2\n  print(v)\n}\n";
        let m = arita_syntax::parse(src).expect("parser does not decide Map vs Vec");
        // ADR-286 S1b (§0.1d P2): the fn is no longer omitted from the module — the whole
        // emit fails with E0006.
        let err = emit_rust_with_entry(&m, "main")
            .expect_err("non-Map IndexAssign must fail the module emit");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit index assignment `x[..] = ..` in fn `main`"),
            "got: {err}"
        );
        assert!(!err.contains("__arita_mk"), "{err}");
        assert!(
            !err.contains("__arita_vec_set"),
            "no Vec helper for a non-Vec target: {err}"
        );
    }

    #[test]
    fn adr282_map_declared_in_nested_block_is_emitted() {
        let rs = emit_main(
            "  let c: Int = 1\n  if c == 1 {\n    let mut m: Map<Text, Int> = Map::new()\n    m[\"a\"] = 4\n    print(m.len())\n  } else {\n    print(0)\n  }",
        );
        assert!(rs.contains("fn main("), "{rs}");
        assert!(
            rs.contains("{ let __arita_mk = \"a\".to_string(); let __arita_mv = 4; m.insert(__arita_mk, __arita_mv); }"),
            "nested-block Map decl must reach the gate:\n{rs}"
        );
    }

    #[test]
    fn adr282_nested_non_map_target_is_never_silently_dropped() {
        // ADR-283 FASE 2: non-Map, non-Vec target (Vec now emits).
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Int = 1\n  let c: Int = 1\n  if c == 1 {\n    v[0] = 2\n  } else {\n    print(0)\n  }\n  print(v)\n}\n";
        let m = arita_syntax::parse(src).expect("parser does not decide Map vs Vec");
        // ADR-286 S1b (§0.1d P2): nested IndexAssign fails the module emit with E0006.
        let err = emit_rust_with_entry(&m, "main")
            .expect_err("nested non-Map IndexAssign must fail the module emit");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit index assignment `x[..] = ..` in fn `main`"),
            "got: {err}"
        );
    }

    // ---- ADR-286 S1b JOIN-SAFETY (§0.1d (b)): no silent omissions in codegen ----

    fn s1b_parse(src: &str) -> Module {
        arita_syntax::parse(src).expect("s1b fixture parses")
    }

    fn s1b_main_mod(body: Vec<Stmt>) -> Module {
        let mut m = s1b_parse("module hello\nfn main() -> Io<()> {\n  print(1)\n}\n");
        m.functions[0].body = body;
        m
    }

    #[test]
    fn s1b_p3_discarded_value_expression_stmt_is_e0006() {
        // `n + 1` as a non-final stmt used to vanish from the Rust (build exit 0).
        let m = s1b_parse(
            "module sa\n\nfn f(n: Int) -> Int {\n  n + 1\n  n\n}\n\nfn main() -> Io<()> {\n  print(f(2))\n}\n",
        );
        let err = emit_rust_with_entry(&m, "main").expect_err("P3 must not drop the stmt");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with(
                "E0006: cannot emit expression statement whose value is discarded in fn `f`"
            ),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_p3_unemittable_nested_stmt_is_e0006() {
        // A block whose inner stmt cannot be emitted is a `None` for the whole `if`.
        let m = s1b_parse(
            "module sb\n\nfn main() -> Io<()> {\n  let n: Int = 2\n  if n == 1 {\n    n + 1\n  } else {\n    print(0)\n  }\n  print(n)\n}\n",
        );
        let err = emit_rust_with_entry(&m, "main").expect_err("nested P3 must not drop");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit `if` statement in fn `main`"),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_p6_join_over_spawn_of_result_fn_is_e0006() {
        let m = s1b_parse(
            "module p8\n\nasync fn f() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  await join(spawn(f()))\n  print(\"after\")\n}\n",
        );
        let err = emit_rust_with_entry(&m, "main").expect_err("P6 over a Result fn");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with(
                "E0006: cannot emit join over spawn of non-Io<()> async fn 'f' (its result would be discarded)"
            ),
            "got: {err}"
        );
        assert!(err.contains("fn `main`"), "got: {err}");
    }

    #[test]
    fn s1b_p6_join_over_spawn_of_io_fn_still_emits() {
        let m = s1b_parse(
            "module p14\n\nasync fn g() -> Io<()> {\n  print(\"worker\")\n}\n\nasync fn main() -> Io<()> {\n  await join(spawn(g()))\n  print(\"after\")\n}\n",
        );
        let rs = emit_rust_with_entry(&m, "main").expect("Io<()> spawn keeps emitting");
        assert!(
            rs.contains("{ let _ = tokio::spawn(g()).await; }"),
            "P6 over an Io<()> task must be unchanged:\n{rs}"
        );
    }

    #[test]
    fn s1b_p6_join_over_task_ident_is_unchanged() {
        let m = s1b_parse(
            "module p1\n\nasync fn g() -> Io<()> {\n  print(\"worker\")\n}\n\nasync fn main() -> Io<()> {\n  let t: Task = spawn(g())\n  await join(t)\n  print(\"after\")\n}\n",
        );
        let rs = emit_rust_with_entry(&m, "main").expect("Task ident join emits");
        assert!(rs.contains("{ let _ = t.await; }"), "{rs}");
    }

    #[test]
    fn s1b_p6_join_over_spawn_of_undeclared_fn_is_e0006() {
        // Built directly (HIR would reject): the callee is not a fn of the module/deps.
        let spawn = Expr::Call(Call {
            callee: "spawn".into(),
            args: vec![Expr::Call(Call {
                callee: "ghost".into(),
                args: vec![],
            })],
        });
        let stmt = Stmt::Expr(Expr::Await {
            inner: Box::new(Expr::Call(Call {
                callee: "join".into(),
                args: vec![spawn],
            })),
        });
        let mut m = s1b_main_mod(vec![stmt]);
        m.functions[0].is_async = true;
        let err = emit_rust_with_entry(&m, "main").expect_err("unknown spawn target");
        assert!(
            err.starts_with("E0006: cannot emit join over spawn of 'ghost'"),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_shadowed_builtin_spawn_under_await_is_e0006() {
        let m = s1b_parse(
            "module a_spawn\n\nasync fn spawn() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  await spawn()\n  print(\"after\")\n}\n",
        );
        let err = emit_rust_with_entry(&m, "main").expect_err("shadowed call must not vanish");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with(
                "E0006: cannot emit call `spawn(..)` in fn `main`: user fn `spawn` shadows the builtin `spawn`"
            ),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_p4_unemittable_test_stmt_is_e0006() {
        // An assert whose side cannot be emitted used to vanish (the test passed empty).
        let mut m = s1b_main_mod(vec![Stmt::Expr(Expr::Call(Call {
            callee: "print".into(),
            args: vec![Expr::LitStr("hi".into())],
        }))]);
        m.tests.push(arita_syntax::Test {
            name: "empty assert".into(),
            body: vec![Stmt::Assert {
                lhs: Expr::MethodCall {
                    receiver: Box::new(Expr::Path(Path {
                        segments: vec!["v".into()],
                    })),
                    method: "retain".into(),
                    args: vec![],
                },
                rhs: Expr::LitInt(1),
            }],
        });
        let err = emit_rust(&m).expect_err("P4 must not emit an empty test");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit `assert` statement in test `empty assert`"),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_p1_emit_rust_has_no_fn_main_fallback() {
        // E0230 (entry is not `fn() -> Io<()>`) used to become `fn main() {}` on the test path.
        let mut m = s1b_main_mod(vec![]);
        m.functions[0].ret_ty = Type::Int;
        let err = emit_rust(&m).expect_err("P1 propagates the error");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert!(
            err.starts_with("E0006: cannot emit module `hello`: E0230"),
            "got: {err}"
        );
    }

    // ---- ADR-286 S1b decision (2): signatures never leak between emits ----

    const S1B_MOD_A: &str =
        "module a\n\nasync fn g() -> Io<()> {\n  print(\"g\")\n}\n\nasync fn main() -> Io<()> {\n  await join(spawn(g()))\n}\n";
    // B calls `g` without declaring it (HIR would reject; codegen is the last defence).
    const S1B_MOD_B: &str =
        "module b\n\nasync fn main() -> Io<()> {\n  await join(spawn(g()))\n  print(\"after\")\n}\n";

    fn s1b_sigs_len() -> usize {
        EMIT_FN_SIGS.with(|s| s.borrow().len())
    }

    #[test]
    fn s1b_sigs_do_not_leak_from_module_a_to_b() {
        let a = s1b_parse(S1B_MOD_A);
        let b = s1b_parse(S1B_MOD_B);
        let rs = emit_rust_with_entry(&a, "main").expect("A emits");
        assert!(rs.contains("{ let _ = tokio::spawn(g()).await; }"), "{rs}");
        assert_eq!(s1b_sigs_len(), 0, "table restored after A");
        let err = emit_rust_with_entry(&b, "main").expect_err("g is not inherited from A");
        assert!(
            err.starts_with("E0006: cannot emit join over spawn of 'g': not an async fn of this module or its deps"),
            "got: {err}"
        );
        assert_eq!(s1b_sigs_len(), 0, "table restored after B");
    }

    #[test]
    fn s1b_sigs_do_not_leak_b_then_a_and_failed_emit_leaves_nothing() {
        let a = s1b_parse(S1B_MOD_A);
        let b = s1b_parse(S1B_MOD_B);
        // Failing emit first: no stale signatures and no stale diag for the next emit.
        assert!(emit_rust_with_entry(&b, "main").is_err());
        assert_eq!(s1b_sigs_len(), 0);
        assert!(take_emit_diag().is_none(), "stale diag after failed emit");
        assert!(emit_rust_with_entry(&a, "main").is_ok());
        // A fails midway (P3) and B must still not see its `g`.
        let bad = s1b_parse(
            "module c\n\nasync fn g() -> Io<()> {\n  print(\"g\")\n}\n\nfn f(n: Int) -> Int {\n  n + 1\n  n\n}\n\nasync fn main() -> Io<()> {\n  print(1)\n}\n",
        );
        let err = emit_rust_with_entry(&bad, "main").expect_err("P3 in the middle");
        assert!(err.starts_with("E0006:"), "got: {err}");
        assert_eq!(s1b_sigs_len(), 0, "failed emit leaves no signatures");
        let err = emit_rust_with_entry(&b, "main").expect_err("g not inherited from failed C");
        assert!(
            err.starts_with("E0006: cannot emit join over spawn of 'g'"),
            "got: {err}"
        );
    }

    #[test]
    fn s1b_every_entry_point_restores_sigs() {
        let a = s1b_parse(S1B_MOD_A);
        let lib = s1b_parse("module l\n\nasync fn lg() -> Io<()> {\n  print(\"lg\")\n}\n");
        let _ = emit_rust(&a);
        assert_eq!(s1b_sigs_len(), 0, "emit_rust");
        let _ = emit_rust_with_entry(&a, "main");
        assert_eq!(s1b_sigs_len(), 0, "emit_rust_with_entry");
        let _ = emit_rust_lib(&lib);
        assert_eq!(s1b_sigs_len(), 0, "emit_rust_lib");
        let _ = emit_rust_program(&a, std::slice::from_ref(&lib), "main");
        assert_eq!(s1b_sigs_len(), 0, "emit_rust_program");
        let ext = [ExternFnSig::from_function(&lib.functions[0])];
        let _ = emit_rust_with_entry_and_sigs(&a, "main", &ext);
        assert_eq!(s1b_sigs_len(), 0, "emit_rust_with_entry_and_sigs");
    }

    #[test]
    fn s1b_guard_restores_state_on_panic() {
        let ext = [ExternFnSig {
            name: "zz".into(),
            is_async: true,
            ret_ty: Type::IoUnit,
        }];
        let r = std::panic::catch_unwind(|| {
            let m = s1b_parse(S1B_MOD_A);
            let _g = enter_fn_sigs(&ext, std::iter::once(&m));
            assert!(lookup_fn_sig("zz").is_some());
            std::panic::resume_unwind(Box::new("boom"));
        });
        assert!(r.is_err());
        assert_eq!(s1b_sigs_len(), 0, "unwinding restored the table");
        assert!(lookup_fn_sig("zz").is_none());
    }

    /// Mirrors `package.rs`: the bin has `use lib::lib_fn`, HIR got the injected lib fn, then the
    /// injected fn is stripped from the emitted module and its signature passed as an extern.
    #[test]
    fn s1b_package_flow_strip_then_extern_sigs() {
        let lib = s1b_parse("module lib\n\nasync fn lib_fn() -> Io<()> {\n  print(\"lib\")\n}\n\nasync fn lib_res() -> Result<Int, Int> {\n  Err(7)\n}\n");
        for (name, ok) in [("lib_fn", true), ("lib_res", false)] {
            let mut bin = s1b_parse(&format!(
                "module pk\n\nasync fn main() -> Io<()> {{\n  await join(spawn({name}()))\n  print(\"after\")\n}}\n"
            ));
            let injected = lib.functions.iter().find(|f| f.name == name).expect(name);
            bin.functions.push(injected.clone());
            let externs = vec![ExternFnSig::from_function(injected)];
            bin.functions.retain(|f| f.name != name);
            // Without externs the stripped fn is unknown: the old (S1b first land) behaviour.
            let err = emit_rust_with_entry(&bin, "main").expect_err("stripped fn unknown");
            assert!(
                err.starts_with(
                    "E0006: cannot emit join over spawn of '{name}': not an async fn"
                        .replace("{name}", name)
                        .as_str()
                ),
                "got: {err}"
            );
            let r = emit_rust_with_entry_and_sigs(&bin, "main", &externs);
            if ok {
                let rs = r.expect("Io<()> lib fn must emit");
                assert!(
                    rs.contains(&format!("{{ let _ = tokio::spawn({name}()).await; }}")),
                    "{rs}"
                );
            } else {
                let err = r.expect_err("Result lib fn must be E0006");
                assert!(
                    err.starts_with(&format!(
                        "E0006: cannot emit join over spawn of non-Io<()> async fn '{name}'"
                    )),
                    "got: {err}"
                );
            }
            assert_eq!(s1b_sigs_len(), 0);
        }
    }

    #[test]
    fn s1b_extern_sigs_allow_io_unit_and_reject_result() {
        let b = s1b_parse(S1B_MOD_B);
        let io = [ExternFnSig {
            name: "g".into(),
            is_async: true,
            ret_ty: Type::IoUnit,
        }];
        let rs = emit_rust_with_entry_and_sigs(&b, "main", &io).expect("extern Io<()> fn");
        assert!(rs.contains("{ let _ = tokio::spawn(g()).await; }"), "{rs}");
        let res = [ExternFnSig {
            name: "g".into(),
            is_async: true,
            ret_ty: Type::Int,
        }];
        let err = emit_rust_with_entry_and_sigs(&b, "main", &res).expect_err("extern non-Io fn");
        assert!(
            err.starts_with(
                "E0006: cannot emit join over spawn of non-Io<()> async fn 'g' (its result would be discarded)"
            ),
            "got: {err}"
        );
        assert_eq!(s1b_sigs_len(), 0);
    }

    #[test]
    fn adr282_walkers_visit_index_assign() {
        let body = vec![Stmt::IndexAssign {
            target: "m".into(),
            key: Expr::LitStr("a".into()),
            value: Expr::MethodCall {
                receiver: Box::new(Expr::LitInt(7)),
                method: "div_ceil".into(),
                args: vec![Expr::LitInt(2)],
            },
            op: None, // ADR-290
            span: arita_syntax::Span { start: 0, end: 0 },
        }];
        let m = Module {
            name: "t".into(),
            functions: vec![Function {
                name: "main".into(),
                params: vec![],
                ret_ty: Type::IoUnit,
                is_async: false,
                is_pub: false,
                body,
            }],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        // ADR-283 FASE 2: an IndexAssign alone no longer implies HashMap — only when its
        // target is declared Map in the same body (a Vec target must not import it).
        assert!(
            !module_uses_map(&m),
            "undeclared target ⇒ no HashMap import"
        );
        assert!(module_uses_div_round(&m), "walker must recurse into value");
        assert!(!module_uses_vec_set(&m) && !module_uses_vec_insert(&m));
        let mut with_map = m.clone();
        with_map.functions[0].body.insert(
            0,
            Stmt::Let {
                mutable: true,
                name: "m".into(),
                ty: Type::Map(Box::new(Type::String), Box::new(Type::Int)),
                init: Expr::Call(Call {
                    callee: "Map::new".into(),
                    args: vec![],
                }),
            },
        );
        assert!(
            module_uses_map(&with_map),
            "Map-typed target ⇒ HashMap import"
        );
        assert!(!module_uses_vec_set(&with_map));
        let mut with_vec = m.clone();
        with_vec.functions[0].body.insert(
            0,
            Stmt::Let {
                mutable: true,
                name: "m".into(),
                ty: Type::Vec(Box::new(Type::Int)),
                init: Expr::Call(Call {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
        );
        assert!(
            module_uses_vec_set(&with_vec),
            "Vec-typed target ⇒ set helper"
        );
    }

    #[test]
    fn adr282_strip_one_outer_parens_is_balanced() {
        assert_eq!(strip_one_outer_parens("(m.len() as i64)"), "m.len() as i64");
        assert_eq!(strip_one_outer_parens("(x + 1)"), "x + 1");
        assert_eq!(strip_one_outer_parens("((x))"), "(x)");
        assert_eq!(strip_one_outer_parens("(a) + (b)"), "(a) + (b)");
        assert_eq!(strip_one_outer_parens("7"), "7");
        assert_eq!(
            strip_one_outer_parens("\"(\".to_string()"),
            "\"(\".to_string()"
        );
        assert_eq!(strip_one_outer_parens("(f(\")\"))"), "f(\")\")");
    }
}

#[cfg(test)]
mod adr283_vec_helpers_r1 {
    //! ADR-283 FASE 1 (CORE-0.9-VEC-ASSIGN-20260926) — R1: helpers `__arita_vec_*` sin `as usize`
    //! + oracle `core09-vec-helpers-semantics-unchanged` (rustc real: copia literal antigua vs nueva).
    use super::*;

    /// Copia literal de los helpers ADR-260/265 previos a ADR-283 (de `lib.rs.bak_283_codegen_pre`),
    /// renombrados `__old_*` para compilarlos junto a los nuevos.
    const OLD_VEC_HELPERS: &str = r#"
fn __old_arita_vec_insert<T>(v: &mut Vec<T>, i: i64, x: T) -> Result<(), i64> {
    if i < 0 || (i as usize) > v.len() {
        Err(0)
    } else {
        v.insert(i as usize, x);
        Ok(())
    }
}

fn __old_arita_vec_set<T>(v: &mut Vec<T>, i: i64, x: T) -> Result<(), i64> {
    if i < 0 || (i as usize) >= v.len() {
        Err(0)
    } else if let Some(slot) = v.get_mut(i as usize) {
        *slot = x;
        Ok(())
    } else {
        Err(0)
    }
}
"#;

    fn emit_main(body: &str) -> String {
        let src = format!("module t\nfn main() -> Io<()> {{\n{body}\n}}\n");
        let m = arita_syntax::parse(&src).expect("ADR-283 source must parse");
        emit_rust_with_entry(&m, "main").expect("ADR-283 emit must succeed")
    }

    /// Bodies of every `fn __arita_vec_*` in `rs` (from `fn` to the closing `\n}` at col 0).
    fn vec_helper_bodies(rs: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = rs;
        while let Some(start) = rest.find("fn __arita_vec_") {
            let tail = &rest[start..];
            let end = tail.find("\n}").map(|e| e + 2).unwrap_or(tail.len());
            out.push(tail[..end].to_string());
            rest = &tail[end..];
        }
        out
    }

    #[test]
    fn adr283_vec_helpers_have_no_as_usize() {
        // Constants as injected…
        for (name, h) in [
            ("ARITA_VEC_INSERT_HELPER", ARITA_VEC_INSERT_HELPER),
            ("ARITA_VEC_SET_HELPER", ARITA_VEC_SET_HELPER),
        ] {
            assert!(!h.contains("as usize"), "{name} contains `as usize`:\n{h}");
            assert!(
                h.contains("usize::try_from(i)"),
                "{name} lacks try_from:\n{h}"
            );
            assert!(
                h.contains("return Err(0);"),
                "{name} lacks Err(0) on conversion fail"
            );
            assert!(
                !h.contains(".unwrap()") && !h.contains(".expect("),
                "{name}"
            );
        }
        // …and as actually emitted for a program using both set and insert.
        let rs = emit_main(
            "  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  let a: Result<(), Int> = v.insert(0, 5)\n  let b: Result<(), Int> = v.set(0, 7)\n  print(v.len())",
        );
        let bodies = vec_helper_bodies(&rs);
        assert_eq!(bodies.len(), 2, "both helpers must be emitted:\n{rs}");
        let mut names: Vec<&str> = bodies
            .iter()
            .map(|b| {
                if b.contains("fn __arita_vec_set") {
                    "set"
                } else {
                    "insert"
                }
            })
            .collect();
        names.sort_unstable();
        assert_eq!(names, vec!["insert", "set"]);
        for b in &bodies {
            assert!(
                !b.contains("as usize"),
                "emitted helper has `as usize`:\n{b}"
            );
            assert_eq!(b.matches("usize::try_from(i)").count(), 1, "{b}");
        }
    }

    /// (case, op, i, expected Result, expected vec) — vec inicial `[1, 2, 3]` (len 3) salvo `empty-*`.
    const CASES: &[(&str, &str, &str, &str, &str)] = &[
        ("set-valid-0", "set", "0", "Ok(())", "[9, 2, 3]"),
        ("set-valid-last", "set", "2", "Ok(())", "[1, 2, 9]"),
        ("set-len", "set", "3", "Err(0)", "[1, 2, 3]"),
        ("set-oob", "set", "4", "Err(0)", "[1, 2, 3]"),
        ("set-neg-1", "set", "-1", "Err(0)", "[1, 2, 3]"),
        ("set-i64-min", "set", "i64::MIN", "Err(0)", "[1, 2, 3]"),
        ("set-i64-max", "set", "i64::MAX", "Err(0)", "[1, 2, 3]"),
        ("set-2pow32", "set", "4294967296", "Err(0)", "[1, 2, 3]"),
        ("set-empty-0", "set", "0", "Err(0)", "[]"),
        ("insert-valid-0", "insert", "0", "Ok(())", "[9, 1, 2, 3]"),
        ("insert-valid-1", "insert", "1", "Ok(())", "[1, 9, 2, 3]"),
        ("insert-len", "insert", "3", "Ok(())", "[1, 2, 3, 9]"),
        ("insert-oob", "insert", "4", "Err(0)", "[1, 2, 3]"),
        ("insert-neg-1", "insert", "-1", "Err(0)", "[1, 2, 3]"),
        (
            "insert-i64-min",
            "insert",
            "i64::MIN",
            "Err(0)",
            "[1, 2, 3]",
        ),
        (
            "insert-i64-max",
            "insert",
            "i64::MAX",
            "Err(0)",
            "[1, 2, 3]",
        ),
        (
            "insert-2pow32",
            "insert",
            "4294967296",
            "Err(0)",
            "[1, 2, 3]",
        ),
        ("insert-empty-0", "insert", "0", "Ok(())", "[9]"),
    ];

    #[test]
    #[cfg_attr(
        miri,
        ignore = "spawns rustc/clippy-driver (real oracle); Miri cannot posix_spawn"
    )]
    fn core09_vec_helpers_semantics_unchanged() {
        let mut prog = String::from("#![forbid(unsafe_code)]\n");
        prog.push_str(OLD_VEC_HELPERS);
        prog.push_str(ARITA_VEC_INSERT_HELPER);
        prog.push_str(ARITA_VEC_SET_HELPER);
        prog.push_str("fn main() {\n");
        for (case, op, i, _, _) in CASES {
            let init = if case.contains("empty") {
                "Vec::new()"
            } else {
                "vec![1i64, 2, 3]"
            };
            prog.push_str(&format!(
                "    {{ let mut a: Vec<i64> = {init}; let ra = __old_arita_vec_{op}(&mut a, {i}, 9); \
                 let mut b: Vec<i64> = {init}; let rb = __arita_vec_{op}(&mut b, {i}, 9); \
                 println!(\"{case}|{{:?}} {{:?}}|{{:?}} {{:?}}\", ra, a, rb, b); }}\n"
            ));
        }
        prog.push_str("}\n");
        let dir = std::env::temp_dir().join(format!("arita283_sem_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let src = dir.join("sem.rs");
        let bin = dir.join("sem");
        std::fs::write(&src, &prog).expect("write sem.rs");
        // Same rustc invocation style as `arita build` (default edition) + -D warnings.
        let c = std::process::Command::new("rustc")
            .arg("-D")
            .arg("warnings")
            .arg(&src)
            .arg("-o")
            .arg(&bin)
            .output()
            .expect("rustc must be available (real oracle, no skip)");
        assert!(
            c.status.success(),
            "rustc failed:\n{}\n{prog}",
            String::from_utf8_lossy(&c.stderr)
        );
        let run = std::process::Command::new(&bin).output().expect("run sem");
        assert!(run.status.success(), "helper program must not panic");
        let stdout = String::from_utf8_lossy(&run.stdout).to_string();
        println!("{stdout}");
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), CASES.len(), "{stdout}");
        for (line, (case, _, _, res, vec)) in lines.iter().zip(CASES) {
            let expected_side = format!("{res} {vec}");
            assert_eq!(
                *line,
                format!("{case}|{expected_side}|{expected_side}"),
                "old vs new vs expected differ for {case}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod adr283_vec_assign_emit {
    //! ADR-283 FASE 2 (CORE-0.9-VEC-ASSIGN-20260926) — `v[i] = x` on a Vec local ≡ `v.set(i, x)?`
    //! (R2 temps before `&mut`, helper emitted as only use, no HashMap import), deferred-shape
    //! marker = codegen ERROR (no panic), TryFrom import clippy-clean on editions 2015 + 2021.
    use super::*;

    fn emit_src(src: &str) -> String {
        let m = arita_syntax::parse(src).expect("ADR-283 source must parse");
        emit_rust_with_entry(&m, "main").expect("ADR-283 emit must succeed")
    }

    const FN_RESULT_SRC: &str = "module t\n\
fn f(n: Int) -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v.push(2)\n\
  v[0] = n\n\
  v[1] = v.len()\n\
  let o: Option<Int> = v.get(1)\n\
  match o {\n\
    Some(x) => { Ok(x) }\n\
    None => { Err(1) }\n\
  }\n\
}\n\
fn main() -> Io<()> {\n\
  match f(7) {\n\
    Ok(x) => { print(x) }\n\
    Err(e) => { print(e) }\n\
  }\n\
}\n";

    #[test]
    fn adr283_vec_assign_emits_set_helper_with_temps() {
        let rs = emit_src(FN_RESULT_SRC);
        assert!(
            rs.contains("    { let __arita_vi = 0; let __arita_vx = n; __arita_vec_set(&mut v, __arita_vi, __arita_vx)?; }\n"),
            "{rs}"
        );
        // R2 + paren strip: index/value temps hold the bare expressions (no outer parens).
        assert!(
            rs.contains("{ let __arita_vi = 1; let __arita_vx = v.len() as i64; __arita_vec_set(&mut v, __arita_vi, __arita_vx)?; }"),
            "{rs}"
        );
        // Helper emitted exactly once although `set` is never called by name (only use = v[i]=x).
        assert_eq!(rs.matches("fn __arita_vec_set<T>").count(), 1, "{rs}");
        // Vec-only program must not import HashMap (unused_imports).
        assert!(!rs.contains("use std::collections::HashMap"), "{rs}");
        // Emit-ban: no Rust index-assign, IndexMut, unwrap/expect/panic, discarded set result.
        for l in rs.lines() {
            let t = l.trim_start();
            assert!(
                !(t.starts_with("v[") && t.contains("] =")),
                "Rust index assign: {l}"
            );
        }
        for banned in [
            "IndexMut",
            "std::ops::Index",
            ".unwrap()",
            ".expect(",
            "panic!",
            "let _ = __arita_vec_set",
            "arita:deferred-shape",
        ] {
            assert!(!rs.contains(banned), "banned `{banned}` in:\n{rs}");
        }
        for l in rs.lines().filter(|l| l.contains("__arita_vi")) {
            assert!(!l.contains("as usize"), "R1 in lowering: {l}");
        }
    }

    #[test]
    fn adr283_vec_assign_eq_set_same_statement() {
        let sugar = emit_src(
            "module t\nfn f() -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 5\n  let n: Int = v.len()\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f() {\n    Ok(x) => { print(x) }\n    Err(e) => { print(e) }\n  }\n}\n",
        );
        let set = emit_src(
            "module t\nfn f() -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  let r: Result<(), Int> = v.set(0, 5)\n  r?\n  let n: Int = v.len()\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f() {\n    Ok(x) => { print(x) }\n    Err(e) => { print(e) }\n  }\n}\n",
        );
        assert!(
            set.contains("let r: Result<(), i64> = __arita_vec_set(&mut v, 0, 5);")
                && set.contains("    r?;\n"),
            "{set}"
        );
        assert!(
            sugar.contains("__arita_vec_set(&mut v, __arita_vi, __arita_vx)?;"),
            "{sugar}"
        );
        // Same helper text in both programs (same runtime semantics, same `?` mapping).
        let helper = |rs: &str| {
            let s = rs.find("fn __arita_vec_set").expect("helper");
            let e = rs[s..].find("\n}").expect("end") + s;
            rs[s..e].to_string()
        };
        assert_eq!(helper(&sugar), helper(&set));
    }

    #[test]
    fn adr283_non_vec_non_map_target_fails_emit() {
        // A target with no Vec/Map local type never emits (fn fails; no silent drop).
        let src = "module t\nfn f() -> Result<Int, Int> {\n  let mut n: Int = 1\n  n[0] = 5\n  let k: Int = n\n  Ok(k)\n}\nfn main() -> Io<()> {\n  print(1)\n}\n";
        let m = arita_syntax::parse(src).expect("parse");
        let f = m.functions.iter().find(|f| f.name == "f").expect("f");
        // ADR-286 S1b: `emit_function` returns `Result`; the failure is an exact E0006.
        let err = emit_function(f).expect_err("non-Vec/Map target must fail the fn emit");
        assert!(
            err.starts_with("E0006: cannot emit index assignment `x[..] = ..` in fn `f`"),
            "got: {err}"
        );
    }

    #[test]
    fn adr283_map_assign_in_if_still_map_insert_and_imports_hashmap() {
        let rs = emit_src(
            "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  let c: Bool = true\n  if c {\n    m[\"a\"] = 3\n  }\n  print(m.len())\n}\n",
        );
        assert!(rs.contains("use std::collections::HashMap;"), "{rs}");
        assert!(rs.contains("m.insert(__arita_mk, __arita_mv);"), "{rs}");
        assert!(!rs.contains("__arita_vec_set"), "{rs}");
    }

    #[test]
    fn adr283_map_and_vec_assign_in_same_fn() {
        let rs = emit_src(
            "module t\nfn f() -> Result<Int, Int> {\n  let mut m: Map<Text, Int> = Map::new()\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  m[\"k\"] = 4\n  v[0] = m.len()\n  let n: Int = v.len()\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f() {\n    Ok(x) => { print(x) }\n    Err(e) => { print(e) }\n  }\n}\n",
        );
        assert!(rs.contains("use std::collections::HashMap;"), "{rs}");
        assert_eq!(rs.matches("fn __arita_vec_set<T>").count(), 1, "{rs}");
        assert!(rs.contains("m.insert(__arita_mk, __arita_mv);"), "{rs}");
        assert!(
            rs.contains("__arita_vec_set(&mut v, __arita_vi, __arita_vx)?;"),
            "{rs}"
        );
    }

    fn marker_stmt() -> Stmt {
        Stmt::Expr(Expr::Call(Call {
            callee: arita_syntax::DEFERRED_SHAPE_MARKER.to_string(),
            args: vec![
                Expr::LitStr("E0001".into()),
                Expr::LitStr("deferred shape".into()),
                Expr::LitInt(0),
                Expr::LitInt(1),
            ],
        }))
    }

    #[test]
    fn adr283_deferred_shape_marker_is_codegen_error_not_panic() {
        // (a) Real parser path: main without print + `v[0] = 2` → parser defers E0001 as marker.
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n}\n";
        let parsed = arita_syntax::parse(src).expect("parser defers shape error as marker");
        let has_marker = parsed.functions.iter().any(|f| {
            f.body.iter().any(|s| {
                matches!(s, Stmt::Expr(Expr::Call(c)) if c.callee == arita_syntax::DEFERRED_SHAPE_MARKER)
            })
        });
        assert!(
            has_marker,
            "parser must have prepended the marker: {parsed:?}"
        );
        let r = std::panic::catch_unwind(|| emit_rust_with_entry(&parsed, "main"));
        let Ok(res) = r else {
            unreachable!("codegen panicked on deferred-shape marker");
        };
        let err = res.expect_err("marker must be a codegen error");
        assert!(err.starts_with("E0001"), "{err}");
        assert!(!err.contains("<arita:deferred-shape>"), "{err}");

        // (b) Synthetic marker nested inside an `if` of a Result fn, all public entry points.
        let mut m = arita_syntax::parse(
            "module t\nfn g() -> Result<Int, Int> {\n  let c: Bool = true\n  if c {\n    print(1)\n  }\n  Ok(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n",
        )
        .expect("parse");
        let g = m.functions.iter_mut().find(|f| f.name == "g").expect("g");
        if let Some(Stmt::If { then_body, .. }) = g.body.get_mut(1) {
            then_body.insert(0, marker_stmt());
        } else {
            unreachable!("expected if at g.body[1]");
        }
        let entry = std::panic::catch_unwind(|| emit_rust_with_entry(&m, "main"));
        assert!(
            matches!(entry, Ok(Err(ref e)) if e.starts_with("E0001")),
            "{entry:?}"
        );
        let prog = std::panic::catch_unwind(|| emit_rust_program(&m, &[], "main"));
        assert!(
            matches!(prog, Ok(Err(ref e)) if e.starts_with("E0001")),
            "{prog:?}"
        );
        let mut lib = m.clone();
        lib.functions.retain(|f| f.name != "main");
        let libr = std::panic::catch_unwind(|| emit_rust_lib(&lib));
        assert!(
            matches!(libr, Ok(Err(ref e)) if e.starts_with("E0001")),
            "{libr:?}"
        );
    }

    fn run_clippy(src: &std::path::Path, bin: &std::path::Path, edition: &str) -> (bool, String) {
        let c = std::process::Command::new("clippy-driver")
            .args([
                "--edition",
                edition,
                "-D",
                "warnings",
                "-D",
                "unused_imports",
            ])
            .args(["-D", "redundant_imports", "-D", "clippy::all"])
            .arg(src)
            .arg("-o")
            .arg(bin)
            .output()
            .expect("clippy-driver must be available (real oracle, no skip)");
        (
            c.status.success(),
            String::from_utf8_lossy(&c.stderr).to_string(),
        )
    }

    /// Ingeniero FASE 2 cond. 1: the in-helper `use std::convert::TryFrom as _;` must be
    /// clippy-clean (`-D warnings`, `unused_imports`, `redundant_imports`) on 2015 AND 2021.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "spawns rustc/clippy-driver (real oracle); Miri cannot posix_spawn"
    )]
    fn adr283_tryfrom_import_clippy_2015_2021() {
        let rs = emit_src(
            "module t\nfn f() -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  let a: Result<(), Int> = v.insert(0, 5)\n  a?\n  v[1] = 9\n  let b: Result<(), Int> = v.set(0, 3)\n  b?\n  let n: Int = v.len()\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f() {\n    Ok(x) => { print(x) }\n    Err(e) => { print(e) }\n  }\n}\n",
        );
        assert_eq!(
            rs.matches("use std::convert::TryFrom as _;").count(),
            2,
            "{rs}"
        );
        assert_eq!(rs.matches("usize::try_from(i)").count(), 2, "{rs}");
        let dir = std::env::temp_dir().join(format!("arita283_clippy_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let src = dir.join("prog.rs");
        std::fs::write(&src, &rs).expect("write prog.rs");
        for edition in ["2015", "2021"] {
            let bin = dir.join(format!("prog{edition}"));
            let (ok, stderr) = run_clippy(&src, &bin, edition);
            assert!(ok, "clippy edition {edition} failed:\n{stderr}\n{rs}");
            let run = std::process::Command::new(&bin).output().expect("run prog");
            assert!(run.status.success());
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                "2\n",
                "edition {edition}"
            );
        }
        // Control: the previous plain `use std::convert::TryFrom;` IS flagged on 2021 by
        // `redundant_imports` — proves the oracle is sensitive to the lint it guards.
        let plain = rs.replace(
            "use std::convert::TryFrom as _;",
            "use std::convert::TryFrom;",
        );
        let src2 = dir.join("plain.rs");
        std::fs::write(&src2, &plain).expect("write plain.rs");
        let (ok21, stderr21) = run_clippy(&src2, &dir.join("plain2021"), "2021");
        assert!(!ok21, "control must fail on 2021");
        assert!(stderr21.contains("imported redundantly"), "{stderr21}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod b282_emit_clippy {
    //! ADR-283 FASE 2 — backlog ADR-282 fixed in place, each with a real emit-clippy oracle:
    //! B-282-1 `m.get("lit")` (no `&"lit".to_string()`, clippy `unnecessary_to_owned`) and
    //! B-282-2 `let n: Int = m.len()` (no `(m.len() as i64)`, rustc `unused_parens`).
    use super::*;

    fn clippy_clean(rs: &str, tag: &str) -> String {
        let dir = std::env::temp_dir().join(format!("arita_b282_{tag}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let src = dir.join("prog.rs");
        std::fs::write(&src, rs).expect("write");
        let mut stdout = String::new();
        for edition in ["2015", "2021"] {
            let bin = dir.join(format!("prog{edition}"));
            let c = std::process::Command::new("clippy-driver")
                .args(["--edition", edition, "-D", "warnings", "-D", "clippy::all"])
                .arg(&src)
                .arg("-o")
                .arg(&bin)
                .output()
                .expect("clippy-driver must be available (real oracle, no skip)");
            assert!(
                c.status.success(),
                "clippy {edition} failed:\n{}\n{rs}",
                String::from_utf8_lossy(&c.stderr)
            );
            let run = std::process::Command::new(&bin).output().expect("run");
            assert!(run.status.success());
            stdout = String::from_utf8_lossy(&run.stdout).to_string();
        }
        let _ = std::fs::remove_dir_all(&dir);
        stdout
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "spawns rustc/clippy-driver (real oracle); Miri cannot posix_spawn"
    )]
    fn b282_1_map_get_string_literal_key_is_clippy_clean() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 7\n  let o: Option<Int> = m.get(\"a\")\n  match o {\n    Some(x) => { print(x) }\n    None => { print(0) }\n  }\n}\n";
        let m = arita_syntax::parse(src).expect("parse");
        let rs = emit_rust_with_entry(&m, "main").expect("emit");
        assert!(rs.contains("m.get(\"a\").cloned()"), "{rs}");
        assert!(!rs.contains(".get(&\"a\".to_string())"), "{rs}");
        assert_eq!(clippy_clean(&rs, "get"), "7\n");
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "spawns rustc/clippy-driver (real oracle); Miri cannot posix_spawn"
    )]
    fn b282_2_let_int_len_has_no_outer_parens_and_is_clippy_clean() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 7\n  let mut v: Vec<Int> = Vec::new()\n  let n: Int = m.len()\n  v.push(n)\n  let k: Int = v.len()\n  print(n)\n  print(k)\n}\n";
        let m = arita_syntax::parse(src).expect("parse");
        let rs = emit_rust_with_entry(&m, "main").expect("emit");
        assert!(rs.contains("    let n: i64 = m.len() as i64;\n"), "{rs}");
        assert!(rs.contains("    let k: i64 = v.len() as i64;\n"), "{rs}");
        assert_eq!(clippy_clean(&rs, "len"), "1\n1\n");
    }
}

#[cfg(test)]
mod adr289_unused_parens {
    //! ADR-289 (CORE-EMIT-UNUSED-PARENS) — no redundant outer parens at `Bare`/`Head` positions;
    //! `Operand` positions, macro arguments and the 12 precedence sites keep today's bytes.
    use super::*;
    use arita_syntax::FnParam;

    fn p(n: &str) -> Expr {
        Expr::Path(Path {
            segments: vec![n.into()],
        })
    }
    fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
        Expr::Binary {
            op,
            lhs: Box::new(l),
            rhs: Box::new(r),
        }
    }
    fn let_int(name: &str, init: Expr) -> Stmt {
        Stmt::Let {
            mutable: false,
            name: name.into(),
            ty: Type::Int,
            init,
        }
    }
    fn func(name: &str, ret: Type, body: Vec<Stmt>) -> Function {
        Function {
            name: name.into(),
            params: if name == "main" {
                vec![]
            } else {
                vec![
                    FnParam {
                        name: "a".into(),
                        ty: Type::Int,
                    },
                    FnParam {
                        name: "b".into(),
                        ty: Type::Int,
                    },
                ]
            },
            ret_ty: ret,
            is_async: false,
            is_pub: false,
            body,
        }
    }
    fn emit(fns: Vec<Function>) -> String {
        let m = Module {
            name: "t".into(),
            functions: fns,
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        emit_rust(&m).expect("emit_rust")
    }
    fn main_with(body: Vec<Stmt>) -> String {
        emit(vec![func("main", Type::IoUnit, body)])
    }

    #[test]
    fn au1_let_assign_if_while_are_bare() {
        let rs = main_with(vec![
            let_int("s", bin(BinOp::Add, p("a"), p("b"))),
            Stmt::If {
                cond: bin(BinOp::Lt, p("a"), p("b")),
                then_body: vec![Stmt::Assign {
                    name: "s".into(),
                    value: bin(BinOp::Sub, p("s"), Expr::LitInt(1)),
                }],
                else_body: None,
            },
            Stmt::While {
                cond: bin(BinOp::Gt, p("s"), Expr::LitInt(0)),
                body: vec![Stmt::Assign {
                    name: "s".into(),
                    value: bin(BinOp::Sub, p("s"), Expr::LitInt(1)),
                }],
            },
        ]);
        assert!(rs.contains("let s: i64 = a + b;"), "{rs}");
        assert!(rs.contains("if a < b {"), "{rs}");
        assert!(rs.contains("s = s - 1;"), "{rs}");
        assert!(rs.contains("while s > 0 {"), "{rs}");
        assert!(!rs.contains("= ("), "{rs}");
    }

    #[test]
    fn au2_fn_tail_ok_err_some_are_bare() {
        let rs = emit(vec![
            func(
                "t1",
                Type::Int,
                vec![Stmt::Expr(bin(BinOp::Mul, p("a"), p("b")))],
            ),
            func(
                "t2",
                Type::Result(Box::new(Type::Int), Box::new(Type::Int)),
                vec![Stmt::Expr(Expr::Ok(Box::new(bin(
                    BinOp::Add,
                    p("a"),
                    Expr::LitInt(1),
                ))))],
            ),
            func(
                "t3",
                Type::Result(Box::new(Type::Int), Box::new(Type::Int)),
                vec![Stmt::Expr(Expr::Err(Box::new(bin(
                    BinOp::Sub,
                    p("a"),
                    p("b"),
                ))))],
            ),
            func(
                "t4",
                Type::Option(Box::new(Type::Int)),
                vec![Stmt::Expr(Expr::Some(Box::new(bin(
                    BinOp::Add,
                    p("b"),
                    Expr::LitInt(2),
                ))))],
            ),
            func("main", Type::IoUnit, vec![]),
        ]);
        assert!(rs.contains("\n    a * b\n}"), "{rs}");
        assert!(rs.contains("Ok(a + 1)"), "{rs}");
        assert!(rs.contains("Err(a - b)"), "{rs}");
        assert!(rs.contains("Some(b + 2)"), "{rs}");
        assert!(
            !rs.contains("((") && !rs.contains("Ok((") && !rs.contains("Some(("),
            "{rs}"
        );
    }

    #[test]
    fn au3_user_call_args_and_borrow_are_bare() {
        let rs = main_with(vec![
            let_int(
                "r",
                Expr::Call(Call {
                    callee: "add".into(),
                    args: vec![
                        bin(BinOp::Add, p("a"), Expr::LitInt(1)),
                        bin(BinOp::Mul, p("b"), Expr::LitInt(2)),
                    ],
                }),
            ),
            let_int(
                "x",
                Expr::Borrow {
                    mutable: false,
                    inner: Box::new(p("r")),
                },
            ),
            let_int(
                "y",
                Expr::Borrow {
                    mutable: true,
                    inner: Box::new(p("r")),
                },
            ),
        ]);
        assert!(rs.contains("let r: i64 = add(a + 1, b * 2);"), "{rs}");
        assert!(rs.contains("let x: i64 = *&r;"), "{rs}");
        assert!(rs.contains("let y: i64 = *&mut r;"), "{rs}");
    }

    #[test]
    fn au4_match_scrutinee_and_arms_are_bare() {
        let rs = main_with(vec![
            let_int(
                "x",
                Expr::Match {
                    scrutinee: Box::new(bin(BinOp::Add, p("a"), Expr::LitInt(1))),
                    arms: vec![
                        arita_syntax::MatchExprArm {
                            pat: Pat::LitInt(0),
                            value: bin(BinOp::Add, p("a"), Expr::LitInt(2)),
                        },
                        arita_syntax::MatchExprArm {
                            pat: Pat::Wildcard,
                            value: bin(BinOp::Sub, p("a"), Expr::LitInt(3)),
                        },
                    ],
                },
            ),
            Stmt::Match {
                scrutinee: bin(BinOp::Mul, p("a"), p("b")),
                arms: vec![arita_syntax::MatchArm {
                    pat: Pat::Wildcard,
                    body: vec![Stmt::Break],
                }],
            },
        ]);
        assert!(rs.contains("match a + 1 {"), "{rs}");
        assert!(rs.contains("0 => a + 2,"), "{rs}");
        assert!(rs.contains("_ => a - 3,"), "{rs}");
        assert!(rs.contains("match a * b {"), "{rs}");
    }

    #[test]
    fn au5_operand_children_keep_parens() {
        let rs = main_with(vec![let_int(
            "x",
            bin(BinOp::Mul, bin(BinOp::Add, p("a"), Expr::LitInt(1)), p("b")),
        )]);
        assert!(rs.contains("let x: i64 = (a + 1) * b;"), "{rs}");
    }

    #[test]
    fn au6_method_arg_and_temp_are_bare_but_amp_and_precedence_sites_are_not() {
        let rs = main_with(vec![
            Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(p("x")),
                method: "checked_add".into(),
                args: vec![bin(BinOp::Add, p("a"), Expr::LitInt(1))],
            }),
            Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(p("x")),
                method: "is_multiple_of".into(),
                args: vec![bin(BinOp::Add, p("a"), Expr::LitInt(2))],
            }),
            // untouched precedence site: `(({n})).max(0) as usize`
            Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(p("v")),
                method: "resize".into(),
                args: vec![bin(BinOp::Add, p("a"), Expr::LitInt(3)), Expr::LitInt(0)],
            }),
            // untouched `&{a}` site
            Stmt::Expr(Expr::MethodCall {
                receiver: Box::new(p("s")),
                method: "push_str".into(),
                args: vec![bin(BinOp::Add, p("a"), Expr::LitInt(4))],
            }),
        ]);
        assert!(rs.contains("x.checked_add(a + 1)"), "{rs}");
        assert!(rs.contains("let __d = a + 2;"), "{rs}");
        assert!(
            rs.contains("v.resize(((a + 3)).max(0) as usize, 0)"),
            "{rs}"
        );
        assert!(rs.contains("s.push_str(&(a + 4))"), "{rs}");
    }

    #[test]
    fn au7_macro_args_are_untouched() {
        let rs = main_with(vec![
            let_int("x", Expr::LitInt(1)),
            Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![bin(BinOp::Add, p("x"), Expr::LitInt(1))],
            })),
        ]);
        assert!(rs.contains("println!(\"{}\", (x + 1));"), "{rs}");
    }

    #[test]
    fn au8_head_keeps_parens_around_exterior_struct_literal() {
        let rec = |v: i64| Expr::RecordLit {
            name: "P".into(),
            fields: vec![("x".into(), Expr::LitInt(v))],
        };
        let e = bin(BinOp::Eq, rec(1), rec(2));
        let bare = emit_expr_ctx(&e, ExprCtx::Bare).unwrap();
        let head = emit_expr_ctx(&e, ExprCtx::Head).unwrap();
        assert_eq!(bare, "P { x: 1 } == P { x: 2 }");
        assert_eq!(head, "(P { x: 1 } == P { x: 2 })");
        let e2 = bin(BinOp::Lt, p("a"), p("b"));
        assert_eq!(emit_expr_ctx(&e2, ExprCtx::Head).unwrap(), "a < b");
        assert_eq!(emit_expr_ctx(&e2, ExprCtx::Operand).unwrap(), "(a < b)");
    }

    #[test]
    fn au9_strip_redundant_outer_parens_guards() {
        assert_eq!(strip_redundant_outer_parens("()"), "()");
        assert_eq!(strip_redundant_outer_parens("(a, b)"), "(a, b)");
        assert_eq!(strip_redundant_outer_parens("(x + 1)"), "x + 1");
        assert_eq!(strip_redundant_outer_parens("(f(a, b))"), "f(a, b)");
        assert_eq!(strip_redundant_outer_parens("(a) + (b)"), "(a) + (b)");
        assert_eq!(strip_redundant_outer_parens("x"), "x");
        // `Ok(())` must survive the Bare path
        let rs = emit(vec![
            func(
                "t",
                Type::Result(Box::new(Type::Unit), Box::new(Type::Int)),
                vec![Stmt::Expr(Expr::Ok(Box::new(Expr::LitUnit)))],
            ),
            func("main", Type::IoUnit, vec![]),
        ]);
        assert!(rs.contains("Ok(())"), "{rs}");
    }

    #[test]
    fn au10_cast_in_bare_positions_and_operand_children() {
        // len is a cast `(v.len() as i64)`: bare in let (ADR-288) and in a call argument,
        // kept as operand of a comparison.
        let src = "module t\nfn id(x: Int) -> Int {\n  x\n}\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  let k: Int = id(v.len())\n  print(k)\n}\n";
        let m = arita_syntax::parse(src).expect("parses");
        let rs = emit_rust_with_entry(&m, "main").expect("emit");
        assert!(rs.contains("let k: i64 = id(v.len() as i64);"), "{rs}");
        assert!(
            !rs.contains("((v.len() as i64)") && !rs.contains("= (v.len()"),
            "{rs}"
        );
        let lt = bin(
            BinOp::Lt,
            Expr::MethodCall {
                receiver: Box::new(p("v")),
                method: "len".into(),
                args: vec![],
            },
            Expr::LitInt(3),
        );
        let s = emit_expr_ctx(&lt, ExprCtx::Head).unwrap();
        assert_eq!(s, "(v.len() as i64) < 3");
    }
}

#[cfg(test)]
mod adr290_index_mut {
    //! ADR-290 slice A (CORE-0.10-INDEX-MUT-20261002) — compound index-assign `v[i] op= x`
    //! (op in `+=`, `-=`, `*=`) on a `Vec<Int>` local: `__arita_vec_update` + `i64::checked_*`,
    //! R2 temps before the `&mut`, single `Err(0)` for OOB / negative / overflow, no Rust
    //! `IndexMut` / `v[i] = v[i] op x` / `+=`, plain `=` (ADR-283) and Map (ADR-282) unchanged.
    //! The Parser lowers `v[i] op= x` to `Stmt::IndexAssign { op: Some(BinOp::{Add,Sub,Mul}), .. }`;
    //! tests that need an operator the grammar cannot produce set `op` on the AST directly.
    use super::*;

    fn emit_src(src: &str) -> String {
        let m = arita_syntax::parse(src).expect("ADR-290 source must parse");
        emit_rust_with_entry(&m, "main").expect("ADR-290 emit must succeed")
    }

    const OPS_SRC: &str = "module t\n\
fn f(n: Int) -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v.push(2)\n\
  v.push(3)\n\
  v[0] += n\n\
  v[1] -= 4\n\
  v[2] *= v.len()\n\
  Ok(n)\n\
}\n\
fn main() -> Io<()> {\n\
  match f(7) {\n\
    Ok(x) => { print(x) }\n\
    Err(e) => { print(e) }\n\
  }\n\
}\n";

    /// Set `op` on every top-level `IndexAssign` of fn `fname` (AST-level, Parser-independent).
    fn set_op(m: &mut Module, fname: &str, op: Option<BinOp>) {
        let f = m
            .functions
            .iter_mut()
            .find(|f| f.name == fname)
            .expect("fn present");
        for s in &mut f.body {
            if let Stmt::IndexAssign { op: o, .. } = s {
                *o = op.clone();
            }
        }
    }

    const PLAIN_SRC: &str = "module t\n\
fn f(n: Int) -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v[0] = n\n\
  Ok(n)\n\
}\n\
fn main() -> Io<()> {\n\
  print(1)\n\
}\n";

    #[test]
    fn adr290_compound_emits_update_helper_with_temps_in_order() {
        let rs = emit_src(OPS_SRC);
        // i → x into temps BEFORE `&mut v`; `i64::checked_<op>` per operator; trailing `?`.
        for site in [
            "    { let __arita_vi = 0; let __arita_vx = n; __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_add)?; }\n",
            "    { let __arita_vi = 1; let __arita_vx = 4; __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_sub)?; }\n",
            // rhs reads `v` (v.len()): evaluated into the temp before the mutable borrow (R2);
            // ADR-289: the outer parens of the cast are stripped.
            "    { let __arita_vi = 2; let __arita_vx = v.len() as i64; __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_mul)?; }\n",
        ] {
            assert!(rs.contains(site), "missing site:\n{site}\nin:\n{rs}");
        }
        // Source order is preserved: add, then sub, then mul.
        let a = rs.find("i64::checked_add").expect("add");
        let b = rs.find("i64::checked_sub").expect("sub");
        let c = rs.find("i64::checked_mul").expect("mul");
        assert!(a < b && b < c, "{rs}");
        // Helper exactly once; the plain-set helper is NOT dragged in by a compound-only module.
        assert_eq!(rs.matches("fn __arita_vec_update(").count(), 1, "{rs}");
        assert!(!rs.contains("fn __arita_vec_set"), "{rs}");
        assert!(!rs.contains("use std::collections::HashMap"), "{rs}");
        for l in rs.lines().filter(|l| l.contains("__arita_vi")) {
            assert!(!l.contains("as usize"), "R1 in lowering: {l}");
        }
    }

    #[test]
    fn adr290_helper_const_shape() {
        let h = ARITA_VEC_UPDATE_HELPER;
        assert!(!h.contains("as usize"), "R1:\n{h}");
        assert!(h.contains("usize::try_from(i)"), "{h}");
        assert!(h.contains("use std::convert::TryFrom as _;"), "{h}");
        assert!(
            h.contains("op: fn(i64, i64) -> Option<i64>") && h.contains("op(*slot, x)"),
            "{h}"
        );
        // Three early `Err(0)` exits (try_from, get_mut, checked op) and a write only on success.
        assert_eq!(h.matches("return Err(0);").count(), 3, "{h}");
        assert!(h.contains("*slot = new;"), "{h}");
        // No arithmetic operator on the slot (identical in debug and release), no panic channel,
        // no silent wrap/saturate.
        for banned in [
            "*slot +",
            "*slot -",
            "*slot *",
            "slot +",
            "slot -",
            "+=",
            "-=",
            "*= ",
            ".unwrap()",
            ".expect(",
            "wrapping_",
            "saturating_",
        ] {
            assert!(!h.contains(banned), "banned `{banned}` in:\n{h}");
        }
        assert!(!h.contains(concat!("panic!", "(")), "{h}");
    }

    #[test]
    fn adr290_rhs_is_evaluated_once_before_the_borrow() {
        // A user call and a `?` call in the rhs: each appears exactly once, in the `let __arita_vx`
        // temp, before the `&mut v` of the helper call (D4: index → rhs → read → op → write).
        let src = "module t\n\
fn nxt(k: Int) -> Int {\n\
  k\n\
}\n\
fn g() -> Result<Int, Int> {\n\
  Ok(2)\n\
}\n\
fn f() -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v[0] += nxt(3)\n\
  v[0] -= g()?\n\
  Ok(1)\n\
}\n\
fn main() -> Io<()> {\n\
  print(1)\n\
}\n";
        let rs = emit_src(src);
        assert_eq!(rs.matches("nxt(3)").count(), 1, "{rs}");
        assert_eq!(rs.matches("g()?").count(), 1, "{rs}");
        for l in rs.lines().filter(|l| l.contains("__arita_vec_update(&mut")) {
            let vx = l.find("let __arita_vx = ").expect("vx temp");
            let call = l.find("__arita_vec_update(&mut").expect("call");
            let vi = l.find("let __arita_vi = ").expect("vi temp");
            assert!(
                vi < vx && vx < call,
                "temps must precede the &mut call: {l}"
            );
        }
        // The `?` of the rhs sits inside the temp (it returns BEFORE `v` is touched), distinct
        // from the helper's own trailing `?`.
        assert!(
            rs.contains("let __arita_vx = g()?; __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_sub)?;"),
            "{rs}"
        );
    }

    #[test]
    fn adr290_emit_has_no_read_then_write_or_panic_channels() {
        let rs = emit_src(OPS_SRC);
        for l in rs.lines() {
            let t = l.trim_start();
            assert!(
                !(t.starts_with("v[")
                    && (t.contains("] =")
                        || t.contains("] +=")
                        || t.contains("] -=")
                        || t.contains("] *="))),
                "Rust indexed assign / AddAssign: {l}"
            );
        }
        for banned in [
            "IndexMut",
            "std::ops::Index",
            "] += ",
            "] -= ",
            "] *= ",
            "= v[",
            ".unwrap()",
            ".expect(",
            "panic!",
            "let _ = __arita_vec_update",
            ".ok()",
            "wrapping_",
            "saturating_",
            "arita:deferred-shape",
        ] {
            assert!(!rs.contains(banned), "banned `{banned}` in:\n{rs}");
        }
    }

    #[test]
    fn adr290_helper_injected_in_lib_and_bin_once() {
        let lib_src = "module t\n\
pub fn bump(i: Int, x: Int) -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v[i] += x\n\
  Ok(1)\n\
}\n";
        let lib = arita_syntax::parse(lib_src).expect("lib source must parse");
        let rs = emit_rust_lib(&lib).expect("lib emit");
        assert_eq!(rs.matches("fn __arita_vec_update(").count(), 1, "{rs}");
        assert!(
            rs.contains("__arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_add)?;"),
            "{rs}"
        );
        // Bin side: same helper (OPS_SRC main path).
        assert_eq!(
            emit_src(OPS_SRC).matches("fn __arita_vec_update(").count(),
            1
        );
    }

    #[test]
    fn adr290_plain_assign_and_set_unchanged() {
        // No compound ⇒ byte-identical ADR-283 emit and NO update helper in the prelude.
        let rs = emit_src(PLAIN_SRC);
        assert!(
            rs.contains("    { let __arita_vi = 0; let __arita_vx = n; __arita_vec_set(&mut v, __arita_vi, __arita_vx)?; }\n"),
            "{rs}"
        );
        assert_eq!(rs.matches("fn __arita_vec_set<T>").count(), 1, "{rs}");
        assert!(!rs.contains("__arita_vec_update"), "{rs}");
        // Both in one module ⇒ each helper exactly once.
        let both = "module t\n\
fn f(n: Int) -> Result<Int, Int> {\n\
  let mut v: Vec<Int> = Vec::new()\n\
  v.push(1)\n\
  v[0] = n\n\
  v[0] += n\n\
  Ok(n)\n\
}\n\
fn main() -> Io<()> {\n\
  print(1)\n\
}\n";
        let rs2 = emit_src(both);
        assert_eq!(rs2.matches("fn __arita_vec_set<T>").count(), 1, "{rs2}");
        assert_eq!(rs2.matches("fn __arita_vec_update(").count(), 1, "{rs2}");
        // Map assign (ADR-282) untouched.
        let map = emit_src(
            "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 7\n  print(m.len())\n}\n",
        );
        assert!(
            map.contains("{ let __arita_mk = \"a\".to_string(); let __arita_mv = 7; m.insert(__arita_mk, __arita_mv); }"),
            "{map}"
        );
        assert!(!map.contains("__arita_vec_update"), "{map}");
    }

    #[test]
    fn adr290_walkers_distinguish_plain_and_compound() {
        let mut plain = arita_syntax::parse(PLAIN_SRC).expect("parse");
        assert!(module_uses_vec_set(&plain) && !module_uses_vec_update(&plain));
        set_op(&mut plain, "f", Some(BinOp::Add));
        assert!(!module_uses_vec_set(&plain), "compound is not a plain set");
        assert!(module_uses_vec_update(&plain));
        set_op(&mut plain, "f", None);
        assert!(module_uses_vec_set(&plain) && !module_uses_vec_update(&plain));
    }

    #[test]
    fn adr290_unsupported_compound_is_never_silently_dropped() {
        // (a) operator outside {+,-,*} (defensive: the Parser rejects it with E0006) → emit error.
        for op in [BinOp::Div, BinOp::Rem, BinOp::Eq, BinOp::Lt] {
            let mut m = arita_syntax::parse(PLAIN_SRC).expect("parse");
            set_op(&mut m, "f", Some(op));
            let f = m.functions.iter().find(|f| f.name == "f").expect("f");
            let err = emit_function(f).expect_err("unsupported operator must fail the fn emit");
            assert!(
                err.starts_with("E0006: cannot emit index assignment `x[..] = ..` in fn `f`"),
                "got: {err}"
            );
        }
        // (b) element type ≠ Int (HIR gives E0333; codegen refuses defensively). The grammar only
        // offers `Vec<Int>`, so retype the already-parsed `let` at AST level.
        let mut m = arita_syntax::parse(PLAIN_SRC).expect("parse");
        let f = m
            .functions
            .iter_mut()
            .find(|f| f.name == "f")
            .expect("f present");
        for s in &mut f.body {
            if let Stmt::Let { ty, .. } = s {
                *ty = Type::Vec(Box::new(Type::String));
            }
        }
        set_op(&mut m, "f", Some(BinOp::Add));
        let f = m.functions.iter().find(|f| f.name == "f").expect("f");
        assert!(
            emit_function(f).is_err(),
            "Vec<Text> compound must not emit"
        );
        // (c) Map target (HIR gives E0006): must not degrade to `insert`.
        let map = "module t\nfn f() -> Result<Int, Int> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n  Ok(1)\n}\nfn main() -> Io<()> {\n  print(1)\n}\n";
        let mut m = arita_syntax::parse(map).expect("parse");
        set_op(&mut m, "f", Some(BinOp::Add));
        let f = m.functions.iter().find(|f| f.name == "f").expect("f");
        assert!(emit_function(f).is_err(), "Map compound must not emit");
    }

    fn run_clippy(src: &std::path::Path, bin: &std::path::Path, edition: &str) -> (bool, String) {
        let c = std::process::Command::new("clippy-driver")
            .args(["--edition", edition, "-D", "warnings"])
            .args(["-D", "unused_imports", "-D", "redundant_imports"])
            .args([
                "-D",
                "unused_parens",
                "-D",
                "unused_braces",
                "-D",
                "clippy::all",
            ])
            .arg(src)
            .arg("-o")
            .arg(bin)
            .output()
            .expect("clippy-driver must be available (real oracle, no skip)");
        (
            c.status.success(),
            String::from_utf8_lossy(&c.stderr).to_string(),
        )
    }

    /// Runtime semantics of the emitted program (overflow / OOB / negative ⇒ `Err(0)`, success),
    /// emit clippy-clean on 2015 + 2021, identical stdout with and without optimisation.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "spawns rustc/clippy-driver (real oracle); Miri cannot posix_spawn"
    )]
    fn adr290_emit_clippy_and_runtime_2015_2021() {
        let src = r#"module t
fn addm(x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(9223372036854775807)
  v[0] += x
  let o: Option<Int> = v.get(0)
  match o {
    Some(y) => { Ok(y) }
    None => { Err(1) }
  }
}
fn subm(x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(-9223372036854775807)
  v[0] -= x
  let o: Option<Int> = v.get(0)
  match o {
    Some(y) => { Ok(y) }
    None => { Err(1) }
  }
}
fn mulm(x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(9223372036854775807)
  v[0] *= x
  let o: Option<Int> = v.get(0)
  match o {
    Some(y) => { Ok(y) }
    None => { Err(1) }
  }
}
fn bump(i: Int, x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  v.push(3)
  v[i] += x
  let n: Int = v.len()
  Ok(n)
}
fn main() -> Io<()> {
  match addm(0) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match addm(1) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match subm(1) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match subm(2) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match mulm(1) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match mulm(2) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match mulm(-1) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match bump(1, 5) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match bump(3, 5) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
  match bump(-1, 5) {
    Ok(a) => { print(a) }
    Err(e) => { print(e) }
  }
}
"#;
        let expected = "9223372036854775807\n0\n-9223372036854775808\n0\n9223372036854775807\n0\n-9223372036854775807\n3\n0\n0\n";
        let rs = emit_src(src);
        assert_eq!(rs.matches("fn __arita_vec_update(").count(), 1, "{rs}");
        let dir = std::env::temp_dir().join(format!("arita290_clippy_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let file = dir.join("prog.rs");
        std::fs::write(&file, &rs).expect("write prog.rs");
        for edition in ["2015", "2021"] {
            let bin = dir.join(format!("prog{edition}"));
            let (ok, stderr) = run_clippy(&file, &bin, edition);
            assert!(ok, "clippy edition {edition} failed:\n{stderr}\n{rs}");
            let run = std::process::Command::new(&bin).output().expect("run prog");
            assert!(run.status.success(), "edition {edition}: {run:?}");
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                expected,
                "edition {edition}"
            );
            // Optimised build (release-like: overflow-checks off) must behave identically.
            let opt = dir.join(format!("prog{edition}_opt"));
            let c = std::process::Command::new("rustc")
                .args(["--edition", edition, "-O"])
                .arg(&file)
                .arg("-o")
                .arg(&opt)
                .output()
                .expect("rustc");
            assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
            let run2 = std::process::Command::new(&opt).output().expect("run opt");
            assert!(run2.status.success());
            assert_eq!(
                String::from_utf8_lossy(&run2.stdout),
                expected,
                "opt {edition}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
