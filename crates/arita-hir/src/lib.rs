//! ARITA HIR + ownership/borrow + F2.1–F2.3 + match-as-expr + async v0 (ADR-025 / ADR-027).
//!
//! Pipeline: AST → [`lower_ast`] → [`check`] → emit from AST if Ok.
//! Diagnostics: **E0201** use of moved value, **E0202** borrow conflict,
//! **E0203** type mismatch (match-expr arms / method arity),
//! **E0205** invalid let / mut binding (`&mut` on non-`mut`),
//! **E0318** borrow cannot escape (ADR-235 return/record field),
//! **E0206** method not in F2 std whitelist (ADR-026),
//! **E0220** if/while condition type ≠ Bool,
//! **E0221** non-exhaustive Bool match, **E0222** Int match without `_`,
//! **E0223** match pattern/scrutinee type mismatch (or scrutinee ≠ Bool|Int),
//! **E0224** break/continue outside while,
//! **E0225** vacuous match arms (ADR-031 / identical constant arm values),
//! **E0226** vacuous while false (ADR-040 / LitBool `false` cond only),
//! **E0227** vacuous if false (ADR-041 / LitBool `false` cond, no else),
//! **E0216** integer division by zero (ADR-044 / LitInt `0` divisor on `/` `%` or div-shaped call),
//! **E0217** integer overflow (ADR-045 / LitInt or same-scope lit-bound Path on `+` `-` `*`),
//! **E0240** await outside async function, **E0241** async feature not allowed here,
//! **E0242** borrow held across await,
//! **E0270** non-exhaustive Result match (ADR-047),
//! **E0273** non-exhaustive Option match (ADR-050),
//! **E0274** option none swallowed (ADR-051),
//! **E0275** if-let without else (ADR-053/054),
//! **E0277** vacuous while-let none (ADR-056 / HirExpr::None scrutinee),
//! **E0278** integer abs overflow (ADR-069 / lit or lit-bound Path == i64::MIN),
//! **E0279** invalid clamp range (ADR-072 / lit or lit-bound lo > hi),
//! **E0280** negative repeat count (ADR-075/076 String|Vec / lit or lit-bound n < 0),
//! **E0282** vacuous while-let result (ADR-078 / Ok-on-Err lit or Err-on-Ok lit),
//! **E0285** checked overflow defaulted away (ADR-101 / checked_* + unwrap_or lit),
//! **E0286** vacuous wrapping/saturating assert (ADR-102 / ±0 or ×1),
//! **E0287** vec get miss defaulted as success (ADR-103 / get + unwrap_or lit),
//! **E0289** find miss defaulted to index zero (ADR-125 / find|rfind + unwrap_or 0),
//! **E0290** ilog none defaulted to zero (ADR-126 / ilog2|ilog10 + unwrap_or 0),
//! **E0291** negative rotate count (ADR-131 / lit n < 0),
//! **E0292** negative char boundary index (ADR-143 / lit i < 0),
//! **E0293** negative shift count (ADR-150/151 / lit s < 0 on checked_|wrapping_shl/shr),
//! **E0294** negative reserve count (ADR-154 / lit n < 0),
//! **E0295** negative swap index (ADR-158 / lit i\|j < 0),
//! **E0296** negative split_off index (ADR-162 / lit at < 0),
//! **E0297** negative Vec rotate count (ADR-170 / lit n < 0),
//! **E0298** negative shrink_to capacity (ADR-208 / lit min_cap < 0),
//! **E0311** negative insert index (ADR-228/260 / lit i < 0),
//! **E0319** negative set index (ADR-270/265 / lit i < 0),
//! **E0288** negative truncate (ADR-118 Vec / ADR-160 String / lit or lit-bound n < 0),
//! **ADR-110** Int div_euclid/rem_euclid (E0216 zero / E0217 MIN÷−1),
//! **E0281** negative pow exponent (ADR-079 / lit or lit-bound exp < 0),
//! **E0283** integer pow overflow (ADR-079 / lit checked_pow None),
//! **E0272** result error swallowed (ADR-048),
//! **E0340** io result discarded (ADR-272 / host read_text|write_text discard or default-as-success).
//! **E0341** required arg miss as ok (ADR-273 / host cli_arg|json_get_int unwrap_or lit or argv Index).
//! **E0342** result return type required (ADR-277 / fn → Result theater: Ok/Err without Result ret, or Result ret with non-Result last value).
//! **E0343** question mark outside result fn (ADR-278 / `?` only inside `fn → Result`).

use arita_syntax::{
    BinOp, Call, Expr, FnParam, Function, MatchArm, Module, Pat, Path, Stmt, Test, Type,
};
use std::collections::HashMap;

/// HIR module: name, functions, and tests (mirrors F2 `Module`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirModule {
    pub name: String,
    pub functions: Vec<HirFunction>,
    pub tests: Vec<HirTest>,
    /// ADR-233 user record defs.
    pub records: Vec<HirRecordDef>,
    /// ADR-234 user enum defs (unit variants).
    pub enums: Vec<HirEnumDef>,
    /// ADR-293: fn names imported by `use module::item` (exempt from E0347).
    pub imports: Vec<String>,
}

/// ADR-233: lowered `record` definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirRecordDef {
    pub name: String,
    pub fields: Vec<HirRecordField>,
    /// ADR-256: pub record crosses crate boundary.
    pub is_pub: bool,
}

/// ADR-234: lowered `enum` definition (unit variants).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirEnumDef {
    pub name: String,
    pub variants: Vec<String>,
}

/// ADR-233: one record field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirRecordField {
    pub name: String,
    pub ty: HirType,
}

/// HIR function: name, params, return type, body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub ret_ty: HirType,
    pub body: Vec<HirStmt>,
    /// ADR-027 async fn.
    pub is_async: bool,
    /// ADR-256: pub fn crosses crate boundary.
    pub is_pub: bool,
}

/// HIR function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirParam {
    pub name: String,
    pub ty: HirType,
}

/// HIR type (F2 subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirType {
    Int,
    Bool,
    String,
    /// ADR-236 bytes (`Vec<u8>`).
    Bytes,
    /// ADR-243 opaque task handle.
    Task,
    /// ADR-047 unit `()`.
    Unit,
    Vec(Box<HirType>),
    /// ADR-237 map (`HashMap`).
    Map(Box<HirType>, Box<HirType>),
    /// ADR-047 `Result<T, E>`.
    Result(Box<HirType>, Box<HirType>),
    /// ADR-050 `Option<T>`.
    Option(Box<HirType>),
    /// ADR-233 user `record` type.
    Named(String),
    IoUnit,
}

/// HIR statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStmt {
    Let {
        mutable: bool,
        name: String,
        ty: HirType,
        init: HirExpr,
    },
    Expr(HirExpr),
    Assert {
        lhs: HirExpr,
        rhs: HirExpr,
    },
    If {
        cond: HirExpr,
        then_body: Vec<HirStmt>,
        else_body: Option<Vec<HirStmt>>,
    },
    IfLet {
        pat: HirPat,
        scrutinee: HirExpr,
        then_body: Vec<HirStmt>,
        else_body: Option<Vec<HirStmt>>,
    },
    /// ADR-055: `while let Some(x) = opt { … }`.
    WhileLet {
        pat: HirPat,
        scrutinee: HirExpr,
        body: Vec<HirStmt>,
    },
    While {
        cond: HirExpr,
        body: Vec<HirStmt>,
    },
    Assign {
        name: String,
        value: HirExpr,
    },
    /// ADR-282: `target[key] = value`; checked with the same helpers as `put` (Map only; else E0314).
    IndexAssign {
        target: String,
        key: HirExpr,
        value: HirExpr,
        /// ADR-290 slice A: `None` = plain `=`; `Some(Add|Sub|Mul)` = `+=` / `-=` / `*=`.
        op: Option<HirBinOp>,
        /// Syntax span of the statement (E0314 `@a..b`).
        span: arita_syntax::Span,
    },
    Match {
        scrutinee: HirExpr,
        arms: Vec<HirMatchArm>,
    },
    /// F2.3 unlabeled break.
    Break,
    /// F2.3 unlabeled continue.
    Continue,
}

/// HIR match arm (statement form).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirMatchArm {
    pub pat: HirPat,
    pub body: Vec<HirStmt>,
}

/// HIR match-as-expr arm (ADR-025).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirMatchExprArm {
    pub pat: HirPat,
    pub value: HirExpr,
}

/// HIR match pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPat {
    LitBool(bool),
    LitInt(i64),
    Wildcard,
    /// ADR-047 `Ok(binding)`.
    Ok(String),
    /// ADR-047 `Err(binding)`.
    Err(String),
    /// ADR-050 `Some(binding)`.
    Some(String),
    /// ADR-050 `None`.
    None,
    /// ADR-234 unit variant.
    Variant {
        enum_name: Option<String>,
        variant: String,
    },
}

/// HIR expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExpr {
    Call(HirCall),
    LitStr(String),
    LitInt(i64),
    LitBool(bool),
    Path(HirPath),
    Binary {
        op: HirBinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
    },
    Borrow {
        mutable: bool,
        inner: Box<HirExpr>,
    },
    MethodCall {
        receiver: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
    },
    /// ADR-025 match expression.
    Match {
        scrutinee: Box<HirExpr>,
        arms: Vec<HirMatchExprArm>,
    },
    /// ADR-027: await call.
    Await {
        inner: Box<HirExpr>,
    },
    /// ADR-278: postfix `?` (Result early-return; typed as Ok payload).
    Try {
        inner: Box<HirExpr>,
    },
    /// ADR-047 unit literal.
    LitUnit,
    /// ADR-047 `Ok(expr)`.
    Ok(Box<HirExpr>),
    /// ADR-047 `Err(expr)`.
    Err(Box<HirExpr>),
    /// ADR-050 `Some(expr)`.
    Some(Box<HirExpr>),
    /// ADR-050 `None`.
    None,
    /// ADR-233 `Name { fields }`.
    RecordLit {
        name: String,
        fields: Vec<(String, HirExpr)>,
    },
    /// ADR-233 shared field get.
    FieldAccess {
        base: Box<HirExpr>,
        field: String,
    },
    /// ADR-234 `Enum::Variant`.
    EnumPath {
        enum_name: String,
        variant: String,
    },
}

/// HIR call expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirCall {
    pub callee: String,
    pub args: Vec<HirExpr>,
}

/// HIR path (identifier segments).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPath {
    pub segments: Vec<String>,
}

/// HIR binary operators (Int arithmetic + Int comparisons → Bool).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// HIR test item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTest {
    pub name: String,
    pub body: Vec<HirStmt>,
}

/// Ownership/borrow check diagnostic (v0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    Coded { code: &'static str, message: String },
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coded { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

fn err(code: &'static str, message: &'static str) -> CheckError {
    CheckError::Coded {
        code,
        message: message.to_string(),
    }
}

fn is_copy(ty: &HirType) -> bool {
    match ty {
        HirType::Int | HirType::Bool | HirType::Unit => true,
        HirType::Result(t, e) => is_copy(t) && is_copy(e),
        HirType::Option(t) => is_copy(t),
        _ => false,
    }
}

fn is_move(ty: &HirType) -> bool {
    match ty {
        HirType::String
        | HirType::Bytes
        | HirType::Vec(_)
        | HirType::Map(_, _)
        | HirType::Task
        | HirType::Named(_) => true,
        HirType::Result(t, e) => is_move(t) || is_move(e),
        HirType::Option(t) => is_move(t),
        _ => false,
    }
}

#[derive(Debug, Clone)]
struct BindingState {
    ty: HirType,
    mutable: bool,
    moved: bool,
    /// ADR-045: LitInt init (or LitInt assign) for same-scope overflow fold.
    known_int: Option<i64>,
    /// ADR-101: init was `checked_*` MethodCall → Option; unwrap_or(lit) → E0285.
    from_checked_arith: bool,
    /// ADR-103: init was `get` MethodCall → Option; unwrap_or(lit) → E0287.
    from_vec_get: bool,
    /// ADR-125: init was `find`/`rfind` → Option; unwrap_or(0) → E0289.
    from_find_idx: bool,
    /// ADR-126: init was `ilog2`/`ilog10` → Option; unwrap_or(0) → E0290.
    from_ilog: bool,
    /// ADR-272: init was host.read_text|write_text → Result; discard/unwrap_or lit → E0340.
    from_host_io: bool,
    /// ADR-273: init was host.cli_arg|json_get_int → Option; unwrap_or lit → E0341.
    from_host_cli_json: bool,
}

#[derive(Debug, Clone)]
struct Loan {
    place: String,
    exclusive: bool,
}

struct CheckCtx<'a> {
    bindings: HashMap<String, BindingState>,
    loans: Vec<Loan>,
    /// Nesting depth of enclosing `while` loops (F2.3 break/continue legality).
    loop_depth: usize,
    /// ADR-027: inside `async fn` body.
    in_async: bool,
    /// Names of async fns in the module (for await callee check).
    async_fns: &'a std::collections::HashSet<String>,
    /// ADR-044: fn names whose body is `a / b` or `a % b` (2 Int params).
    div_rem_fns: &'a std::collections::HashSet<String>,
    /// ADR-233: record name → fields (name, ty).
    records: &'a HashMap<String, Vec<(String, HirType)>>,
    /// ADR-234: enum name → variant names.
    enums: &'a HashMap<String, Vec<String>>,
    /// ADR-263: user fn name → return type (calls typed Int before this).
    fn_rets: &'a HashMap<String, HirType>,
    /// ADR-283: `Some(E)` only inside a SYNC `fn … -> Result<_, E>` (the only legal context for
    /// `v[i] = x`); `None` in main/Io/non-Result/async fns and tests → E0344.
    result_err_ty: Option<HirType>,
    /// ADR-283: span of the NON-Map `IndexAssign` (Vec/List/String/other/unbound) that errored;
    /// such an error wins over a parser shape error deferred behind it (marker stmt), and over
    /// E0343 only if it precedes the first `?` in source order (§0).
    index_assign_failed_span: Option<arita_syntax::Span>,
    /// ADR-293: names imported by `use` (exempt from E0347).
    imports: std::collections::HashSet<String>,
    /// ADR-295 (S2 MUST-USE): the body about to be checked by `check_body` is in return-tail
    /// position of a fn whose return type is not `Io<()>` (its last value is a return, not a
    /// discard). Set for fn bodies by `bind_fn_params`; propagated by `check_body` to the
    /// bodies of a tail `if`/`else`, `if let`/`else` and `match`; `false` for tests and loops.
    tail_ret: bool,
}

impl<'a> CheckCtx<'a> {
    fn new(
        in_async: bool,
        async_fns: &'a std::collections::HashSet<String>,
        div_rem_fns: &'a std::collections::HashSet<String>,
        records: &'a HashMap<String, Vec<(String, HirType)>>,
        enums: &'a HashMap<String, Vec<String>>,
        fn_rets: &'a HashMap<String, HirType>,
    ) -> Self {
        Self {
            bindings: HashMap::new(),
            loans: Vec::new(),
            loop_depth: 0,
            in_async,
            async_fns,
            div_rem_fns,
            records,
            enums,
            fn_rets,
            result_err_ty: None,
            index_assign_failed_span: None,
            imports: std::collections::HashSet::new(),
            tail_ret: false,
        }
    }

    fn place_name(path: &HirPath) -> Option<&str> {
        path.segments.first().map(|s| s.as_str())
    }

    fn fold_i64(&self, e: &HirExpr) -> Option<i64> {
        match e {
            HirExpr::LitInt(n) => Some(*n),
            HirExpr::Path(p) => {
                let name = Self::place_name(p)?;
                self.bindings.get(name).and_then(|b| b.known_int)
            }
            _ => None,
        }
    }

    /// Path use: if moved → E0201. In move context, mark move-types moved.
    fn use_path(&mut self, path: &HirPath, move_context: bool) -> Result<(), CheckError> {
        let Some(name) = Self::place_name(path) else {
            return Ok(());
        };
        let Some(b) = self.bindings.get_mut(name) else {
            // unbound: not an ownership error in v0
            return Ok(());
        };
        if b.moved {
            return Err(err("E0201", "use of moved value"));
        }
        if move_context && is_move(&b.ty) && !is_copy(&b.ty) {
            b.moved = true;
        }
        Ok(())
    }

    /// Method receiver / ADR-282 index-assign target: moved place → E0201.
    fn check_receiver_not_moved(&self, place: Option<&str>) -> Result<(), CheckError> {
        if let Some(b) = place.and_then(|name| self.bindings.get(name)) {
            if b.moved {
                return Err(err("E0201", "use of moved value"));
            }
        }
        Ok(())
    }

    /// ADR-049 §1b: exclusive receiver (push/put/set/…, ADR-282 `m[k] = v`) needs `let mut` → E0202.
    fn check_exclusive_receiver_mutable(&self, place: Option<&str>) -> Result<(), CheckError> {
        if let Some(b) = place.and_then(|name| self.bindings.get(name)) {
            if !b.mutable {
                return Err(err("E0202", "borrow conflict"));
            }
        }
        Ok(())
    }

    /// Temporary receiver loan (caller pops): conflict with live loans → E0202.
    fn push_receiver_loan(&mut self, name: &str, exclusive: bool) -> Result<(), CheckError> {
        if exclusive {
            if self.loans.iter().any(|l| l.place == name) {
                return Err(err("E0202", "borrow conflict"));
            }
        } else if self.loans.iter().any(|l| l.place == name && l.exclusive) {
            return Err(err("E0202", "borrow conflict"));
        }
        self.loans.push(Loan {
            place: name.to_string(),
            exclusive,
        });
        Ok(())
    }

    /// ADR-237 `put` + ADR-282 `m[k] = v`: Map write typing (non-Map → E0206; K/V mismatch → E0203).
    fn type_of_map_write(
        &self,
        recv_ty: &HirType,
        key: &HirExpr,
        value: &HirExpr,
    ) -> Result<HirType, CheckError> {
        let HirType::Map(k, v) = recv_ty else {
            return Err(err("E0206", "method not in F2 std whitelist"));
        };
        let kt = self.type_of_expr(key)?;
        let vt = self.type_of_expr(value)?;
        if &kt != k.as_ref() || &vt != v.as_ref() {
            return Err(err("E0203", "type mismatch"));
        }
        Ok(HirType::IoUnit)
    }

    /// ADR-265 `set` + ADR-283 `v[i] = x`: Vec write typing (non-Vec → E0206; i≠Int / x≠T → E0203).
    fn type_of_vec_set(
        &self,
        recv_ty: &HirType,
        idx: &HirExpr,
        value: &HirExpr,
    ) -> Result<HirType, CheckError> {
        let HirType::Vec(inner) = recv_ty else {
            return Err(err("E0206", "method not in F2 std whitelist"));
        };
        let idx_ty = self.type_of_expr(idx)?;
        if idx_ty != HirType::Int {
            return Err(err("E0203", "type mismatch"));
        }
        let val_ty = self.type_of_expr(value)?;
        if &val_ty != inner.as_ref() {
            return Err(err("E0203", "type mismatch"));
        }
        Ok(HirType::Result(
            Box::new(HirType::Unit),
            Box::new(HirType::Int),
        ))
    }

    /// ADR-270 `set` + ADR-283 `v[i] = x`: foldable index < 0 (lit / known_int ident) → E0319.
    fn check_negative_set_index(&self, idx: &HirExpr) -> Result<(), CheckError> {
        if let Some(n) = self.fold_i64(idx) {
            if n < 0 {
                return Err(err("E0319", "negative set index"));
            }
        }
        Ok(())
    }

    /// ADR-282 Map (`put`) / ADR-283 Vec (`set(i, x)?`) index-assign checks.
    fn check_index_assign(
        &mut self,
        target: &str,
        key: &HirExpr,
        value: &HirExpr,
        op: Option<&HirBinOp>,
        span: &arita_syntax::Span,
    ) -> Result<(), CheckError> {
        // ADR-282/283: HIR is the sole authority — Map (`put` semantics) and Vec/List
        // (`v.set(i, x)?` semantics) are decided here; String/other/unbound keep E0314 with
        // the statement span in the parser's `@a..b` format.
        let recv_ty = match self.bindings.get(target) {
            // ADR-290 D6: `x[i] op= y` on a Map / String / Bytes keeps the parser-era E0006
            // (the slice-A grammar no longer rejects it in the parser).
            Some(b)
                if op.is_some()
                    && matches!(b.ty, HirType::Map(_, _) | HirType::String | HirType::Bytes) =>
            {
                return Err(CheckError::Coded {
                    code: "E0006",
                    message: format!(
                        "construct outside F1.1 (parse failure) @{}..{}",
                        span.start, span.end
                    ),
                });
            }
            Some(b) if matches!(b.ty, HirType::Map(_, _) | HirType::Vec(_)) => b.ty.clone(),
            _ => {
                return Err(CheckError::Coded {
                    code: "E0314",
                    message: format!(
                        "IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{}..{}",
                        span.start, span.end
                    ),
                })
            }
        };
        if matches!(recv_ty, HirType::Vec(_)) {
            // ADR-283 precedence: context (E0344) > negative index (E0319) > types (E0203).
            let Some(fn_err_ty) = self.result_err_ty.clone() else {
                return Err(CheckError::Coded {
                    code: "E0344",
                    message: format!(
                        "index assign outside result fn @{}..{}",
                        span.start, span.end
                    ),
                });
            };
            self.check_negative_set_index(key)?;
            if op.is_some() {
                // ADR-290 Pin 3: E0344 > E0319 > E0333 (element type ≠ Int) > E0203.
                if let HirType::Vec(inner) = &recv_ty {
                    if inner.as_ref() != &HirType::Int {
                        return Err(CheckError::Coded {
                            code: "E0333",
                            message: "compound index-assign: unsupported element type in v0"
                                .to_string(),
                        });
                    }
                }
            }
            self.type_of_vec_set(&recv_ty, key, value)?;
            // Implicit `?` on `Result<(), Int>`: the fn's E must be Int (D4 → E0203).
            if fn_err_ty != HirType::Int {
                return Err(err("E0203", "type mismatch"));
            }
        } else {
            // ADR-282 Map: same helpers and order as `put`: K/V types (E0203).
            self.type_of_map_write(&recv_ty, key, value)?;
        }
        // Then moved (E0201) → mut (E0202), as `put` / `set`.
        self.check_receiver_not_moved(Some(target))?;
        self.check_exclusive_receiver_mutable(Some(target))?;
        // ADR-282 §0 order: k, then v (RHS may read the receiver, option B), then insert.
        self.eval_expr(key, true)?;
        self.eval_expr(value, true)?;
        self.push_receiver_loan(target, true)?;
        self.loans.pop();
        Ok(())
    }

    fn check_borrow(&mut self, mutable: bool, inner: &HirExpr) -> Result<(), CheckError> {
        let HirExpr::Path(path) = inner else {
            // v0: only Path places
            return Ok(());
        };
        let Some(place) = Self::place_name(path).map(|s| s.to_string()) else {
            return Ok(());
        };
        let binding = self.bindings.get(place.as_str());
        if let Some(b) = binding {
            if b.moved {
                // borrow of moved place → E0202 (Arquitecto pin)
                return Err(err("E0202", "borrow conflict"));
            }
            if mutable && !b.mutable {
                return Err(err("E0205", "invalid let / mut binding"));
            }
        }
        let exclusive = mutable;
        if exclusive {
            if self.loans.iter().any(|l| l.place == place) {
                return Err(err("E0202", "borrow conflict"));
            }
        } else if self.loans.iter().any(|l| l.place == place && l.exclusive) {
            return Err(err("E0202", "borrow conflict"));
        }
        self.loans.push(Loan { place, exclusive });
        Ok(())
    }

    /// Evaluate expression in a given context.
    /// `move_context`: true for let-init RHS and call args (by-value).
    fn eval_expr(&mut self, e: &HirExpr, move_context: bool) -> Result<(), CheckError> {
        match e {
            HirExpr::LitStr(_) | HirExpr::LitInt(_) | HirExpr::LitBool(_) | HirExpr::LitUnit => {
                Ok(())
            }
            HirExpr::Ok(inner) | HirExpr::Err(inner) | HirExpr::Some(inner) => {
                self.eval_expr(inner, move_context)
            }
            HirExpr::Try { inner } => {
                // Force type_of (Option? OUT → E0203; Result → Ok payload)
                let _ = self.type_of_expr(e)?;
                self.eval_expr(inner, move_context)
            }
            HirExpr::None => Ok(()),
            HirExpr::EnumPath { .. } => Ok(()),
            HirExpr::RecordLit { fields, .. } => {
                // Force typecheck (unknown/missing field) then eval field exprs.
                let _ = self.type_of_expr(e)?;
                for (_, fe) in fields {
                    self.eval_expr(fe, true)?;
                }
                Ok(())
            }
            HirExpr::FieldAccess { base, .. } => {
                let _ = self.type_of_expr(e)?;
                // Shared access: do not move the record binding.
                self.eval_expr(base, false)?;
                Ok(())
            }
            HirExpr::Path(p) => self.use_path(p, move_context),
            HirExpr::Borrow { mutable, inner } => self.check_borrow(*mutable, inner),
            HirExpr::Binary { op, lhs, rhs } => {
                // ADR-044 / E0216: `/` or `%` with RHS LitInt 0 — reject before emit
                if matches!(op, HirBinOp::Div | HirBinOp::Rem)
                    && matches!(rhs.as_ref(), HirExpr::LitInt(0))
                {
                    return Err(err("E0216", "integer division by zero"));
                }
                // ADR-045 / E0217: `+` `-` `*` with both sides lit-foldable overflowing i64
                if matches!(op, HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul) {
                    if let (Some(l), Some(r)) = (self.fold_i64(lhs), self.fold_i64(rhs)) {
                        let ok = match op {
                            HirBinOp::Add => l.checked_add(r).is_some(),
                            HirBinOp::Sub => l.checked_sub(r).is_some(),
                            HirBinOp::Mul => l.checked_mul(r).is_some(),
                            _ => true,
                        };
                        if !ok {
                            return Err(err("E0217", "integer overflow"));
                        }
                    }
                }
                // Int operands: copy uses
                self.eval_expr(lhs, false)?;
                self.eval_expr(rhs, false)?;
                Ok(())
            }
            HirExpr::Call(c) => {
                if c.callee == "spawn" || c.callee == "join" {
                    if !self.in_async {
                        return Err(err("E0240", "await outside async function"));
                    }
                    if c.callee == "spawn" {
                        let _ = self.type_of_spawn(c)?;
                        // Spawn arg is async call — do not require await (E0241).
                        for arg in &c.args {
                            if let HirExpr::Call(inner) = arg {
                                if self.async_fns.contains(&inner.callee) {
                                    for a in &inner.args {
                                        self.eval_expr(a, true)?;
                                    }
                                    continue;
                                }
                            }
                            self.eval_expr(arg, true)?;
                        }
                        return Ok(());
                    }
                    // join must be awaited — bare join(t) as stmt is illegal
                    return Err(err("E0241", "async feature not allowed here"));
                }

                // ADR-244 TIMEOUT-CANCEL
                match c.callee.as_str() {
                    "busy_spin" | "hang_forever" => {
                        return Err(err(
                            "E0320",
                            "hang theater / busy-spin not allowed (use timeout)",
                        ));
                    }
                    "timeout" => {
                        if !self.in_async {
                            return Err(err("E0313", "timeout requires async function"));
                        }
                        if c.args.len() != 2 {
                            return Err(err("E0313", "timeout expects (ms, async_call)"));
                        }
                        match &c.args[0] {
                            HirExpr::LitInt(_) => {}
                            other => {
                                let t = self.type_of_expr(other)?;
                                if t != HirType::Int {
                                    return Err(err("E0313", "timeout ms must be Int"));
                                }
                            }
                        }
                        self.eval_expr(&c.args[0], false)?;
                        match &c.args[1] {
                            HirExpr::Call(inner) if self.async_fns.contains(&inner.callee) => {
                                for a in &inner.args {
                                    self.eval_expr(a, true)?;
                                }
                            }
                            HirExpr::Call(inner)
                                if matches!(inner.callee.as_str(), "delay" | "until_cancelled") =>
                            {
                                for a in &inner.args {
                                    self.eval_expr(a, true)?;
                                }
                            }
                            _ => {
                                return Err(err(
                                    "E0313",
                                    "timeout body must be async call or delay/until_cancelled",
                                ));
                            }
                        }
                        return Ok(());
                    }
                    "delay" => {
                        if !self.in_async {
                            return Err(err("E0313", "delay requires async function"));
                        }
                        if c.args.len() != 1 {
                            return Err(err("E0313", "delay expects (ms)"));
                        }
                        self.eval_expr(&c.args[0], false)?;
                        if self.type_of_expr(&c.args[0])? != HirType::Int {
                            return Err(err("E0313", "delay ms must be Int"));
                        }
                        return Ok(());
                    }
                    "cancel_token" => {
                        if !c.args.is_empty() {
                            return Err(err("E0313", "cancel_token expects no args"));
                        }
                        return Ok(());
                    }
                    "cancel" => {
                        if c.args.len() != 1 {
                            return Err(err("E0313", "cancel expects (token)"));
                        }
                        self.eval_expr(&c.args[0], false)?;
                        return Ok(());
                    }
                    "until_cancelled" => {
                        if !self.in_async {
                            return Err(err("E0313", "until_cancelled requires async function"));
                        }
                        if c.args.len() != 1 {
                            return Err(err("E0313", "until_cancelled expects (token)"));
                        }
                        self.eval_expr(&c.args[0], false)?;
                        return Ok(());
                    }
                    // ADR-245: free http_* OUT — surface is types/methods (E0206)
                    "http_listen" | "http_serve" | "http_route_health" | "http_route_echo"
                    | "http_bound_port" | "http_shutdown" | "http_route" | "http_response" => {
                        return Err(err(
                            "E0206",
                            "http free-fn not in surface (use HttpServer.bind / .serve)",
                        ));
                    }
                    other if other.contains("reqwest") => {
                        return Err(err(
                            "E0321",
                            "reqwest not in surface (use HttpClient.get / post_text)",
                        ));
                    }
                    // ADR-293: free ident call to a fn neither declared, imported nor builtin.
                    name if !name.contains("::")
                        && name != "print"
                        && name != arita_syntax::DEFERRED_SHAPE_MARKER
                        && !BUILTIN_FN_NAMES.contains(&name)
                        && !self.fn_rets.contains_key(name)
                        && !self.imports.contains(name) =>
                    {
                        return Err(CheckError::Coded {
                            code: "E0347",
                            message: format!("call to undeclared function `{name}`"),
                        });
                    }
                    _ => {}
                }
                if self.async_fns.contains(&c.callee) {
                    return Err(err("E0241", "async feature not allowed here"));
                }
                // ADR-044 / E0216: call to fn { a/b }|{ a%b } with 2nd arg LitInt 0
                if self.div_rem_fns.contains(&c.callee)
                    && matches!(c.args.get(1), Some(HirExpr::LitInt(0)))
                {
                    return Err(err("E0216", "integer division by zero"));
                }
                for arg in &c.args {
                    self.eval_expr(arg, true)?;
                }
                Ok(())
            }
            HirExpr::Await { inner } => {
                if !self.in_async {
                    return Err(err("E0240", "await outside async function"));
                }
                match inner.as_ref() {
                    HirExpr::Call(c) if self.async_fns.contains(&c.callee) => {
                        // ADR-039 / E0242: any live loan at await point (before rustc).
                        if !self.loans.is_empty() {
                            return Err(err("E0242", "borrow held across await"));
                        }
                        for arg in &c.args {
                            self.eval_expr(arg, true)?;
                        }
                        Ok(())
                    }
                    HirExpr::Call(c) if c.callee == "join" => {
                        // ADR-243: await join(task)
                        if !self.loans.is_empty() {
                            return Err(err("E0242", "borrow held across await"));
                        }
                        let _ = self.type_of_join(c)?;
                        for arg in &c.args {
                            self.eval_expr(arg, true)?;
                        }
                        Ok(())
                    }
                    _ => Err(err("E0241", "async feature not allowed here")),
                }
            }
            HirExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                // Type/whitelist/arity first (E0203 / E0206); shared/exclusive loan after.
                let _ = self.type_of_expr(e)?;
                // Temporary loan on receiver place for duration of this call only.
                let place = match receiver.as_ref() {
                    HirExpr::Path(p) => Self::place_name(p).map(|s| s.to_string()),
                    _ => None,
                };
                self.check_receiver_not_moved(place.as_deref())?;
                let recv_ty_loan = self.type_of_expr(receiver)?;
                let exclusive = method == "push"
                    || method == "insert"
                    || method == "set"
                    || method == "put"
                    || method == "push_str"
                    || method == "clear"
                    || method == "pop"
                    || method == "remove"
                    || method == "swap_remove"
                    || method == "truncate"
                    || method == "resize"
                    || method == "make_ascii_lowercase"
                    || method == "make_ascii_uppercase"
                    || method == "reserve"
                    || method == "try_reserve"
                    || method == "try_reserve_exact"
                    || method == "shrink_to"
                    || method == "shrink_to_fit"
                    || method == "swap"
                    || method == "split_off"
                    || method == "append"
                    || method == "extend"
                    || method == "fill"
                    || method == "dedup"
                    || method == "sort_unstable"
                    || method == "reverse"
                    || method == "take"
                    || (method == "replace" && matches!(recv_ty_loan, HirType::Option(_)))
                    || ((method == "rotate_left" || method == "rotate_right")
                        && matches!(recv_ty_loan, HirType::Vec(_)));
                let shared = method == "len"
                    || method == "capacity"
                    || method == "is_empty"
                    || method == "is_sorted"
                    || method == "binary_search"
                    || method == "is_ascii"
                    || method == "clone"
                    || method == "contains"
                    || method == "starts_with"
                    || method == "ends_with"
                    || method == "eq_ignore_ascii_case"
                    || method == "is_char_boundary"
                    || method == "floor_char_boundary"
                    || method == "ceil_char_boundary"
                    || method == "trim"
                    || method == "trim_ascii"
                    || method == "trim_ascii_start"
                    || method == "trim_ascii_end"
                    || method == "replace"
                    || method == "to_uppercase"
                    || method == "to_lowercase"
                    || method == "to_ascii_lowercase"
                    || method == "to_ascii_uppercase"
                    || method == "parse_int"
                    || method == "to_string"
                    || method == "as_bytes"
                    || method == "abs"
                    || method == "signum"
                    || method == "is_positive"
                    || method == "is_negative"
                    || method == "wrapping_neg"
                    || method == "saturating_neg"
                    || method == "min"
                    || method == "max"
                    || method == "clamp"
                    || method == "trim_start"
                    || method == "trim_end"
                    || method == "strip_prefix"
                    || method == "strip_suffix"
                    || method == "repeat"
                    || method == "pow"
                    || method == "checked_add"
                    || method == "checked_sub"
                    || method == "checked_mul"
                    || method == "checked_div"
                    || method == "checked_rem"
                    || method == "checked_neg"
                    || method == "checked_div_euclid"
                    || method == "checked_rem_euclid"
                    || method == "checked_shl"
                    || method == "checked_shr"
                    || method == "get"
                    || method == "first"
                    || method == "last"
                    || method == "saturating_add"
                    || method == "saturating_sub"
                    || method == "saturating_mul"
                    || method == "saturating_div"
                    || method == "saturating_rem"
                    || method == "unwrap_or"
                    || method == "is_some"
                    || method == "is_none"
                    || method == "is_ok"
                    || method == "is_err"
                    || method == "ok_or"
                    || method == "ok"
                    || method == "err"
                    || method == "wrapping_add"
                    || method == "wrapping_sub"
                    || method == "wrapping_mul"
                    || method == "wrapping_div"
                    || method == "wrapping_rem"
                    || method == "wrapping_shl"
                    || method == "wrapping_shr"
                    || method == "div_euclid"
                    || method == "rem_euclid"
                    || method == "div_ceil"
                    || method == "div_floor"
                    || method == "find"
                    || method == "rfind"
                    || method == "abs_diff"
                    || method == "midpoint"
                    || method == "is_multiple_of"
                    || method == "is_power_of_two"
                    || method == "checked_next_power_of_two"
                    || method == "next_multiple_of"
                    || method == "checked_next_multiple_of"
                    || method == "previous_multiple_of"
                    || method == "checked_previous_multiple_of"
                    || method == "ilog2"
                    || method == "ilog10"
                    || method == "count_ones"
                    || method == "count_zeros"
                    || method == "leading_zeros"
                    || method == "trailing_zeros"
                    || method == "leading_ones"
                    || method == "trailing_ones"
                    || method == "reverse_bits"
                    || method == "rotate_left"
                    || method == "rotate_right"
                    || method == "swap_bytes"
                    || method == "to_be"
                    || method == "to_le"
                    || method == "from_be"
                    || method == "from_le"
                    || method == "then_some"
                    || method == "flatten"
                    || method == "transpose"
                    || method == "or"
                    || method == "and"
                    || method == "xor"
                    || method == "status"
                    || method == "body_text"
                    || method == "header"
                    || method == "method"
                    || method == "path";
                // ADR-049 §1b: exclusive methods require mut binding (push/clear).
                if exclusive {
                    self.check_exclusive_receiver_mutable(place.as_deref())?;
                }
                let mut pushed = false;
                if let Some(ref name) = place {
                    if exclusive || shared {
                        self.push_receiver_loan(name, exclusive)?;
                        pushed = true;
                    }
                }
                let mut vec_arg_loaned = false;
                for arg in args {
                    if method == "append" {
                        // ADR-222: exclusive mut borrow of arg; do not move Vec
                        let HirExpr::Path(p) = arg else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        let Some(name) = Self::place_name(p) else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        if let Some(b) = self.bindings.get(name) {
                            if b.moved {
                                return Err(err("E0201", "use of moved value"));
                            }
                            if !b.mutable {
                                return Err(err("E0202", "borrow conflict"));
                            }
                        }
                        if self.loans.iter().any(|l| l.place == name) {
                            return Err(err("E0202", "borrow conflict"));
                        }
                        self.loans.push(Loan {
                            place: name.to_string(),
                            exclusive: true,
                        });
                        vec_arg_loaned = true;
                    } else if method == "extend" {
                        // ADR-224: shared borrow of arg; do not move Vec (b intact)
                        let HirExpr::Path(p) = arg else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        let Some(name) = Self::place_name(p) else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        if let Some(b) = self.bindings.get(name) {
                            if b.moved {
                                return Err(err("E0201", "use of moved value"));
                            }
                        }
                        if self.loans.iter().any(|l| l.place == name && l.exclusive) {
                            return Err(err("E0202", "borrow conflict"));
                        }
                        self.loans.push(Loan {
                            place: name.to_string(),
                            exclusive: false,
                        });
                        vec_arg_loaned = true;
                    } else {
                        self.eval_expr(arg, true)?;
                    }
                }
                if vec_arg_loaned {
                    self.loans.pop();
                }
                if pushed {
                    self.loans.pop();
                }
                Ok(())
            }
            HirExpr::Match { scrutinee, arms } => {
                self.eval_expr(scrutinee, false)?;
                for arm in arms {
                    self.eval_expr(&arm.value, move_context)?;
                }
                // Exhaustiveness + arm-type unify (E0221–E0223 / E0203)
                let _ = self.type_of_expr(e)?;
                Ok(())
            }
        }
    }

    fn type_of_expr(&self, e: &HirExpr) -> Result<HirType, CheckError> {
        match e {
            HirExpr::LitInt(_) => Ok(HirType::Int),
            HirExpr::LitBool(_) => Ok(HirType::Bool),
            HirExpr::LitStr(_) => Ok(HirType::String),
            HirExpr::LitUnit => Ok(HirType::Unit),
            // Ok/Err without let expected-type: synthesize Result with Unit other side.
            HirExpr::Ok(inner) => {
                let pt = self.type_of_expr(inner)?;
                Ok(HirType::Result(Box::new(pt), Box::new(HirType::Unit)))
            }
            HirExpr::Err(inner) => {
                let pt = self.type_of_expr(inner)?;
                Ok(HirType::Result(Box::new(HirType::Unit), Box::new(pt)))
            }
            HirExpr::Some(inner) => {
                let pt = self.type_of_expr(inner)?;
                Ok(HirType::Option(Box::new(pt)))
            }
            HirExpr::None => Ok(HirType::Option(Box::new(HirType::Unit))),
            HirExpr::Path(p) => {
                let Some(name) = Self::place_name(p) else {
                    return Ok(HirType::Int); // v0 fallback
                };
                if let Some(b) = self.bindings.get(name) {
                    Ok(b.ty.clone())
                } else {
                    // unbound: not a type error in ownership v0
                    Ok(HirType::Int)
                }
            }
            HirExpr::RecordLit { name, fields } => {
                let Some(decl) = self.records.get(name) else {
                    return Err(err("E0203", "type mismatch"));
                };
                // ADR-235: borrow cannot escape into record fields
                for (_fname, fexpr) in fields {
                    if matches!(fexpr, HirExpr::Borrow { .. }) {
                        return Err(err("E0318", "borrow cannot escape"));
                    }
                }
                // unknown fields
                for (fname, _fexpr) in fields {
                    if !decl.iter().any(|(n, _)| n == fname) {
                        return Err(err("E0314", "unknown field"));
                    }
                }
                // missing fields
                for (n, _) in decl {
                    if !fields.iter().any(|(fname, _)| fname == n) {
                        return Err(err("E0315", "missing field"));
                    }
                }
                // duplicate fields
                let mut seen = std::collections::HashSet::new();
                for (fname, fexpr) in fields {
                    if !seen.insert(fname.clone()) {
                        return Err(err("E0314", "unknown field"));
                    }
                    let expect = decl
                        .iter()
                        .find(|(n, _)| n == fname)
                        .map(|(_, t)| t.clone())
                        .unwrap();
                    let got = self.type_of_expr(fexpr)?;
                    if got != expect {
                        return Err(err("E0203", "type mismatch"));
                    }
                    // ADR-246: ServicePolicy ints must be > 0 (E0323)
                    if name == "ServicePolicy" {
                        if let Some(v) = self.fold_i64(fexpr) {
                            if v <= 0 {
                                return Err(err("E0323", "invalid policy value"));
                            }
                        }
                    }
                }
                Ok(HirType::Named(name.clone()))
            }
            HirExpr::EnumPath { enum_name, variant } => {
                let Some(vars) = self.enums.get(enum_name) else {
                    return Err(err("E0317", "unknown variant"));
                };
                if !vars.iter().any(|v| v == variant) {
                    return Err(err("E0317", "unknown variant"));
                }
                if self.records.contains_key(enum_name) {
                    return Err(err("E0317", "unknown variant"));
                }
                Ok(HirType::Named(enum_name.clone()))
            }
            HirExpr::FieldAccess { base, field } => {
                let bty = self.type_of_expr(base)?;
                let HirType::Named(rname) = bty else {
                    return Err(err("E0314", "unknown field"));
                };
                let Some(decl) = self.records.get(&rname) else {
                    return Err(err("E0314", "unknown field"));
                };
                let Some((_, fty)) = decl.iter().find(|(n, _)| n == field) else {
                    return Err(err("E0314", "unknown field"));
                };
                Ok(fty.clone())
            }
            HirExpr::Borrow { .. } => Ok(HirType::Int), // v0: borrow typed as Int placeholder
            HirExpr::Binary { op, lhs, rhs } => {
                match op {
                    HirBinOp::Add
                    | HirBinOp::Sub
                    | HirBinOp::Mul
                    | HirBinOp::Div
                    | HirBinOp::Rem => {
                        self.type_of_expr(lhs)?;
                        self.type_of_expr(rhs)?;
                        Ok(HirType::Int)
                    }
                    HirBinOp::Eq
                    | HirBinOp::Ne
                    | HirBinOp::Lt
                    | HirBinOp::Le
                    | HirBinOp::Gt
                    | HirBinOp::Ge => {
                        let lt = self.type_of_expr(lhs)?;
                        let rt = self.type_of_expr(rhs)?;
                        // Int comparisons only in v0. Non-Int operands → Int (non-Bool)
                        // so require_bool_cond emits E0220 when used as if/while cond.
                        if lt != HirType::Int || rt != HirType::Int {
                            return Ok(HirType::Int);
                        }
                        Ok(HirType::Bool)
                    }
                }
            }
            HirExpr::Call(c) if c.callee == "Vec::new" || c.callee == "List::new" => {
                Ok(HirType::Vec(Box::new(HirType::Int)))
            }
            HirExpr::Call(c) if c.callee == "Map::new" => Ok(HirType::Map(
                Box::new(HirType::String),
                Box::new(HirType::Int),
            )),
            HirExpr::Call(c) if c.callee == "spawn" => self.type_of_spawn(c),
            HirExpr::Call(c) if c.callee == "join" => self.type_of_join(c),
            HirExpr::Call(c) if c.callee == "timeout" => Ok(HirType::Result(
                Box::new(HirType::Unit),
                Box::new(HirType::String),
            )),
            HirExpr::Call(c) if c.callee == "delay" => Ok(HirType::Unit),
            HirExpr::Call(c) if c.callee == "cancel_token" => Ok(HirType::Int),
            HirExpr::Call(c) if c.callee == "cancel" => Ok(HirType::Unit),
            HirExpr::Call(c) if c.callee == "until_cancelled" => Ok(HirType::Unit),
            // ADR-245 free http_* rejected in eval_expr (E0206); no typeof success path.
            HirExpr::Call(c) if c.callee.starts_with("host::") => self.type_of_host_call(c),
            // ADR-263: user fn calls carry declared ret_ty (was always Int).
            HirExpr::Call(c) => {
                if let Some(t) = self.fn_rets.get(&c.callee) {
                    Ok(t.clone())
                } else {
                    Ok(HirType::Int)
                }
            }
            HirExpr::MethodCall {
                receiver,
                method,
                args,
            } => self.type_of_method_call(receiver, method, args),
            HirExpr::Match { scrutinee, arms } => {
                let sty = self.type_of_expr(scrutinee)?;
                match &sty {
                    HirType::Result(ok_ty, err_ty) => {
                        let mut has_ok = false;
                        let mut has_err = false;
                        let mut arm_ty: Option<HirType> = None;
                        for arm in arms {
                            match &arm.pat {
                                HirPat::Ok(_) => has_ok = true,
                                HirPat::Err(_) => has_err = true,
                                HirPat::Wildcard
                                | HirPat::LitBool(_)
                                | HirPat::LitInt(_)
                                | HirPat::Some(_)
                                | HirPat::None
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            // Bindings in arm scope for value typing
                            let bind = match &arm.pat {
                                HirPat::Ok(n) => Some((n.clone(), ok_ty.as_ref().clone())),
                                HirPat::Err(n) => Some((n.clone(), err_ty.as_ref().clone())),
                                _ => None,
                            };
                            // type_of_expr is &self — cannot insert bindings.
                            // For match-expr values, Ok/Err bindings may appear in value;
                            // v0: values typically don't use bindings for Result oracles (stmt form).
                            // Still type the value; if Path uses binding name unbound → Int fallback (existing).
                            let _ = (ok_ty, err_ty, bind);
                            let vt = self.type_of_expr(&arm.value)?;
                            if vt != HirType::Bool && vt != HirType::Int {
                                return Err(err("E0203", "type mismatch"));
                            }
                            match &arm_ty {
                                None => arm_ty = Some(vt),
                                Some(t) if *t != vt => {
                                    return Err(err("E0203", "type mismatch"));
                                }
                                Some(_) => {}
                            }
                        }
                        if !(has_ok && has_err) {
                            return Err(err("E0270", "non-exhaustive result match"));
                        }
                        // ADR-272: host IO Result match-expr — E0340 empty/default Err theater
                        let scrut_ref: &HirExpr = scrutinee.as_ref();
                        let host_io_scrut = expr_from_host_io(scrut_ref)
                            || matches!(
                                scrut_ref,
                                HirExpr::Path(p) if Self::place_name(p)
                                    .and_then(|n| self.bindings.get(n))
                                    .map(|b| b.from_host_io)
                                    .unwrap_or(false)
                            );
                        if host_io_scrut {
                            if host_io_result_err_theater_expr(arms) {
                                return Err(err("E0340", "io result discarded"));
                            }
                        } else if result_err_swallowed_expr(arms) {
                            return Err(err("E0272", "result error swallowed"));
                        }
                        if vacuous_match_expr_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        arm_ty.ok_or_else(|| err("E0203", "type mismatch"))
                    }
                    HirType::Option(inner_ty) => {
                        let mut has_some = false;
                        let mut has_none = false;
                        let mut arm_ty: Option<HirType> = None;
                        for arm in arms {
                            match &arm.pat {
                                HirPat::Some(_) => has_some = true,
                                HirPat::None => has_none = true,
                                HirPat::Wildcard
                                | HirPat::LitBool(_)
                                | HirPat::LitInt(_)
                                | HirPat::Ok(_)
                                | HirPat::Err(_)
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            let _ = inner_ty;
                            let vt = self.type_of_expr(&arm.value)?;
                            if vt != HirType::Bool && vt != HirType::Int {
                                return Err(err("E0203", "type mismatch"));
                            }
                            match &arm_ty {
                                None => arm_ty = Some(vt),
                                Some(t) if *t != vt => {
                                    return Err(err("E0203", "type mismatch"));
                                }
                                Some(_) => {}
                            }
                        }
                        if !(has_some && has_none) {
                            return Err(err("E0273", "non-exhaustive option match"));
                        }
                        // ADR-273: host cli/json Option match-expr — E0341 empty/default None theater
                        let scrut_ref: &HirExpr = scrutinee.as_ref();
                        let host_cli_scrut = expr_from_host_cli_json(scrut_ref)
                            || matches!(
                                scrut_ref,
                                HirExpr::Path(p) if Self::place_name(p)
                                    .and_then(|n| self.bindings.get(n))
                                    .map(|b| b.from_host_cli_json)
                                    .unwrap_or(false)
                            );
                        if host_cli_scrut {
                            if host_cli_none_theater_expr(arms) {
                                return Err(err("E0341", "required arg miss as ok"));
                            }
                        } else if option_none_swallowed_expr(arms) {
                            return Err(err("E0274", "option none swallowed"));
                        }
                        if vacuous_match_expr_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        arm_ty.ok_or_else(|| err("E0203", "type mismatch"))
                    }
                    HirType::Bool | HirType::Int => {
                        let mut has_true = false;
                        let mut has_false = false;
                        let mut has_wild = false;
                        let mut arm_ty: Option<HirType> = None;
                        for arm in arms {
                            match &arm.pat {
                                HirPat::Wildcard => has_wild = true,
                                HirPat::LitBool(true) => {
                                    if sty != HirType::Bool {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                    has_true = true;
                                }
                                HirPat::LitBool(false) => {
                                    if sty != HirType::Bool {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                    has_false = true;
                                }
                                HirPat::LitInt(_) => {
                                    if sty != HirType::Int {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                }
                                HirPat::Ok(_)
                                | HirPat::Err(_)
                                | HirPat::Some(_)
                                | HirPat::None
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            let vt = self.type_of_expr(&arm.value)?;
                            // ADR-025 pin: arm values Bool|Int only; mismatch → E0203
                            if vt != HirType::Bool && vt != HirType::Int {
                                return Err(err("E0203", "type mismatch"));
                            }
                            match &arm_ty {
                                None => arm_ty = Some(vt),
                                Some(t) if *t != vt => {
                                    return Err(err("E0203", "type mismatch"));
                                }
                                Some(_) => {}
                            }
                        }
                        if sty == HirType::Bool && !(has_wild || (has_true && has_false)) {
                            return Err(err("E0221", "non-exhaustive Bool match"));
                        } else if sty == HirType::Int && !has_wild {
                            return Err(err("E0222", "Int match requires `_` arm"));
                        }
                        // ADR-031 / E0225: all arms same constant lit → vacuous theater
                        if vacuous_match_expr_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        arm_ty.ok_or_else(|| err("E0203", "type mismatch"))
                    }
                    HirType::Named(ename) if self.enums.contains_key(ename) => {
                        let variants = self.enums.get(ename).unwrap();
                        let mut covered: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        let mut has_wild = false;
                        let mut arm_ty: Option<HirType> = None;
                        for arm in arms {
                            match &arm.pat {
                                HirPat::Wildcard => has_wild = true,
                                HirPat::Variant { enum_name, variant } => {
                                    if let Some(en) = enum_name {
                                        if en != ename {
                                            return Err(err(
                                                "E0223",
                                                "match pattern/scrutinee type mismatch",
                                            ));
                                        }
                                    }
                                    if !variants.iter().any(|v| v == variant) {
                                        return Err(err("E0317", "unknown variant"));
                                    }
                                    covered.insert(variant.clone());
                                }
                                _ => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            let vt = self.type_of_expr(&arm.value)?;
                            if let Some(ref t) = arm_ty {
                                if t != &vt {
                                    return Err(err("E0203", "type mismatch"));
                                }
                            } else {
                                arm_ty = Some(vt);
                            }
                        }
                        if !(has_wild || variants.iter().all(|v| covered.contains(v))) {
                            return Err(err("E0316", "non-exhaustive enum match"));
                        }
                        if vacuous_match_expr_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        arm_ty.ok_or_else(|| err("E0203", "type mismatch"))
                    }
                    _ => Err(err("E0223", "match pattern/scrutinee type mismatch")),
                }
            }
            HirExpr::Await { .. } => Ok(HirType::IoUnit),
            // ADR-278: `expr?` — Result → Ok payload; Option OUT → E0203 (no new E0xxx)
            HirExpr::Try { inner } => {
                let inner_ty = self.type_of_expr(inner)?;
                match inner_ty {
                    HirType::Result(ok, _) => Ok(*ok),
                    HirType::Option(_) => Err(err("E0203", "type mismatch")),
                    _ => Err(err("E0203", "type mismatch")),
                }
            }
        }
    }

    fn type_of_spawn(&self, c: &HirCall) -> Result<HirType, CheckError> {
        // ADR-243
        if c.args.len() != 1 {
            return Err(err("E0203", "type mismatch"));
        }
        let HirExpr::Call(inner) = &c.args[0] else {
            return Err(err("E0241", "async feature not allowed here"));
        };
        // ADR-286 §0.1d (a): the spawned fn must return Io<()> (any other ret, Int included,
        // is E0203). Checked on the `spawn` itself, so `await join(spawn(f()))` anchors here.
        if let Some(t) = self
            .fn_rets
            .get(&inner.callee)
            .filter(|t| **t != HirType::IoUnit)
        {
            return Err(CheckError::Coded {
                code: "E0203",
                message: format!(
                    "type mismatch: `spawn` expects async fn returning Io<()>, `{}` returns {}",
                    inner.callee,
                    hir_type_surface(t)
                ),
            });
        }
        if !self.async_fns.contains(&inner.callee) {
            return Err(err("E0241", "async feature not allowed here"));
        }
        if !inner.args.is_empty() {
            return Err(err("E0203", "type mismatch"));
        }
        Ok(HirType::Task)
    }

    fn type_of_join(&self, c: &HirCall) -> Result<HirType, CheckError> {
        if c.args.len() != 1 {
            return Err(err("E0203", "type mismatch"));
        }
        if self.type_of_expr(&c.args[0])? != HirType::Task {
            return Err(err("E0203", "type mismatch"));
        }
        Ok(HirType::IoUnit)
    }

    /// ADR-238: curated `host.*` files/JSON/CLI bindings.
    fn type_of_host_call(&self, c: &HirCall) -> Result<HirType, CheckError> {
        let name = c.callee.strip_prefix("host::").unwrap_or(c.callee.as_str());
        match name {
            "mark" => {
                // ADR-035 demo
                if !c.args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "read_text" => {
                if c.args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.type_of_expr(&c.args[0])? != HirType::String {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Result(
                    Box::new(HirType::String),
                    Box::new(HirType::Int),
                ))
            }
            "write_text" => {
                if c.args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.type_of_expr(&c.args[0])? != HirType::String
                    || self.type_of_expr(&c.args[1])? != HirType::String
                {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Result(
                    Box::new(HirType::Int),
                    Box::new(HirType::Int),
                ))
            }
            "cli_arg" => {
                if c.args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.type_of_expr(&c.args[0])? != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::String)))
            }
            "json_get_int" => {
                if c.args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.type_of_expr(&c.args[0])? != HirType::String
                    || self.type_of_expr(&c.args[1])? != HirType::String
                {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            _ => Err(err("E0206", "method not in F2 std whitelist")),
        }
    }

    /// ADR-026 / CUT STD-MIN-20260913: whitelist method typing.
    fn type_of_method_call(
        &self,
        receiver: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Result<HirType, CheckError> {
        // ADR-245 associated ctors: HttpServer.bind / HttpResponse.ok_text|status
        if let HirExpr::Path(p) = receiver {
            let tname = match p.segments.as_slice() {
                [s] => s.as_str(),
                _ => "",
            };
            if tname == "HttpServer" && method == "bind" {
                if args.len() != 2 {
                    return Err(err(
                        "E0321",
                        "HttpServer.bind expects (addr: Text, port: Int)",
                    ));
                }
                if self.type_of_expr(&args[0])? != HirType::String {
                    return Err(err("E0321", "HttpServer.bind addr must be Text"));
                }
                if self.type_of_expr(&args[1])? != HirType::Int {
                    return Err(err("E0321", "HttpServer.bind port must be Int"));
                }
                return Ok(HirType::Result(
                    Box::new(HirType::Named("HttpServer".into())),
                    Box::new(HirType::Named("IoError".into())),
                ));
            }
            if tname == "HttpResponse" && method == "ok_text" {
                if args.len() != 1 {
                    return Err(err("E0321", "HttpResponse.ok_text expects (body: Text)"));
                }
                if self.type_of_expr(&args[0])? != HirType::String {
                    return Err(err("E0321", "HttpResponse.ok_text body must be Text"));
                }
                return Ok(HirType::Named("HttpResponse".into()));
            }
            if tname == "HttpResponse" && method == "status" {
                if args.len() != 2 {
                    return Err(err(
                        "E0321",
                        "HttpResponse.status expects (code: Int, body: Text)",
                    ));
                }
                if self.type_of_expr(&args[0])? != HirType::Int
                    || self.type_of_expr(&args[1])? != HirType::String
                {
                    return Err(err("E0321", "HttpResponse.status type mismatch"));
                }
                return Ok(HirType::Named("HttpResponse".into()));
            }
            // ADR-246
            if tname == "ServicePolicy" && method == "default" {
                if !args.is_empty() {
                    return Err(err("E0321", "ServicePolicy.default expects no args"));
                }
                return Ok(HirType::Named("ServicePolicy".into()));
            }
            // ADR-247 curated client
            if tname == "HttpClient" && method == "url" {
                if args.len() != 2 {
                    return Err(err(
                        "E0321",
                        "HttpClient.url expects (port: Int, path: Text)",
                    ));
                }
                if self.type_of_expr(&args[0])? != HirType::Int
                    || self.type_of_expr(&args[1])? != HirType::String
                {
                    return Err(err("E0321", "HttpClient.url type mismatch"));
                }
                return Ok(HirType::String);
            }
            if tname == "HttpClient" && method == "get" {
                if args.len() != 1 {
                    return Err(err("E0321", "HttpClient.get expects (url: Text)"));
                }
                if self.type_of_expr(&args[0])? != HirType::String {
                    return Err(err("E0321", "HttpClient.get url must be Text"));
                }
                return Ok(HirType::Result(
                    Box::new(HirType::Named("HttpResponse".into())),
                    Box::new(HirType::Named("IoError".into())),
                ));
            }
            if tname == "HttpClient" && method == "post_text" {
                if args.len() != 2 {
                    return Err(err(
                        "E0321",
                        "HttpClient.post_text expects (url: Text, body: Text)",
                    ));
                }
                if self.type_of_expr(&args[0])? != HirType::String
                    || self.type_of_expr(&args[1])? != HirType::String
                {
                    return Err(err("E0321", "HttpClient.post_text type mismatch"));
                }
                return Ok(HirType::Result(
                    Box::new(HirType::Named("HttpResponse".into())),
                    Box::new(HirType::Named("IoError".into())),
                ));
            }
            if tname == "HttpClient" && method == "post_json" {
                return Err(err(
                    "E0321",
                    "HttpClient.post_json OUT in v0 (Json not IN; use post_text)",
                ));
            }
        }

        let recv_ty = self.type_of_expr(receiver)?;
        // ADR-245 instance methods on HTTP opaque types
        if let HirType::Named(n) = &recv_ty {
            match (n.as_str(), method) {
                ("HttpServer", "serve") => {
                    if !self.in_async {
                        return Err(err("E0321", "HttpServer.serve requires async function"));
                    }
                    if args.len() != 1 {
                        return Err(err("E0321", "HttpServer.serve expects (handler)"));
                    }
                    // handler: path to async fn (signature checked lightly)
                    let HirExpr::Path(hp) = &args[0] else {
                        return Err(err("E0321", "HttpServer.serve handler must be fn path"));
                    };
                    let hname = match hp.segments.as_slice() {
                        [s] => s.as_str(),
                        _ => {
                            return Err(err("E0321", "HttpServer.serve handler must be fn path"));
                        }
                    };
                    if !self.async_fns.contains(hname) {
                        return Err(err("E0321", "HttpServer.serve handler must be async fn"));
                    }
                    return Ok(HirType::Result(
                        Box::new(HirType::Unit),
                        Box::new(HirType::Named("IoError".into())),
                    ));
                }
                ("HttpServer", "port") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpServer.port expects no args"));
                    }
                    return Ok(HirType::Int);
                }
                ("HttpServer", "shutdown") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpServer.shutdown expects no args"));
                    }
                    return Ok(HirType::Unit);
                }
                ("HttpServer", "set_policy") => {
                    if args.len() != 1 {
                        return Err(err(
                            "E0321",
                            "HttpServer.set_policy expects (ServicePolicy)",
                        ));
                    }
                    let pty = self.type_of_expr(&args[0])?;
                    if pty != HirType::Named("ServicePolicy".into()) {
                        return Err(err("E0321", "HttpServer.set_policy expects ServicePolicy"));
                    }
                    // E0323 for invalid policy record lit / known ints
                    if let HirExpr::RecordLit { name, fields } = &args[0] {
                        if name == "ServicePolicy" {
                            for (_fn, fe) in fields {
                                if let Some(v) = self.fold_i64(fe) {
                                    if v <= 0 {
                                        return Err(err("E0323", "invalid policy value"));
                                    }
                                }
                            }
                        }
                    }
                    return Ok(HirType::Result(
                        Box::new(HirType::Named("HttpServer".into())),
                        Box::new(HirType::Named("IoError".into())),
                    ));
                }
                ("HttpServer", "policy") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpServer.policy expects no args"));
                    }
                    return Ok(HirType::Named("ServicePolicy".into()));
                }
                ("HttpRequest", "method") | ("HttpRequest", "path") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpRequest accessor expects no args"));
                    }
                    return Ok(HirType::String);
                }
                ("HttpRequest", "header") => {
                    if args.len() != 1 || self.type_of_expr(&args[0])? != HirType::String {
                        return Err(err("E0321", "HttpRequest.header expects (name: Text)"));
                    }
                    return Ok(HirType::Option(Box::new(HirType::String)));
                }
                ("HttpRequest", "body_text") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpRequest.body_text expects no args"));
                    }
                    return Ok(HirType::Result(
                        Box::new(HirType::String),
                        Box::new(HirType::Named("IoError".into())),
                    ));
                }
                ("HttpResponse", "set_header") => {
                    if args.len() != 2
                        || self.type_of_expr(&args[0])? != HirType::String
                        || self.type_of_expr(&args[1])? != HirType::String
                    {
                        return Err(err("E0321", "HttpResponse.set_header expects (Text, Text)"));
                    }
                    return Ok(HirType::Named("HttpResponse".into()));
                }
                ("HttpResponse", "status") => {
                    // ADR-247 instance accessor (associated status(code,body) is 2-arg path above)
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpResponse.status accessor expects no args"));
                    }
                    return Ok(HirType::Int);
                }
                ("HttpResponse", "body_text") => {
                    if !args.is_empty() {
                        return Err(err("E0321", "HttpResponse.body_text expects no args"));
                    }
                    return Ok(HirType::Result(
                        Box::new(HirType::String),
                        Box::new(HirType::Named("IoError".into())),
                    ));
                }
                _ => {}
            }
        }
        let is_vec_or_string = matches!(
            recv_ty,
            HirType::Vec(_) | HirType::String | HirType::Bytes | HirType::Map(_, _)
        );
        match method {
            "as_bytes" => {
                // ADR-236: Text/String → Bytes (copy via as_bytes().to_vec())
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bytes)
            }
            "len" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "capacity" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "binary_search" => {
                // ADR-190: v.binary_search(x) → Result<Int,Int>; Vec only
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Result(
                    Box::new(HirType::Int),
                    Box::new(HirType::Int),
                ))
            }
            "is_sorted" => {
                // ADR-186: v.is_sorted() → Bool; Vec only
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "is_empty" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "is_ascii" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "is_char_boundary" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-143 / E0292: lit or lit-bound i < 0
                if let Some(i) = self.fold_i64(&args[0]) {
                    if i < 0 {
                        return Err(err("E0292", "negative char boundary index"));
                    }
                }
                Ok(HirType::Bool)
            }
            "floor_char_boundary" | "ceil_char_boundary" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-144 / E0292 reuse: lit or lit-bound i < 0
                if let Some(i) = self.fold_i64(&args[0]) {
                    if i < 0 {
                        return Err(err("E0292", "negative char boundary index"));
                    }
                }
                Ok(HirType::Int)
            }
            "push" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "push_str" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::String {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::IoUnit)
            }
            "resize" => {
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Vec(inner) = recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let n_ty = self.type_of_expr(&args[0])?;
                if n_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                let fill_ty = self.type_of_expr(&args[1])?;
                if fill_ty != *inner {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-119 / E0288 reuse: lit or lit-bound n < 0
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0288", "negative truncate"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "truncate" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-118 Vec; ADR-160 String (floor_char_boundary in emit)
                if !matches!(recv_ty, HirType::Vec(_) | HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-118 / E0288: lit or lit-bound n < 0
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0288", "negative truncate"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "clear" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "try_reserve_exact" => {
                // ADR-206: mut v/s.try_reserve_exact(n) → Result<(), Int>; lit n<0 → E0294
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0294", "negative reserve count"));
                    }
                }
                Ok(HirType::Result(
                    Box::new(HirType::Unit),
                    Box::new(HirType::Int),
                ))
            }
            "try_reserve" => {
                // ADR-204: mut v/s.try_reserve(n) → Result<(), Int>; lit n<0 → E0294
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0294", "negative reserve count"));
                    }
                }
                Ok(HirType::Result(
                    Box::new(HirType::Unit),
                    Box::new(HirType::Int),
                ))
            }
            "reserve" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-154 / E0294: lit or lit-bound n < 0
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0294", "negative reserve count"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "shrink_to" => {
                // ADR-208: mut v/s.shrink_to(min_cap) → (); lit min_cap < 0 → E0298
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0298", "negative shrink_to capacity"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "shrink_to_fit" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !is_vec_or_string {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }

            "reverse" => {
                // ADR-182: mut v.reverse() → ()
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "sort_unstable" => {
                // ADR-180: mut v.sort_unstable() → (); Ord elems (Vec<Int>)
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "dedup" => {
                // ADR-176: mut v.dedup() → (); consecutive only
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "fill" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Vec(inner) = recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != *inner {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::IoUnit)
            }
            "swap" => {
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Vec(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let i_ty = self.type_of_expr(&args[0])?;
                let j_ty = self.type_of_expr(&args[1])?;
                if i_ty != HirType::Int || j_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-158 / E0295: lit or lit-bound i|j < 0
                if let Some(i) = self.fold_i64(&args[0]) {
                    if i < 0 {
                        return Err(err("E0295", "negative swap index"));
                    }
                }
                if let Some(j) = self.fold_i64(&args[1]) {
                    if j < 0 {
                        return Err(err("E0295", "negative swap index"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "extend" => {
                // ADR-224: mut a.extend(b) → (); arg shared Vec (b intact)
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Vec(inner) = &recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let arg_ty = self.type_of_expr(&args[0])?;
                match &arg_ty {
                    HirType::Vec(arg_inner) if **arg_inner == **inner => {}
                    _ => return Err(err("E0203", "type mismatch")),
                }
                Ok(HirType::IoUnit)
            }
            "append" => {
                // ADR-222: mut a.append(mut b) → (); both Vec<T>; arg Path must be mut
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Vec(inner) = &recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let arg_ty = self.type_of_expr(&args[0])?;
                match &arg_ty {
                    HirType::Vec(arg_inner) if **arg_inner == **inner => {}
                    _ => return Err(err("E0203", "type mismatch")),
                }
                let HirExpr::Path(p) = &args[0] else {
                    return Err(err("E0203", "type mismatch"));
                };
                let Some(name) = Self::place_name(p) else {
                    return Err(err("E0203", "type mismatch"));
                };
                if let Some(b) = self.bindings.get(name) {
                    if !b.mutable {
                        return Err(err("E0202", "borrow conflict"));
                    }
                }
                Ok(HirType::IoUnit)
            }
            "split_off" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-162 / E0296: lit or lit-bound at < 0
                if let Some(at) = self.fold_i64(&args[0]) {
                    if at < 0 {
                        return Err(err("E0296", "negative split_off index"));
                    }
                }
                match recv_ty {
                    HirType::String => Ok(HirType::String),
                    HirType::Vec(inner) => Ok(HirType::Vec(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "make_ascii_lowercase" | "make_ascii_uppercase" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::IoUnit)
            }
            "pop" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Vec(inner) => Ok(HirType::Option(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "get" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                match &recv_ty {
                    HirType::Vec(inner) => {
                        if arg_ty != HirType::Int {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::Option(Box::new(inner.as_ref().clone())))
                    }
                    HirType::Map(k, v) => {
                        if &arg_ty != k.as_ref() {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::Option(Box::new(v.as_ref().clone())))
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "put" => {
                // ADR-237: Map write (no surface `insert`)
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                self.type_of_map_write(&recv_ty, &args[0], &args[1])
            }
            "insert" => {
                // ADR-260 / ADR-228: Vec.insert(i, x) -> Result<(), Int>; lit i<0 → E0311
                // Map.write stays via `put` (insert on Map remains E0206).
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Vec(inner) = &recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let idx_ty = self.type_of_expr(&args[0])?;
                if idx_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                let val_ty = self.type_of_expr(&args[1])?;
                if &val_ty != inner.as_ref() {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0311", "negative insert index"));
                    }
                }
                Ok(HirType::Result(
                    Box::new(HirType::Unit),
                    Box::new(HirType::Int),
                ))
            }
            "set" => {
                // ADR-265 / ADR-270 / CORE-0.6-SET-FALLIBLE-20260926:
                // Vec.set(i, x) -> Result<(), Int>; lit i<0 → E0319
                // Replace in-bounds only (does NOT grow len, unlike insert).
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                // Order unchanged by ADR-283: types (E0206/E0203) → negative index (E0319).
                let ret = self.type_of_vec_set(&recv_ty, &args[0], &args[1])?;
                self.check_negative_set_index(&args[0])?;
                Ok(ret)
            }
            "swap_remove" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Vec(inner) => Ok(HirType::Option(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "remove" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Vec(inner) => Ok(HirType::Option(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "first" | "last" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Vec(inner) => Ok(HirType::Option(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "clone" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::String => Ok(HirType::String),
                    HirType::Vec(inner) => Ok(HirType::Vec(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "find" | "rfind" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::String {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                match &recv_ty {
                    HirType::String => {
                        if arg_ty != HirType::String {
                            return Err(err("E0203", "type mismatch"));
                        }
                    }
                    HirType::Vec(inner) => {
                        if arg_ty != **inner {
                            return Err(err("E0203", "type mismatch"));
                        }
                    }
                    _ => return Err(err("E0206", "method not in F2 std whitelist")),
                }
                Ok(HirType::Bool)
            }
            "starts_with" => {
                // ADR-196: String unchanged; Vec<T>.starts_with(Vec<T>) → Bool
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                match &recv_ty {
                    HirType::String => {
                        if arg_ty != HirType::String {
                            return Err(err("E0203", "type mismatch"));
                        }
                    }
                    HirType::Vec(inner) => match &arg_ty {
                        HirType::Vec(arg_inner) if **arg_inner == **inner => {}
                        _ => return Err(err("E0203", "type mismatch")),
                    },
                    _ => return Err(err("E0206", "method not in F2 std whitelist")),
                }
                Ok(HirType::Bool)
            }

            "ends_with" => {
                // ADR-196: String unchanged; Vec<T>.ends_with(Vec<T>) → Bool
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                match &recv_ty {
                    HirType::String => {
                        if arg_ty != HirType::String {
                            return Err(err("E0203", "type mismatch"));
                        }
                    }
                    HirType::Vec(inner) => match &arg_ty {
                        HirType::Vec(arg_inner) if **arg_inner == **inner => {}
                        _ => return Err(err("E0203", "type mismatch")),
                    },
                    _ => return Err(err("E0206", "method not in F2 std whitelist")),
                }
                Ok(HirType::Bool)
            }

            "eq_ignore_ascii_case" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::String {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Bool)
            }
            "trim" | "trim_start" | "trim_end" | "trim_ascii" | "trim_ascii_start"
            | "trim_ascii_end" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::String)
            }
            "to_uppercase" | "to_lowercase" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::String)
            }
            "to_ascii_lowercase" | "to_ascii_uppercase" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::String)
            }
            "parse_int" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Result(
                    Box::new(HirType::Int),
                    Box::new(HirType::String),
                ))
            }
            "to_string" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int | HirType::Bool) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::String)
            }
            "transpose" => {
                // ADR-200: Option<Result<T,E>> → Result<Option<T>,E>
                //          Result<Option<T>,E> → Option<Result<T,E>>
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Option(inner) => {
                        let HirType::Result(payload, err_ty) = *inner else {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        };
                        Ok(HirType::Result(Box::new(HirType::Option(payload)), err_ty))
                    }
                    HirType::Result(inner, err_ty) => {
                        let HirType::Option(payload) = *inner else {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        };
                        Ok(HirType::Option(Box::new(HirType::Result(payload, err_ty))))
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "flatten" => {
                // ADR-184: Option<Option<T>> → Option<T>
                // ADR-198: Result<Result<T,E>,E> → Result<T,E> (same E)
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Option(inner) => {
                        let HirType::Option(payload) = *inner else {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        };
                        Ok(HirType::Option(payload))
                    }
                    HirType::Result(inner, e_outer) => {
                        let HirType::Result(payload, e_inner) = *inner else {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        };
                        if e_outer != e_inner {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        }
                        Ok(HirType::Result(payload, e_outer))
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "then_some" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Bool) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                // ADR-164: Option of arg type (Int/Bool/String/Unit surface)
                match arg_ty {
                    HirType::Int | HirType::Bool | HirType::String | HirType::Unit => {}
                    _ => return Err(err("E0203", "type mismatch")),
                }
                Ok(HirType::Option(Box::new(arg_ty)))
            }

            "wrapping_neg" | "saturating_neg" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "is_positive" | "is_negative" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "signum" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "abs" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                if self.fold_i64(receiver) == Some(i64::MIN) {
                    return Err(err("E0278", "integer abs overflow"));
                }
                Ok(HirType::Int)
            }
            "count_ones" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "count_zeros" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "leading_zeros" | "trailing_zeros" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "leading_ones" | "trailing_ones" => {
                // ADR-216: n.leading_ones/trailing_ones() → Int
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "reverse_bits" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "rotate_left" | "rotate_right" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Int => {
                        // ADR-131 / E0291: lit or lit-bound n < 0
                        if let Some(n) = self.fold_i64(&args[0]) {
                            if n < 0 {
                                return Err(err("E0291", "negative rotate count"));
                            }
                        }
                        Ok(HirType::Int)
                    }
                    HirType::Vec(_) => {
                        // ADR-170 / E0297: lit or lit-bound n < 0
                        if let Some(n) = self.fold_i64(&args[0]) {
                            if n < 0 {
                                return Err(err("E0297", "negative vec rotate count"));
                            }
                        }
                        Ok(HirType::IoUnit)
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "swap_bytes" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "to_be" | "to_le" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "from_be" | "from_le" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Int)
            }
            "ilog10" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "ilog2" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "previous_multiple_of" => {
                // ADR-194: n.previous_multiple_of(m) → Int; lit m==0 → E0216
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Int)
            }
            "checked_previous_multiple_of" => {
                // ADR-194: n.checked_previous_multiple_of(m) → Option<Int>; lit m==0 → E0216
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "next_multiple_of" => {
                // ADR-192: n.next_multiple_of(m) → Int; lit m==0 → E0216
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Int)
            }
            "checked_next_multiple_of" => {
                // ADR-192: n.checked_next_multiple_of(m) → Option<Int>; lit m==0 → E0216
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_next_power_of_two" => {
                // ADR-220: n.checked_next_power_of_two() → Option<Int>; n≤0/overflow → None
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "is_power_of_two" => {
                // ADR-218: n.is_power_of_two() → Bool
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "is_multiple_of" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-117: d==0 → false at runtime (no E0216)
                Ok(HirType::Bool)
            }
            "abs_diff" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "midpoint" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "min" | "max" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "clamp" => {
                if args.len() != 2 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let lo_ty = self.type_of_expr(&args[0])?;
                let hi_ty = self.type_of_expr(&args[1])?;
                if lo_ty != HirType::Int || hi_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let (Some(lo), Some(hi)) = (self.fold_i64(&args[0]), self.fold_i64(&args[1])) {
                    if lo > hi {
                        return Err(err("E0279", "invalid clamp range"));
                    }
                }
                Ok(HirType::Int)
            }
            "strip_prefix" | "strip_suffix" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::String) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::String {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::String)))
            }
            "repeat" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let ret_ty = match &recv_ty {
                    HirType::String => HirType::String,
                    HirType::Vec(inner) => HirType::Vec(inner.clone()),
                    _ => return Err(err("E0206", "method not in F2 std whitelist")),
                };
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(n) = self.fold_i64(&args[0]) {
                    if n < 0 {
                        return Err(err("E0280", "negative repeat count"));
                    }
                }
                Ok(ret_ty)
            }
            "pow" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if let Some(exp) = self.fold_i64(&args[0]) {
                    if exp < 0 {
                        return Err(err("E0281", "negative pow exponent"));
                    }
                    if let Some(base) = self.fold_i64(receiver) {
                        if exp > u32::MAX as i64 || base.checked_pow(exp as u32).is_none() {
                            return Err(err("E0283", "integer pow overflow"));
                        }
                    }
                }
                Ok(HirType::Int)
            }
            "checked_neg" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_add" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_sub" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_mul" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_div" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_rem" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "checked_shl" | "checked_shr" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-150 / E0293: lit or lit-bound s < 0
                if let Some(s) = self.fold_i64(&args[0]) {
                    if s < 0 {
                        return Err(err("E0293", "negative shift count"));
                    }
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "wrapping_shl" | "wrapping_shr" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-151 / E0293 reuse: lit or lit-bound s < 0
                if let Some(s) = self.fold_i64(&args[0]) {
                    if s < 0 {
                        return Err(err("E0293", "negative shift count"));
                    }
                }
                Ok(HirType::Int)
            }
            "saturating_add" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("saturating_add", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "saturating_sub" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("saturating_sub", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "saturating_mul" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("saturating_mul", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }

            "wrapping_add" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("wrapping_add", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }

            "wrapping_sub" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("wrapping_sub", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }

            "checked_div_euclid" | "checked_rem_euclid" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Option(Box::new(HirType::Int)))
            }
            "wrapping_div" | "wrapping_rem" => {
                // ADR-212: wrapping_div/rem → Int; lit b==0 → E0216; MIN/-1 → MIN/0 (no E0217)
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Int)
            }
            "saturating_rem" => {
                // ADR-214: a.saturating_rem(b) → Int; lit b==0 → E0216; MIN/-1 → 0
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Int)
            }
            "saturating_div" => {
                // ADR-210: a.saturating_div(b) → Int; lit b==0 → E0216; MIN/-1 → MAX (no E0217)
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                Ok(HirType::Int)
            }
            "div_ceil" | "div_floor" => {
                // ADR-188: a.div_ceil/div_floor(b) → Int; E0216 zero / E0217 MIN÷−1
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                if self.fold_i64(receiver) == Some(i64::MIN) && self.fold_i64(&args[0]) == Some(-1)
                {
                    return Err(err("E0217", "integer overflow"));
                }
                Ok(HirType::Int)
            }
            "div_euclid" | "rem_euclid" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-110 / E0216: lit divisor 0
                if self.fold_i64(&args[0]) == Some(0) {
                    return Err(err("E0216", "integer division by zero"));
                }
                // ADR-110 / E0217: MIN.div_euclid(-1) overflows i64
                if method == "div_euclid"
                    && self.fold_i64(receiver) == Some(i64::MIN)
                    && self.fold_i64(&args[0]) == Some(-1)
                {
                    return Err(err("E0217", "integer overflow"));
                }
                Ok(HirType::Int)
            }
            "wrapping_mul" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-102 / E0286: identity wrapping/saturating is vacuous theater
                if is_vacuous_wrap_sat_identity("wrapping_mul", args) {
                    return Err(err("E0286", "vacuous wrapping/saturating assert"));
                }
                if !matches!(recv_ty, HirType::Int) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                if arg_ty != HirType::Int {
                    return Err(err("E0203", "type mismatch"));
                }
                Ok(HirType::Int)
            }
            "is_some" | "is_none" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Option(_)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "ok_or" => {
                // ADR-202: Option<T>.ok_or(e) → Result<T, E>
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Option(inner) = recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                let err_ty = self.type_of_expr(&args[0])?;
                match &err_ty {
                    HirType::Int | HirType::Bool | HirType::String | HirType::Unit => {}
                    _ => return Err(err("E0203", "type mismatch")),
                }
                Ok(HirType::Result(inner, Box::new(err_ty)))
            }
            "ok" => {
                // ADR-202: Result<T,E>.ok() → Option<T>
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Result(ok_ty, _) = recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                Ok(HirType::Option(ok_ty))
            }
            "err" => {
                // ADR-202: Result<T,E>.err() → Option<E>
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                let HirType::Result(_, err_ty) = recv_ty else {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                };
                Ok(HirType::Option(err_ty))
            }
            "is_ok" | "is_err" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                if !matches!(recv_ty, HirType::Result(_, _)) {
                    return Err(err("E0206", "method not in F2 std whitelist"));
                }
                Ok(HirType::Bool)
            }
            "unwrap_or" => {
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                // ADR-101 / E0285: checked_* → Option + unwrap_or(lit) swallows overflow
                if is_const_lit(&args[0]) {
                    let from_checked = match receiver {
                        HirExpr::MethodCall { method: m, .. } if is_checked_arith_method(m) => true,
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_checked_arith)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_checked {
                        return Err(err("E0285", "checked overflow defaulted away"));
                    }
                    let from_get = match receiver {
                        HirExpr::MethodCall { method: m, .. } if m == "get" => true,
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_vec_get)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_get {
                        return Err(err("E0287", "vec get miss defaulted as success"));
                    }
                    let from_find = match receiver {
                        HirExpr::MethodCall { method: m, .. } if m == "find" || m == "rfind" => {
                            true
                        }
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_find_idx)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_find && self.fold_i64(&args[0]) == Some(0) {
                        return Err(err("E0289", "find miss defaulted to index zero"));
                    }
                    let from_ilog_m = match receiver {
                        HirExpr::MethodCall { method: m, .. } if m == "ilog2" || m == "ilog10" => {
                            true
                        }
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_ilog)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_ilog_m && self.fold_i64(&args[0]) == Some(0) {
                        return Err(err("E0290", "ilog none defaulted to zero"));
                    }
                    // ADR-272 / E0340: host.read_text|write_text Result + unwrap_or(lit) theater
                    let from_host = match receiver {
                        HirExpr::Call(c) if host_io_callee(&c.callee) => true,
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_host_io)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_host {
                        return Err(err("E0340", "io result discarded"));
                    }
                    // ADR-273 / E0341: host.cli_arg|json_get_int Option + unwrap_or(lit) theater
                    let from_cli = match receiver {
                        HirExpr::Call(c) if host_cli_json_callee(&c.callee) => true,
                        HirExpr::Path(p) => Self::place_name(p)
                            .and_then(|n| self.bindings.get(n))
                            .map(|b| b.from_host_cli_json)
                            .unwrap_or(false),
                        _ => false,
                    };
                    if from_cli {
                        return Err(err("E0341", "required arg miss as ok"));
                    }
                }
                let arg_ty = self.type_of_expr(&args[0])?;
                match &recv_ty {
                    HirType::Option(inner) => {
                        if arg_ty != **inner {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok((**inner).clone())
                    }
                    HirType::Result(ok, _) => {
                        if arg_ty != **ok {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok((**ok).clone())
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }

            "or" | "and" | "xor" => {
                // ADR-172 Option.or/and/xor; ADR-174 Result.or/and (xor VOID → E0206)
                if args.len() != 1 {
                    return Err(err("E0203", "type mismatch"));
                }
                let other_ty = self.type_of_expr(&args[0])?;
                match recv_ty {
                    HirType::Option(inner) => {
                        let HirType::Option(other_inner) = other_ty else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        if other_inner != inner {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::Option(inner))
                    }
                    HirType::Result(ok, err_ty) => {
                        // ADR-174: xor not in Rust Result std
                        if method == "xor" {
                            return Err(err("E0206", "method not in F2 std whitelist"));
                        }
                        let HirType::Result(other_ok, other_err) = other_ty else {
                            return Err(err("E0203", "type mismatch"));
                        };
                        if other_ok != ok || other_err != err_ty {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::Result(ok, err_ty))
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "take" => {
                if !args.is_empty() {
                    return Err(err("E0203", "type mismatch"));
                }
                match recv_ty {
                    HirType::Option(inner) => Ok(HirType::Option(inner)),
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            "replace" => {
                match recv_ty {
                    HirType::Option(inner) => {
                        // ADR-168: mut o.replace(x) → Option (previous)
                        if args.len() != 1 {
                            return Err(err("E0203", "type mismatch"));
                        }
                        let arg_ty = self.type_of_expr(&args[0])?;
                        if arg_ty != *inner {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::Option(inner))
                    }
                    HirType::String => {
                        if args.len() != 2 {
                            return Err(err("E0203", "type mismatch"));
                        }
                        let from_ty = self.type_of_expr(&args[0])?;
                        let to_ty = self.type_of_expr(&args[1])?;
                        if from_ty != HirType::String || to_ty != HirType::String {
                            return Err(err("E0203", "type mismatch"));
                        }
                        Ok(HirType::String)
                    }
                    _ => Err(err("E0206", "method not in F2 std whitelist")),
                }
            }
            _ => Err(err("E0206", "method not in F2 std whitelist")),
        }
    }

    fn require_bool_cond(&self, cond: &HirExpr) -> Result<(), CheckError> {
        let ty = self.type_of_expr(cond)?;
        if ty != HirType::Bool {
            return Err(err("E0220", "if/while condition type ≠ Bool"));
        }
        Ok(())
    }

    /// ADR-294 (B-292-2, option A): forget `known_int` of every name assigned or re-declared
    /// (`let`) anywhere inside `body`. Only ever clears, so it can remove an E0216/E0217-class
    /// rejection but never create one.
    fn known_clear_clobbered(&mut self, body: &[HirStmt]) {
        let mut names = std::collections::HashSet::<String>::new();
        collect_clobbered(body, &mut names);
        self.known_clear(&names);
    }

    fn known_clear(&mut self, names: &std::collections::HashSet<String>) {
        for n in names {
            if let Some(b) = self.bindings.get_mut(n.as_str()) {
                b.known_int = None;
            }
        }
    }

    /// ADR-294: entering the next sibling arm of a `match`: drop what the earlier arms clobbered,
    /// then record this arm's own clobbers for the arms that follow.
    fn known_enter_sibling(
        &mut self,
        seen: &mut std::collections::HashSet<String>,
        body: &[HirStmt],
    ) {
        self.known_clear(seen);
        collect_clobbered(body, seen);
    }

    /// ADR-294: after an `if` / `if let` / `while` / `while let` / `match`, no `known_int` of a
    /// name touched inside it survives (no path merge; conservative).
    fn check_stmt(&mut self, s: &HirStmt) -> Result<(), CheckError> {
        let r = self.check_stmt_inner(s);
        if matches!(
            s,
            HirStmt::If { .. }
                | HirStmt::IfLet { .. }
                | HirStmt::While { .. }
                | HirStmt::WhileLet { .. }
                | HirStmt::Match { .. }
        ) {
            self.known_clear_clobbered(std::slice::from_ref(s));
        }
        r
    }

    fn check_stmt_inner(&mut self, s: &HirStmt) -> Result<(), CheckError> {
        match s {
            HirStmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                // First evaluate init (move context for Path RHS)
                self.eval_expr(init, true)?;
                // ADR-047: Result let — check Ok/Err payload vs T/E (E0203)
                if let HirType::Result(ok_ty, err_ty) = ty {
                    match init {
                        HirExpr::Ok(payload) => {
                            let pt = self.type_of_expr(payload)?;
                            if pt != **ok_ty {
                                return Err(err("E0203", "type mismatch"));
                            }
                        }
                        HirExpr::Err(payload) => {
                            let pt = self.type_of_expr(payload)?;
                            if pt != **err_ty {
                                return Err(err("E0203", "type mismatch"));
                            }
                        }
                        other => {
                            let it = self.type_of_expr(other)?;
                            if it != *ty {
                                return Err(err("E0203", "type mismatch"));
                            }
                        }
                    }
                }
                if let HirType::Option(inner_ty) = ty {
                    match init {
                        HirExpr::Some(payload) => {
                            let pt = self.type_of_expr(payload)?;
                            if pt != **inner_ty {
                                return Err(err("E0203", "type mismatch"));
                            }
                        }
                        HirExpr::None => {
                            // None synthesizes Option<Unit> in type_of; require Unit payload type
                            // or allow None for any Option via expected ty (pin: accept for any T).
                        }
                        other => {
                            let it = self.type_of_expr(other)?;
                            if it != *ty {
                                return Err(err("E0203", "type mismatch"));
                            }
                        }
                    }
                }
                // ADR-233: record let annotation must match construct/access type
                if matches!(ty, HirType::Named(_)) {
                    let it = self.type_of_expr(init)?;
                    if it != *ty {
                        return Err(err("E0203", "type mismatch"));
                    }
                }
                let known_int = match init {
                    HirExpr::LitInt(n) => Some(*n),
                    _ => None,
                };
                let from_checked_arith = expr_from_checked_arith(init);
                let from_vec_get = expr_from_vec_get(init);
                let from_find_idx = expr_from_find_idx(init);
                let from_ilog = expr_from_ilog(init);
                let from_host_io = expr_from_host_io(init);
                let from_host_cli_json = expr_from_host_cli_json(init);
                // ADR-272: `let _ = host.read_text|write_text(...)` discards Result → E0340
                if name == "_" && from_host_io {
                    return Err(err("E0340", "io result discarded"));
                }
                self.bindings.insert(
                    name.clone(),
                    BindingState {
                        ty: ty.clone(),
                        mutable: *mutable,
                        moved: false,
                        known_int,
                        from_checked_arith,
                        from_vec_get,
                        from_find_idx,
                        from_ilog,
                        from_host_io,
                        from_host_cli_json,
                    },
                );
                Ok(())
            }
            HirStmt::Expr(e) => {
                self.eval_expr(e, false)?;
                // ADR-272: bare host.read_text|write_text Result as stmt → discard → E0340
                if expr_is_host_io_call(e) {
                    return Err(err("E0340", "io result discarded"));
                }
                Ok(())
            }
            HirStmt::Assert { lhs, rhs } => {
                self.eval_expr(lhs, false)?;
                self.eval_expr(rhs, false)?;
                Ok(())
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                self.eval_expr(cond, false)?;
                self.require_bool_cond(cond)?;
                // ADR-041 / E0227: LitBool `false` only, no else — unreachable then theater
                if else_body.is_none() && matches!(cond, HirExpr::LitBool(false)) {
                    return Err(err("E0227", "vacuous if false"));
                }
                self.check_body(then_body)?;
                if let Some(eb) = else_body {
                    // ADR-294: sibling branch must not see known_int set by the `then` body.
                    self.known_clear_clobbered(then_body);
                    self.check_body(eb)?;
                }
                Ok(())
            }
            HirStmt::IfLet {
                pat,
                scrutinee,
                then_body,
                else_body,
            } => {
                self.eval_expr(scrutinee, false)?;
                let sty = self.type_of_expr(scrutinee)?;
                if else_body.is_none() {
                    return Err(err("E0275", "if-let without else"));
                }
                let (binding, bind_ty) = match (&sty, pat) {
                    (HirType::Option(inner), HirPat::Some(n)) => (n.clone(), (**inner).clone()),
                    (HirType::Result(ok_ty, _), HirPat::Ok(n)) => (n.clone(), (**ok_ty).clone()),
                    (HirType::Result(_, err_ty), HirPat::Err(n)) => (n.clone(), (**err_ty).clone()),
                    (HirType::Option(_), _) | (HirType::Result(_, _), _) => {
                        return Err(err("E0203", "type mismatch"));
                    }
                    _ => return Err(err("E0203", "type mismatch")),
                };
                let old = self.bindings.insert(
                    binding.clone(),
                    BindingState {
                        ty: bind_ty,
                        mutable: false,
                        moved: false,
                        known_int: None,
                        from_checked_arith: false,
                        from_vec_get: false,
                        from_find_idx: false,
                        from_ilog: false,
                        from_host_io: false,
                        from_host_cli_json: false,
                    },
                );
                self.check_body(then_body)?;
                if let Some(b) = old {
                    self.bindings.insert(binding.clone(), b);
                } else {
                    self.bindings.remove(&binding);
                }
                if let Some(eb) = else_body {
                    // ADR-294: sibling branch must not see known_int set by the `then` body.
                    self.known_clear_clobbered(then_body);
                    self.check_body(eb)?;
                }
                Ok(())
            }
            HirStmt::WhileLet {
                pat,
                scrutinee,
                body,
            } => {
                // ADR-294: scrutinee and body re-run after the assignments below.
                self.known_clear_clobbered(body);
                self.eval_expr(scrutinee, false)?;
                // ADR-056 / E0277: lit None scrutinee — always-absent theater
                if matches!(scrutinee, HirExpr::None) {
                    return Err(err("E0277", "vacuous while-let none"));
                }
                // ADR-078 / E0282: lit Result ctor never matches while-let pat
                let vacuous_result = match (pat, scrutinee) {
                    (HirPat::Ok(_), HirExpr::Err(inner)) | (HirPat::Err(_), HirExpr::Ok(inner)) => {
                        matches!(
                            inner.as_ref(),
                            HirExpr::LitInt(_)
                                | HirExpr::LitBool(_)
                                | HirExpr::LitStr(_)
                                | HirExpr::LitUnit
                        )
                    }
                    _ => false,
                };
                if vacuous_result {
                    return Err(err("E0282", "vacuous while-let result"));
                }
                let sty = self.type_of_expr(scrutinee)?;
                let (binding, bind_ty) = match (&sty, pat) {
                    (HirType::Option(inner), HirPat::Some(n)) => (n.clone(), (**inner).clone()),
                    (HirType::Result(ok_ty, _), HirPat::Ok(n)) => (n.clone(), (**ok_ty).clone()),
                    (HirType::Result(_, err_ty), HirPat::Err(n)) => (n.clone(), (**err_ty).clone()),
                    (HirType::Option(_), _) | (HirType::Result(_, _), _) => {
                        return Err(err("E0203", "type mismatch"));
                    }
                    _ => return Err(err("E0203", "type mismatch")),
                };
                self.loop_depth += 1;
                let old = self.bindings.insert(
                    binding.clone(),
                    BindingState {
                        ty: bind_ty,
                        mutable: false,
                        moved: false,
                        known_int: None,
                        from_checked_arith: false,
                        from_vec_get: false,
                        from_find_idx: false,
                        from_ilog: false,
                        from_host_io: false,
                        from_host_cli_json: false,
                    },
                );
                let r = self.check_body(body);
                if let Some(b) = old {
                    self.bindings.insert(binding.clone(), b);
                } else {
                    self.bindings.remove(&binding);
                }
                self.loop_depth -= 1;
                r
            }
            HirStmt::While { cond, body } => {
                // ADR-294: cond and body re-run after the assignments below; drop known_int.
                self.known_clear_clobbered(body);
                self.eval_expr(cond, false)?;
                self.require_bool_cond(cond)?;
                // ADR-040 / E0226: LitBool `false` only — unreachable body theater
                if matches!(cond, HirExpr::LitBool(false)) {
                    return Err(err("E0226", "vacuous while false"));
                }
                self.loop_depth += 1;
                let r = self.check_body(body);
                self.loop_depth -= 1;
                r
            }
            HirStmt::Break | HirStmt::Continue => {
                if self.loop_depth == 0 {
                    return Err(err("E0224", "break/continue outside while"));
                }
                Ok(())
            }
            HirStmt::Assign { name, value } => {
                self.eval_expr(value, true)?;
                let Some(b) = self.bindings.get_mut(name) else {
                    // unbound assign: not ownership error in v0
                    return Ok(());
                };
                if b.moved {
                    return Err(err("E0201", "use of moved value"));
                }
                if !b.mutable {
                    return Err(err("E0205", "invalid let / mut binding"));
                }
                // ADR-045: refresh / clear lit fold knowledge on mut assign
                b.known_int = match value {
                    HirExpr::LitInt(n) => Some(*n),
                    _ => None,
                };
                Ok(())
            }
            HirStmt::IndexAssign {
                target,
                key,
                value,
                op,
                span,
            } => {
                let is_map = matches!(
                    self.bindings.get(target.as_str()).map(|b| &b.ty),
                    Some(HirType::Map(_, _))
                );
                let r = self.check_index_assign(target, key, value, op.as_ref(), span);
                // ADR-290 D6: a compound on a Map is the (former parser) E0006, so it also
                // wins over a deferred shape error, like every non-Map index-assign failure.
                if r.is_err() && (!is_map || op.is_some()) {
                    self.index_assign_failed_span = Some(span.clone());
                }
                r
            }
            HirStmt::Match { scrutinee, arms } => {
                self.eval_expr(scrutinee, false)?;
                let sty = self.type_of_expr(scrutinee)?;
                match &sty {
                    HirType::Result(ok_ty, err_ty) => {
                        let mut has_ok = false;
                        let mut has_err = false;
                        let mut sib = std::collections::HashSet::<String>::new();
                        for arm in arms {
                            self.known_enter_sibling(&mut sib, &arm.body);
                            let bind = match &arm.pat {
                                HirPat::Ok(n) => {
                                    has_ok = true;
                                    Some((n.clone(), ok_ty.as_ref().clone()))
                                }
                                HirPat::Err(n) => {
                                    has_err = true;
                                    Some((n.clone(), err_ty.as_ref().clone()))
                                }
                                HirPat::Wildcard
                                | HirPat::LitBool(_)
                                | HirPat::LitInt(_)
                                | HirPat::Some(_)
                                | HirPat::None
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            };
                            let prev = if let Some((ref n, ref bt)) = bind {
                                let old = self.bindings.insert(
                                    n.clone(),
                                    BindingState {
                                        ty: bt.clone(),
                                        mutable: false,
                                        moved: false,
                                        known_int: None,
                                        from_checked_arith: false,
                                        from_vec_get: false,
                                        from_find_idx: false,
                                        from_ilog: false,
                                        from_host_io: false,
                                        from_host_cli_json: false,
                                    },
                                );
                                Some((n.clone(), old))
                            } else {
                                None
                            };
                            self.check_body(&arm.body)?;
                            if let Some((n, old)) = prev {
                                if let Some(b) = old {
                                    self.bindings.insert(n, b);
                                } else {
                                    self.bindings.remove(&n);
                                }
                            }
                        }
                        if !(has_ok && has_err) {
                            return Err(err("E0270", "non-exhaustive result match"));
                        }
                        // ADR-272: host IO Result — E0340 for empty/default Err theater; skip E0272
                        // so explicit fail messages (print("read_failed")) remain OK.
                        let host_io_scrut = expr_from_host_io(scrutinee)
                            || matches!(
                                scrutinee,
                                HirExpr::Path(p) if Self::place_name(p)
                                    .and_then(|n| self.bindings.get(n))
                                    .map(|b| b.from_host_io)
                                    .unwrap_or(false)
                            );
                        if host_io_scrut {
                            if host_io_result_err_theater_stmt(arms) {
                                return Err(err("E0340", "io result discarded"));
                            }
                        } else if result_err_swallowed_stmt(arms) || result_err_dead_sink_stmt(arms)
                        {
                            // ADR-048 v1 · ADR-286 v2 (dead sink in `Err` arm), same pass/order.
                            return Err(err("E0272", "result error swallowed"));
                        }
                        if vacuous_match_stmt_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        Ok(())
                    }
                    HirType::Option(inner_ty) => {
                        let mut has_some = false;
                        let mut has_none = false;
                        let mut sib = std::collections::HashSet::<String>::new();
                        for arm in arms {
                            self.known_enter_sibling(&mut sib, &arm.body);
                            let bind = match &arm.pat {
                                HirPat::Some(n) => {
                                    has_some = true;
                                    Some((n.clone(), inner_ty.as_ref().clone()))
                                }
                                HirPat::None => {
                                    has_none = true;
                                    None
                                }
                                HirPat::Wildcard
                                | HirPat::LitBool(_)
                                | HirPat::LitInt(_)
                                | HirPat::Ok(_)
                                | HirPat::Err(_)
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            };
                            let prev = if let Some((ref n, ref bt)) = bind {
                                let old = self.bindings.insert(
                                    n.clone(),
                                    BindingState {
                                        ty: bt.clone(),
                                        mutable: false,
                                        moved: false,
                                        known_int: None,
                                        from_checked_arith: false,
                                        from_vec_get: false,
                                        from_find_idx: false,
                                        from_ilog: false,
                                        from_host_io: false,
                                        from_host_cli_json: false,
                                    },
                                );
                                Some((n.clone(), old))
                            } else {
                                None
                            };
                            self.check_body(&arm.body)?;
                            if let Some((n, old)) = prev {
                                if let Some(b) = old {
                                    self.bindings.insert(n, b);
                                } else {
                                    self.bindings.remove(&n);
                                }
                            }
                        }
                        if !(has_some && has_none) {
                            return Err(err("E0273", "non-exhaustive option match"));
                        }
                        // ADR-273: host cli/json Option — E0341 for empty/default None theater; skip E0274
                        // so explicit fail messages (print("usage") / print("missing_n")) remain OK.
                        let host_cli_scrut = expr_from_host_cli_json(scrutinee)
                            || matches!(
                                scrutinee,
                                HirExpr::Path(p) if Self::place_name(p)
                                    .and_then(|n| self.bindings.get(n))
                                    .map(|b| b.from_host_cli_json)
                                    .unwrap_or(false)
                            );
                        if host_cli_scrut {
                            if host_cli_none_theater_stmt(arms) {
                                return Err(err("E0341", "required arg miss as ok"));
                            }
                        } else if option_none_swallowed_stmt(arms) {
                            return Err(err("E0274", "option none swallowed"));
                        }
                        if vacuous_match_stmt_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        Ok(())
                    }
                    HirType::Bool | HirType::Int => {
                        let mut has_true = false;
                        let mut has_false = false;
                        let mut has_wild = false;
                        let mut sib = std::collections::HashSet::<String>::new();
                        for arm in arms {
                            self.known_enter_sibling(&mut sib, &arm.body);
                            match &arm.pat {
                                HirPat::Wildcard => has_wild = true,
                                HirPat::LitBool(true) => {
                                    if sty != HirType::Bool {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                    has_true = true;
                                }
                                HirPat::LitBool(false) => {
                                    if sty != HirType::Bool {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                    has_false = true;
                                }
                                HirPat::LitInt(_) => {
                                    if sty != HirType::Int {
                                        return Err(err(
                                            "E0223",
                                            "match pattern/scrutinee type mismatch",
                                        ));
                                    }
                                }
                                HirPat::Ok(_)
                                | HirPat::Err(_)
                                | HirPat::Some(_)
                                | HirPat::None
                                | HirPat::Variant { .. } => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            self.check_body(&arm.body)?;
                        }
                        if sty == HirType::Bool && !(has_wild || (has_true && has_false)) {
                            return Err(err("E0221", "non-exhaustive Bool match"));
                        } else if sty == HirType::Int && !has_wild {
                            return Err(err("E0222", "Int match requires `_` arm"));
                        }
                        // ADR-031 / E0225: stmt arms all `{ lit }` same constant
                        if vacuous_match_stmt_arms(arms) {
                            return Err(err("E0225", "vacuous match arms"));
                        }
                        Ok(())
                    }
                    HirType::Named(ename) if self.enums.contains_key(ename) => {
                        let variants = self.enums.get(ename).unwrap();
                        let mut covered: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        let mut has_wild = false;
                        let mut sib = std::collections::HashSet::<String>::new();
                        for arm in arms {
                            self.known_enter_sibling(&mut sib, &arm.body);
                            match &arm.pat {
                                HirPat::Wildcard => has_wild = true,
                                HirPat::Variant { enum_name, variant } => {
                                    if let Some(en) = enum_name {
                                        if en != ename {
                                            return Err(err(
                                                "E0223",
                                                "match pattern/scrutinee type mismatch",
                                            ));
                                        }
                                    }
                                    if !variants.iter().any(|v| v == variant) {
                                        return Err(err("E0317", "unknown variant"));
                                    }
                                    covered.insert(variant.clone());
                                }
                                _ => {
                                    return Err(err(
                                        "E0223",
                                        "match pattern/scrutinee type mismatch",
                                    ));
                                }
                            }
                            self.check_body(&arm.body)?;
                        }
                        if !(has_wild || variants.iter().all(|v| covered.contains(v))) {
                            return Err(err("E0316", "non-exhaustive enum match"));
                        }
                        Ok(())
                    }
                    _ => Err(err("E0223", "match pattern/scrutinee type mismatch")),
                }
            }
        }
    }

    fn check_body(&mut self, body: &[HirStmt]) -> Result<(), CheckError> {
        // ADR-295: `tail_ret` on entry says whether this body is in return-tail position.
        let body_tail = self.tail_ret;
        let r = self.check_body_stmts(body, body_tail);
        self.tail_ret = body_tail;
        r
    }

    fn check_body_stmts(&mut self, body: &[HirStmt], body_tail: bool) -> Result<(), CheckError> {
        for (i, s) in body.iter().enumerate() {
            let is_tail = body_tail && i + 1 == body.len();
            // Only the bodies of a tail `if`/`else`, `if let`/`else` or `match` inherit the tail.
            self.tail_ret = is_tail && stmt_propagates_tail(s);
            self.check_stmt(s)?;
            self.must_use_after_stmt(s, &body[i + 1..], body_tail)?;
        }
        Ok(())
    }

    /// ADR-295 (S2 MUST-USE, E0272): runs after `check_stmt` accepted `s` (so the RHS error, E0347,
    /// E0203 and E0340 win). `rest` = the statements after `s` in the same block; `body_tail` =
    /// the block is in return-tail position. Purely lexical: no `BindingState` / `known_int`.
    fn must_use_after_stmt(
        &self,
        s: &HirStmt,
        rest: &[HirStmt],
        body_tail: bool,
    ) -> Result<(), CheckError> {
        match s {
            // P2(a) `let _: Result<..> = e` and P2(b) a `Result` binding never used afterwards.
            HirStmt::Let {
                name,
                ty: HirType::Result(_, _),
                ..
            } => {
                if name == "_" || !must_use_binding_used(name, rest, body_tail) {
                    return Err(err("E0272", "result error swallowed"));
                }
                Ok(())
            }
            // P2(c) a `Result`-typed expression statement that is not the return tail.
            HirStmt::Expr(e) if !(body_tail && rest.is_empty()) => {
                if matches!(self.type_of_expr(e), Ok(HirType::Result(_, _))) {
                    return Err(err("E0272", "result error swallowed"));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn is_vacuous_wrap_sat_identity(method: &str, args: &[HirExpr]) -> bool {
    args.len() == 1
        && matches!(
            (method, &args[0]),
            (
                "wrapping_add" | "saturating_add" | "wrapping_sub" | "saturating_sub",
                HirExpr::LitInt(0),
            ) | ("wrapping_mul" | "saturating_mul", HirExpr::LitInt(1))
        )
}

fn is_checked_arith_method(method: &str) -> bool {
    matches!(
        method,
        "checked_add"
            | "checked_sub"
            | "checked_mul"
            | "checked_div"
            | "checked_rem"
            | "checked_neg"
            | "checked_div_euclid"
            | "checked_rem_euclid"
            | "checked_shl"
            | "checked_shr"
    )
}

fn expr_from_vec_get(e: &HirExpr) -> bool {
    matches!(e, HirExpr::MethodCall { method, .. } if method == "get")
}

fn expr_from_find_idx(e: &HirExpr) -> bool {
    matches!(e, HirExpr::MethodCall { method, .. } if method == "find" || method == "rfind")
}

fn expr_from_ilog(e: &HirExpr) -> bool {
    matches!(e, HirExpr::MethodCall { method, .. } if method == "ilog2" || method == "ilog10")
}

/// ADR-272: callee is host.read_text or host.write_text (HIR `host::<fn>`).
fn host_io_callee(callee: &str) -> bool {
    let name = callee.strip_prefix("host::").unwrap_or(callee);
    name == "read_text" || name == "write_text"
}

fn expr_is_host_io_call(e: &HirExpr) -> bool {
    matches!(e, HirExpr::Call(c) if host_io_callee(&c.callee))
}

fn expr_from_host_io(e: &HirExpr) -> bool {
    expr_is_host_io_call(e)
}

/// ADR-272: empty / default lit theater on Err arm (not explicit fail message).
fn host_io_empty_theater_expr(e: &HirExpr) -> bool {
    match e {
        HirExpr::LitStr(s) if s.is_empty() => true,
        HirExpr::LitInt(0) | HirExpr::LitUnit => true,
        HirExpr::Call(c) if c.callee == "print" && c.args.len() == 1 => {
            matches!(
                &c.args[0],
                HirExpr::LitStr(s) if s.is_empty()
            ) || matches!(&c.args[0], HirExpr::LitInt(0) | HirExpr::LitUnit)
        }
        _ => false,
    }
}

fn host_io_empty_theater_stmt_body(body: &[HirStmt]) -> bool {
    if body.is_empty() {
        return true;
    }
    if body.len() != 1 {
        return false;
    }
    match &body[0] {
        HirStmt::Expr(e) => host_io_empty_theater_expr(e),
        _ => false,
    }
}

fn host_io_result_err_theater_stmt(arms: &[HirMatchArm]) -> bool {
    arms.iter().any(|arm| {
        err_binding_unused_in_body(&arm.pat, &arm.body)
            && host_io_empty_theater_stmt_body(&arm.body)
    })
}

fn host_io_result_err_theater_expr(arms: &[HirMatchExprArm]) -> bool {
    arms.iter().any(|arm| {
        err_binding_unused_in_value(&arm.pat, &arm.value) && host_io_empty_theater_expr(&arm.value)
    })
}

/// ADR-273: callee is host.cli_arg or host.json_get_int (HIR `host::<fn>`).
fn host_cli_json_callee(callee: &str) -> bool {
    let name = callee.strip_prefix("host::").unwrap_or(callee);
    name == "cli_arg" || name == "json_get_int"
}

fn expr_is_host_cli_json_call(e: &HirExpr) -> bool {
    matches!(e, HirExpr::Call(c) if host_cli_json_callee(&c.callee))
}

fn expr_from_host_cli_json(e: &HirExpr) -> bool {
    expr_is_host_cli_json_call(e)
}

/// ADR-273: empty / default lit theater on None arm (not explicit fail message).
fn host_cli_none_theater_stmt(arms: &[HirMatchArm]) -> bool {
    arms.iter()
        .any(|arm| matches!(arm.pat, HirPat::None) && host_io_empty_theater_stmt_body(&arm.body))
}

fn host_cli_none_theater_expr(arms: &[HirMatchExprArm]) -> bool {
    arms.iter()
        .any(|arm| matches!(arm.pat, HirPat::None) && host_io_empty_theater_expr(&arm.value))
}

fn expr_from_checked_arith(e: &HirExpr) -> bool {
    matches!(
        e,
        HirExpr::MethodCall { method, .. } if is_checked_arith_method(method)
    )
}

/// ADR-031 / E0225: arm value is a pure constant lit (`{ lit }` / lit).
fn is_const_lit(e: &HirExpr) -> bool {
    matches!(
        e,
        HirExpr::LitInt(_) | HirExpr::LitBool(_) | HirExpr::LitStr(_)
    )
}

/// True when ≥2 arms and every arm value is the **same** constant lit (vacuous / theater).
fn vacuous_match_expr_arms(arms: &[HirMatchExprArm]) -> bool {
    if arms.len() < 2 {
        return false;
    }
    let Some(first) = arms.first().map(|a| &a.value) else {
        return false;
    };
    if !is_const_lit(first) {
        return false;
    }
    arms.iter()
        .all(|a| is_const_lit(&a.value) && a.value == *first)
}

/// Stmt-form v0: each body is exactly one `Expr(lit)` and all lits are equal.
fn vacuous_match_stmt_arms(arms: &[HirMatchArm]) -> bool {
    if arms.len() < 2 {
        return false;
    }
    let mut first: Option<&HirExpr> = None;
    for arm in arms {
        if arm.body.len() != 1 {
            return false;
        }
        let HirStmt::Expr(e) = &arm.body[0] else {
            return false;
        };
        if !is_const_lit(e) {
            return false;
        }
        match first {
            None => first = Some(e),
            Some(prev) if prev == e => {}
            Some(_) => return false,
        }
    }
    first.is_some()
}

/// True when `path` is a single-segment ident equal to `name`.
fn path_is_ident(path: &HirPath, name: &str) -> bool {
    path.segments.len() == 1 && path.segments[0] == name
}

/// ADR-048 / E0272: true if `name` appears as a value use in `e`.
fn expr_uses_ident(e: &HirExpr, name: &str) -> bool {
    match e {
        HirExpr::Path(path) => path_is_ident(path, name),
        HirExpr::Call(c) => c.args.iter().any(|a| expr_uses_ident(a, name)),
        HirExpr::Binary { lhs, rhs, .. } => {
            expr_uses_ident(lhs, name) || expr_uses_ident(rhs, name)
        }
        HirExpr::Borrow { inner, .. }
        | HirExpr::Ok(inner)
        | HirExpr::Err(inner)
        | HirExpr::Some(inner)
        | HirExpr::Await { inner }
        | HirExpr::Try { inner } => expr_uses_ident(inner, name),
        HirExpr::MethodCall { receiver, args, .. } => {
            expr_uses_ident(receiver, name) || args.iter().any(|a| expr_uses_ident(a, name))
        }
        HirExpr::Match { scrutinee, arms } => {
            expr_uses_ident(scrutinee, name) || arms.iter().any(|a| {
                let shadows =
                    matches!(&a.pat, HirPat::Ok(n) | HirPat::Err(n) | HirPat::Some(n) if n == name);
                !shadows && expr_uses_ident(&a.value, name)
            })
        }
        HirExpr::RecordLit { fields, .. } => fields.iter().any(|(_, e)| expr_uses_ident(e, name)),
        HirExpr::FieldAccess { base, .. } => expr_uses_ident(base, name),
        HirExpr::LitStr(_)
        | HirExpr::LitInt(_)
        | HirExpr::LitBool(_)
        | HirExpr::LitUnit
        | HirExpr::None
        | HirExpr::EnumPath { .. } => false,
    }
}

/// ADR-294: names assigned (`Assign`) or re-declared (`Let`) anywhere inside `body`, nested
/// control flow included. Used to drop `known_int` at branch / loop / block boundaries.
fn collect_clobbered(body: &[HirStmt], out: &mut std::collections::HashSet<String>) {
    for s in body {
        match s {
            HirStmt::Let { name, .. } | HirStmt::Assign { name, .. } => {
                out.insert(name.clone());
            }
            HirStmt::If {
                then_body,
                else_body,
                ..
            }
            | HirStmt::IfLet {
                then_body,
                else_body,
                ..
            } => {
                collect_clobbered(then_body, out);
                if let Some(eb) = else_body {
                    collect_clobbered(eb, out);
                }
            }
            HirStmt::While { body, .. } | HirStmt::WhileLet { body, .. } => {
                collect_clobbered(body, out);
            }
            HirStmt::Match { arms, .. } => {
                for a in arms {
                    collect_clobbered(&a.body, out);
                }
            }
            HirStmt::Expr(_)
            | HirStmt::Assert { .. }
            | HirStmt::IndexAssign { .. }
            | HirStmt::Break
            | HirStmt::Continue => {}
        }
    }
}

fn stmt_uses_ident(s: &HirStmt, name: &str) -> bool {
    match s {
        HirStmt::Let { init, .. } => expr_uses_ident(init, name),
        HirStmt::Expr(e) | HirStmt::Assign { value: e, .. } => expr_uses_ident(e, name),
        HirStmt::IndexAssign {
            target, key, value, ..
        } => target == name || expr_uses_ident(key, name) || expr_uses_ident(value, name),
        HirStmt::Assert { lhs, rhs } => expr_uses_ident(lhs, name) || expr_uses_ident(rhs, name),
        HirStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_uses_ident(cond, name)
                || body_uses_ident(then_body, name)
                || else_body
                    .as_ref()
                    .map(|b| body_uses_ident(b, name))
                    .unwrap_or(false)
        }
        HirStmt::IfLet {
            pat,
            scrutinee,
            then_body,
            else_body,
        } => {
            let shadows =
                matches!(pat, HirPat::Some(n) | HirPat::Ok(n) | HirPat::Err(n) if n == name);
            expr_uses_ident(scrutinee, name)
                || (!shadows && body_uses_ident(then_body, name))
                || else_body
                    .as_ref()
                    .map(|b| body_uses_ident(b, name))
                    .unwrap_or(false)
        }
        HirStmt::WhileLet {
            pat,
            scrutinee,
            body,
        } => {
            let shadows =
                matches!(pat, HirPat::Some(n) | HirPat::Ok(n) | HirPat::Err(n) if n == name);
            expr_uses_ident(scrutinee, name) || (!shadows && body_uses_ident(body, name))
        }
        HirStmt::While { cond, body } => expr_uses_ident(cond, name) || body_uses_ident(body, name),
        HirStmt::Match { scrutinee, arms } => {
            expr_uses_ident(scrutinee, name) || arms.iter().any(|a| {
                let shadows =
                    matches!(&a.pat, HirPat::Ok(n) | HirPat::Err(n) | HirPat::Some(n) if n == name);
                !shadows && body_uses_ident(&a.body, name)
            })
        }
        HirStmt::Break | HirStmt::Continue => false,
    }
}

fn body_uses_ident(body: &[HirStmt], name: &str) -> bool {
    body.iter().any(|s| stmt_uses_ident(s, name))
}

/// ADR-295 A-1: a statement whose nested bodies are return-tail when the statement itself is
/// (`if`/`else`, `if let`/`else`, `match`); loops and `if` without `else` are never.
fn stmt_propagates_tail(s: &HirStmt) -> bool {
    match s {
        HirStmt::If { else_body, .. } | HirStmt::IfLet { else_body, .. } => else_body.is_some(),
        HirStmt::Match { .. } => true,
        _ => false,
    }
}

/// ADR-295 P4 + S2-D2: is the `Result` binding `name` used in `rest` (the rest of its block)?
/// A use is a mention as receiver, argument or operand, a scrutinee / condition, or a bare
/// ident in return-tail position (`tail`). A bare ident statement elsewhere is not a use, and the
/// only transitive form is the direct alias `let y: T = name` (used iff `y` is used afterwards).
/// A later `let name` shadows: only its initializer still reads the old binding.
fn must_use_binding_used(name: &str, rest: &[HirStmt], tail: bool) -> bool {
    for (i, s) in rest.iter().enumerate() {
        let st = tail && i + 1 == rest.len();
        let used = match s {
            HirStmt::Expr(HirExpr::Path(p)) if path_is_ident(p, name) => st,
            HirStmt::Let {
                name: y,
                init: HirExpr::Path(p),
                ..
            } if path_is_ident(p, name) && y != name => {
                y != "_" && must_use_binding_used(y, &rest[i + 1..], tail)
            }
            HirStmt::Let { name: y, init, .. } if y == name => {
                return expr_uses_ident(init, name);
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let sub = st && else_body.is_some();
                expr_uses_ident(cond, name)
                    || must_use_binding_used(name, then_body, sub)
                    || else_body
                        .as_ref()
                        .is_some_and(|b| must_use_binding_used(name, b, sub))
            }
            HirStmt::IfLet {
                pat,
                scrutinee,
                then_body,
                else_body,
            } => {
                let sub = st && else_body.is_some();
                expr_uses_ident(scrutinee, name)
                    || (!pat_binds_name(pat, name) && must_use_binding_used(name, then_body, sub))
                    || else_body
                        .as_ref()
                        .is_some_and(|b| must_use_binding_used(name, b, sub))
            }
            HirStmt::While { cond, body } => {
                expr_uses_ident(cond, name) || must_use_binding_used(name, body, false)
            }
            HirStmt::WhileLet {
                pat,
                scrutinee,
                body,
            } => {
                expr_uses_ident(scrutinee, name)
                    || (!pat_binds_name(pat, name) && must_use_binding_used(name, body, false))
            }
            HirStmt::Match { scrutinee, arms } => {
                expr_uses_ident(scrutinee, name)
                    || arms.iter().any(|a| {
                        !pat_binds_name(&a.pat, name) && must_use_binding_used(name, &a.body, st)
                    })
            }
            other => stmt_uses_ident(other, name),
        };
        if used {
            return true;
        }
    }
    false
}

fn pat_binds_name(pat: &HirPat, name: &str) -> bool {
    matches!(pat, HirPat::Ok(n) | HirPat::Err(n) | HirPat::Some(n) if n == name)
}

fn theater_expr(e: &HirExpr) -> bool {
    if is_const_lit(e) {
        return true;
    }
    matches!(
        e,
        HirExpr::Call(c)
            if c.callee == "print"
                && c.args.len() == 1
                && is_const_lit(&c.args[0])
    )
}

fn theater_stmt_body(body: &[HirStmt]) -> bool {
    if body.is_empty() {
        return true;
    }
    if body.len() != 1 {
        return false;
    }
    match &body[0] {
        HirStmt::Expr(e) => theater_expr(e),
        HirStmt::Assert { lhs, rhs } => theater_expr(lhs) && theater_expr(rhs),
        _ => false,
    }
}

fn err_binding_unused_in_body(pat: &HirPat, body: &[HirStmt]) -> bool {
    match pat {
        HirPat::Err(n) if n == "_" => true,
        HirPat::Err(n) => !body_uses_ident(body, n),
        _ => false,
    }
}

fn err_binding_unused_in_value(pat: &HirPat, value: &HirExpr) -> bool {
    match pat {
        HirPat::Err(n) if n == "_" => true,
        HirPat::Err(n) => !expr_uses_ident(value, n),
        _ => false,
    }
}

fn result_err_swallowed_stmt(arms: &[HirMatchArm]) -> bool {
    arms.iter()
        .any(|arm| err_binding_unused_in_body(&arm.pat, &arm.body) && theater_stmt_body(&arm.body))
}

fn result_err_swallowed_expr(arms: &[HirMatchExprArm]) -> bool {
    arms.iter()
        .any(|arm| err_binding_unused_in_value(&arm.pat, &arm.value) && theater_expr(&arm.value))
}

// ---------------------------------------------------------------------------
// ADR-286 (CORE-0.10-DEAD-SINK-20260927, slice 1): E0272 v2 — dead sink in an `Err` arm.
// Same `check_body` pass and same place as v1 (stmt-form `match` on a non-host-IO Result).
// ---------------------------------------------------------------------------

/// ADR-286 P4 ⚑2: `x == x` / `x != x` with the SAME single-segment ident on both sides
/// (purely syntactic). Returns `(ident, is_eq)`.
fn dead_sink_self_compare(cond: &HirExpr) -> Option<(&str, bool)> {
    let HirExpr::Binary { op, lhs, rhs } = cond else {
        return None;
    };
    let is_eq = match op {
        HirBinOp::Eq => true,
        HirBinOp::Ne => false,
        _ => return None,
    };
    match (lhs.as_ref(), rhs.as_ref()) {
        (HirExpr::Path(a), HirExpr::Path(b))
            if a.segments.len() == 1 && b.segments.len() == 1 && a.segments == b.segments =>
        {
            Some((a.segments[0].as_str(), is_eq))
        }
        _ => None,
    }
}

/// ADR-286 P3(ii) (precisión Ingeniero 27-09): in an `Err` arm WITH a dead sink, only an
/// explicit `Err(…)` exit satisfies P3(ii). `Ok(…)`, a Result call or a Result binding in tail
/// position is NOT handling (a tail that reads the error is a use under P3(i) instead).
/// An intermediate `?` is irrelevant here: it neither rescues nor condemns the arm (corrección
/// Ingeniero 27-09); `?` over a value derived from the error is still a use (P3(i)).
fn dead_sink_arm_propagates(body: &[HirStmt]) -> bool {
    dead_sink_body_exits_err(body)
}

/// Tail of `body` is an explicit `Err(…)` exit.
fn dead_sink_body_exits_err(body: &[HirStmt]) -> bool {
    body.last().is_some_and(dead_sink_stmt_exits_err)
}

/// ADR-286 P3(ii) (Arquitecto 2): a compound tail (`if`/`else`, `match`) counts only if ALL
/// its branches end in an explicit `Err(…)`, recursively; an `if` without `else` never does.
fn dead_sink_stmt_exits_err(s: &HirStmt) -> bool {
    match s {
        HirStmt::Expr(e) => dead_sink_expr_exits_err(e),
        HirStmt::If {
            then_body,
            else_body: Some(else_body),
            ..
        } => dead_sink_body_exits_err(then_body) && dead_sink_body_exits_err(else_body),
        HirStmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|a| dead_sink_body_exits_err(&a.body))
        }
        _ => false,
    }
}

fn dead_sink_expr_exits_err(e: &HirExpr) -> bool {
    match e {
        HirExpr::Err(_) => true,
        HirExpr::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|a| dead_sink_expr_exits_err(&a.value))
        }
        _ => false,
    }
}

/// Names declared by `let` anywhere inside `body` (arm-local bindings).
fn dead_sink_collect_lets(body: &[HirStmt], out: &mut std::collections::HashSet<String>) {
    for s in body {
        match s {
            HirStmt::Let { name, .. } => {
                out.insert(name.clone());
            }
            HirStmt::If {
                then_body,
                else_body,
                ..
            }
            | HirStmt::IfLet {
                then_body,
                else_body,
                ..
            } => {
                dead_sink_collect_lets(then_body, out);
                if let Some(b) = else_body {
                    dead_sink_collect_lets(b, out);
                }
            }
            HirStmt::While { body, .. } | HirStmt::WhileLet { body, .. } => {
                dead_sink_collect_lets(body, out);
            }
            HirStmt::Match { arms, .. } => {
                for a in arms {
                    dead_sink_collect_lets(&a.body, out);
                }
            }
            _ => {}
        }
    }
}

/// ADR-286 P2(d)/P4 walker over one `Err(n)` arm. `sinks` = `n` plus every binding the
/// error value flows into (directly or transitively) without a real use.
struct DeadSinkWalk<'a> {
    sinks: std::collections::HashSet<String>,
    arm_lets: &'a std::collections::HashSet<String>,
    /// A `let`/assign bound the error (or a derived sink) — P2(d) needs at least one.
    sank: bool,
    grew: bool,
    used: bool,
}

impl DeadSinkWalk<'_> {
    fn is_sink(&self, name: &str, ignore: Option<&str>) -> bool {
        ignore != Some(name) && self.sinks.contains(name)
    }

    /// P4 use positions: call/method args and receiver, `?` operand, match scrutinee.
    fn expr_real_use(&self, e: &HirExpr, ignore: Option<&str>) -> bool {
        match e {
            HirExpr::Call(c) => c.args.iter().any(|a| self.expr_reads(a, ignore)),
            HirExpr::MethodCall { receiver, args, .. } => {
                self.expr_reads(receiver, ignore) || args.iter().any(|a| self.expr_reads(a, ignore))
            }
            HirExpr::Try { inner } => self.expr_reads(inner, ignore),
            HirExpr::Match { scrutinee, arms } => {
                self.expr_reads(scrutinee, ignore)
                    || arms.iter().any(|a| self.expr_real_use(&a.value, ignore))
            }
            HirExpr::Binary { lhs, rhs, .. } => {
                self.expr_real_use(lhs, ignore) || self.expr_real_use(rhs, ignore)
            }
            HirExpr::Borrow { inner, .. }
            | HirExpr::Ok(inner)
            | HirExpr::Err(inner)
            | HirExpr::Some(inner)
            | HirExpr::Await { inner } => self.expr_real_use(inner, ignore),
            HirExpr::RecordLit { fields, .. } => {
                fields.iter().any(|(_, f)| self.expr_real_use(f, ignore))
            }
            HirExpr::FieldAccess { base, .. } => self.expr_real_use(base, ignore),
            HirExpr::Path(_)
            | HirExpr::LitStr(_)
            | HirExpr::LitInt(_)
            | HirExpr::LitBool(_)
            | HirExpr::LitUnit
            | HirExpr::None
            | HirExpr::EnumPath { .. } => false,
        }
    }

    /// The value of `e` carries a sink (operand/constructor/field chain, no use position).
    fn expr_flows(&self, e: &HirExpr, ignore: Option<&str>) -> bool {
        match e {
            HirExpr::Path(p) => p.segments.len() == 1 && self.is_sink(&p.segments[0], ignore),
            HirExpr::Binary { lhs, rhs, .. } => {
                self.expr_flows(lhs, ignore) || self.expr_flows(rhs, ignore)
            }
            HirExpr::Borrow { inner, .. }
            | HirExpr::Ok(inner)
            | HirExpr::Err(inner)
            | HirExpr::Some(inner)
            | HirExpr::Await { inner } => self.expr_flows(inner, ignore),
            HirExpr::Match { arms, .. } => arms.iter().any(|a| self.expr_flows(&a.value, ignore)),
            HirExpr::RecordLit { fields, .. } => {
                fields.iter().any(|(_, f)| self.expr_flows(f, ignore))
            }
            HirExpr::FieldAccess { base, .. } => self.expr_flows(base, ignore),
            HirExpr::Call(_)
            | HirExpr::MethodCall { .. }
            | HirExpr::Try { .. }
            | HirExpr::LitStr(_)
            | HirExpr::LitInt(_)
            | HirExpr::LitBool(_)
            | HirExpr::LitUnit
            | HirExpr::None
            | HirExpr::EnumPath { .. } => false,
        }
    }

    /// Any read of a sink in `e` (flowing or in a use position).
    fn expr_reads(&self, e: &HirExpr, ignore: Option<&str>) -> bool {
        self.expr_flows(e, ignore) || self.expr_real_use(e, ignore)
    }

    fn add_sink(&mut self, name: &str) {
        self.sank = true;
        // `let _: T = e` binds nothing readable (the value is dropped).
        if name != "_" && self.sinks.insert(name.to_string()) {
            self.grew = true;
        }
    }

    fn walk_body(&mut self, body: &[HirStmt], tail: bool, ignore: Option<&str>) {
        let last = body.len().saturating_sub(1);
        for (i, s) in body.iter().enumerate() {
            self.walk_stmt(s, tail && i == last, ignore);
        }
    }

    fn walk_stmt(&mut self, s: &HirStmt, tail: bool, ignore: Option<&str>) {
        match s {
            HirStmt::Let { name, init, .. } => {
                if self.expr_real_use(init, ignore) {
                    self.used = true;
                } else if self.expr_flows(init, ignore) {
                    self.add_sink(name);
                }
            }
            HirStmt::Assign { name, value } => {
                if self.expr_real_use(value, ignore) {
                    self.used = true;
                } else if self.expr_flows(value, ignore) {
                    if self.arm_lets.contains(name) {
                        self.add_sink(name);
                    } else {
                        // Assigned to a binding outside the arm: observable later → use.
                        self.used = true;
                    }
                }
            }
            HirStmt::Expr(e) => {
                // P4: a flowing read is a use only as the value/exit (tail) of the arm.
                if self.expr_real_use(e, ignore) || (tail && self.expr_flows(e, ignore)) {
                    self.used = true;
                }
            }
            HirStmt::IndexAssign {
                target, key, value, ..
            } => {
                if self.is_sink(target, ignore)
                    || self.expr_reads(key, ignore)
                    || self.expr_reads(value, ignore)
                {
                    self.used = true;
                }
            }
            HirStmt::Assert { lhs, rhs } => {
                if self.expr_reads(lhs, ignore) || self.expr_reads(rhs, ignore) {
                    self.used = true;
                }
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                if let Some((x, is_eq)) = dead_sink_self_compare(cond) {
                    // P4 ⚑2: the self-comparison is not a use; reads of `x` in the
                    // unreachable branch (`else` for `==`, `then` for `!=`) are not uses.
                    let (then_ign, else_ign) = if is_eq {
                        (ignore, Some(x))
                    } else {
                        (Some(x), ignore)
                    };
                    self.walk_body(then_body, tail, then_ign);
                    if let Some(b) = else_body {
                        self.walk_body(b, tail, else_ign);
                    }
                    return;
                }
                if self.expr_reads(cond, ignore) {
                    self.used = true;
                }
                self.walk_body(then_body, tail, ignore);
                if let Some(b) = else_body {
                    self.walk_body(b, tail, ignore);
                }
            }
            HirStmt::IfLet {
                scrutinee,
                then_body,
                else_body,
                ..
            } => {
                if self.expr_reads(scrutinee, ignore) {
                    self.used = true;
                }
                self.walk_body(then_body, tail, ignore);
                if let Some(b) = else_body {
                    self.walk_body(b, tail, ignore);
                }
            }
            HirStmt::While { cond, body } => {
                if self.expr_reads(cond, ignore) {
                    self.used = true;
                }
                self.walk_body(body, false, ignore);
            }
            HirStmt::WhileLet {
                scrutinee, body, ..
            } => {
                if self.expr_reads(scrutinee, ignore) {
                    self.used = true;
                }
                self.walk_body(body, false, ignore);
            }
            HirStmt::Match { scrutinee, arms } => {
                if self.expr_reads(scrutinee, ignore) {
                    self.used = true;
                }
                for a in arms {
                    self.walk_body(&a.body, tail, ignore);
                }
            }
            HirStmt::Break | HirStmt::Continue => {}
        }
    }
}

/// ADR-286 P2(d): `Err(n) => { … }` whose only handling of `n` is binding it (directly or
/// transitively) to bindings without a real use (P4), and the arm does not satisfy P3.
/// Arms with no such binding keep E0272 v1 unchanged (P8).
fn err_arm_dead_sink(arm: &HirMatchArm) -> bool {
    let HirPat::Err(n) = &arm.pat else {
        return false;
    };
    if dead_sink_arm_propagates(&arm.body) {
        return false;
    }
    let mut arm_lets = std::collections::HashSet::new();
    dead_sink_collect_lets(&arm.body, &mut arm_lets);
    let mut walk = DeadSinkWalk {
        sinks: std::collections::HashSet::from([n.clone()]),
        arm_lets: &arm_lets,
        sank: false,
        grew: false,
        used: false,
    };
    loop {
        walk.grew = false;
        walk.walk_body(&arm.body, true, None);
        if walk.used {
            return false;
        }
        if !walk.grew {
            break;
        }
    }
    walk.sank
}

/// ADR-286: E0272 v2 over a stmt-form Result match (after the v1 check, same place).
fn result_err_dead_sink_stmt(arms: &[HirMatchArm]) -> bool {
    arms.iter().any(err_arm_dead_sink)
}

/// ADR-051: None-arm success-theater (empty / default lit / print non-string lit).
/// `print("none")` string message = explicit absence → not swallow.
fn none_theater_expr(e: &HirExpr) -> bool {
    matches!(
        e,
        HirExpr::LitInt(_) | HirExpr::LitBool(_) | HirExpr::LitUnit
    ) || matches!(
        e,
        HirExpr::Call(c) if c.callee == "print" && c.args.len() == 1 && {
            match &c.args[0] {
                HirExpr::LitInt(_) | HirExpr::LitBool(_) | HirExpr::LitUnit => true,
                // Explicit absence token for pos oracle; other string lits = success-theater.
                HirExpr::LitStr(s) => s != "none",
                _ => false,
            }
        }
    )
}

fn none_theater_stmt_body(body: &[HirStmt]) -> bool {
    if body.is_empty() {
        return true;
    }
    if body.len() != 1 {
        return false;
    }
    match &body[0] {
        HirStmt::Expr(e) => none_theater_expr(e),
        HirStmt::Assert { lhs, rhs } => none_theater_expr(lhs) && none_theater_expr(rhs),
        _ => false,
    }
}

fn option_none_swallowed_stmt(arms: &[HirMatchArm]) -> bool {
    arms.iter()
        .any(|arm| matches!(arm.pat, HirPat::None) && none_theater_stmt_body(&arm.body))
}

fn option_none_swallowed_expr(arms: &[HirMatchExprArm]) -> bool {
    arms.iter()
        .any(|arm| matches!(arm.pat, HirPat::None) && none_theater_expr(&arm.value))
}

/// ADR-044: `fn(a: Int, b: Int) -> Int { a / b }` or `{ a % b }` (params in order).
fn is_int_div_rem_by_second_param(f: &HirFunction) -> bool {
    if f.params.len() != 2 {
        return false;
    }
    if f.params[0].ty != HirType::Int || f.params[1].ty != HirType::Int {
        return false;
    }
    if f.ret_ty != HirType::Int {
        return false;
    }
    if f.body.len() != 1 {
        return false;
    }
    let HirStmt::Expr(HirExpr::Binary { op, lhs, rhs }) = &f.body[0] else {
        return false;
    };
    if !matches!(op, HirBinOp::Div | HirBinOp::Rem) {
        return false;
    }
    match (lhs.as_ref(), rhs.as_ref()) {
        (HirExpr::Path(lp), HirExpr::Path(rp)) => {
            path_is_ident(lp, &f.params[0].name) && path_is_ident(rp, &f.params[1].name)
        }
        _ => false,
    }
}

/// ADR-278: true if `e` contains any `HirExpr::Try`.
fn expr_contains_try(e: &HirExpr) -> bool {
    match e {
        HirExpr::Try { .. } => true,
        HirExpr::Call(c) => c.args.iter().any(expr_contains_try),
        HirExpr::Binary { lhs, rhs, .. } => expr_contains_try(lhs) || expr_contains_try(rhs),
        HirExpr::Borrow { inner, .. }
        | HirExpr::Ok(inner)
        | HirExpr::Err(inner)
        | HirExpr::Some(inner)
        | HirExpr::Await { inner } => expr_contains_try(inner),
        HirExpr::MethodCall { receiver, args, .. } => {
            expr_contains_try(receiver) || args.iter().any(expr_contains_try)
        }
        HirExpr::Match { scrutinee, arms } => {
            expr_contains_try(scrutinee) || arms.iter().any(|a| expr_contains_try(&a.value))
        }
        HirExpr::RecordLit { fields, .. } => fields.iter().any(|(_, e)| expr_contains_try(e)),
        HirExpr::FieldAccess { base, .. } => expr_contains_try(base),
        HirExpr::LitStr(_)
        | HirExpr::LitInt(_)
        | HirExpr::LitBool(_)
        | HirExpr::LitUnit
        | HirExpr::None
        | HirExpr::Path(_)
        | HirExpr::EnumPath { .. } => false,
    }
}

fn stmt_contains_try(s: &HirStmt) -> bool {
    match s {
        HirStmt::Let { init, .. } => expr_contains_try(init),
        HirStmt::Expr(e) | HirStmt::Assign { value: e, .. } => expr_contains_try(e),
        HirStmt::IndexAssign { key, value, .. } => {
            expr_contains_try(key) || expr_contains_try(value)
        }
        HirStmt::Assert { lhs, rhs } => expr_contains_try(lhs) || expr_contains_try(rhs),
        HirStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_contains_try(cond)
                || body_contains_try(then_body)
                || else_body.as_ref().is_some_and(|b| body_contains_try(b))
        }
        HirStmt::IfLet {
            scrutinee,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_try(scrutinee)
                || body_contains_try(then_body)
                || else_body.as_ref().is_some_and(|b| body_contains_try(b))
        }
        HirStmt::While { cond, body } => expr_contains_try(cond) || body_contains_try(body),
        HirStmt::WhileLet {
            scrutinee, body, ..
        } => expr_contains_try(scrutinee) || body_contains_try(body),
        HirStmt::Match { scrutinee, arms } => {
            expr_contains_try(scrutinee) || arms.iter().any(|a| body_contains_try(&a.body))
        }
        HirStmt::Break | HirStmt::Continue => false,
    }
}

fn body_contains_try(body: &[HirStmt]) -> bool {
    body.iter().any(stmt_contains_try)
}

/// ADR-283 §0 (one diagnostic per fn, first offender in source order): walks `body` in source
/// order. `Some(true)` if a statement whose own expressions contain `?` comes first;
/// `Some(false)` if the given `IndexAssign` (by span; any `IndexAssign` if `ia_span` is `None`)
/// comes first — within the same statement the index-assign wins; `None` if neither is found.
fn qmark_before_index_assign(
    body: &[HirStmt],
    ia_span: Option<&arita_syntax::Span>,
) -> Option<bool> {
    for s in body {
        if let HirStmt::IndexAssign { span, .. } = s {
            if ia_span.is_none() || ia_span == Some(span) {
                return Some(false);
            }
        }
        let (own_try, nested): (bool, Vec<&[HirStmt]>) = match s {
            HirStmt::Let { init, .. } => (expr_contains_try(init), vec![]),
            HirStmt::Expr(e) | HirStmt::Assign { value: e, .. } => (expr_contains_try(e), vec![]),
            HirStmt::IndexAssign { key, value, .. } => {
                (expr_contains_try(key) || expr_contains_try(value), vec![])
            }
            HirStmt::Assert { lhs, rhs } => {
                (expr_contains_try(lhs) || expr_contains_try(rhs), vec![])
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let mut n: Vec<&[HirStmt]> = vec![then_body];
                n.extend(else_body.as_deref());
                (expr_contains_try(cond), n)
            }
            HirStmt::IfLet {
                scrutinee,
                then_body,
                else_body,
                ..
            } => {
                let mut n: Vec<&[HirStmt]> = vec![then_body];
                n.extend(else_body.as_deref());
                (expr_contains_try(scrutinee), n)
            }
            HirStmt::While { cond, body } => (expr_contains_try(cond), vec![body]),
            HirStmt::WhileLet {
                scrutinee, body, ..
            } => (expr_contains_try(scrutinee), vec![body]),
            HirStmt::Match { scrutinee, arms } => (
                expr_contains_try(scrutinee),
                arms.iter().map(|a| a.body.as_slice()).collect(),
            ),
            HirStmt::Break | HirStmt::Continue => (false, vec![]),
        };
        if own_try {
            return Some(true);
        }
        for b in nested {
            if let Some(r) = qmark_before_index_assign(b, ia_span) {
                return Some(r);
            }
        }
    }
    None
}

/// ADR-278 / E0343: `?` only inside `fn … -> Result<_,_>`.
fn check_question_mark_outside_result_fn(f: &HirFunction) -> Result<(), CheckError> {
    if !body_contains_try(&f.body) {
        return Ok(());
    }
    if matches!(f.ret_ty, HirType::Result(_, _)) {
        return Ok(());
    }
    Err(err("E0343", "question mark outside result fn"))
}

/// ADR-277 / E0342: Result return theater — pinned cases only.
///
/// A) last body value is `Ok`/`Err` (or types as `Result`) but `ret_ty` is not `Result`
/// B) `ret_ty` is `Result` but last body value type is clearly non-`Result`
/// Skips `IoUnit` / main-style functions. Only inspects last `HirStmt::Expr`.
fn check_fn_result_return_theater(ctx: &CheckCtx<'_>, f: &HirFunction) -> Result<(), CheckError> {
    if matches!(f.ret_ty, HirType::IoUnit) {
        return Ok(());
    }
    let Some(HirStmt::Expr(last)) = f.body.last() else {
        return Ok(());
    };
    let last_is_ok_err = matches!(last, HirExpr::Ok(_) | HirExpr::Err(_));
    let last_ty = ctx.type_of_expr(last)?;
    let last_is_result = last_is_ok_err || matches!(last_ty, HirType::Result(_, _));
    let ret_is_result = matches!(f.ret_ty, HirType::Result(_, _));
    // A: Ok/Err / Result-typed last without Result return
    if last_is_result && !ret_is_result {
        return Err(err("E0342", "result return type required"));
    }
    // B: Result return with clearly non-Result last value
    if ret_is_result && !last_is_result {
        return Err(err("E0342", "result return type required"));
    }
    Ok(())
}

/// Lexical ownership/borrow check v0 over HIR functions and test bodies.
/// Params as immutable bindings + ADR-283 legal `v[i] = x` context (SYNC `fn -> Result<_, E>`).
fn bind_fn_params(ctx: &mut CheckCtx<'_>, f: &HirFunction) {
    for p in &f.params {
        ctx.bindings.insert(
            p.name.clone(),
            BindingState {
                ty: p.ty.clone(),
                mutable: false,
                moved: false,
                known_int: None,
                from_checked_arith: false,
                from_vec_get: false,
                from_find_idx: false,
                from_ilog: false,
                from_host_io: false,
                from_host_cli_json: false,
            },
        );
    }
    if !f.is_async {
        if let HirType::Result(_, e) = &f.ret_ty {
            ctx.result_err_ty = Some(e.as_ref().clone());
        }
    }
    // ADR-295: the tail of a fn that is not `Io<()>` is a return (E0342 judges its type).
    ctx.tail_ret = f.ret_ty != HirType::IoUnit;
}

/// ADR-283: `[marker, rest..]` → (exact deferred parser error, rest). See
/// [`arita_syntax::DEFERRED_SHAPE_MARKER`]; only E0001 / E0006 are ever deferred.
fn split_deferred_shape_marker(body: &[HirStmt]) -> Option<(CheckError, &[HirStmt])> {
    let (HirStmt::Expr(HirExpr::Call(c)), rest) = body.split_first()? else {
        return None;
    };
    if c.callee != arita_syntax::DEFERRED_SHAPE_MARKER {
        return None;
    }
    let [HirExpr::LitStr(code), HirExpr::LitStr(message), HirExpr::LitInt(start), HirExpr::LitInt(end)] =
        c.args.as_slice()
    else {
        return None;
    };
    let code: &'static str = match code.as_str() {
        "E0001" => "E0001",
        "E0006" => "E0006",
        _ => return None,
    };
    Some((
        CheckError::Coded {
            code,
            message: format!("{message} @{start}..{end}"),
        },
        rest,
    ))
}

/// ADR-286 §0.1d (a): surface spelling of a HIR type for diagnostics (E0203 spawn contract).
fn hir_type_surface(t: &HirType) -> String {
    match t {
        HirType::Int => "Int".into(),
        HirType::Bool => "Bool".into(),
        HirType::String => "String".into(),
        HirType::Bytes => "Bytes".into(),
        HirType::Task => "Task".into(),
        HirType::Unit => "()".into(),
        HirType::IoUnit => "Io<()>".into(),
        HirType::Vec(i) => format!("Vec<{}>", hir_type_surface(i)),
        HirType::Map(k, v) => format!("Map<{}, {}>", hir_type_surface(k), hir_type_surface(v)),
        HirType::Result(o, e) => {
            format!("Result<{}, {}>", hir_type_surface(o), hir_type_surface(e))
        }
        HirType::Option(i) => format!("Option<{}>", hir_type_surface(i)),
        HirType::Named(n) => n.clone(),
    }
}

/// ADR-286 §0.1d (c): free builtins dispatched by name in HIR/codegen. A user fn declared
/// with one of these names would be shadowed by the builtin arm (silent drop in codegen).
/// Closed list; applies only to fn declaration names (not `is_keyword`: vars/params stay legal).
const BUILTIN_FN_NAMES: &[&str] = &[
    "spawn",
    "join",
    "timeout",
    "delay",
    "cancel_token",
    "cancel",
    "until_cancelled",
    "busy_spin",
    "hang_forever",
    "http_listen",
    "http_serve",
    "http_route_health",
    "http_route_echo",
    "http_bound_port",
    "http_shutdown",
    "http_route",
    "http_response",
];

/// ADR-286 §0.1d (c): E0007 for a user fn declaration named like a free builtin.
fn check_builtin_fn_name(f: &HirFunction) -> Result<(), CheckError> {
    if BUILTIN_FN_NAMES.contains(&f.name.as_str()) {
        return Err(CheckError::Coded {
            code: "E0007",
            message: format!("illegal ident / keyword misuse: `{}`", f.name),
        });
    }
    Ok(())
}

pub fn check(hir: &HirModule) -> Result<(), CheckError> {
    let imports: std::collections::HashSet<String> = hir.imports.iter().cloned().collect();
    let async_fns: std::collections::HashSet<String> = hir
        .functions
        .iter()
        .filter(|f| f.is_async)
        .map(|f| f.name.clone())
        .collect();
    let div_rem_fns: std::collections::HashSet<String> = hir
        .functions
        .iter()
        .filter(|f| is_int_div_rem_by_second_param(f))
        .map(|f| f.name.clone())
        .collect();
    let mut records: HashMap<String, Vec<(String, HirType)>> = hir
        .records
        .iter()
        .map(|r| {
            (
                r.name.clone(),
                r.fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
            )
        })
        .collect();
    // ADR-246 builtin ServicePolicy (Int fields; E0323 on invalid lits)
    records.entry("ServicePolicy".into()).or_insert_with(|| {
        vec![
            ("max_body_bytes".into(), HirType::Int),
            ("max_header_bytes".into(), HirType::Int),
            ("max_header_count".into(), HirType::Int),
            ("idle_timeout_ms".into(), HirType::Int),
        ]
    });
    let enums: HashMap<String, Vec<String>> = hir
        .enums
        .iter()
        .map(|e| (e.name.clone(), e.variants.clone()))
        .collect();
    let fn_rets: HashMap<String, HirType> = hir
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.ret_ty.clone()))
        .collect();
    // ADR-283: a parser shape error (main E0001 / `fn -> Int` E0006) deferred behind an
    // IndexAssign is resolved FIRST (the parser would have stopped there): the non-Map
    // index-assign error wins if the body yields one; otherwise the exact deferred error.
    for f in &hir.functions {
        // ADR-286 §0.1d (c): the decl name precedes its shape/body in source order.
        check_builtin_fn_name(f)?;
        let Some((deferred, rest)) = split_deferred_shape_marker(&f.body) else {
            continue;
        };
        let mut ctx = CheckCtx::new(
            f.is_async,
            &async_fns,
            &div_rem_fns,
            &records,
            &enums,
            &fn_rets,
        );
        ctx.imports = imports.clone();
        bind_fn_params(&mut ctx, f);
        let res = ctx.check_body(rest);
        // ADR-283 §0: an outside-Result `?` (E0343) wins if it comes first in source order —
        // before the failing index-assign, or (E0006 `fn -> Int` shape) before any index-assign.
        if let Err(qmark) = check_question_mark_outside_result_fn(f) {
            let ia_span = ctx.index_assign_failed_span.as_ref();
            let positional =
                ia_span.is_some() || matches!(deferred, CheckError::Coded { code: "E0006", .. });
            if positional && qmark_before_index_assign(rest, ia_span) == Some(true) {
                return Err(qmark);
            }
        }
        return match res {
            Err(e) if ctx.index_assign_failed_span.is_some() => Err(e),
            _ => Err(deferred),
        };
    }
    if !async_fns.is_empty() {
        let main = hir.functions.iter().find(|f| f.name == "main");
        if let Some(m) = main {
            if !m.is_async {
                return Err(err("E0241", "async feature not allowed here"));
            }
        }
        for f in &hir.functions {
            if f.is_async && f.ret_ty != HirType::IoUnit {
                // ADR-245: async Result handlers (HttpResponse, IoError)
                if !matches!(&f.ret_ty, HirType::Result(_, _)) {
                    return Err(err("E0241", "async feature not allowed here"));
                }
            }
        }
    }
    for f in &hir.functions {
        // ADR-235: borrow as fn result escapes
        if f.ret_ty != HirType::IoUnit {
            if let Some(HirStmt::Expr(HirExpr::Borrow { .. })) = f.body.last() {
                return Err(err("E0318", "borrow cannot escape"));
            }
        }
        let mut ctx = CheckCtx::new(
            f.is_async,
            &async_fns,
            &div_rem_fns,
            &records,
            &enums,
            &fn_rets,
        );
        ctx.imports = imports.clone();
        bind_fn_params(&mut ctx, f);
        // ADR-278: E0343 before body so outside-Result `?` wins over type noise; ADR-283 §0:
        // except an E0344 `v[i] = x` that precedes the first `?` in source order.
        if let Err(qmark) = check_question_mark_outside_result_fn(f) {
            return match ctx.check_body(&f.body) {
                Err(e @ CheckError::Coded { code: "E0344", .. })
                    if qmark_before_index_assign(
                        &f.body,
                        ctx.index_assign_failed_span.as_ref(),
                    ) == Some(false) =>
                {
                    Err(e)
                }
                _ => Err(qmark),
            };
        }
        ctx.check_body(&f.body)?;
        // ADR-277: E0342 Result return theater (after body so Path bindings resolve)
        check_fn_result_return_theater(&ctx, f)?;
    }
    let empty = std::collections::HashSet::new();
    for t in &hir.tests {
        // ADR-278: tests are not Result fns — any `?` → E0343
        if body_contains_try(&t.body) {
            return Err(err("E0343", "question mark outside result fn"));
        }
        // Tests may call module div/rem helpers — pass the same div_rem set.
        let mut ctx = CheckCtx::new(false, &empty, &div_rem_fns, &records, &enums, &fn_rets);
        ctx.imports = imports.clone();
        ctx.check_body(&t.body)?;
    }
    Ok(())
}

/// Pure structural map from F2 AST [`Module`] to [`HirModule`].
///
/// Mechanical only: no validation; use [`check`] for ownership/borrow.
pub fn lower_ast(module: &Module) -> HirModule {
    HirModule {
        name: module.name.clone(),
        functions: module.functions.iter().map(lower_function).collect(),
        tests: module.tests.iter().map(lower_test).collect(),
        records: module
            .records
            .iter()
            .map(|r| HirRecordDef {
                name: r.name.clone(),
                fields: r
                    .fields
                    .iter()
                    .map(|f| HirRecordField {
                        name: f.name.clone(),
                        ty: lower_type(&f.ty),
                    })
                    .collect(),
                is_pub: r.is_pub,
            })
            .collect(),
        enums: module
            .enums
            .iter()
            .map(|e| HirEnumDef {
                name: e.name.clone(),
                variants: e.variants.clone(),
            })
            .collect(),
        imports: module.uses.iter().map(|u| u.item.clone()).collect(),
    }
}

fn lower_function(f: &Function) -> HirFunction {
    HirFunction {
        name: f.name.clone(),
        params: f.params.iter().map(lower_param).collect(),
        ret_ty: lower_type(&f.ret_ty),
        body: f.body.iter().map(lower_stmt).collect(),
        is_async: f.is_async,
        is_pub: f.is_pub,
    }
}

fn lower_param(p: &FnParam) -> HirParam {
    HirParam {
        name: p.name.clone(),
        ty: lower_type(&p.ty),
    }
}

fn lower_type(ty: &Type) -> HirType {
    match ty {
        Type::Int => HirType::Int,
        Type::Bool => HirType::Bool,
        Type::String => HirType::String,
        Type::Bytes => HirType::Bytes,
        Type::Task => HirType::Task,
        Type::Unit => HirType::Unit,
        Type::IoUnit => HirType::IoUnit,
        Type::Vec(inner) => HirType::Vec(Box::new(lower_type(inner))),
        Type::Map(k, v) => HirType::Map(Box::new(lower_type(k)), Box::new(lower_type(v))),
        Type::Result(t, e) => HirType::Result(Box::new(lower_type(t)), Box::new(lower_type(e))),
        Type::Option(t) => HirType::Option(Box::new(lower_type(t))),
        // Defense: pest order footgun Named("Int") etc. must not escape as opaque names.
        Type::Named(n) => match n.as_str() {
            "Int" => HirType::Int,
            "Bool" => HirType::Bool,
            "Text" | "String" => HirType::String,
            "Bytes" => HirType::Bytes,
            "Task" => HirType::Task,
            "Unit" => HirType::Unit,
            _ => HirType::Named(n.clone()),
        },
    }
}

fn lower_stmt(s: &Stmt) -> HirStmt {
    match s {
        Stmt::Let {
            mutable,
            name,
            ty,
            init,
        } => HirStmt::Let {
            mutable: *mutable,
            name: name.clone(),
            ty: lower_type(ty),
            init: lower_expr(init),
        },
        Stmt::Expr(e) => HirStmt::Expr(lower_expr(e)),
        Stmt::Assert { lhs, rhs } => HirStmt::Assert {
            lhs: lower_expr(lhs),
            rhs: lower_expr(rhs),
        },
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => HirStmt::If {
            cond: lower_expr(cond),
            then_body: then_body.iter().map(lower_stmt).collect(),
            else_body: else_body
                .as_ref()
                .map(|b| b.iter().map(lower_stmt).collect()),
        },
        Stmt::IfLet {
            pat,
            scrutinee,
            then_body,
            else_body,
        } => HirStmt::IfLet {
            pat: lower_pat(pat),
            scrutinee: lower_expr(scrutinee),
            then_body: then_body.iter().map(lower_stmt).collect(),
            else_body: else_body
                .as_ref()
                .map(|b| b.iter().map(lower_stmt).collect()),
        },
        Stmt::WhileLet {
            pat,
            scrutinee,
            body,
        } => HirStmt::WhileLet {
            pat: lower_pat(pat),
            scrutinee: lower_expr(scrutinee),
            body: body.iter().map(lower_stmt).collect(),
        },
        Stmt::While { cond, body } => HirStmt::While {
            cond: lower_expr(cond),
            body: body.iter().map(lower_stmt).collect(),
        },
        Stmt::Assign { name, value } => HirStmt::Assign {
            name: name.clone(),
            value: lower_expr(value),
        },
        Stmt::IndexAssign {
            target,
            key,
            value,
            op,
            span,
        } => HirStmt::IndexAssign {
            target: target.clone(),
            key: lower_expr(key),
            value: lower_expr(value),
            op: op.as_ref().map(lower_binop),
            span: span.clone(),
        },
        Stmt::Match { scrutinee, arms } => HirStmt::Match {
            scrutinee: lower_expr(scrutinee),
            arms: arms.iter().map(lower_match_arm).collect(),
        },
        Stmt::Break => HirStmt::Break,
        Stmt::Continue => HirStmt::Continue,
    }
}

fn lower_match_arm(arm: &MatchArm) -> HirMatchArm {
    HirMatchArm {
        pat: lower_pat(&arm.pat),
        body: arm.body.iter().map(lower_stmt).collect(),
    }
}

fn lower_pat(pat: &Pat) -> HirPat {
    match pat {
        Pat::LitBool(b) => HirPat::LitBool(*b),
        Pat::LitInt(n) => HirPat::LitInt(*n),
        Pat::Wildcard => HirPat::Wildcard,
        Pat::Ok(n) => HirPat::Ok(n.clone()),
        Pat::Err(n) => HirPat::Err(n.clone()),
        Pat::Some(n) => HirPat::Some(n.clone()),
        Pat::None => HirPat::None,
        Pat::Variant { enum_name, variant } => HirPat::Variant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
        },
    }
}

fn lower_expr(e: &Expr) -> HirExpr {
    match e {
        Expr::Call(c) => HirExpr::Call(lower_call(c)),
        Expr::LitStr(s) => HirExpr::LitStr(s.clone()),
        Expr::LitInt(n) => HirExpr::LitInt(*n),
        Expr::LitBool(b) => HirExpr::LitBool(*b),
        Expr::Path(p) => HirExpr::Path(lower_path(p)),
        Expr::Binary { op, lhs, rhs } => HirExpr::Binary {
            op: lower_binop(op),
            lhs: Box::new(lower_expr(lhs)),
            rhs: Box::new(lower_expr(rhs)),
        },
        Expr::Borrow { mutable, inner } => HirExpr::Borrow {
            mutable: *mutable,
            inner: Box::new(lower_expr(inner)),
        },
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => HirExpr::MethodCall {
            receiver: Box::new(lower_expr(receiver)),
            method: method.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
        Expr::Match { scrutinee, arms } => HirExpr::Match {
            scrutinee: Box::new(lower_expr(scrutinee)),
            arms: arms
                .iter()
                .map(|a| HirMatchExprArm {
                    pat: lower_pat(&a.pat),
                    value: lower_expr(&a.value),
                })
                .collect(),
        },
        Expr::Await { inner } => HirExpr::Await {
            inner: Box::new(lower_expr(inner)),
        },
        Expr::Try { inner } => HirExpr::Try {
            inner: Box::new(lower_expr(inner)),
        },
        Expr::LitUnit => HirExpr::LitUnit,
        Expr::Ok(inner) => HirExpr::Ok(Box::new(lower_expr(inner))),
        Expr::Err(inner) => HirExpr::Err(Box::new(lower_expr(inner))),
        Expr::Some(inner) => HirExpr::Some(Box::new(lower_expr(inner))),
        Expr::None => HirExpr::None,
        Expr::RecordLit { name, fields } => HirExpr::RecordLit {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), lower_expr(e)))
                .collect(),
        },
        Expr::FieldAccess { base, field } => HirExpr::FieldAccess {
            base: Box::new(lower_expr(base)),
            field: field.clone(),
        },
        Expr::EnumPath { enum_name, variant } => HirExpr::EnumPath {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
        },
    }
}

fn lower_call(c: &Call) -> HirCall {
    HirCall {
        callee: c.callee.clone(),
        args: c.args.iter().map(lower_expr).collect(),
    }
}

fn lower_path(p: &Path) -> HirPath {
    HirPath {
        segments: p.segments.clone(),
    }
}

fn lower_binop(op: &BinOp) -> HirBinOp {
    match op {
        BinOp::Add => HirBinOp::Add,
        BinOp::Sub => HirBinOp::Sub,
        BinOp::Mul => HirBinOp::Mul,
        BinOp::Div => HirBinOp::Div,
        BinOp::Rem => HirBinOp::Rem,
        BinOp::Eq => HirBinOp::Eq,
        BinOp::Ne => HirBinOp::Ne,
        BinOp::Lt => HirBinOp::Lt,
        BinOp::Le => HirBinOp::Le,
        BinOp::Gt => HirBinOp::Gt,
        BinOp::Ge => HirBinOp::Ge,
    }
}

fn lower_test(t: &Test) -> HirTest {
    HirTest {
        name: t.name.clone(),
        body: t.body.iter().map(lower_stmt).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arita_syntax::{Call, Expr, Function, Module, Stmt, Type};

    #[test]
    fn lower_hello_module_name_and_functions() {
        let module = Module {
            name: "hello".into(),
            functions: vec![Function {
                name: "main".into(),
                params: vec![],
                ret_ty: Type::IoUnit,
                is_async: false,
                is_pub: false,
                body: vec![Stmt::Expr(Expr::Call(Call {
                    callee: "print".into(),
                    args: vec![Expr::LitStr("hello".into())],
                }))],
            }],
            tests: vec![],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        let hir = lower_ast(&module);
        assert_eq!(hir.name, "hello");
        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].name, "main");
        assert!(hir.functions[0].params.is_empty());
        assert_eq!(hir.functions[0].ret_ty, HirType::IoUnit);
        assert!(hir.tests.is_empty());
        assert!(check(&hir).is_ok());
    }

    #[test]
    fn hir_types_constructible() {
        let _ = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "demo".into(),
            functions: vec![HirFunction {
                name: "main".into(),
                params: vec![HirParam {
                    name: "x".into(),
                    ty: HirType::Int,
                }],
                ret_ty: HirType::IoUnit,
                is_async: false,
                is_pub: false,
                body: vec![
                    HirStmt::Let {
                        mutable: false,
                        name: "n".into(),
                        ty: HirType::Int,
                        init: HirExpr::LitInt(1),
                    },
                    HirStmt::Expr(HirExpr::Borrow {
                        mutable: true,
                        inner: Box::new(HirExpr::Path(HirPath {
                            segments: vec!["n".into()],
                        })),
                    }),
                    HirStmt::Expr(HirExpr::Binary {
                        op: HirBinOp::Add,
                        lhs: Box::new(HirExpr::LitInt(1)),
                        rhs: Box::new(HirExpr::LitInt(2)),
                    }),
                    HirStmt::Expr(HirExpr::MethodCall {
                        receiver: Box::new(HirExpr::Path(HirPath {
                            segments: vec!["xs".into()],
                        })),
                        method: "push".into(),
                        args: vec![HirExpr::LitInt(3)],
                    }),
                    HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitBool(true)],
                    })),
                    HirStmt::Assert {
                        lhs: HirExpr::LitInt(1),
                        rhs: HirExpr::LitInt(1),
                    },
                ],
            }],
            tests: vec![HirTest {
                name: "t".into(),
                body: vec![HirStmt::Assert {
                    lhs: HirExpr::LitStr("a".into()),
                    rhs: HirExpr::LitStr("a".into()),
                }],
            }],
            records: vec![],
            enums: vec![],
        };
        assert_eq!(
            HirType::Vec(Box::new(HirType::String)),
            HirType::Vec(Box::new(HirType::String))
        );
    }

    fn main_body(body: Vec<HirStmt>) -> HirModule {
        HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "m".into(),
            functions: vec![HirFunction {
                name: "main".into(),
                params: vec![],
                ret_ty: HirType::IoUnit,
                is_async: false,
                is_pub: false,
                body,
            }],
            tests: vec![],
            records: vec![],
            enums: vec![],
        }
    }

    fn path(name: &str) -> HirExpr {
        HirExpr::Path(HirPath {
            segments: vec![name.into()],
        })
    }

    #[test]
    fn move_then_use_errors() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("hi".into()),
            },
            HirStmt::Let {
                mutable: false,
                name: "t".into(),
                ty: HirType::String,
                init: path("s"),
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("s")],
            })),
        ]);
        let e = check(&hir).expect_err("must E0201");
        assert_eq!(e.to_string(), "E0201: use of moved value");
    }

    #[test]
    fn double_mut_errors() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "x".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Let {
                mutable: false,
                name: "a".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: true,
                    inner: Box::new(path("x")),
                },
            },
            HirStmt::Let {
                mutable: false,
                name: "b".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: true,
                    inner: Box::new(path("x")),
                },
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitInt(1)],
            })),
        ]);
        let e = check(&hir).expect_err("must E0202");
        assert_eq!(e.to_string(), "E0202: borrow conflict");
    }

    #[test]
    fn copy_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(42),
            },
            HirStmt::Let {
                mutable: false,
                name: "m".into(),
                ty: HirType::Int,
                init: path("n"),
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("n")],
            })),
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("m")],
            })),
        ]);
        assert!(check(&hir).is_ok());
    }

    #[test]
    fn shared_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "x".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Let {
                mutable: false,
                name: "a".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: false,
                    inner: Box::new(path("x")),
                },
            },
            HirStmt::Let {
                mutable: false,
                name: "b".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: false,
                    inner: Box::new(path("x")),
                },
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitInt(1)],
            })),
        ]);
        assert!(check(&hir).is_ok());
    }

    #[test]
    fn e0205_mut_on_immut() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "x".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Let {
                mutable: false,
                name: "a".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: true,
                    inner: Box::new(path("x")),
                },
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitInt(1)],
            })),
        ]);
        let e = check(&hir).expect_err("must E0205");
        assert_eq!(e.to_string(), "E0205: invalid let / mut binding");
    }

    #[test]
    fn e0220_if_int_cond() {
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitInt(1),
            then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("x".into())],
            }))],
            else_body: None,
        }]);
        let e = check(&hir).expect_err("must E0220");
        assert_eq!(e.to_string(), "E0220: if/while condition type ≠ Bool");
    }

    #[test]
    fn if_bool_cond_ok() {
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitBool(true),
            then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("yes".into())],
            }))],
            else_body: None,
        }]);
        assert!(check(&hir).is_ok());
    }

    #[test]
    fn e0221_bool_nonexhaustive() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "flag".into(),
                ty: HirType::Bool,
                init: HirExpr::LitBool(true),
            },
            HirStmt::Match {
                scrutinee: path("flag"),
                arms: vec![HirMatchArm {
                    pat: HirPat::LitBool(true),
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("yes".into())],
                    }))],
                }],
            },
        ]);
        let e = check(&hir).expect_err("must E0221");
        assert_eq!(e.to_string(), "E0221: non-exhaustive Bool match");
    }

    #[test]
    fn e0222_int_no_wildcard() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Match {
                scrutinee: path("n"),
                arms: vec![HirMatchArm {
                    pat: HirPat::LitInt(1),
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("a".into())],
                    }))],
                }],
            },
        ]);
        let e = check(&hir).expect_err("must E0222");
        assert_eq!(e.to_string(), "E0222: Int match requires `_` arm");
    }

    #[test]
    fn e0223_pat_type_mismatch() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Match {
                scrutinee: path("n"),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::LitBool(true),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("x".into())],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Wildcard,
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("y".into())],
                        }))],
                    },
                ],
            },
        ]);
        let e = check(&hir).expect_err("must E0223");
        assert_eq!(
            e.to_string(),
            "E0223: match pattern/scrutinee type mismatch"
        );
    }

    #[test]
    fn match_bool_exhaustive_ok() {
        let hir = main_body(vec![HirStmt::Match {
            scrutinee: HirExpr::LitBool(true),
            arms: vec![
                HirMatchArm {
                    pat: HirPat::LitBool(true),
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("yes".into())],
                    }))],
                },
                HirMatchArm {
                    pat: HirPat::LitBool(false),
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("no".into())],
                    }))],
                },
            ],
        }]);
        assert!(check(&hir).is_ok());
    }

    #[test]
    fn e0224_break_outside_while() {
        let hir = main_body(vec![HirStmt::Break]);
        let e = check(&hir).expect_err("must E0224");
        assert_eq!(e.to_string(), "E0224: break/continue outside while");
    }

    #[test]
    fn e0224_continue_in_if_only() {
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitBool(true),
            then_body: vec![HirStmt::Continue],
            else_body: None,
        }]);
        let e = check(&hir).expect_err("must E0224");
        assert_eq!(e.to_string(), "E0224: break/continue outside while");
    }

    #[test]
    fn break_inside_while_ok() {
        let hir = main_body(vec![HirStmt::While {
            cond: HirExpr::LitBool(true),
            body: vec![HirStmt::Break],
        }]);
        assert!(check(&hir).is_ok(), "break in while ok");
    }

    #[test]
    fn continue_inside_while_if_ok() {
        let hir = main_body(vec![HirStmt::While {
            cond: HirExpr::LitBool(true),
            body: vec![HirStmt::If {
                cond: HirExpr::LitBool(true),
                then_body: vec![HirStmt::Continue],
                else_body: None,
            }],
        }]);
        assert!(check(&hir).is_ok(), "continue in if-in-while ok");
    }

    #[test]
    fn match_expr_bool_ok() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitBool(true)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitBool(true),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::LitBool(false),
                        value: HirExpr::LitInt(0),
                    },
                ],
            },
        }]);
        assert!(check(&hir).is_ok(), "match expr bool ok");
    }

    #[test]
    fn match_expr_int_ok() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitInt(1)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitInt(1),
                        value: HirExpr::LitInt(10),
                    },
                    HirMatchExprArm {
                        pat: HirPat::Wildcard,
                        value: HirExpr::LitInt(20),
                    },
                ],
            },
        }]);
        assert!(check(&hir).is_ok(), "match expr int ok");
    }

    #[test]
    fn e0203_match_expr_arm_type_mismatch() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitBool(true)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitBool(true),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::LitBool(false),
                        value: HirExpr::LitBool(false),
                    },
                ],
            },
        }]);
        let e = check(&hir).expect_err("must E0203");
        assert_eq!(e.to_string(), "E0203: type mismatch");
    }

    #[test]
    fn e0217_lit_add_overflow_rejected() {
        // ADR-045 oracle: MAX + 1
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::LitInt(i64::MAX)),
                rhs: Box::new(HirExpr::LitInt(1)),
            },
        }]);
        let e = check(&hir).expect_err("must E0217 lit +");
        assert_eq!(e.to_string(), "E0217: integer overflow");
    }

    #[test]
    fn e0217_lit_sub_overflow_rejected() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Sub,
                lhs: Box::new(HirExpr::LitInt(i64::MIN)),
                rhs: Box::new(HirExpr::LitInt(1)),
            },
        }]);
        let e = check(&hir).expect_err("must E0217 lit -");
        assert_eq!(e.to_string(), "E0217: integer overflow");
    }

    #[test]
    fn e0217_lit_mul_overflow_rejected() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Mul,
                lhs: Box::new(HirExpr::LitInt(i64::MAX)),
                rhs: Box::new(HirExpr::LitInt(2)),
            },
        }]);
        let e = check(&hir).expect_err("must E0217 lit *");
        assert_eq!(e.to_string(), "E0217: integer overflow");
    }

    #[test]
    fn e0217_runtime_morph_max_plus_one_rejected() {
        // Minimum morph closing release wrap: let a = MAX; a + 1
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "a".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(i64::MAX),
            },
            HirStmt::Let {
                mutable: false,
                name: "x".into(),
                ty: HirType::Int,
                init: HirExpr::Binary {
                    op: HirBinOp::Add,
                    lhs: Box::new(path("a")),
                    rhs: Box::new(HirExpr::LitInt(1)),
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0217 morph a+1");
        assert_eq!(e.to_string(), "E0217: integer overflow");
    }

    #[test]
    fn e0217_in_range_lit_ok() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::LitInt(1)),
                rhs: Box::new(HirExpr::LitInt(1)),
            },
        }]);
        assert!(check(&hir).is_ok(), "1+1 must not E0217");
    }

    #[test]
    fn e0217_path_arith_in_range_ok() {
        // Clean F2 shape: let a=2; let b=3; a+b — foldable but in range
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "a".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(2),
            },
            HirStmt::Let {
                mutable: false,
                name: "b".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(3),
            },
            HirStmt::Let {
                mutable: false,
                name: "c".into(),
                ty: HirType::Int,
                init: HirExpr::Binary {
                    op: HirBinOp::Add,
                    lhs: Box::new(path("a")),
                    rhs: Box::new(path("b")),
                },
            },
        ]);
        assert!(check(&hir).is_ok(), "a+b in range must not E0217");
    }

    // ── ADR-294 KNOWN-INT-SCOPE (B-292-2, option A) ──
    // Sources live in `const` items outside the `#[test]` bodies (B-286-5 / VT006).
    // NOT VALIDATED: written without compiling or running (Lex gate).
    const ADR294_FP1_ELSE_BLEED: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    a = 9223372036854775807\n  } else {\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP2_MATCH_ARM_BLEED: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let flag: Bool = true\n  match flag {\n    true => {\n      a = 9223372036854775807\n    }\n    false => {\n      let r: Int = a + 1\n      print(r)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP3_LOOP_CARRIED: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let mut i: Int = 0\n  while i < 2 {\n    if i == 1 {\n      let r: Int = a + 1\n      print(r)\n    }\n    a = 0\n    i = i + 1\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP4_SHADOW_LEAK: &str = "module b292\nfn main() -> Io<()> {\n  let a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    let a: Int = 9223372036854775807\n    print(a)\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_FP7_E0216_ELSE_BLEED: &str = "module b292\nfn main() -> Io<()> {\n  let mut d: Int = 1\n  let n: Int = 3\n  let x: Int = 7\n  if n < 5 {\n    d = 0\n  } else {\n    let q: Int = x.div_euclid(d)\n    print(q)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP8_SHADOW_ELSE: &str = "module b294\nfn main() -> Io<()> {\n  let a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    let a: Int = 9223372036854775807\n    print(a)\n  } else {\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP9_NESTED_CLOBBER: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    if n < 4 {\n      a = 9223372036854775807\n    }\n  } else {\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP10_IFLET_ELSE: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let o: Option<Int> = Some(1)\n  if let Some(x) = o {\n    a = 9223372036854775807\n    print(x)\n  } else {\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FP11_WHILELET_CARRIED: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let mut o: Option<Int> = Some(1)\n  while let Some(x) = o {\n    if x == 2 {\n      let r: Int = a + 1\n      print(r)\n    }\n    a = 0\n    o = Some(2)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FPW5_ADR_HYPOTHESIS: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    a = 9223372036854775807\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_FPW6_WHILE_ASSIGN: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let mut i: Int = 0\n  while i < 2 {\n    a = 9223372036854775807\n    i = i + 1\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C1_STRAIGHT_REASSIGN_LIT: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  a = 0\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C2_REASSIGN_AFTER_CHECK: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let r: Int = a + 1\n  a = 9223372036854775807\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C3_REASSIGN_NONLIT_CLEARS: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let n: Int = 0\n  a = n\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C4_STRAIGHT_SHADOW: &str = "module b292\nfn main() -> Io<()> {\n  let a: Int = 9223372036854775807\n  let a: Int = 0\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C5_TRUE_POSITIVE: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_C6_ASSIGN_OVERFLOW_EXPR: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  a = a + 1\n  print(a)\n  print(\"ok\")\n}\n";
    const ADR294_CT1_BRANCH_KEEPS_PRESTATE: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let n: Int = 3\n  if n < 5 {\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_CT2_ASSIGN_THEN_USE_SAME_BRANCH: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    a = 9223372036854775807\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_CT3_LOOP_ASSIGN_THEN_USE: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let mut i: Int = 0\n  while i < 2 {\n    a = 9223372036854775807\n    let r: Int = a + 1\n    print(r)\n    i = i + 1\n  }\n  print(\"ok\")\n}\n";
    const ADR294_CT4_ARM_ASSIGN_THEN_USE: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let flag: Bool = true\n  match flag {\n    true => {\n      a = 9223372036854775807\n      let r: Int = a + 1\n      print(r)\n    }\n    false => {\n      print(1)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_CT5_E0216_STRAIGHT: &str = "module b294\nfn main() -> Io<()> {\n  let d: Int = 0\n  let x: Int = 7\n  let q: Int = x.div_euclid(d)\n  print(q)\n  print(\"ok\")\n}\n";
    const ADR294_CT6_ELSE_OWN_ASSIGN: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    print(1)\n  } else {\n    a = 9223372036854775807\n    let r: Int = a + 1\n    print(r)\n  }\n  print(\"ok\")\n}\n";
    const ADR294_CT7_USE_BEFORE_LOOP: &str = "module b294\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let r: Int = a + 1\n  let mut i: Int = 0\n  while i < 2 {\n    a = 0\n    i = i + 1\n  }\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_CT8_UNCLOBBERED_SURVIVES: &str = "module b294\nfn main() -> Io<()> {\n  let a: Int = 9223372036854775807\n  let n: Int = 3\n  if n < 5 {\n    print(1)\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_FN1_PREP_BRANCH_OVERWRITE: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let n: Int = 1\n  if n > 5 {\n    a = 0\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_FN2_LOOP_CARRIED: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let mut i: Int = 0\n  while i < 2 {\n    let r: Int = a + 1\n    print(r)\n    a = 9223372036854775807\n    i = i + 1\n  }\n  print(\"ok\")\n}\n";
    const ADR294_FN3_ELSE_WINS: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let n: Int = 3\n  if n < 5 {\n    a = 9223372036854775807\n  } else {\n    a = 1\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    const ADR294_FN4_MATCH_LAST_WINS: &str = "module b292\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let flag: Bool = true\n  match flag {\n    true => {\n      a = 9223372036854775807\n    }\n    false => {\n      a = 1\n    }\n  }\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";

    const ADR294_E0217: &str = "E0217: integer overflow";
    const ADR294_E0216: &str = "E0216: integer division by zero";

    /// Valid programs that the sticky `known_int` used to reject (before ADR-294).
    const ADR294_ACCEPTED: [&str; 11] = [
        ADR294_FP1_ELSE_BLEED,
        ADR294_FP2_MATCH_ARM_BLEED,
        ADR294_FP3_LOOP_CARRIED,
        ADR294_FP4_SHADOW_LEAK,
        ADR294_FP7_E0216_ELSE_BLEED,
        ADR294_FP8_SHADOW_ELSE,
        ADR294_FP9_NESTED_CLOBBER,
        ADR294_FP10_IFLET_ELSE,
        ADR294_FP11_WHILELET_CARRIED,
        ADR294_FPW5_ADR_HYPOTHESIS,
        ADR294_FPW6_WHILE_ASSIGN,
    ];

    const ADR294_MP_A1_RESULT_SIBLING: &str = "module mp\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      a = 9223372036854775807\n    }\n    Err(e) => {\n      let t: Int = a + 1\n      print(t)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_MP_A1_OPTION_SIBLING: &str = "module mp\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let o0: Option<Int> = Some(1)\n  match o0 {\n    Some(x) => {\n      a = 9223372036854775807\n    }\n    None => {\n      let t: Int = a + 1\n      print(t)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_MP_A1_ENUM_SIBLING: &str = "module mp\nenum Color {\n  Red,\n  Green,\n}\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let c: Color = Color::Red\n  match c {\n    Color::Red => {\n      a = 9223372036854775807\n    }\n    Color::Green => {\n      let t: Int = a + 1\n      print(t)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_MP_A2_RESULT_POST: &str = "module mp\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      a = 0\n    }\n    Err(e) => {\n      a = 0\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_A2_OPTION_POST: &str = "module mp\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let o0: Option<Int> = Some(1)\n  match o0 {\n    Some(x) => {\n      a = 0\n    }\n    None => {\n      a = 0\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_A2_ENUM_POST: &str = "module mp\nenum Color {\n  Red,\n  Green,\n}\nfn main() -> Io<()> {\n  let mut a: Int = 9223372036854775807\n  let c: Color = Color::Red\n  match c {\n    Color::Red => {\n      a = 0\n    }\n    Color::Green => {\n      a = 0\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_A3_RESULT_POST_ONESIDE: &str = "module mp\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      a = 9223372036854775807\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_B1_RESULT_BINDING_SHADOW: &str = "module mp\nfn main() -> Io<()> {\n  let x: Int = 9223372036854775807\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      let t: Int = x + 1\n      print(t)\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_MP_B1_OPTION_BINDING_SHADOW: &str = "module mp\nfn main() -> Io<()> {\n  let x: Int = 9223372036854775807\n  let o0: Option<Int> = Some(1)\n  match o0 {\n    Some(x) => {\n      let t: Int = x + 1\n      print(t)\n    }\n    None => {\n      print(\"none\")\n    }\n  }\n  print(\"ok\")\n}\n";
    const ADR294_MP_B2_RESULT_BINDING_AFTER: &str = "module mp\nfn main() -> Io<()> {\n  let x: Int = 9223372036854775807\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      print(x)\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n  let t: Int = x + 1\n  print(t)\n}\n";
    const ADR294_MP_B3_RESULT_BINDING_ZERO_OUTER: &str = "module mp\nfn main() -> Io<()> {\n  let x: Int = 0\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      let t: Int = x + 1\n      print(t)\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n  let u: Int = x + 1\n  print(u)\n}\n";
    const ADR294_MP_C1_RESULT_NEG: &str = "module mp\nfn main() -> Io<()> {\n  let a: Int = 9223372036854775807\n  let mut b: Int = 0\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      b = 1\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_C1_OPTION_NEG: &str = "module mp\nfn main() -> Io<()> {\n  let a: Int = 9223372036854775807\n  let mut b: Int = 0\n  let o0: Option<Int> = Some(1)\n  match o0 {\n    Some(x) => {\n      b = 1\n    }\n    None => {\n      print(\"none\")\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_C1_ENUM_NEG: &str = "module mp\nenum Color {\n  Red,\n  Green,\n}\nfn main() -> Io<()> {\n  let a: Int = 9223372036854775807\n  let mut b: Int = 0\n  let c: Color = Color::Red\n  match c {\n    Color::Red => {\n      b = 1\n    }\n    Color::Green => {\n      print(0)\n    }\n  }\n  let t: Int = a + 1\n  print(t)\n}\n";
    const ADR294_MP_C2_RESULT_IN_ARM_NEG: &str = "module mp\nfn main() -> Io<()> {\n  let r0: Result<Int, Int> = Ok(1)\n  match r0 {\n    Ok(x) => {\n      let a: Int = 9223372036854775807\n      let t: Int = a + 1\n      print(t)\n    }\n    Err(e) => {\n      print(e)\n    }\n  }\n}\n";

    #[test]
    fn adr294_sibling_branch_loop_and_scope_do_not_keep_known_int() {
        for src in ADR294_ACCEPTED {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn adr294_straight_line_controls_keep_their_codes() {
        for src in [
            ADR294_C1_STRAIGHT_REASSIGN_LIT,
            ADR294_C2_REASSIGN_AFTER_CHECK,
            ADR294_C3_REASSIGN_NONLIT_CLEARS,
            ADR294_C4_STRAIGHT_SHADOW,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
        for src in [ADR294_C5_TRUE_POSITIVE, ADR294_C6_ASSIGN_OVERFLOW_EXPR] {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR294_E0217.to_string()),
                "{src}"
            );
        }
        assert_eq!(
            adr283_check_src(ADR294_CT5_E0216_STRAIGHT),
            Err(ADR294_E0216.to_string())
        );
    }

    #[test]
    fn adr294_rejection_inside_the_same_branch_or_before_the_loop_is_kept() {
        for src in [
            ADR294_CT1_BRANCH_KEEPS_PRESTATE,
            ADR294_CT2_ASSIGN_THEN_USE_SAME_BRANCH,
            ADR294_CT3_LOOP_ASSIGN_THEN_USE,
            ADR294_CT4_ARM_ASSIGN_THEN_USE,
            ADR294_CT6_ELSE_OWN_ASSIGN,
            ADR294_CT7_USE_BEFORE_LOOP,
            ADR294_CT8_UNCLOBBERED_SURVIVES,
        ] {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR294_E0217.to_string()),
                "{src}"
            );
        }
    }

    #[test]
    fn adr294_known_false_negatives_stay_accepted_backlog() {
        // Documented backlog (ADR-292 D5 / B-292-2): path-dependent overflow is not claimed
        // statically; ADR-292 D1 makes it a uniform runtime panic. If a flow analysis ever
        // lands, these four flip to E0217 and this test must be updated deliberately.
        for src in [
            ADR294_FN1_PREP_BRANCH_OVERWRITE,
            ADR294_FN2_LOOP_CARRIED,
            ADR294_FN3_ELSE_WINS,
            ADR294_FN4_MATCH_LAST_WINS,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn adr294_match_pattern_binding_sibling_arm_does_not_see_assignment() {
        // Result / Option / user enum: an arm that assigns a known Int must not leak it into the
        // sibling arm (the enum arm checks per statement with `check_stmt`).
        for src in [
            ADR294_MP_A1_RESULT_SIBLING,
            ADR294_MP_A1_OPTION_SIBLING,
            ADR294_MP_A1_ENUM_SIBLING,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn adr294_match_pattern_binding_post_statement_use_after_every_arm_assigns() {
        // Old value is MAX, every arm overwrites it with 0: `a + 1` after the match is fine.
        for src in [
            ADR294_MP_A2_RESULT_POST,
            ADR294_MP_A2_OPTION_POST,
            ADR294_MP_A2_ENUM_POST,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn adr294_match_pattern_binding_path_dependent_post_use_is_accepted() {
        // Only one arm assigns MAX: overflow depends on the path (ADR-292 D1 panics at runtime);
        // no path merge, so the known value is dropped after the match. Documented loss.
        assert_eq!(adr283_check_src(ADR294_MP_A3_RESULT_POST_ONESIDE), Ok(()));
    }

    #[test]
    fn adr294_match_pattern_binding_shadow_known_value_stays_outside() {
        // The pattern binding `x` shadows an outer `x` with a known value: the known value must
        // not leak into the arm (Ok), and the outer value must survive the match (E0217 after).
        for src in [
            ADR294_MP_B1_RESULT_BINDING_SHADOW,
            ADR294_MP_B1_OPTION_BINDING_SHADOW,
            ADR294_MP_B3_RESULT_BINDING_ZERO_OUTER,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
        assert_eq!(
            adr283_check_src(ADR294_MP_B2_RESULT_BINDING_AFTER),
            Err(ADR294_E0217.to_string())
        );
    }

    #[test]
    fn adr294_match_pattern_binding_real_overflow_outside_the_arms_is_kept() {
        // `a` is never assigned or re-declared inside the match: still E0217 after it; and an
        // overflow wholly inside one arm is still E0217.
        for src in [
            ADR294_MP_C1_RESULT_NEG,
            ADR294_MP_C1_OPTION_NEG,
            ADR294_MP_C1_ENUM_NEG,
            ADR294_MP_C2_RESULT_IN_ARM_NEG,
        ] {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR294_E0217.to_string()),
                "{src}"
            );
        }
    }

    #[test]
    fn e0270_result_nonexhaustive() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::String)),
                init: HirExpr::Ok(Box::new(HirExpr::LitInt(1))),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![HirMatchArm {
                    pat: HirPat::Ok("x".into()),
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::Path(HirPath {
                            segments: vec!["x".into()],
                        })],
                    }))],
                }],
            },
        ]);
        let e = check(&hir).expect_err("must E0270");
        assert_eq!(e.to_string(), "E0270: non-exhaustive result match");
    }

    #[test]
    fn e0272_err_default_lit_swallowed() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                init: HirExpr::Err(Box::new(HirExpr::LitInt(1))),
            },
            HirStmt::Let {
                mutable: false,
                name: "v".into(),
                ty: HirType::Int,
                init: HirExpr::Match {
                    scrutinee: Box::new(HirExpr::Path(HirPath {
                        segments: vec!["r".into()],
                    })),
                    arms: vec![
                        HirMatchExprArm {
                            pat: HirPat::Ok("x".into()),
                            value: HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            }),
                        },
                        HirMatchExprArm {
                            pat: HirPat::Err("e".into()),
                            value: HirExpr::LitInt(0),
                        },
                    ],
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0272");
        assert_eq!(e.to_string(), "E0272: result error swallowed");
    }

    #[test]
    fn e0272_err_unused_print_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::String)),
                init: HirExpr::Err(Box::new(HirExpr::LitStr("fail".into()))),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("x".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("e".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("ok".into())],
                        }))],
                    },
                ],
            },
        ]);
        let e = check(&hir).expect_err("must E0272");
        assert_eq!(e.to_string(), "E0272: result error swallowed");
    }

    #[test]
    fn e0272_err_used_not_swallowed() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::String)),
                init: HirExpr::Err(Box::new(HirExpr::LitStr("fail".into()))),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("x".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("e".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["e".into()],
                            })],
                        }))],
                    },
                ],
            },
        ]);
        assert!(check(&hir).is_ok(), "using Err binding must not E0272");
    }

    #[test]
    fn e0340_host_io_discard_expr_stmt() {
        let hir = main_body(vec![HirStmt::Expr(HirExpr::Call(HirCall {
            callee: "host::read_text".into(),
            args: vec![HirExpr::LitStr("p".into())],
        }))]);
        let e = check(&hir).expect_err("must E0340");
        assert!(e.to_string().contains("E0340"), "got {e}");
        assert!(e.to_string().contains("io result discarded"), "got {e}");
    }

    #[test]
    fn e0340_host_io_unwrap_or_empty() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::String), Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "host::read_text".into(),
                    args: vec![HirExpr::LitStr("p".into())],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "t".into(),
                ty: HirType::String,
                init: HirExpr::MethodCall {
                    receiver: Box::new(HirExpr::Path(HirPath {
                        segments: vec!["r".into()],
                    })),
                    method: "unwrap_or".into(),
                    args: vec![HirExpr::LitStr("".into())],
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0340");
        assert!(e.to_string().contains("E0340"), "got {e}");
    }

    #[test]
    fn e0340_host_io_err_empty_lit_match_stmt() {
        // Stmt form: Err(_) => print("") is empty theater → E0340 (match-expr String OUT of Bool|Int).
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::String), Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "host::read_text".into(),
                    args: vec![HirExpr::LitStr("p".into())],
                }),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("t".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["t".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("_".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("".into())],
                        }))],
                    },
                ],
            },
        ]);
        let e = check(&hir).expect_err("must E0340");
        assert!(e.to_string().contains("E0340"), "got {e}");
    }

    #[test]
    fn e0340_host_io_err_zero_lit_match_expr() {
        // write_text → Result<Int,Int>; Err(_) => 0 default theater via match-expr.
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "host::write_text".into(),
                    args: vec![HirExpr::LitStr("p".into()), HirExpr::LitStr("b".into())],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "v".into(),
                ty: HirType::Int,
                init: HirExpr::Match {
                    scrutinee: Box::new(HirExpr::Path(HirPath {
                        segments: vec!["r".into()],
                    })),
                    arms: vec![
                        HirMatchExprArm {
                            pat: HirPat::Ok("x".into()),
                            value: HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            }),
                        },
                        HirMatchExprArm {
                            pat: HirPat::Err("_".into()),
                            value: HirExpr::LitInt(0),
                        },
                    ],
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0340");
        assert!(e.to_string().contains("E0340"), "got {e}");
    }

    #[test]
    fn e0340_host_io_err_fail_message_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::String), Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "host::read_text".into(),
                    args: vec![HirExpr::LitStr("p".into())],
                }),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("t".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["t".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("_".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("read_failed".into())],
                        }))],
                    },
                ],
            },
        ]);
        assert!(
            check(&hir).is_ok(),
            "explicit fail message must not E0340/E0272"
        );
    }

    #[test]
    fn e0341_host_cli_unwrap_or_empty() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::String)),
                init: HirExpr::Call(HirCall {
                    callee: "host::cli_arg".into(),
                    args: vec![HirExpr::LitInt(1)],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "p".into(),
                ty: HirType::String,
                init: HirExpr::MethodCall {
                    receiver: Box::new(HirExpr::Path(HirPath {
                        segments: vec!["o".into()],
                    })),
                    method: "unwrap_or".into(),
                    args: vec![HirExpr::LitStr("".into())],
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0341");
        assert!(e.to_string().contains("E0341"), "got {e}");
        assert!(e.to_string().contains("required arg miss as ok"), "got {e}");
    }

    #[test]
    fn e0341_host_json_unwrap_or_zero() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "host::json_get_int".into(),
                    args: vec![
                        HirExpr::LitStr("{\"n\":1}".into()),
                        HirExpr::LitStr("n".into()),
                    ],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::MethodCall {
                    receiver: Box::new(HirExpr::Path(HirPath {
                        segments: vec!["o".into()],
                    })),
                    method: "unwrap_or".into(),
                    args: vec![HirExpr::LitInt(0)],
                },
            },
        ]);
        let e = check(&hir).expect_err("must E0341");
        assert!(e.to_string().contains("E0341"), "got {e}");
    }

    #[test]
    fn e0341_host_cli_none_empty_lit_match_stmt() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::String)),
                init: HirExpr::Call(HirCall {
                    callee: "host::cli_arg".into(),
                    args: vec![HirExpr::LitInt(1)],
                }),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["o".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Some("p".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["p".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::None,
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("".into())],
                        }))],
                    },
                ],
            },
        ]);
        let e = check(&hir).expect_err("must E0341");
        assert!(e.to_string().contains("E0341"), "got {e}");
    }

    #[test]
    fn e0341_host_cli_none_fail_message_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::String)),
                init: HirExpr::Call(HirCall {
                    callee: "host::cli_arg".into(),
                    args: vec![HirExpr::LitInt(1)],
                }),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["o".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Some("p".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["p".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::None,
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("usage".into())],
                        }))],
                    },
                ],
            },
        ]);
        assert!(
            check(&hir).is_ok(),
            "explicit fail message must not E0341/E0274"
        );
    }

    #[test]
    fn e0274_none_default_lit() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::Int)),
                init: HirExpr::None,
            },
            HirStmt::Let {
                mutable: false,
                name: "v".into(),
                ty: HirType::Int,
                init: HirExpr::Match {
                    scrutinee: Box::new(path("o")),
                    arms: vec![
                        HirMatchExprArm {
                            pat: HirPat::Some("x".into()),
                            value: path("x"),
                        },
                        HirMatchExprArm {
                            pat: HirPat::None,
                            value: HirExpr::LitInt(0),
                        },
                    ],
                },
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("v")],
            })),
        ]);
        let e = check(&hir).expect_err("must E0274");
        assert_eq!(e.to_string(), "E0274: option none swallowed");
    }

    #[test]
    fn e0274_none_print_string_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "o".into(),
                ty: HirType::Option(Box::new(HirType::Int)),
                init: HirExpr::None,
            },
            HirStmt::Match {
                scrutinee: path("o"),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Some("x".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![path("x")],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::None,
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("none".into())],
                        }))],
                    },
                ],
            },
        ]);
        assert!(check(&hir).is_ok(), "print string on None must not E0274");
    }

    #[test]
    fn result_ok_err_exhaustive_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::String)),
                init: HirExpr::Ok(Box::new(HirExpr::LitInt(42))),
            },
            HirStmt::Match {
                scrutinee: HirExpr::Path(HirPath {
                    segments: vec!["r".into()],
                }),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("x".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            })],
                        }))],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("e".into()),
                        body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::Path(HirPath {
                                segments: vec!["e".into()],
                            })],
                        }))],
                    },
                ],
            },
        ]);
        assert!(check(&hir).is_ok(), "Result match should pass");
    }

    #[test]
    fn e0221_match_expr_non_exhaustive() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitBool(true)),
                arms: vec![HirMatchExprArm {
                    pat: HirPat::LitBool(true),
                    value: HirExpr::LitInt(1),
                }],
            },
        }]);
        let e = check(&hir).expect_err("must E0221");
        assert_eq!(e.to_string(), "E0221: non-exhaustive Bool match");
    }

    #[test]
    fn e0225_match_expr_bool_same_const() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitBool(true)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitBool(true),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::LitBool(false),
                        value: HirExpr::LitInt(1),
                    },
                ],
            },
        }]);
        let e = check(&hir).expect_err("must E0225");
        assert_eq!(e.to_string(), "E0225: vacuous match arms");
    }

    #[test]
    fn e0225_match_expr_int_same_const() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitInt(0)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitInt(0),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::LitInt(1),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::Wildcard,
                        value: HirExpr::LitInt(1),
                    },
                ],
            },
        }]);
        let e = check(&hir).expect_err("must E0225");
        assert_eq!(e.to_string(), "E0225: vacuous match arms");
    }

    #[test]
    fn match_expr_different_const_not_e0225() {
        // Control: f22-04/05 style — different arm values still PASS
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Match {
                scrutinee: Box::new(HirExpr::LitBool(true)),
                arms: vec![
                    HirMatchExprArm {
                        pat: HirPat::LitBool(true),
                        value: HirExpr::LitInt(1),
                    },
                    HirMatchExprArm {
                        pat: HirPat::LitBool(false),
                        value: HirExpr::LitInt(0),
                    },
                ],
            },
        }]);
        assert!(
            check(&hir).is_ok(),
            "different arm constants must not E0225"
        );
    }

    #[test]
    fn e0226_while_false_lit_rejected() {
        let hir = main_body(vec![
            HirStmt::While {
                cond: HirExpr::LitBool(false),
                body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "print".into(),
                    args: vec![HirExpr::LitStr("x".into())],
                }))],
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("ok".into())],
            })),
        ]);
        let e = check(&hir).expect_err("must E0226");
        assert_eq!(e.to_string(), "E0226: vacuous while false");
    }

    #[test]
    fn e0226_while_false_empty_body_rejected() {
        let hir = main_body(vec![HirStmt::While {
            cond: HirExpr::LitBool(false),
            body: vec![],
        }]);
        let e = check(&hir).expect_err("must E0226 empty");
        assert_eq!(e.to_string(), "E0226: vacuous while false");
    }

    #[test]
    fn while_false_via_bool_binding_ok() {
        // Clean control: non-literal Bool cond must NOT E0226 (ADR-040 OUT)
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "b".into(),
                ty: HirType::Bool,
                init: HirExpr::LitBool(false),
            },
            HirStmt::While {
                cond: path("b"),
                body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "print".into(),
                    args: vec![HirExpr::LitStr("x".into())],
                }))],
            },
        ]);
        assert!(
            check(&hir).is_ok(),
            "while b with Bool binding must not E0226"
        );
    }

    #[test]
    fn while_true_lit_ok() {
        let hir = main_body(vec![HirStmt::While {
            cond: HirExpr::LitBool(true),
            body: vec![HirStmt::Break],
        }]);
        assert!(check(&hir).is_ok(), "while true must not E0226");
    }

    #[test]
    fn e0277_while_let_none_lit() {
        let hir = main_body(vec![HirStmt::WhileLet {
            pat: HirPat::Some("x".into()),
            scrutinee: HirExpr::None,
            body: vec![HirStmt::Expr(HirExpr::LitInt(1))],
        }]);
        let e = check(&hir).expect_err("must E0277");
        assert_eq!(e.to_string(), "E0277: vacuous while-let none");
    }

    #[test]
    fn e0282_while_let_ok_on_err_lit() {
        let hir = main_body(vec![HirStmt::WhileLet {
            pat: HirPat::Ok("x".into()),
            scrutinee: HirExpr::Err(Box::new(HirExpr::LitInt(0))),
            body: vec![HirStmt::Expr(HirExpr::LitInt(1))],
        }]);
        let e = check(&hir).expect_err("must E0282");
        assert_eq!(e.to_string(), "E0282: vacuous while-let result");
    }

    #[test]
    fn e0282_while_let_err_on_ok_lit() {
        let hir = main_body(vec![HirStmt::WhileLet {
            pat: HirPat::Err("e".into()),
            scrutinee: HirExpr::Ok(Box::new(HirExpr::LitInt(0))),
            body: vec![HirStmt::Expr(HirExpr::LitInt(1))],
        }]);
        let e = check(&hir).expect_err("must E0282");
        assert_eq!(e.to_string(), "E0282: vacuous while-let result");
    }

    #[test]
    fn e0278_abs_min_rejected() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(i64::MIN),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(HirExpr::Path(HirPath {
                    segments: vec!["n".into()],
                })),
                method: "abs".into(),
                args: vec![],
            }),
        ]);
        let e = check(&hir).expect_err("must E0278");
        assert_eq!(e.to_string(), "E0278: integer abs overflow");
    }

    #[test]
    fn e0279_clamp_lo_gt_hi_rejected() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(0),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(HirExpr::Path(HirPath {
                    segments: vec!["n".into()],
                })),
                method: "clamp".into(),
                args: vec![HirExpr::LitInt(5), HirExpr::LitInt(1)],
            }),
        ]);
        let e = check(&hir).expect_err("must E0279");
        assert_eq!(e.to_string(), "E0279: invalid clamp range");
    }

    #[test]
    fn e0280_repeat_neg_rejected() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("ab".into()),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(HirExpr::Path(HirPath {
                    segments: vec!["s".into()],
                })),
                method: "repeat".into(),
                args: vec![HirExpr::LitInt(-1)],
            }),
        ]);
        let e = check(&hir).expect_err("must E0280");
        assert_eq!(e.to_string(), "E0280: negative repeat count");
    }

    #[test]
    fn e0281_pow_neg_exp() {
        let hir = main_body(vec![HirStmt::Expr(HirExpr::MethodCall {
            receiver: Box::new(HirExpr::LitInt(2)),
            method: "pow".into(),
            args: vec![HirExpr::LitInt(-1)],
        })]);
        let e = check(&hir).expect_err("must E0281");
        assert_eq!(e.to_string(), "E0281: negative pow exponent");
    }

    #[test]
    fn e0283_pow_overflow() {
        let hir = main_body(vec![HirStmt::Expr(HirExpr::MethodCall {
            receiver: Box::new(HirExpr::LitInt(2)),
            method: "pow".into(),
            args: vec![HirExpr::LitInt(63)],
        })]);
        let e = check(&hir).expect_err("must E0283");
        assert_eq!(e.to_string(), "E0283: integer pow overflow");
    }

    #[test]
    fn e0227_if_false_lit_no_else_rejected() {
        let hir = main_body(vec![
            HirStmt::If {
                cond: HirExpr::LitBool(false),
                then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "print".into(),
                    args: vec![HirExpr::LitStr("x".into())],
                }))],
                else_body: None,
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("ok".into())],
            })),
        ]);
        let e = check(&hir).expect_err("must E0227");
        assert_eq!(e.to_string(), "E0227: vacuous if false");
    }

    #[test]
    fn e0227_if_false_empty_then_rejected() {
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitBool(false),
            then_body: vec![],
            else_body: None,
        }]);
        let e = check(&hir).expect_err("must E0227 empty");
        assert_eq!(e.to_string(), "E0227: vacuous if false");
    }

    #[test]
    fn if_false_with_else_ok_out() {
        // ADR-041 OUT: if false … else { … } must NOT E0227
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitBool(false),
            then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("x".into())],
            }))],
            else_body: Some(vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("else".into())],
            }))]),
        }]);
        assert!(check(&hir).is_ok(), "if false else must not E0227");
    }

    #[test]
    fn if_false_via_bool_binding_ok() {
        // Clean: non-literal Bool cond must NOT E0227
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "b".into(),
                ty: HirType::Bool,
                init: HirExpr::LitBool(false),
            },
            HirStmt::If {
                cond: path("b"),
                then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "print".into(),
                    args: vec![HirExpr::LitStr("x".into())],
                }))],
                else_body: None,
            },
        ]);
        assert!(check(&hir).is_ok(), "if b with Bool binding must not E0227");
    }

    #[test]
    fn if_true_lit_ok() {
        let hir = main_body(vec![HirStmt::If {
            cond: HirExpr::LitBool(true),
            then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("ok".into())],
            }))],
            else_body: None,
        }]);
        assert!(check(&hir).is_ok(), "if true must not E0227");
    }

    #[test]
    fn string_len_and_is_empty_ok_no_move() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("hi".into()),
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::MethodCall {
                    receiver: Box::new(path("s")),
                    method: "len".into(),
                    args: vec![],
                }],
            })),
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::MethodCall {
                    receiver: Box::new(path("s")),
                    method: "is_empty".into(),
                    args: vec![],
                }],
            })),
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("s")],
            })),
        ]);
        assert!(check(&hir).is_ok(), "string len/is_empty ok");
    }

    #[test]
    fn e0206_int_len_rejected() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "n".into(),
                ty: HirType::Int,
                init: HirExpr::LitInt(1),
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::MethodCall {
                    receiver: Box::new(path("n")),
                    method: "len".into(),
                    args: vec![],
                }],
            })),
        ]);
        let e = check(&hir).expect_err("must E0206");
        assert_eq!(e.to_string(), "E0206: method not in F2 std whitelist");
    }

    #[test]
    fn e0206_unknown_method_rejected() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "retain".into(),
                args: vec![],
            }),
        ]);
        let e = check(&hir).expect_err("must E0206");
        assert_eq!(e.to_string(), "E0206: method not in F2 std whitelist");
    }

    #[test]
    fn insert_fallible_ok_and_e0311() {
        // ADR-260: in-range insert types as Result<(), Int>
        let ok = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Unit), Box::new(HirType::Int)),
                init: HirExpr::MethodCall {
                    receiver: Box::new(path("v")),
                    method: "insert".into(),
                    args: vec![HirExpr::LitInt(0), HirExpr::LitInt(1)],
                },
            },
            // ADR-295 (S2-D2): `r` is read as a receiver, so the Result binding is used.
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("r")),
                method: "is_ok".into(),
                args: vec![],
            }),
        ]);
        check(&ok).expect("insert in-range must pass");

        let neg = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "insert".into(),
                args: vec![HirExpr::LitInt(-1), HirExpr::LitInt(0)],
            }),
        ]);
        let e = check(&neg).expect_err("must E0311");
        assert!(e.to_string().contains("E0311"), "got {e}");
    }

    #[test]
    fn set_fallible_ok_and_e0319() {
        // ADR-265 / CORE-0.6-SET-FALLIBLE-20260926: in-bounds set → Result<(), Int>
        let ok = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Result(Box::new(HirType::Unit), Box::new(HirType::Int)),
                init: HirExpr::MethodCall {
                    receiver: Box::new(path("v")),
                    method: "set".into(),
                    args: vec![HirExpr::LitInt(0), HirExpr::LitInt(1)],
                },
            },
            // ADR-295 (S2-D2): `r` is read as a receiver, so the Result binding is used.
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("r")),
                method: "is_ok".into(),
                args: vec![],
            }),
        ]);
        check(&ok).expect("set in-range must pass");

        let neg = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "set".into(),
                args: vec![HirExpr::LitInt(-1), HirExpr::LitInt(0)],
            }),
        ]);
        let e = check(&neg).expect_err("must E0319");
        assert!(e.to_string().contains("E0319"), "got {e}");
        assert!(e.to_string().contains("negative set index"), "got {e}");
    }

    #[test]
    fn clear_on_mut_vec_ok() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "clear".into(),
                args: vec![],
            }),
        ]);
        assert!(check(&hir).is_ok(), "clear on mut Vec must pass");
    }

    #[test]
    fn e0202_clear_requires_mut() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "clear".into(),
                args: vec![],
            }),
        ]);
        let e = check(&hir).expect_err("must E0202");
        assert_eq!(e.to_string(), "E0202: borrow conflict");
    }

    #[test]
    fn e0202_push_requires_mut() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: false,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::MethodCall {
                receiver: Box::new(path("v")),
                method: "push".into(),
                args: vec![HirExpr::LitInt(1)],
            }),
        ]);
        let e = check(&hir).expect_err("must E0202");
        assert_eq!(e.to_string(), "E0202: borrow conflict");
    }

    #[test]
    fn e0203_len_bad_arity() {
        let hir = main_body(vec![
            HirStmt::Let {
                mutable: true,
                name: "v".into(),
                ty: HirType::Vec(Box::new(HirType::Int)),
                init: HirExpr::Call(HirCall {
                    callee: "Vec::new".into(),
                    args: vec![],
                }),
            },
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::MethodCall {
                    receiver: Box::new(path("v")),
                    method: "len".into(),
                    args: vec![HirExpr::LitInt(1)],
                }],
            })),
        ]);
        let e = check(&hir).expect_err("must E0203");
        assert_eq!(e.to_string(), "E0203: type mismatch");
    }

    /// Async module: `tick` + `main` both async Io<()>.
    fn async_tick_main(main_body: Vec<HirStmt>) -> HirModule {
        HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "m".into(),
            functions: vec![
                HirFunction {
                    name: "tick".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    is_async: true,
                    is_pub: false,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("tick".into())],
                    }))],
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    is_async: true,
                    is_pub: false,
                    body: main_body,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        }
    }

    #[test]
    fn e0242_borrow_across_await() {
        let hir = async_tick_main(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("hi".into()),
            },
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: false,
                    inner: Box::new(path("s")),
                },
            },
            HirStmt::Expr(HirExpr::Await {
                inner: Box::new(HirExpr::Call(HirCall {
                    callee: "tick".into(),
                    args: vec![],
                })),
            }),
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("r")],
            })),
        ]);
        let e = check(&hir).expect_err("must E0242");
        assert_eq!(e.to_string(), "E0242: borrow held across await");
    }

    #[test]
    fn e0242_borrow_across_await_even_if_unused() {
        // Loan still live at await even when `r` is never used after.
        let hir = async_tick_main(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("hi".into()),
            },
            HirStmt::Let {
                mutable: false,
                name: "r".into(),
                ty: HirType::Int,
                init: HirExpr::Borrow {
                    mutable: false,
                    inner: Box::new(path("s")),
                },
            },
            HirStmt::Expr(HirExpr::Await {
                inner: Box::new(HirExpr::Call(HirCall {
                    callee: "tick".into(),
                    args: vec![],
                })),
            }),
        ]);
        let e = check(&hir).expect_err("must E0242");
        assert_eq!(e.to_string(), "E0242: borrow held across await");
    }

    #[test]
    fn string_move_across_await_ok() {
        // Control: owned String used after await (no borrow) must PASS.
        let hir = async_tick_main(vec![
            HirStmt::Let {
                mutable: false,
                name: "s".into(),
                ty: HirType::String,
                init: HirExpr::LitStr("hi".into()),
            },
            HirStmt::Expr(HirExpr::Await {
                inner: Box::new(HirExpr::Call(HirCall {
                    callee: "tick".into(),
                    args: vec![],
                })),
            }),
            HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![path("s")],
            })),
        ]);
        assert!(check(&hir).is_ok(), "string move across await ok");
    }

    /// ADR-044 helper: `fn div(a,b){a/b}` (+ optional rem twin) + main + tests.
    fn module_with_div_helpers(
        main_body: Vec<HirStmt>,
        tests: Vec<HirTest>,
        include_rem: bool,
    ) -> HirModule {
        let mut functions = vec![HirFunction {
            name: "div".into(),
            params: vec![
                HirParam {
                    name: "a".into(),
                    ty: HirType::Int,
                },
                HirParam {
                    name: "b".into(),
                    ty: HirType::Int,
                },
            ],
            ret_ty: HirType::Int,
            is_async: false,
            is_pub: false,
            body: vec![HirStmt::Expr(HirExpr::Binary {
                op: HirBinOp::Div,
                lhs: Box::new(path("a")),
                rhs: Box::new(path("b")),
            })],
        }];
        if include_rem {
            functions.push(HirFunction {
                name: "rem".into(),
                params: vec![
                    HirParam {
                        name: "a".into(),
                        ty: HirType::Int,
                    },
                    HirParam {
                        name: "b".into(),
                        ty: HirType::Int,
                    },
                ],
                ret_ty: HirType::Int,
                is_async: false,
                is_pub: false,
                body: vec![HirStmt::Expr(HirExpr::Binary {
                    op: HirBinOp::Rem,
                    lhs: Box::new(path("a")),
                    rhs: Box::new(path("b")),
                })],
            });
        }
        functions.push(HirFunction {
            name: "main".into(),
            params: vec![],
            ret_ty: HirType::IoUnit,
            is_async: false,
            is_pub: false,
            body: main_body,
        });
        HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "m".into(),
            functions,
            tests,
            records: vec![],
            enums: vec![],
        }
    }

    #[test]
    fn e0216_lit_div0_rejected() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Div,
                lhs: Box::new(HirExpr::LitInt(10)),
                rhs: Box::new(HirExpr::LitInt(0)),
            },
        }]);
        let e = check(&hir).expect_err("must E0216 lit /");
        assert_eq!(e.to_string(), "E0216: integer division by zero");
    }

    #[test]
    fn e0216_lit_rem0_rejected() {
        let hir = main_body(vec![HirStmt::Let {
            mutable: false,
            name: "x".into(),
            ty: HirType::Int,
            init: HirExpr::Binary {
                op: HirBinOp::Rem,
                lhs: Box::new(HirExpr::LitInt(10)),
                rhs: Box::new(HirExpr::LitInt(0)),
            },
        }]);
        let e = check(&hir).expect_err("must E0216 lit %");
        assert_eq!(e.to_string(), "E0216: integer division by zero");
    }

    #[test]
    fn e0216_div_call_lit0_in_test_rejected() {
        // Canonical ADR-044 fixture shape: assert div(10, 0) == 0
        let hir = module_with_div_helpers(
            vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("ok".into())],
            }))],
            vec![HirTest {
                name: "t_div0".into(),
                body: vec![HirStmt::Assert {
                    lhs: HirExpr::Call(HirCall {
                        callee: "div".into(),
                        args: vec![HirExpr::LitInt(10), HirExpr::LitInt(0)],
                    }),
                    rhs: HirExpr::LitInt(0),
                }],
            }],
            false,
        );
        let e = check(&hir).expect_err("must E0216 call");
        assert_eq!(e.to_string(), "E0216: integer division by zero");
    }

    #[test]
    fn e0216_rem_call_lit0_rejected() {
        let hir = module_with_div_helpers(
            vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::Call(HirCall {
                    callee: "rem".into(),
                    args: vec![HirExpr::LitInt(10), HirExpr::LitInt(0)],
                })],
            }))],
            vec![],
            true,
        );
        let e = check(&hir).expect_err("must E0216 rem call");
        assert_eq!(e.to_string(), "E0216: integer division by zero");
    }

    #[test]
    fn e0216_div_call_nonzero_ok() {
        let hir = module_with_div_helpers(
            vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::Call(HirCall {
                    callee: "div".into(),
                    args: vec![HirExpr::LitInt(10), HirExpr::LitInt(2)],
                })],
            }))],
            vec![],
            false,
        );
        assert!(check(&hir).is_ok(), "div(10,2) must not E0216");
    }

    #[test]
    fn e0216_guarded_div_path_ok() {
        // Clean: if b != 0 { print(div(10, b)) } — 2nd arg not LitInt 0
        let hir = module_with_div_helpers(
            vec![
                HirStmt::Let {
                    mutable: false,
                    name: "b".into(),
                    ty: HirType::Int,
                    init: HirExpr::LitInt(2),
                },
                HirStmt::If {
                    cond: HirExpr::Binary {
                        op: HirBinOp::Ne,
                        lhs: Box::new(path("b")),
                        rhs: Box::new(HirExpr::LitInt(0)),
                    },
                    then_body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::Call(HirCall {
                            callee: "div".into(),
                            args: vec![HirExpr::LitInt(10), path("b")],
                        })],
                    }))],
                    else_body: None,
                },
            ],
            vec![],
            false,
        );
        assert!(check(&hir).is_ok(), "guarded div(10,b) must not E0216");
    }

    #[test]
    fn e0216_div_fn_def_alone_ok() {
        // Defining fn div(a,b){a/b} without a LitInt-0 call site must PASS
        let hir = module_with_div_helpers(
            vec![HirStmt::Expr(HirExpr::Call(HirCall {
                callee: "print".into(),
                args: vec![HirExpr::LitStr("ok".into())],
            }))],
            vec![],
            false,
        );
        assert!(
            check(&hir).is_ok(),
            "div helper definition alone must not E0216"
        );
    }

    #[test]
    fn timeout_ok_in_async() {
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "quick".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("fast".into())],
                    }))],
                    is_async: true,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![
                        HirStmt::Let {
                            mutable: false,
                            name: "r".into(),
                            ty: HirType::Result(Box::new(HirType::Unit), Box::new(HirType::String)),
                            init: HirExpr::Call(HirCall {
                                callee: "timeout".into(),
                                args: vec![
                                    HirExpr::LitInt(1000),
                                    HirExpr::Call(HirCall {
                                        callee: "quick".into(),
                                        args: vec![],
                                    }),
                                ],
                            }),
                        },
                        // ADR-295 (S2-D2): `r` is read as a receiver, so the Result binding is used.
                        HirStmt::Expr(HirExpr::MethodCall {
                            receiver: Box::new(HirExpr::Path(HirPath {
                                segments: vec!["r".into()],
                            })),
                            method: "is_ok".into(),
                            args: vec![],
                        }),
                    ],
                    is_async: true,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        assert!(check(&hir).is_ok(), "timeout in async should pass");
    }

    #[test]
    fn timeout_outside_async_e0313() {
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![HirFunction {
                name: "main".into(),
                params: vec![],
                ret_ty: HirType::IoUnit,
                body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "timeout".into(),
                    args: vec![
                        HirExpr::LitInt(1),
                        HirExpr::Call(HirCall {
                            callee: "print".into(),
                            args: vec![HirExpr::LitStr("x".into())],
                        }),
                    ],
                }))],
                is_async: false,
                is_pub: false,
            }],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        let e = check(&hir).expect_err("timeout outside async");
        assert!(e.to_string().starts_with("E0313:"), "{}", e);
    }

    #[test]
    fn busy_spin_e0320() {
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![HirFunction {
                name: "main".into(),
                params: vec![],
                ret_ty: HirType::IoUnit,
                body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                    callee: "busy_spin".into(),
                    args: vec![],
                }))],
                is_async: false,
                is_pub: false,
            }],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        let e = check(&hir).expect_err("busy_spin");
        assert!(e.to_string().starts_with("E0320:"), "{}", e);
    }

    // ADR-277 / E0342 — Result return theater (Ingeniero soft: pinned only)
    fn helper_result_fn(ret_ty: HirType, last: HirExpr) -> HirModule {
        HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "load".into(),
                    params: vec![HirParam {
                        name: "x".into(),
                        ty: HirType::Int,
                    }],
                    ret_ty,
                    body: vec![HirStmt::Expr(last)],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitInt(1)],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        }
    }

    #[test]
    fn e0342_err_body_without_result_ret() {
        // -> Int { Err(1) }
        let hir = helper_result_fn(HirType::Int, HirExpr::Err(Box::new(HirExpr::LitInt(1))));
        let e = check(&hir).expect_err("must E0342");
        assert_eq!(e.to_string(), "E0342: result return type required");
    }

    #[test]
    fn e0342_litint_body_with_result_ret() {
        // -> Result<Int,Int> { 0 }
        let hir = helper_result_fn(
            HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
            HirExpr::LitInt(0),
        );
        let e = check(&hir).expect_err("must E0342");
        assert_eq!(e.to_string(), "E0342: result return type required");
    }

    #[test]
    fn e0342_ok_body_with_result_ret_ok() {
        // -> Result + Ok(...) → no E0342
        let hir = helper_result_fn(
            HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
            HirExpr::Ok(Box::new(HirExpr::LitInt(1))),
        );
        assert!(
            check(&hir).is_ok(),
            "Ok body with Result ret must not E0342"
        );
    }

    #[test]
    fn e0342_path_result_binding_ok() {
        // -> Result + last Path of Result binding → no E0342
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "load".into(),
                    params: vec![HirParam {
                        name: "x".into(),
                        ty: HirType::Int,
                    }],
                    ret_ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                    body: vec![
                        HirStmt::Let {
                            mutable: false,
                            name: "r".into(),
                            ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                            init: HirExpr::Ok(Box::new(HirExpr::Path(HirPath {
                                segments: vec!["x".into()],
                            }))),
                        },
                        HirStmt::Expr(HirExpr::Path(HirPath {
                            segments: vec!["r".into()],
                        })),
                    ],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitInt(1)],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        assert!(check(&hir).is_ok(), "Path of Result binding must not E0342");
    }

    #[test]
    fn e0343_try_inside_io_main() {
        // fn main() -> Io<()> { let t: String = host.read_text(...) ? ; ... }
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![HirFunction {
                name: "main".into(),
                params: vec![],
                ret_ty: HirType::IoUnit,
                body: vec![
                    HirStmt::Let {
                        mutable: false,
                        name: "t".into(),
                        ty: HirType::String,
                        init: HirExpr::Try {
                            inner: Box::new(HirExpr::Call(HirCall {
                                callee: "host::read_text".into(),
                                args: vec![HirExpr::LitStr("a.txt".into())],
                            })),
                        },
                    },
                    HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::Path(HirPath {
                            segments: vec!["t".into()],
                        })],
                    })),
                ],
                is_async: false,
                is_pub: false,
            }],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        let e = check(&hir).expect_err("must E0343");
        assert_eq!(e.to_string(), "E0343: question mark outside result fn");
    }

    #[test]
    fn e0343_try_inside_int_fn() {
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "f".into(),
                    params: vec![],
                    ret_ty: HirType::Int,
                    body: vec![
                        HirStmt::Let {
                            mutable: false,
                            name: "t".into(),
                            ty: HirType::String,
                            init: HirExpr::Try {
                                inner: Box::new(HirExpr::Call(HirCall {
                                    callee: "host::read_text".into(),
                                    args: vec![HirExpr::LitStr("a.txt".into())],
                                })),
                            },
                        },
                        HirStmt::Expr(HirExpr::LitInt(0)),
                    ],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitInt(1)],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        let e = check(&hir).expect_err("must E0343");
        assert_eq!(e.to_string(), "E0343: question mark outside result fn");
    }

    #[test]
    fn e0343_try_inside_result_fn_ok() {
        // ? inside fn → Result is allowed (no E0343); body ends with Ok
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "load".into(),
                    params: vec![HirParam {
                        name: "path".into(),
                        ty: HirType::String,
                    }],
                    ret_ty: HirType::Result(Box::new(HirType::String), Box::new(HirType::Int)),
                    body: vec![
                        HirStmt::Let {
                            mutable: false,
                            name: "t".into(),
                            ty: HirType::String,
                            init: HirExpr::Try {
                                inner: Box::new(HirExpr::Call(HirCall {
                                    callee: "host::read_text".into(),
                                    args: vec![HirExpr::Path(HirPath {
                                        segments: vec!["path".into()],
                                    })],
                                })),
                            },
                        },
                        HirStmt::Expr(HirExpr::Ok(Box::new(HirExpr::Path(HirPath {
                            segments: vec!["t".into()],
                        })))),
                    ],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitInt(1)],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        assert!(check(&hir).is_ok(), "? inside Result fn must not E0343");
    }

    #[test]
    fn e0343_option_try_rejects_e0203_no_new_code() {
        // Option? OUT: existing E0203, no new E0xxx
        let hir = HirModule {
            imports: vec![], // ADR-293 test literal fix
            name: "t".into(),
            functions: vec![
                HirFunction {
                    name: "f".into(),
                    params: vec![],
                    ret_ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                    body: vec![
                        HirStmt::Let {
                            mutable: false,
                            name: "o".into(),
                            ty: HirType::Option(Box::new(HirType::Int)),
                            init: HirExpr::None,
                        },
                        HirStmt::Let {
                            mutable: false,
                            name: "x".into(),
                            ty: HirType::Int,
                            init: HirExpr::Try {
                                inner: Box::new(HirExpr::Path(HirPath {
                                    segments: vec!["o".into()],
                                })),
                            },
                        },
                        HirStmt::Expr(HirExpr::Ok(Box::new(HirExpr::Path(HirPath {
                            segments: vec!["x".into()],
                        })))),
                    ],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitInt(1)],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![],
            records: vec![],
            enums: vec![],
        };
        let e = check(&hir).expect_err("Option? must reject");
        let s = e.to_string();
        assert!(s.starts_with("E0203:"), "expected E0203, got {s}");
        assert!(!s.contains("E0343") || s.starts_with("E0203:"));
        assert!(!s.starts_with("E0344"));
    }

    // ADR-282 (CORE-0.9-MAP-ASSIGN-20260926): IndexAssign ≡ put diagnostics (shared helpers).
    fn adr282_check(body: &str) -> Result<(), String> {
        let src = format!("module t\nfn main() -> Io<()> {{\n{body}\n  print(1)\n}}\n");
        let m = arita_syntax::parse(&src).map_err(|e| e.to_string())?;
        check(&lower_ast(&m)).map_err(|e| e.to_string())
    }

    fn adr282_pair(prelude: &str, sugar: &str, put: &str) -> (String, String) {
        let a = adr282_check(&format!("{prelude}\n  {sugar}")).expect_err("sugar must reject");
        let b = adr282_check(&format!("{prelude}\n  {put}")).expect_err("put must reject");
        (a, b)
    }

    #[test]
    fn adr282_pos_map_assign_let_mut_ok() {
        assert_eq!(
            adr282_check(
                "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n  m[\"a\"] = 9"
            ),
            Ok(()),
            "let mut Map + m[k] = v must pass HIR"
        );
    }

    #[test]
    fn adr282_pos_rhs_self_ref_len_ok() {
        assert_eq!(
            adr282_check("  let mut m: Map<Text, Int> = Map::new()\n  m[\"b\"] = m.len()"),
            Ok(()),
            "RHS may read receiver (option B)"
        );
    }

    #[test]
    fn adr282_pos_rhs_local_arith_ok() {
        assert_eq!(
            adr282_check(
                "  let mut m: Map<Text, Int> = Map::new()\n  let x: Int = 4\n  m[\"c\"] = x + 1",
            ),
            Ok(()),
            "RHS arith over local ident"
        );
    }

    #[test]
    fn adr282_lowers_to_hir_index_assign() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n  print(1)\n}\n";
        let hir = lower_ast(&arita_syntax::parse(src).expect("parse"));
        let start = src.find("m[\"a\"] = 1").expect("stmt in src");
        assert_eq!(
            hir.functions[0].body[1],
            HirStmt::IndexAssign {
                target: "m".into(),
                key: HirExpr::LitStr("a".into()),
                value: HirExpr::LitInt(1),
                op: None,
                span: arita_syntax::Span {
                    start,
                    end: start + "m[\"a\"] = 1".len(),
                },
            }
        );
    }

    #[test]
    fn adr282_eq_put_non_mut_e0202() {
        let (a, b) = adr282_pair(
            "  let m: Map<Text, Int> = Map::new()",
            "m[\"a\"] = 1",
            "m.put(\"a\", 1)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr282_eq_put_shared_loan_receiver_e0202() {
        let (a, b) = adr282_pair(
            "  let mut m: Map<Text, Int> = Map::new()\n  let r: Map<Text, Int> = borrow m",
            "r[\"a\"] = 1",
            "r.put(\"a\", 1)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr282_eq_put_live_shared_loan_e0202() {
        let (a, b) = adr282_pair(
            "  let mut m: Map<Text, Int> = Map::new()\n  let r: Map<Text, Int> = borrow m",
            "m[\"a\"] = 1\n  print(r.len())",
            "m.put(\"a\", 1)\n  print(r.len())",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr282_eq_put_wrong_key_type_e0203() {
        let (a, b) = adr282_pair(
            "  let mut m: Map<Text, Int> = Map::new()",
            "m[1] = 2",
            "m.put(1, 2)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0203: type mismatch");
    }

    #[test]
    fn adr282_eq_put_wrong_value_type_e0203() {
        let (a, b) = adr282_pair(
            "  let mut m: Map<Text, Int> = Map::new()",
            "m[\"a\"] = \"x\"",
            "m.put(\"a\", \"x\")",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0203: type mismatch");
    }

    #[test]
    fn adr282_eq_put_moved_receiver_e0201() {
        let (a, b) = adr282_pair(
            "  let mut m: Map<Text, Int> = Map::new()\n  let n: Map<Text, Int> = m",
            "m[\"a\"] = 1",
            "m.put(\"a\", 1)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0201: use of moved value");
    }

    #[test]
    fn adr282_string_unbound_stay_e0314() {
        // ADR-283: Vec left this list (main Io → E0344, see adr283_* tests).
        for body in [
            "  let mut s: String = \"abc\"\n  s[0] = 1",
            "  q[\"a\"] = 1",
        ] {
            let e = adr282_check(body).expect_err("non-Map must reject");
            assert!(
                e.starts_with("E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @"),
                "{body}: {e}"
            );
        }
    }

    #[test]
    fn adr282_e0314_carries_index_assign_stmt_span() {
        // ADR-283: receiver switched Vec → String (Vec in main is now E0344).
        let src = "module t\nfn main() -> Io<()> {\n  let s: String = \"abc\"\n  s[0] = 1\n  print(1)\n}\n";
        let start = src.find("s[0] = 1").expect("stmt in src");
        let end = start + "s[0] = 1".len();
        let e = check(&lower_ast(&arita_syntax::parse(src).expect("parse")))
            .expect_err("String receiver → E0314");
        assert_eq!(
            e.to_string(),
            format!("E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{start}..{end}")
        );
        assert_eq!(&src[start..end], "s[0] = 1");
    }

    #[test]
    fn adr282_try_in_rhs_io_main_still_qmark_reject() {
        let src = "module t\nfn load(x: Int) -> Result<Int, Int> { Ok(x) }\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = load(1)?\n  print(1)\n}\n";
        let e = check(&lower_ast(&arita_syntax::parse(src).expect("parse"))).expect_err("E0343");
        assert_eq!(e.to_string(), "E0343: question mark outside result fn");
    }

    #[test]
    fn adr282_put_self_ref_still_rejected_e0006() {
        // put keeps its literal-args rule (parser E0006); the sugar does not inherit it.
        let e = adr282_check("  let mut m: Map<Text, Int> = Map::new()\n  m.put(\"b\", m.len())")
            .expect_err("put self-ref E0006");
        assert!(e.starts_with("E0006:"), "{e}");
    }

    // ADR-283 (CORE-0.9-VEC-ASSIGN-20260926): `v[i] = x` on Vec/List ≡ `v.set(i, x)?`,
    // legal only in a SYNC `fn … -> Result<_, Int>`; E0344 elsewhere.
    const ADR283_MAIN: &str = "fn main() -> Io<()> {\n  match f(1) {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }\n}\n";

    fn adr283_src(body: &str) -> String {
        format!("module t\nfn f(n: Int) -> Result<Int, Int> {{\n{body}\n  Ok(n)\n}}\n{ADR283_MAIN}")
    }

    /// Test helper (ADR-283 gate): byte offset one past the `}` closing the last item.
    fn adr283_close_last(src: &str) -> usize {
        src.rfind('}').expect("close") + 1
    }

    /// Test helper (ADR-283 gate): byte offset one past the `}` right before item `next`.
    fn adr283_close_before(src: &str, next: &str) -> usize {
        src.find(&format!("}}\n{next}")).expect("close") + 1
    }

    fn adr283_check_src(src: &str) -> Result<(), String> {
        let m = arita_syntax::parse(src).map_err(|e| e.to_string())?;
        check(&lower_ast(&m)).map_err(|e| e.to_string())
    }

    fn adr283_check(body: &str) -> Result<(), String> {
        adr283_check_src(&adr283_src(body))
    }

    /// (sugar, `let q: Result<(), Int> = v.set(..)`) — both must reject inside a Result fn.
    fn adr283_pair(prelude: &str, sugar: &str, set: &str) -> (String, String) {
        let a = adr283_check(&format!("{prelude}\n  {sugar}")).expect_err("sugar must reject");
        let b = adr283_check(&format!("{prelude}\n  {set}")).expect_err("set must reject");
        (a, b)
    }

    fn adr283_span_of(src: &str, stmt: &str) -> (usize, usize) {
        let start = src.find(stmt).expect("stmt in src");
        (start, start + stmt.len())
    }

    #[test]
    fn adr283_pos_vec_assign_in_result_fn_nested_ok() {
        let body = "  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v.push(5)\n  v[0] = n\n  let mut i: Int = 0\n  while i < 2 {\n    if i == 0 {\n      v[i] = 9\n    }\n    i = i + 1\n  }\n  let g: Option<Int> = v.get(0)\n  match g {\n    Some(x) => {\n      v[1] = x\n    }\n    None => {\n      v[1] = 0\n    }\n  }";
        assert_eq!(
            adr283_check(body),
            Ok(()),
            "while/if/match nested in Result fn"
        );
    }

    #[test]
    fn adr283_pos_rhs_reads_receiver_r2_ok() {
        // R2: index and value evaluated before the exclusive loan.
        assert_eq!(
            adr283_check("  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = v.len()"),
            Ok(())
        );
    }

    #[test]
    fn adr283_pos_list_receiver_ok() {
        assert_eq!(
            adr283_check("  let mut v: List<Int> = List::new()\n  v.push(0)\n  v[0] = n"),
            Ok(())
        );
    }

    #[test]
    fn adr283_lowers_vec_assign_to_hir_index_assign() {
        let src = adr283_src("  let mut v: Vec<Int> = Vec::new()\n  v[0] = 7");
        let hir = lower_ast(&arita_syntax::parse(&src).expect("parse"));
        let (start, end) = adr283_span_of(&src, "v[0] = 7");
        assert_eq!(
            hir.functions[0].body[1],
            HirStmt::IndexAssign {
                target: "v".into(),
                key: HirExpr::LitInt(0),
                value: HirExpr::LitInt(7),
                op: None,
                span: arita_syntax::Span { start, end },
            }
        );
    }

    #[test]
    fn adr283_e0344_main_io_with_span() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(src, "v[0] = 2");
        assert_eq!(
            adr283_check_src(src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_e0344_io_helper_fn() {
        let src = "module t\nfn h() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n  print(1)\n}\nfn main() -> Io<()> {\n  h()\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(src, "v[0] = 2");
        assert_eq!(
            adr283_check_src(src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_e0344_async_result_fn_and_async_main() {
        let handler = "module t\nasync fn handle(req: HttpRequest) -> Result<HttpResponse, IoError> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = 1\n  let resp: HttpResponse = HttpResponse.ok_text(\"x\")\n  let out: Result<HttpResponse, IoError> = Ok(resp)\n  out\n}\nasync fn main() -> Io<()> {\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(handler, "v[0] = 1");
        assert_eq!(
            adr283_check_src(handler),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            )),
            "async Result fn is not a legal context"
        );
        let amain = "module t\nasync fn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = 1\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(amain, "v[0] = 1");
        assert_eq!(
            adr283_check_src(amain),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_e0344_test_context_in_hir() {
        // Surface `test` blocks reject `let` in the parser (E0007); pin that, then feed the
        // same statements to a HIR test body directly: HIR test ctx → E0344.
        let surface = "module t\nfn main() -> Io<()> {\n  print(1)\n}\ntest t {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n}\n";
        let e = adr283_check_src(surface).expect_err("surface test block");
        assert!(
            e.starts_with("E0007: illegal ident / keyword misuse: `let`"),
            "{e}"
        );
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(src, "v[0] = 2");
        let mut hir = lower_ast(&arita_syntax::parse(src).expect("parse"));
        let body = std::mem::take(&mut hir.functions[0].body);
        hir.functions[0].body = body[3..].to_vec();
        hir.tests.push(HirTest {
            name: "t".into(),
            body: body[..3].to_vec(),
        });
        assert_eq!(
            check(&hir).map_err(|e| e.to_string()),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_int_fn_vec_assign_e0344_map_assign_keeps_e0006() {
        // Parser defers the `fn -> Int` shape E0006 behind an IndexAssign; HIR resolves it.
        let vec_src = "module t\nfn f(n: Int) -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = n\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
        let (start, end) = adr283_span_of(vec_src, "v[0] = n");
        assert_eq!(
            adr283_check_src(vec_src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            )),
            "Vec in non-Result fn"
        );
        let map_src = "module t\nfn f(n: Int) -> Int {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = n\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
        let start = map_src.find("fn f").expect("fn f");
        let end = adr283_close_before(map_src, "fn main");
        assert_eq!(
            adr283_check_src(map_src),
            Err(format!(
                "E0006: non-main fn `f` body must be Expr binary/path/lit/call in this slice @{start}..{end}"
            )),
            "Map in Int fn keeps the exact parser E0006"
        );
    }

    #[test]
    fn adr283_invariant_no_print_map_e0001_vec_hir_error() {
        let no_print = |body: &str| format!("module t\nfn main() -> Io<()> {{\n{body}\n}}\n");
        let e0001 = |src: &str| {
            let start = src.find("fn main").expect("main");
            let end = adr283_close_last(src);
            format!("E0001: no usable print in main @{start}..{end}")
        };
        // Map-assign without print (valid, invalid value, non-mut): exactly E0001.
        for body in [
            "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1",
            "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = \"x\"",
            "  let m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1",
        ] {
            let src = no_print(body);
            assert_eq!(adr283_check_src(&src), Err(e0001(&src)), "{body}");
        }
        // A non-IndexAssign HIR error before the assign does not steal the deferred E0001.
        let src = no_print(
            "  let x: Int = \"a\"\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1",
        );
        assert_eq!(adr283_check_src(&src), Err(e0001(&src)));
        // Vec without print: the HIR error wins (E0344, statement span).
        let src = no_print("  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2");
        let (start, end) = adr283_span_of(&src, "v[0] = 2");
        assert_eq!(
            adr283_check_src(&src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
        // String without print: E0314 with span (neg-e0310-index-string shape).
        let src = no_print("  let mut s: String = \"hi\"\n  s[0] = \"x\"");
        let (start, end) = adr283_span_of(&src, "s[0] = \"x\"");
        assert_eq!(
            adr283_check_src(&src),
            Err(format!(
                "E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_marker_always_errors_never_accepts() {
        // Marker present but body clean (Map assign, no print): HIR must still reject.
        let src = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n}\n";
        let hir = lower_ast(&arita_syntax::parse(src).expect("deferred"));
        assert!(matches!(
            &hir.functions[0].body[0],
            HirStmt::Expr(HirExpr::Call(c)) if c.callee == arita_syntax::DEFERRED_SHAPE_MARKER
        ));
        assert!(matches!(
            check(&hir),
            Err(CheckError::Coded { code: "E0001", .. })
        ));
    }

    #[test]
    fn adr283_precedence_e0344_over_e0319_over_e0203() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[-1] = \"x\"\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(src, "v[-1] = \"x\"");
        assert_eq!(
            adr283_check_src(src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            )),
            "context first"
        );
        assert_eq!(
            adr283_check("  let mut v: Vec<Int> = Vec::new()\n  v[-1] = \"x\""),
            Err("E0319: negative set index".to_string()),
            "negative lit before type"
        );
        // `set` keeps its own order (types before E0319): unchanged by the extraction.
        assert_eq!(
            adr283_check(
                "  let mut v: Vec<Int> = Vec::new()\n  let q: Result<(), Int> = v.set(-1, \"x\")"
            ),
            Err("E0203: type mismatch".to_string())
        );
    }

    #[test]
    fn adr283_eq_set_negative_lit_and_known_int_e0319() {
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()\n  v.push(0)",
            "v[-1] = n",
            "let q: Result<(), Int> = v.set(-1, n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0319: negative set index");
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  let k: Int = -2",
            "v[k] = n",
            "let q: Result<(), Int> = v.set(k, n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0319: negative set index");
    }

    #[test]
    fn adr283_eq_set_wrong_value_and_index_type_e0203() {
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()",
            "v[0] = \"x\"",
            "let q: Result<(), Int> = v.set(0, \"x\")",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0203: type mismatch");
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()",
            "v[\"a\"] = n",
            "let q: Result<(), Int> = v.set(\"a\", n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0203: type mismatch");
    }

    #[test]
    fn adr283_fn_err_type_not_int_e0203() {
        let src = "module t\nfn f(n: Int) -> Result<Int, Text> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = n\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f(1) {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }\n}\n";
        assert_eq!(
            adr283_check_src(src),
            Err("E0203: type mismatch".to_string())
        );
    }

    #[test]
    fn adr283_eq_set_non_mut_e0202() {
        let (a, b) = adr283_pair(
            "  let v: Vec<Int> = Vec::new()",
            "v[0] = n",
            "let q: Result<(), Int> = v.set(0, n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr283_eq_set_shared_loan_receiver_e0202() {
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()\n  let r: Vec<Int> = borrow v",
            "r[0] = n",
            "let q: Result<(), Int> = r.set(0, n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr283_eq_set_live_loan_e0202() {
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()\n  let r: Vec<Int> = borrow v",
            "v[0] = n\n  let k: Int = r.len()",
            "let q: Result<(), Int> = v.set(0, n)\n  let k: Int = r.len()",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0202: borrow conflict");
    }

    #[test]
    fn adr283_eq_set_moved_receiver_e0201() {
        let (a, b) = adr283_pair(
            "  let mut v: Vec<Int> = Vec::new()\n  let w: Vec<Int> = v",
            "v[0] = n",
            "let q: Result<(), Int> = v.set(0, n)",
        );
        assert_eq!(a, b);
        assert_eq!(a, "E0201: use of moved value");
    }

    #[test]
    fn adr283_bad_return_still_e0342() {
        let src = "module t\nfn f(n: Int) -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = n\n  n\n}\nfn main() -> Io<()> {\n  print(1)\n}\n";
        assert_eq!(
            adr283_check_src(src),
            Err("E0342: result return type required".to_string())
        );
    }

    #[test]
    fn adr283_string_receiver_in_result_fn_e0314_span() {
        let src = adr283_src("  let mut s: String = \"abc\"\n  s[0] = n");
        let (start, end) = adr283_span_of(&src, "s[0] = n");
        assert_eq!(
            adr283_check_src(&src),
            Err(format!(
                "E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_int_receiver_e0314_exact_span() {
        let src =
            "module t\nfn main() -> Io<()> {\n  let mut n: Int = 1\n  n[0] = 2\n  print(n)\n}\n";
        let (start, end) = adr283_span_of(src, "n[0] = 2");
        let e =
            check(&lower_ast(&arita_syntax::parse(src).expect("parse"))).expect_err("Int receiver");
        assert!(matches!(&e, CheckError::Coded { code: "E0314", .. }), "{e}");
        assert_eq!(
            e.to_string(),
            format!("E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{start}..{end}")
        );
        assert_eq!(&src[start..end], "n[0] = 2");
    }

    #[test]
    fn adr283_record_receiver_e0314_exact_span() {
        let src = "module t\nrecord P {\n  x: Int,\n}\nfn main() -> Io<()> {\n  let mut p: P = P { x: 1 }\n  p[0] = 2\n  print(1)\n}\n";
        let (start, end) = adr283_span_of(src, "p[0] = 2");
        let e = check(&lower_ast(&arita_syntax::parse(src).expect("parse")))
            .expect_err("record receiver");
        assert!(matches!(&e, CheckError::Coded { code: "E0314", .. }), "{e}");
        assert_eq!(
            e.to_string(),
            format!("E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @{start}..{end}")
        );
        assert_eq!(&src[start..end], "p[0] = 2");
    }

    #[test]
    fn adr283_residual_forms_stay_parser_e0006() {
        for stmt in ["v[0][1] = 1", "v[0].f = 1"] {
            let e = adr283_check(&format!("  let mut v: Vec<Int> = Vec::new()\n  {stmt}"))
                .expect_err("residual form");
            assert!(
                e.starts_with("E0006: construct outside F1.1 (parse failure)"),
                "{stmt}: {e}"
            );
        }
    }

    // ADR-283 §0 L28 (addendum 23:58): one diagnostic per fn — the FIRST offender in source
    // order wins between E0343 (`?` outside Result fn) and E0344 (`v[i] = x` outside Result fn).
    const ADR283_G: &str = "fn g() -> Result<Int, Int> {\n  Ok(1)\n}\n";

    fn adr283_order_src(helper: &str, main: &str) -> String {
        format!("module t\n{ADR283_G}{helper}{main}")
    }

    const ADR283_MAIN_H: &str = "fn main() -> Io<()> {\n  print(h())\n}\n";
    const ADR283_MAIN_K: &str = "fn main() -> Io<()> {\n  k()\n  print(1)\n}\n";

    #[test]
    fn adr283_qmark_before_vec_assign_int_fn_e0343() {
        // neg-core09-vec-assign-qmark-chain-rev shape (deferred `fn -> Int` path).
        let src = adr283_order_src(
            "fn h() -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  let n: Int = g()?\n  v[0] = 2\n  n\n}\n",
            ADR283_MAIN_H,
        );
        assert_eq!(
            adr283_check_src(&src),
            Err("E0343: question mark outside result fn".to_string())
        );
    }

    #[test]
    fn adr283_vec_assign_before_qmark_int_fn_e0344_span() {
        // neg-core09-vec-assign-qmark-chain shape (deferred `fn -> Int` path).
        let src = adr283_order_src(
            "fn h() -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2\n  let n: Int = g()?\n  n\n}\n",
            ADR283_MAIN_H,
        );
        let (start, end) = adr283_span_of(&src, "v[0] = 2");
        assert_eq!(
            adr283_check_src(&src),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_source_order_io_helper_both_orders() {
        // Non-deferred path (fn -> Io<()>): same first-offender rule.
        let ia_first = adr283_order_src(
            "fn k() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2\n  let n: Int = g()?\n  print(n)\n}\n",
            ADR283_MAIN_K,
        );
        let (start, end) = adr283_span_of(&ia_first, "v[0] = 2");
        assert_eq!(
            adr283_check_src(&ia_first),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
        let qmark_first = adr283_order_src(
            "fn k() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  let n: Int = g()?\n  v[0] = 2\n  print(n)\n}\n",
            ADR283_MAIN_K,
        );
        assert_eq!(
            adr283_check_src(&qmark_first),
            Err("E0343: question mark outside result fn".to_string())
        );
    }

    #[test]
    fn adr283_source_order_nested_and_same_stmt() {
        // `?` nested in an earlier `if` body still precedes a later top-level `v[i] = x`.
        let nested_q = adr283_order_src(
            "fn k() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  if true {\n    let n: Int = g()?\n    print(n)\n  }\n  v[0] = 2\n  print(1)\n}\n",
            ADR283_MAIN_K,
        );
        assert_eq!(
            adr283_check_src(&nested_q),
            Err("E0343: question mark outside result fn".to_string())
        );
        // `v[i] = x` nested in an earlier `while` precedes a later `?`.
        let nested_ia = adr283_order_src(
            "fn k() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  let mut i: Int = 0\n  while i < 1 {\n    v[0] = 2\n    i = i + 1\n  }\n  let n: Int = g()?\n  print(n)\n}\n",
            ADR283_MAIN_K,
        );
        let (start, end) = adr283_span_of(&nested_ia, "v[0] = 2");
        assert_eq!(
            adr283_check_src(&nested_ia),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
        // Same statement `v[0] = g()?`: E0344 > … precedence holds inside the statement.
        let same = adr283_order_src(
            "fn h() -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = g()?\n  1\n}\n",
            ADR283_MAIN_H,
        );
        let (start, end) = adr283_span_of(&same, "v[0] = g()?");
        assert_eq!(
            adr283_check_src(&same),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr283_qmark_only_still_e0343_and_noise_does_not_steal_it() {
        // Only `?` (no index-assign) → E0343 exactly as ADR-278.
        let only_q = adr283_order_src(
            "fn h() -> Int {\n  let n: Int = g()?\n  n\n}\n",
            ADR283_MAIN_H,
        );
        assert_eq!(
            adr283_check_src(&only_q),
            Err("E0343: question mark outside result fn".to_string())
        );
        // A non-index-assign type error before the `?` does not steal E0343 (ADR-278).
        let noise = adr283_order_src(
            "fn k() -> Io<()> {\n  let x: Int = \"a\"\n  let n: Int = g()?\n  print(n)\n}\n",
            ADR283_MAIN_K,
        );
        assert_eq!(
            adr283_check_src(&noise),
            Err("E0343: question mark outside result fn".to_string())
        );
    }

    // ADR-286 (CORE-0.10-DEAD-SINK-20260927, S1): E0272 v2 — dead sink in an `Err` arm.
    const ADR286_E0272: &str = "E0272: result error swallowed";

    /// `tally` = ADR-286 §2 neg shape (fn→Result with `v[i] = 7`); `arm` = Err-arm body.
    fn adr286_src(arm: &str) -> String {
        format!(
            "module t\nfn tally(i: Int) -> Result<Int, Int> {{\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[i] = 7\n  let n: Int = v.len()\n  Ok(n)\n}}\nfn main() -> Io<()> {{\n  match tally(99) {{\n    Ok(n) => {{ print(n) }}\n    Err(e) => {{\n{arm}\n    }}\n  }}\n}}\n"
        )
    }

    fn adr286_check(arm: &str) -> Result<(), String> {
        adr283_check_src(&adr286_src(arm))
    }

    #[test]
    fn adr286_dead_sink_e0272() {
        // neg-core10-dead-sink (T09-12 / B-285-4): P2(d).
        assert_eq!(
            adr286_check("      let _c: Int = e\n      print(\"ok\")"),
            Err(ADR286_E0272.to_string())
        );
        // Control (P3 i): same arm reading the sink for real → accepted.
        assert_eq!(
            adr286_check("      let _c: Int = e\n      print(_c)"),
            Ok(())
        );
        // Control: operand of a discarded expression is not a use (P4).
        assert_eq!(
            adr286_check("      let _c: Int = e + 1\n      print(\"ok\")"),
            Err(ADR286_E0272.to_string())
        );
    }

    #[test]
    fn adr286_dead_sink_underscore_e0272() {
        // neg-core10-dead-sink-underscore (⚑1): `let _: Int = e`.
        assert_eq!(
            adr286_check("      let _: Int = e\n      print(\"ok\")"),
            Err(ADR286_E0272.to_string())
        );
        // `Err(_e)` / `Err(_)` with a let-sink: the prefix does not exempt (P4 / P8).
        let under = adr286_src("      let _c: Int = _e\n      print(\"ok\")")
            .replace("Err(e) => {", "Err(_e) => {");
        assert_eq!(adr283_check_src(&under), Err(ADR286_E0272.to_string()));
        // P8: `Err(_eN)` WITHOUT let-sink → E0272 v1 unchanged (theater fires, 2 stmts do not).
        let v1_theater = adr286_src("      print(\"ok\")").replace("Err(e) => {", "Err(_e2) => {");
        assert_eq!(adr283_check_src(&v1_theater), Err(ADR286_E0272.to_string()));
        let v1_two = adr286_src("      print(\"a\")\n      print(\"b\")")
            .replace("Err(e) => {", "Err(_) => {");
        assert_eq!(adr283_check_src(&v1_two), Ok(()));
    }

    #[test]
    fn adr286_dead_sink_transitive_e0272() {
        // neg-core10-dead-sink-transitive: P4 recursive (fixpoint).
        assert_eq!(
            adr286_check("      let _c: Int = e\n      let _d: Int = _c\n      print(\"ok\")"),
            Err(ADR286_E0272.to_string())
        );
        // Control: the end of the chain is really used → accepted.
        assert_eq!(
            adr286_check("      let _c: Int = e\n      let _d: Int = _c\n      print(_d)"),
            Ok(())
        );
    }

    #[test]
    fn adr286_dead_sink_tautology_e0272() {
        // neg-core10-dead-sink-tautology: `if code == code` is not a use (P4 ⚑2).
        assert_eq!(
            adr286_check("      let code: Int = e\n      if code == code { print(\"ok\") }"),
            Err(ADR286_E0272.to_string())
        );
        // ⚑2: reads in the unreachable `else` of `==` do not count (§3 rows 12–13 shape).
        assert_eq!(
            adr286_check(
                "      let code: Int = e\n      if code == code {\n        print(\"ok\")\n      } else {\n        print(code)\n      }"
            ),
            Err(ADR286_E0272.to_string())
        );
        // ⚑2 for `!=`: the unreachable branch is `then`; a read in `else` IS a use.
        assert_eq!(
            adr286_check(
                "      let code: Int = e\n      if code != code {\n        print(code)\n      } else {\n        print(\"ok\")\n      }"
            ),
            Err(ADR286_E0272.to_string())
        );
        assert_eq!(
            adr286_check(
                "      let code: Int = e\n      if code != code {\n        print(\"ok\")\n      } else {\n        print(code)\n      }"
            ),
            Ok(())
        );
        // Control: a real condition (`code > 0`) is a use (P4: condición de `if`).
        assert_eq!(
            adr286_check("      let code: Int = e\n      if code > 0 { print(\"ok\") }"),
            Ok(())
        );
    }

    #[test]
    fn adr286_handled_arms_accepted() {
        // core10-err-used (P3 i).
        assert_eq!(adr286_check("      print(\"err\")\n      print(e)"), Ok(()));
        // core10-err-mapped (P3 ii / D-4): dead binding + explicit `Err(3)` tail.
        let mapped = "module t\nfn inner() -> Result<Int, Int> {\n  Err(1)\n}\nfn outer() -> Result<Int, Int> {\n  match inner() {\n    Ok(x) => { Ok(x) }\n    Err(e) => {\n      let _c: Int = e\n      Err(3)\n    }\n  }\n}\nfn main() -> Io<()> {\n  match outer() {\n    Ok(v) => { print(v) }\n    Err(e) => {\n      print(\"mapped\")\n      print(e)\n    }\n  }\n}\n";
        assert_eq!(adr283_check_src(mapped), Ok(()));
        // Argument of a call is a use (P4), even if the call result is bound to a dead let.
        let call = adr286_src("      let _c: Int = g(e)\n      print(\"ok\")")
            .replace("fn main()", "fn g(x: Int) -> Int {\n  x\n}\nfn main()");
        assert_eq!(adr283_check_src(&call), Ok(()));
    }

    #[test]
    fn adr286_host_io_dead_sink_stays_outside_e0272() {
        // P1 / §0.0: host-IO scrutinee → E0272 not evaluated (E0340 rules, unchanged).
        let src = "module t\nfn main() -> Io<()> {\n  match host.read_text(\"missing.txt\") {\n    Ok(t) => { print(t) }\n    Err(e) => {\n      let _c: Int = e\n      print(\"read_failed\")\n    }\n  }\n}\n";
        assert_eq!(adr283_check_src(src), Ok(()));
    }

    /// P5 inputs: `noise` appended after the `match` (sink first) or prepended to `main`
    /// (noise first). Built outside the `#[test]` body because the unbalanced `}` literals
    /// confuse brace-counting test scanners (Veyra VT006 false positive, 27-09).
    fn adr286_order_srcs(noise: &str) -> (String, String) {
        let sink_first = adr286_src("      let _c: Int = e\n      print(\"ok\")")
            .replace("  }\n}\n", &format!("  }}\n{noise}}}\n"));
        let noise_first = adr286_src("      let _c: Int = e\n      print(\"ok\")").replace(
            "fn main() -> Io<()> {\n",
            &format!("fn main() -> Io<()> {{\n{noise}"),
        );
        (sink_first, noise_first)
    }

    #[test]
    fn adr286_source_order_first_offender() {
        // P5 / ADR-283 L28: between statements the first offender wins (noise = E0206).
        let noise = "  let mut w: Vec<Int> = Vec::new()\n  let b: Int = w.bogus()\n";
        let (sink_first, noise_first) = adr286_order_srcs(noise);
        assert_eq!(adr283_check_src(&sink_first), Err(ADR286_E0272.to_string()));
        let e0206 = Err("E0206: method not in F2 std whitelist".to_string());
        assert_eq!(adr283_check_src(&noise_first), e0206);
        // Inside the match the arm bodies are checked before E0272 (§0.0 order, unchanged).
        assert_eq!(
            adr286_check(
                "      let _c: Int = e\n      let mut w: Vec<Int> = Vec::new()\n      let b: Int = w.bogus()\n      print(\"ok\")"
            ),
            e0206
        );
    }

    // ADR-286 S1 aclaraciones (Arquitecto 1–2 + precisión Ingeniero 27-09): with a dead sink,
    // only an explicit `Err(…)` exit (tail; compound tails if ALL branches are `Err`) is P3(ii).
    /// `outer(flag) -> Result` matching `inner()`; `arm` = Err-arm body; `g(x) = Err(x)`.
    fn adr286_outer_src(arm: &str) -> String {
        format!(
            "module t\nfn inner() -> Result<Int, Int> {{\n  Err(1)\n}}\nfn g(x: Int) -> Result<Int, Int> {{\n  Err(x)\n}}\nfn outer(flag: Bool) -> Result<Int, Int> {{\n  match inner() {{\n    Ok(x) => {{ Ok(x) }}\n    Err(e) => {{\n{arm}\n    }}\n  }}\n}}\nfn main() -> Io<()> {{\n  match outer(true) {{\n    Ok(v) => {{ print(v) }}\n    Err(e) => {{\n      print(\"mapped\")\n      print(e)\n    }}\n  }}\n}}\n"
        )
    }

    fn adr286_outer(arm: &str) -> Result<(), String> {
        adr283_check_src(&adr286_outer_src(arm))
    }

    #[test]
    fn adr286_ok_tail_after_sink_e0272() {
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      Ok(0)"),
            Err(ADR286_E0272.to_string())
        );
        // Result binding not derived from `e` in tail: not an explicit `Err(…)` → E0272.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let r: Result<Int, Int> = Ok(0)\n      r"),
            Err(ADR286_E0272.to_string())
        );
        // Controls (P3 i): the `Ok` tail reads the error (directly / derived) → accepted.
        // (`Ok(e + 1)` is not surface — binary inside `Ok` is E0006 — so the derived form is
        // `let y: Int = e + 1` + `Ok(y)`.)
        assert_eq!(adr286_outer("      let _c: Int = e\n      Ok(e)"), Ok(()));
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let y: Int = e + 1\n      Ok(y)"),
            Ok(())
        );
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let r: Result<Int, Int> = Ok(_c)\n      r"),
            Ok(())
        );
        // P8: without a sink the arm stays on v1 (e used) → accepted.
        assert_eq!(adr286_outer("      Ok(e)"), Ok(()));
    }

    #[test]
    fn adr286_result_call_without_e_after_sink_e0272() {
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      g(1)"),
            Err(ADR286_E0272.to_string())
        );
        // Controls (P3 i): the call receives `e` or a value derived from it → accepted.
        assert_eq!(adr286_outer("      let _c: Int = e\n      g(e)"), Ok(()));
        assert_eq!(adr286_outer("      let _c: Int = e\n      g(_c)"), Ok(()));
    }

    #[test]
    fn adr286_compound_tail_all_err_accepted() {
        assert_eq!(adr286_outer("      let _c: Int = e\n      Err(3)"), Ok(()));
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      if flag { Err(1) } else { Err(2) }"),
            Ok(())
        );
        assert_eq!(
            adr286_outer(
                "      let _c: Int = e\n      match g(2) {\n        Ok(y) => { Err(y) }\n        Err(z) => { Err(z) }\n      }"
            ),
            Ok(())
        );
        // Nested (recursive) compound tail.
        assert_eq!(
            adr286_outer(
                "      let _c: Int = e\n      if flag {\n        Err(1)\n      } else {\n        if flag { Err(2) } else { Err(3) }\n      }"
            ),
            Ok(())
        );
    }

    #[test]
    fn adr286_compound_tail_partial_err_e0272() {
        // One `Ok` branch.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      if flag { Err(1) } else { Ok(2) }"),
            Err(ADR286_E0272.to_string())
        );
        // `if` without `else`.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      if flag { Err(1) }"),
            Err(ADR286_E0272.to_string())
        );
        // `match` with one `Ok` arm.
        assert_eq!(
            adr286_outer(
                "      let _c: Int = e\n      match g(2) {\n        Ok(y) => { Ok(y) }\n        Err(z) => { Err(z) }\n      }"
            ),
            Err(ADR286_E0272.to_string())
        );
        // Nested: the inner `else` is `Ok`.
        assert_eq!(
            adr286_outer(
                "      let _c: Int = e\n      if flag {\n        Err(1)\n      } else {\n        if flag { Err(2) } else { Ok(3) }\n      }"
            ),
            Err(ADR286_E0272.to_string())
        );
        // Control (P3 i): the `Ok` branch reads the error → accepted.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      if flag { Err(1) } else { Ok(e) }"),
            Ok(())
        );
    }

    #[test]
    fn adr286_qmark_does_not_rescue_sink_e0272() {
        // Corrección Ingeniero 27-09: with a dead sink, an intermediate `?` is not handling;
        // only the exit counts (`Ok(v)` here) → E0272.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let v: Int = g(2)?\n      Ok(v)"),
            Err(ADR286_E0272.to_string())
        );
        // Control (P3 i): `?` over a value derived from the error is a use → accepted.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let v: Int = g(_c)?\n      Ok(v)"),
            Ok(())
        );
        // Control: `?` + explicit `Err(…)` exit → P3(ii) by the exit.
        assert_eq!(
            adr286_outer("      let _c: Int = e\n      let v: Int = g(2)?\n      Err(v)"),
            Ok(())
        );
    }

    // ── ADR-286 §0.1d S1b JOIN-SAFETY (CORE-0.10-JOIN-SAFETY-20260927) ──
    // Sources as consts outside `#[test]` (B-286-5 / VT006).

    const ADR286_S1B_E0203: &str = "E0203: type mismatch";

    /// E0203 spawn contract: exact code/head (starts_with) and the message names `spawn`.
    fn adr286_s1b_assert_spawn_e0203(res: Result<(), String>, fn_name: &str, ret: &str) -> String {
        let e = res.expect_err("spawn of non-Io<()> fn must reject");
        assert!(e.starts_with(ADR286_S1B_E0203), "code/head: {e}");
        assert!(e.contains("`spawn`"), "names spawn: {e}");
        assert_eq!(
            e,
            format!(
                "{ADR286_S1B_E0203}: `spawn` expects async fn returning Io<()>, `{fn_name}` returns {ret}"
            )
        );
        e
    }

    /// p2: `let t: Task = spawn(f()); await join(t)` with `f` → Result (today E0100 at rustc).
    const ADR286_S1B_P2_LET_TASK: &str = "module s1b_p2\n\nasync fn f_result() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  let t: Task = spawn(f_result())\n  await join(t)\n  print(\"done\")\n}\n";

    /// p8: `await join(spawn(f()))` with `f` → Result (P6 form; anchor = `spawn`).
    const ADR286_S1B_P8_JOIN_SPAWN: &str = "module s1b_p8\n\nasync fn f_result() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  await join(spawn(f_result()))\n  print(\"done\")\n}\n";

    /// p11: bare `spawn(f())` with `f` → Result.
    const ADR286_S1B_P11_BARE_SPAWN: &str = "module s1b_p11\n\nasync fn f_result() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  spawn(f_result())\n  print(\"done\")\n}\n";

    /// Spawn of a fn → Int (sync: an `async fn -> Int` is already E0241 at the decl, ADR-245).
    const ADR286_S1B_SPAWN_INT: &str = "module s1b_int\n\nfn g() -> Int {\n  1\n}\n\nasync fn main() -> Io<()> {\n  spawn(g())\n  print(\"done\")\n}\n";

    /// Same shape with `async fn g() -> Int`: rejected at the decl (E0241) before any spawn.
    const ADR286_S1B_SPAWN_ASYNC_INT: &str = "module s1b_aint\n\nasync fn g() -> Int {\n  1\n}\n\nasync fn main() -> Io<()> {\n  spawn(g())\n  print(\"done\")\n}\n";

    /// Controls: spawn of an async fn → Io<()> (bare, let + join, inline join).
    const ADR286_S1B_IO_OK: &str = "module s1b_io_ok\n\nasync fn w() -> Io<()> {\n  print(\"w\")\n}\n\nasync fn main() -> Io<()> {\n  spawn(w())\n  print(\"done\")\n}\n";
    const ADR286_S1B_IO_LET_OK: &str = "module s1b_io_let_ok\n\nasync fn w() -> Io<()> {\n  print(\"w\")\n}\n\nasync fn main() -> Io<()> {\n  let t: Task = spawn(w())\n  await join(t)\n  print(\"done\")\n}\n";
    const ADR286_S1B_IO_INLINE_OK: &str = "module s1b_io_inline_ok\n\nasync fn w() -> Io<()> {\n  print(\"w\")\n}\n\nasync fn main() -> Io<()> {\n  await join(spawn(w()))\n  print(\"done\")\n}\n";

    /// ADR-286 §0.1d (c): closed list of free builtin names (the ADR table lists these 17).
    const ADR286_S1B_BUILTIN_NAMES: [&str; 17] = [
        "spawn",
        "join",
        "timeout",
        "delay",
        "cancel_token",
        "cancel",
        "until_cancelled",
        "busy_spin",
        "hang_forever",
        "http_listen",
        "http_serve",
        "http_route_health",
        "http_route_echo",
        "http_bound_port",
        "http_shutdown",
        "http_route",
        "http_response",
    ];

    /// `NAME` is replaced per builtin: sync fn decl.
    const ADR286_S1B_SYNC_DECL_TPL: &str = "module s1b_decl\n\nfn NAME() -> Int {\n  1\n}\n\nfn main() -> Io<()> {\n  print(\"x\")\n}\n";
    /// `NAME` is replaced per builtin: async fn decl → Io<()>.
    const ADR286_S1B_ASYNC_DECL_TPL: &str = "module s1b_adecl\n\nasync fn NAME() -> Io<()> {\n  print(\"n\")\n}\n\nasync fn main() -> Io<()> {\n  print(\"x\")\n}\n";
    /// Motivating case (Codegen): `async fn NAME() -> Result<Int,Int>` + `await NAME()`.
    const ADR286_S1B_ASYNC_RESULT_TPL: &str = "module s1b_motiv\n\nasync fn NAME() -> Result<Int, Int> {\n  Err(7)\n}\n\nasync fn main() -> Io<()> {\n  await NAME()\n  print(\"done\")\n}\n";
    /// Motivating case: `async fn timeout(a: Int, b: Int) -> Result<Int,Int>` + `await timeout(1, 2)`.
    const ADR286_S1B_TIMEOUT_AB: &str = "module s1b_timeout_ab\n\nasync fn timeout(a: Int, b: Int) -> Result<Int, Int> {\n  Err(a)\n}\n\nasync fn main() -> Io<()> {\n  await timeout(1, 2)\n  print(\"done\")\n}\n";

    /// Vars / params named like builtins stay legal (not `is_keyword`).
    const ADR286_S1B_VAR_TIMEOUT: &str =
        "module s1b_var\n\nfn main() -> Io<()> {\n  let timeout: Int = 1\n  print(\"x\")\n}\n";
    const ADR286_S1B_PARAM_NAMES: &str = "module s1b_param\n\nfn add(delay: Int, cancel: Int) -> Int {\n  let spawn: Int = delay + cancel\n  spawn\n}\n\nfn main() -> Io<()> {\n  let join: Int = add(1, 2)\n  print(\"x\")\n}\n";

    fn adr286_s1b_e0007(name: &str) -> String {
        format!("E0007: illegal ident / keyword misuse: `{name}`")
    }

    #[test]
    fn adr286_s1b_p2_let_task_spawn_result_e0203_in_hir() {
        let e = adr286_s1b_assert_spawn_e0203(
            adr283_check_src(ADR286_S1B_P2_LET_TASK),
            "f_result",
            "Result<Int, Int>",
        );
        assert!(e.starts_with(ADR286_S1B_E0203), "{e}");
    }

    #[test]
    fn adr286_s1b_p8_join_spawn_inline_result_e0203() {
        // Anchor = `spawn` (type_of_spawn), not `join`: join would accept any Task.
        // Anchor by precedence (Ingeniero 27-09 NO GO syntax; spans → backlog B-286-11).
        let e = adr286_s1b_assert_spawn_e0203(
            adr283_check_src(ADR286_S1B_P8_JOIN_SPAWN),
            "f_result",
            "Result<Int, Int>",
        );
        assert!(e.starts_with(ADR286_S1B_E0203), "{e}");
    }

    #[test]
    fn adr286_s1b_p11_bare_spawn_result_e0203() {
        let e = adr286_s1b_assert_spawn_e0203(
            adr283_check_src(ADR286_S1B_P11_BARE_SPAWN),
            "f_result",
            "Result<Int, Int>",
        );
        assert!(e.starts_with(ADR286_S1B_E0203), "{e}");
    }

    #[test]
    fn adr286_s1b_spawn_fn_int_e0203() {
        adr286_s1b_assert_spawn_e0203(adr283_check_src(ADR286_S1B_SPAWN_INT), "g", "Int");
        // `async fn -> Int` never reaches spawn: ADR-245 decl check (E0241) is unchanged.
        assert_eq!(
            adr283_check_src(ADR286_S1B_SPAWN_ASYNC_INT),
            Err("E0241: async feature not allowed here".to_string())
        );
    }

    #[test]
    fn adr286_s1b_spawn_iounit_controls_ok() {
        assert_eq!(adr283_check_src(ADR286_S1B_IO_OK), Ok(()), "io-ok");
        assert_eq!(adr283_check_src(ADR286_S1B_IO_LET_OK), Ok(()), "io-let-ok");
        assert_eq!(
            adr283_check_src(ADR286_S1B_IO_INLINE_OK),
            Ok(()),
            "io-inline-ok"
        );
    }

    #[test]
    fn adr286_s1b_e0007_each_builtin_decl_name() {
        for name in ADR286_S1B_BUILTIN_NAMES {
            let sync_src = ADR286_S1B_SYNC_DECL_TPL.replace("NAME", name);
            assert_eq!(
                adr283_check_src(&sync_src),
                Err(adr286_s1b_e0007(name)),
                "sync fn {name}"
            );
            let async_src = ADR286_S1B_ASYNC_DECL_TPL.replace("NAME", name);
            assert_eq!(
                adr283_check_src(&async_src),
                Err(adr286_s1b_e0007(name)),
                "async fn {name}"
            );
        }
    }

    #[test]
    fn adr286_s1b_e0007_async_result_motivating_cases() {
        for name in ["spawn", "timeout", "delay", "until_cancelled", "join"] {
            let src = ADR286_S1B_ASYNC_RESULT_TPL.replace("NAME", name);
            assert_eq!(
                adr283_check_src(&src),
                Err(adr286_s1b_e0007(name)),
                "async fn {name}() -> Result<Int,Int>"
            );
        }
        assert_eq!(
            adr283_check_src(ADR286_S1B_TIMEOUT_AB),
            Err(adr286_s1b_e0007("timeout"))
        );
    }

    #[test]
    fn adr286_s1b_builtin_names_as_vars_and_params_ok() {
        assert_eq!(adr283_check_src(ADR286_S1B_VAR_TIMEOUT), Ok(()));
        assert_eq!(adr283_check_src(ADR286_S1B_PARAM_NAMES), Ok(()));
    }

    // ── ADR-291 MUTEX-REJECT (CORE-0.10-MUTEX-REJECT-20261003), HIR-visible precedence ──
    // The rejection lives in arita-syntax (parse phase), so every case below must come back
    // as the parse error, never as an HIR code. Sources are `const` items outside `#[test]`
    // (B-286-5 / VT006). NOT VALIDATED: written without compiling or running (Lex gate).
    const ADR291_MSG: &str = "E0346: mutex concurrency is not available in this surface";

    const ADR291_CTOR_AFTER_E0206: &str = "module t\nfn main() -> Io<()> {\n  let mut w: Vec<Int> = Vec::new()\n  let b: Int = w.bogus()\n  let m: Int = Mutex.new(0)\n  print(1)\n}\n";
    const ADR291_LOCK_AFTER_E0206: &str = "module t\nfn main() -> Io<()> {\n  let mut w: Vec<Int> = Vec::new()\n  let b: Int = w.bogus()\n  let n: Int = 0\n  let g: Int = n.lock()\n  print(1)\n}\n";
    const ADR291_E0007_DECL_AFTER: &str = "module t\nfn main() -> Io<()> {\n  let m: Int = Mutex.new(0)\n  print(1)\n}\nfn join() -> Io<()> {\n  print(\"j\")\n}\n";
    const ADR291_E0007_CONTROL: &str = "module t\nfn main() -> Io<()> {\n  print(1)\n}\nfn join() -> Io<()> {\n  print(\"j\")\n}\n";
    const ADR291_E0203_LET: &str = "module t\nfn main() -> Io<()> {\n  let m: Result<Int, Int> = Mutex.new(0)\n  print(1)\n}\n";
    const ADR291_UNDECLARED_BEFORE: &str = "module t\nfn main() -> Io<()> {\n  let g: Int = ghost(1)\n  let m: Int = Mutex.new(0)\n  print(1)\n}\n";
    const ADR291_NAMED_PARAM: &str = "module t\nfn f(m: Mutex) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_FREE_MUTEX_NEW: &str =
        "module t\nfn main() -> Io<()> {\n  let _m: Int = mutex_new(0)\n  print(1)\n}\n";

    fn adr291_want(src: &str, anchor: &str) -> String {
        let s = src.find(anchor).expect("anchor in source");
        format!("{ADR291_MSG} @{}..{}", s, s + anchor.len())
    }

    #[test]
    fn adr291_parse_phase_wins_over_e0206() {
        // E0206 (`w.bogus()`) is earlier in the source, but the parse phase runs first.
        assert_eq!(
            adr283_check_src(ADR291_CTOR_AFTER_E0206),
            Err(adr291_want(ADR291_CTOR_AFTER_E0206, "Mutex"))
        );
        assert_eq!(
            adr283_check_src(ADR291_LOCK_AFTER_E0206),
            Err(adr291_want(ADR291_LOCK_AFTER_E0206, "lock"))
        );
    }

    #[test]
    fn adr291_parse_phase_wins_over_e0007_and_e0203() {
        // Control: the builtin-named decl alone is the HIR-phase E0007.
        let e = adr283_check_src(ADR291_E0007_CONTROL).expect_err("fn join is E0007");
        assert!(e.starts_with("E0007"), "{e}");
        // With a Mutex form in `main`, the parse phase wins even though `fn join` is declared.
        assert_eq!(
            adr283_check_src(ADR291_E0007_DECL_AFTER),
            Err(adr291_want(ADR291_E0007_DECL_AFTER, "Mutex"))
        );
        // `let m: Result<..> = Mutex.new(0)` would be an E0203 type mismatch in HIR.
        assert_eq!(
            adr283_check_src(ADR291_E0203_LET),
            Err(adr291_want(ADR291_E0203_LET, "Mutex"))
        );
    }

    #[test]
    fn adr291_wins_over_undeclared_call_before_it() {
        // Holds before and after ADR-293: an undeclared `ghost(1)` is typed Int today and is
        // E0347 (HIR phase) after ADR-293; either way the parse-phase E0346 is reported.
        assert_eq!(
            adr283_check_src(ADR291_UNDECLARED_BEFORE),
            Err(adr291_want(ADR291_UNDECLARED_BEFORE, "Mutex"))
        );
    }

    #[test]
    fn adr291_named_param_never_reaches_hir() {
        assert_eq!(
            adr283_check_src(ADR291_NAMED_PARAM),
            Err(adr291_want(ADR291_NAMED_PARAM, "Mutex"))
        );
    }

    #[test]
    fn adr291_free_call_mutex_new_is_not_an_e0346_form() {
        // `mutex_new(0)` is a free call (B-286-7 / ADR-293 territory). Today it checks Ok; after
        // ADR-293 it is E0347. It must never become E0346 in this slice.
        let r = adr283_check_src(ADR291_FREE_MUTEX_NEW);
        assert!(
            r.as_ref().err().is_none_or(|e| !e.starts_with("E0346")),
            "{r:?}"
        );
    }

    // ADR-290 slice A (IndexMut compound `v[i] op= x`). Sources live in `const` items outside
    // the `#[test]` bodies; the cases that need a non-Int element type retype a lowered `let`.
    const ADR290_POS: &str = "  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] += n\n  v[0] -= 2\n  v[0] *= 3\n  let mut i: Int = 0\n  v[i] += 1";
    const ADR290_POS_LIST: &str = "  let mut v: List<Int> = List::new()\n  v.push(1)\n  v[0] += n";
    const ADR290_LOWER: &str = "module t\nfn f(n: Int) -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += n\n  v[1] -= n\n  v[2] *= n\n  v[3] = n\n  Ok(n)\n}\nfn main() -> Io<()> {\n  print(1)\n}\n";
    const ADR290_MAIN_NEG_LIT: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[-1] += 1\n  print(1)\n}\n";
    const ADR290_MAIN_TEXT_RHS: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += \"x\"\n  print(1)\n}\n";
    const ADR290_MAIN_PLAIN: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += 1\n  print(1)\n}\n";
    const ADR290_INT_FN: &str = "module t\nfn f(n: Int) -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += n\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
    const ADR290_IO_HELPER: &str = "module t\nfn h() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] *= 2\n  print(1)\n}\nfn main() -> Io<()> {\n  h()\n  print(1)\n}\n";
    const ADR290_ERR_TEXT_FN: &str = "module t\nfn f(n: Int) -> Result<Int, Text> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] += n\n  Ok(n)\n}\nfn main() -> Io<()> {\n  match f(1) {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }\n}\n";
    const ADR290_PREC_BOTH: &str = "  let mut v: Vec<Int> = Vec::new()\n  v[-1] += \"x\"";
    const ADR290_PREC_NEG: &str = "  let mut v: Vec<Int> = Vec::new()\n  v[-1] += 1";
    const ADR290_PREC_TYPES: &str = "  let mut v: Vec<Int> = Vec::new()\n  v[\"a\"] += \"x\"";
    const ADR290_PREC_ELEM: &str = "  let mut v: Vec<Int> = Vec::new()\n  v[0] += 1";
    const ADR290_MAP: &str = "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] += 1";
    const ADR290_STRING: &str = "  let mut s: String = \"abc\"\n  s[0] += 1";
    const ADR290_BYTES: &str = "  let mut b: Vec<Int> = Vec::new()\n  b[0] += 1";
    const ADR290_INT: &str = "  let mut c: Int = 0\n  c[0] += 1";
    const ADR290_UNBOUND: &str = "  q[0] += 1";
    const ADR290_MAP_NO_PRINT: &str = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] += 1\n}\n";
    const ADR290_UNSUPPORTED_OPS: [&str; 3] = [
        "  let mut v: Vec<Int> = Vec::new()\n  v[0] /= 2",
        "  let mut v: Vec<Int> = Vec::new()\n  v[0] %= 2",
        "  let mut n: Int = 0\n  n += 1",
    ];
    const ADR290_PLUS_AB: &str = "  let a: Int = 1\n  let c: Int = a + n\n  let mut v: Vec<Int> = Vec::new()\n  v.push(c)\n  v[0] += a + n";
    const ADR290_CORE09_MAP_04: &str =
        include_str!("../../../ejemplos/core09/map-assign/neg/04-compound.arita");
    const ADR290_CORE09_MAP_NEGS: [(&str, &str); 6] = [
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/01-non-mut.arita"),
            "E0202: borrow conflict",
        ),
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/02-shared-loan.arita"),
            "E0202: borrow conflict",
        ),
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/03-live-loan.arita"),
            "E0202: borrow conflict",
        ),
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/05-unwrap.arita"),
            "E0206: method not in F2 std whitelist",
        ),
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/06-type.arita"),
            "E0203: type mismatch",
        ),
        (
            include_str!("../../../ejemplos/core09/map-assign/neg/07-value-type.arita"),
            "E0203: type mismatch",
        ),
    ];
    const ADR290_CORE09_VEC_05: &str =
        include_str!("../../../ejemplos/core09/vec-assign/neg/05-compound.arita");
    const ADR290_CORE09_VEC_06: &str =
        include_str!("../../../ejemplos/core09/vec-assign/neg/06-nested.arita");
    const ADR290_CORE09_VEC_07: &str =
        include_str!("../../../ejemplos/core09/vec-assign/neg/07-field.arita");
    const ADR290_E0006: &str = "E0006: construct outside F1.1 (parse failure)";
    const ADR290_E0333: &str = "E0333: compound index-assign: unsupported element type in v0";

    /// Lower `src`, retype every `let` named `v`/`b` to `ty` (the grammar only offers
    /// `Vec<Int>` / `List<Int>` vectors, so E0333 needs this direct HIR construction).
    fn adr290_check_retyped(src: &str, ty: &HirType) -> Result<(), String> {
        let m = arita_syntax::parse(src).map_err(|e| e.to_string())?;
        let mut hir = lower_ast(&m);
        for f in &mut hir.functions {
            for st in &mut f.body {
                if let HirStmt::Let { name, ty: t, .. } = st {
                    if name == "v" || name == "b" {
                        *t = ty.clone();
                    }
                }
            }
        }
        check(&hir).map_err(|e| e.to_string())
    }

    fn adr290_in_result_fn(body: &str) -> String {
        adr283_src(body)
    }

    fn adr290_in_main(body: &str) -> String {
        format!("module t\nfn main() -> Io<()> {{\n{body}\n  print(1)\n}}\n")
    }

    #[test]
    fn adr290_pos_compound_in_result_fn_ok() {
        assert_eq!(adr283_check(ADR290_POS), Ok(()));
        assert_eq!(adr283_check(ADR290_POS_LIST), Ok(()));
        assert_eq!(adr283_check(ADR290_PLUS_AB), Ok(()));
    }

    #[test]
    fn adr290_lowers_op_to_hir() {
        let hir = lower_ast(&arita_syntax::parse(ADR290_LOWER).expect("parse"));
        let ops: Vec<Option<HirBinOp>> = hir.functions[0]
            .body
            .iter()
            .filter_map(|s| match s {
                HirStmt::IndexAssign { op, .. } => Some(op.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            ops,
            vec![
                Some(HirBinOp::Add),
                Some(HirBinOp::Sub),
                Some(HirBinOp::Mul),
                None
            ]
        );
    }

    #[test]
    fn adr290_e0344_outside_result_fn() {
        for src in [ADR290_MAIN_PLAIN, ADR290_IO_HELPER, ADR290_INT_FN] {
            let e = adr283_check_src(src).expect_err("compound outside Result fn");
            assert!(
                e.starts_with("E0344: index assign outside result fn @"),
                "{src}: {e}"
            );
        }
        let (start, end) = adr283_span_of(ADR290_MAIN_PLAIN, "v[0] += 1");
        assert_eq!(
            adr283_check_src(ADR290_MAIN_PLAIN),
            Err(format!(
                "E0344: index assign outside result fn @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr290_e0319_negative_literal_index() {
        assert_eq!(
            adr283_check(ADR290_PREC_NEG),
            Err("E0319: negative set index".to_string())
        );
    }

    #[test]
    fn adr290_e0333_unsupported_element_type() {
        for ty in [
            HirType::String,
            HirType::Bool,
            HirType::Vec(Box::new(HirType::Int)),
        ] {
            let src = adr290_in_result_fn(ADR290_PREC_ELEM);
            assert_eq!(
                adr290_check_retyped(&src, &HirType::Vec(Box::new(ty.clone()))),
                Err(ADR290_E0333.to_string()),
                "{ty:?}"
            );
        }
    }

    #[test]
    fn adr290_e0203_rhs_key_and_fn_error_types() {
        assert_eq!(
            adr283_check("  let mut v: Vec<Int> = Vec::new()\n  v[0] += \"x\""),
            Err("E0203: type mismatch".to_string()),
            "rhs Text"
        );
        assert_eq!(
            adr283_check("  let mut v: Vec<Int> = Vec::new()\n  v[\"a\"] += n"),
            Err("E0203: type mismatch".to_string()),
            "key Text"
        );
        assert_eq!(
            adr283_check_src(ADR290_ERR_TEXT_FN),
            Err("E0203: type mismatch".to_string()),
            "fn E != Int"
        );
    }

    #[test]
    fn adr290_borrow_rules_unchanged() {
        assert_eq!(
            adr283_check("  let v: Vec<Int> = Vec::new()\n  v[0] += n"),
            Err("E0202: borrow conflict".to_string()),
            "non-mut"
        );
        assert_eq!(
            adr283_check("  let mut v: Vec<Int> = Vec::new()\n  let w: Vec<Int> = v\n  v[0] += n"),
            Err("E0201: use of moved value".to_string()),
            "moved"
        );
        assert_eq!(
            adr283_check(
                "  let mut v: Vec<Int> = Vec::new()\n  let r: Vec<Int> = borrow v\n  r[0] += n"
            ),
            Err("E0202: borrow conflict".to_string()),
            "shared receiver"
        );
        assert_eq!(
            adr283_check(
                "  let mut v: Vec<Int> = Vec::new()\n  let r: Vec<Int> = borrow v\n  v[0] += n\n  let k: Int = r.len()"
            ),
            Err("E0202: borrow conflict".to_string()),
            "live loan"
        );
    }

    #[test]
    fn adr290_compound_on_map_string_bytes_is_hir_e0006() {
        for body in [ADR290_MAP, ADR290_STRING] {
            for src in [adr290_in_result_fn(body), adr290_in_main(body)] {
                let e = adr283_check_src(&src).expect_err("compound on non-Vec collection");
                assert!(e.starts_with(ADR290_E0006), "{body}: {e}");
            }
        }
        let src = adr290_in_result_fn(ADR290_BYTES);
        let e = adr290_check_retyped(&src, &HirType::Bytes).expect_err("Bytes compound");
        assert!(e.starts_with(ADR290_E0006), "{e}");
        let (start, end) = adr283_span_of(&adr290_in_main(ADR290_MAP), "m[\"a\"] += 1");
        assert_eq!(
            adr283_check_src(&adr290_in_main(ADR290_MAP)),
            Err(format!(
                "E0006: construct outside F1.1 (parse failure) @{start}..{end}"
            ))
        );
    }

    #[test]
    fn adr290_compound_on_int_and_unbound_stay_e0314() {
        for body in [ADR290_INT, ADR290_UNBOUND] {
            let e = adr283_check_src(&adr290_in_main(body)).expect_err("non-collection");
            assert!(
                e.starts_with("E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @"),
                "{body}: {e}"
            );
        }
    }

    #[test]
    fn adr290_map_compound_wins_over_deferred_shape() {
        // main without print: the shape E0001 is deferred behind the IndexAssign; the Map
        // compound E0006 (a parse failure before ADR-290) still wins.
        let e = adr283_check_src(ADR290_MAP_NO_PRINT).expect_err("Map compound");
        assert!(e.starts_with(ADR290_E0006), "{e}");
    }

    #[test]
    fn adr290_unsupported_operators_and_simple_compound_stay_parser_e0006() {
        for body in ADR290_UNSUPPORTED_OPS {
            let e = adr283_check(body).expect_err("unsupported operator");
            assert!(e.starts_with(ADR290_E0006), "{body}: {e}");
            assert!(!e.starts_with("E0333"), "{body}: {e}");
        }
    }

    // Pin 3: one case per step of E0344 > E0319 > E0333 > E0203 (each pair discriminated).
    #[test]
    fn adr290_precedence_e0344_over_e0319() {
        let e = adr283_check_src(ADR290_MAIN_NEG_LIT).expect_err("main + negative literal");
        assert!(e.starts_with("E0344:"), "{e}");
        assert_eq!(
            adr283_check(ADR290_PREC_NEG),
            Err("E0319: negative set index".to_string())
        );
    }

    #[test]
    fn adr290_precedence_e0344_over_e0333() {
        let src = adr290_in_main(ADR290_PREC_ELEM);
        let e = adr290_check_retyped(&src, &HirType::Vec(Box::new(HirType::String)))
            .expect_err("main + Vec<Text>");
        assert!(e.starts_with("E0344:"), "{e}");
    }

    #[test]
    fn adr290_precedence_e0319_over_e0333() {
        let neg = "  let mut v: Vec<Int> = Vec::new()\n  v[-1] += 1";
        let text = HirType::Vec(Box::new(HirType::String));
        assert_eq!(
            adr290_check_retyped(&adr290_in_result_fn(neg), &text),
            Err("E0319: negative set index".to_string())
        );
        assert_eq!(
            adr290_check_retyped(&adr290_in_result_fn(ADR290_PREC_ELEM), &text),
            Err(ADR290_E0333.to_string())
        );
    }

    #[test]
    fn adr290_precedence_e0333_over_e0203() {
        // Key Text + rhs Text on a Vec<Text>: E0333 wins; the same statement on Vec<Int> is E0203.
        let text = HirType::Vec(Box::new(HirType::String));
        assert_eq!(
            adr290_check_retyped(&adr290_in_result_fn(ADR290_PREC_TYPES), &text),
            Err(ADR290_E0333.to_string())
        );
        assert_eq!(
            adr283_check(ADR290_PREC_TYPES),
            Err("E0203: type mismatch".to_string())
        );
    }

    #[test]
    fn adr290_precedence_e0344_over_e0203() {
        let e = adr283_check_src(ADR290_MAIN_TEXT_RHS).expect_err("main + Text rhs");
        assert!(e.starts_with("E0344:"), "{e}");
        // Inside a Result fn the negative literal still beats the Text rhs (E0319 > E0203).
        assert_eq!(
            adr283_check(ADR290_PREC_BOTH),
            Err("E0319: negative set index".to_string())
        );
    }

    #[test]
    fn adr290_core09_map_assign_negs_unchanged() {
        let e = adr283_check_src(ADR290_CORE09_MAP_04).expect_err("neg-core09-map-assign-compound");
        assert!(e.starts_with(ADR290_E0006), "{e}");
        for (src, want) in ADR290_CORE09_MAP_NEGS {
            assert_eq!(adr283_check_src(src), Err(want.to_string()));
        }
    }

    #[test]
    fn adr290_core09_vec_assign_nested_field_unchanged_and_05_migrates() {
        for src in [ADR290_CORE09_VEC_06, ADR290_CORE09_VEC_07] {
            let e = adr283_check_src(src).expect_err("nested / field stay E0006");
            assert!(e.starts_with(ADR290_E0006), "{e}");
        }
        // IM-6: `vec-assign/neg/05-compound` (Vec<Int> in `fn -> Result<_, Int>`) is now valid.
        assert_eq!(adr283_check_src(ADR290_CORE09_VEC_05), Ok(()));
    }

    // ---------------------------------------------------------------------------------------
    // ADR-295 (S2 MUST-USE, E0272): descarte de valores `Result` de surface. Los fuentes van
    // en `const` fuera de `#[test]` (B-286-5, VT006). Casos y veredictos: PREP r4 (R4.8).
    // ---------------------------------------------------------------------------------------

    /// `want`: `None` = compila; `Some("E0xxx")` = ese código (prefijo `E0xxx:`); `Some(msg)` =
    /// diagnóstico exacto.
    fn adr295_expect(src: &str, want: Option<&str>) {
        let got = adr283_check_src(src);
        match want {
            None => assert_eq!(got, Ok(()), "must compile:\n{src}"),
            Some(w) if w.len() == 5 => {
                let e = got.expect_err(w);
                assert!(e.starts_with(&format!("{w}:")), "want {w}, got {e}\n{src}");
            }
            Some(w) => assert_eq!(got, Err(w.to_string()), "{src}"),
        }
    }
    const ADR295_POS01: &str = r#"module core10_must_use_bare_stmt
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  print("after")
}
"#;

    const ADR295_POS02: &str = r#"module core10_must_use_let_underscore
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let _: Result<Int, Int> = f(1)
  print("after")
}
"#;

    const ADR295_POS03: &str = r#"module core10_must_use_let_named_dead
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let _r: Result<Int, Int> = f(1)
  print("after")
}
"#;

    const ADR295_POS04: &str = r#"module core10_must_use_let_plain_dead
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  print("after")
}
"#;

    const ADR295_POS05: &str = r#"module core10_must_use_let_transitive
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  let s: Result<Int, Int> = r
  print("after")
}
"#;

    const ADR295_POS06: &str = r#"module core10_must_use_vec_set
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.set(0, 7)
  print(v.len())
}
"#;

    const ADR295_POS07: &str = r#"module core10_must_use_vec_insert
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.insert(0, 7)
  print(v.len())
}
"#;

    const ADR295_POS08: &str = r#"module core10_must_use_nested_if
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let ok: Bool = true
  if ok {
    f(1)
    print("a")
  } else {
    print("b")
  }
}
"#;

    const ADR295_POS09: &str = r#"module core10_must_use_nested_while
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut i: Int = 0
  while i < 1 {
    f(i)
    i = i + 1
  }
  print(i)
}
"#;

    const ADR295_POS10: &str = r#"module core10_must_use_nested_match_arm
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  match f(2) {
    Ok(v) => {
      f(v)
      print(v)
    }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_POS11: &str = r#"module core10_must_use_result_fn_midbody
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  f(1)
  Ok(2)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_POS12: &str = r#"module core10_must_use_async_main
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
async fn greet() -> Io<()> {
  print("g")
}
async fn main() -> Io<()> {
  await greet()
  f(1)
  print("after")
}
"#;

    const ADR295_POS13: &str = r#"module core10_must_use_main_tail
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  print("before")
  f(1)
}
"#;

    const ADR295_POS14: &str = r#"module core10_must_use_io_helper
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn helper() -> Io<()> {
  f(1)
  print("h")
}
fn main() -> Io<()> {
  helper()
  print("after")
}
"#;

    const ADR295_POS15: &str = r#"module core10_must_use_if_let_body
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(3)
  if let Ok(v) = r {
    f(v)
    print(v)
  } else {
    print("e")
  }
}
"#;

    const ADR295_POS16: &str = r#"module core10_must_use_ident_stmt
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  r
  print("after")
}
"#;

    const ADR295_POS17: &str = r#"module core10_must_use_ok_lit_midbody
fn g() -> Result<Int, Int> {
  Ok(1)
  Ok(2)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_POS18: &str = r#"module core10_must_use_midbody_match_tails
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  match f(1) {
    Ok(v) => { f(v) }
    Err(e) => { Err(e) }
  }
  Ok(2)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_POS19: &str = r#"module core10_must_use_int_helper
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn h() -> Int {
  f(1)
  3
}
fn main() -> Io<()> {
  print(h())
}
"#;

    const ADR295_POS20: &str = r#"module core10_must_use_timeout_bare
async fn quick() -> Io<()> {
  print("fast")
}
async fn main() -> Io<()> {
  timeout(1000, quick())
  print("after")
}
"#;

    const ADR295_POS21: &str = r#"module core10_must_use_try_reserve_bare
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.try_reserve(8)
  print(v.len())
}
"#;

    const ADR295_POS22: &str = r#"module core10_must_use_main_tail_arm
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  match f(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      f(e)
    }
  }
}
"#;

    const ADR295_NEG01: &str = r#"module core10_qmark_use
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  let v: Int = f(1)?
  Ok(v)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG02: &str = r#"module core10_let_used
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  match r {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG03: &str = r#"module core10_underscore_read
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let _x: Result<Int, Int> = f(2)
  match _x {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG04: &str = r#"module core10_must_use_tail_return
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  f(1)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG05: &str = r#"module core10_must_use_tail_match
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  match f(1) {
    Ok(v) => { f(v) }
    Err(e) => { Err(e) }
  }
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG06: &str = r#"module core10_must_use_bare_qmark
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  f(1)?
  Ok(2)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG07: &str = r#"module core10_must_use_arg
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn h(r: Result<Int, Int>) -> Result<Int, Int> {
  r
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  match h(r) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG08: &str = r#"module core10_must_use_if_let
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  if let Ok(v) = r {
    print(v)
  } else {
    print("e")
  }
}
"#;

    const ADR295_NEG09: &str = r#"module core10_must_use_ident_try
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  let r: Result<Int, Int> = f(1)
  let v: Int = r?
  Ok(v)
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG10: &str = r#"module core10_must_use_binding_tail
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  let r: Result<Int, Int> = f(1)
  r
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG11: &str = r#"module core10_must_use_index_assign
fn g(i: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v[i] = 7
  let n: Int = v.len()
  Ok(n)
}
fn main() -> Io<()> {
  match g(0) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG12: &str = r#"module core10_must_use_option_unit
fn main() -> Io<()> {
  let _o: Option<Int> = Some(1)
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  print(v.len())
}
"#;

    const ADR295_NEG13: &str = r#"module core10_must_use_async_await
async fn greet() -> Io<()> {
  print("g")
}
async fn main() -> Io<()> {
  await greet()
  print("after")
}
"#;

    const ADR295_NEG14: &str = r#"module core10_must_use_param_unused
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn h(r: Result<Int, Int>, x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  match h(r, 2) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG15: &str = r#"module probe_recv_bare
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  r.is_ok()
  print(0)
}
"#;

    const ADR295_NEG16: &str = r#"module core10_must_use_compound_ia
fn g(n: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v[0] += n
  Ok(n)
}
fn main() -> Io<()> {
  match g(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG17: &str = r#"module core10_must_use_nested_tail
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(o: Int) -> Result<Int, Int> {
  let r: Result<Int, Int> = f(o)
  match r {
    Ok(u) => {
      match f(u) {
        Ok(w) => {
          let ok: Result<Int, Int> = Ok(w)
          ok
        }
        Err(e) => {
          let bad: Result<Int, Int> = Err(e)
          bad
        }
      }
    }
    Err(e) => {
      let bad: Result<Int, Int> = Err(e)
      bad
    }
  }
}
fn main() -> Io<()> {
  match g(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_NEG18: &str = r#"module core10_must_use_tail_if_else
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g() -> Result<Int, Int> {
  let c: Bool = true
  if c {
    f(1)
  } else {
    Err(2)
  }
}
fn main() -> Io<()> {
  match g() {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_PREC01: &str = r#"module core10_must_use_prec_e0206_after
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  let mut w: Vec<Int> = Vec::new()
  print(w.bogus())
}
"#;

    const ADR295_PREC02: &str = r#"module core10_must_use_prec_e0206_before
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut w: Vec<Int> = Vec::new()
  print(w.bogus())
  f(1)
}
"#;

    const ADR295_PREC03: &str = r#"module core10_must_use_prec_e0343_after
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  let n: Int = f(2)?
  print(n)
}
"#;

    const ADR295_PREC04: &str = r#"module core10_must_use_prec_e0343_before
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let n: Int = f(2)?
  f(1)
  print(n)
}
"#;

    const ADR295_PREC05: &str = r#"module core10_must_use_prec_e0344_after
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  f(1)
  v[0] = 2
  print(v.len())
}
"#;

    const ADR295_PREC06: &str = r#"module core10_must_use_prec_e0344_before
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v[0] = 2
  f(1)
  print(v.len())
}
"#;

    const ADR295_PREC07: &str = r#"module core10_must_use_prec_sink
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  match f(2) {
    Ok(v) => { print(v) }
    Err(e) => {
      let _c: Int = e
      print("ok")
    }
  }
}
"#;

    const ADR295_PREC08: &str = r#"module core10_must_use_prec_deferred
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn h() -> Int {
  let mut v: Vec<Int> = Vec::new()
  f(1)
  v[0] = 2
  3
}
fn main() -> Io<()> {
  print(h())
}
"#;

    const ADR295_PREC09: &str = r#"module core10_must_use_prec_let_e0206
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  let mut w: Vec<Int> = Vec::new()
  print(w.bogus())
}
"#;

    const ADR295_PREC10: &str = r#"module core10_must_use_int_helper_tail
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn h() -> Int {
  f(1)
}
fn main() -> Io<()> {
  print(h())
}
"#;

    const ADR295_PREC11: &str = r#"module core10_must_use_prec_e0319_after
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(n: Int) -> Result<Int, Int> {
  f(1)
  let mut v: Vec<Int> = Vec::new()
  v[-1] += 1
  Ok(n)
}
fn main() -> Io<()> {
  match g(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_PREC12: &str = r#"module hoy_p12
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(n: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v[-1] += 1
  f(1)
  Ok(n)
}
fn main() -> Io<()> {
  match g(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    const ADR295_PREC13: &str = r#"module hoy_p13
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  print("after")
}
fn delay() -> Io<()> {
  print("d")
}
"#;

    const ADR295_PREC14: &str = r#"module hoy_prec14
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  ghost(1)
  f(1)
  print("after")
}
"#;

    const ADR295_PREC15: &str = r#"module hoy_prec15
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  f(1)
  ghost(1)
  print("after")
}
"#;

    const ADR295_PREC16: &str = r#"module hoy_prec16
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = ghost(1)
  print("after")
}
"#;

    const ADR295_PREC17: &str = r#"module core10_must_use_prec_arm_let
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  match f(1) {
    Ok(v) => { print(v) }
    Err(e) => {
      let r: Result<Int, Int> = f(2)
      print(e)
      let mut w: Vec<Int> = Vec::new()
      print(w.bogus())
    }
  }
}
"#;

    const ADR295_PREC19: &str = r#"module hoy_prec19
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  ghost(1)
  let n: Int = f(2)?
  print(n)
}
"#;

    const ADR295_PREC20: &str = r#"module core10_must_use_prec_async_bare
async fn h() -> Result<Int, Int> {
  Ok(1)
}
async fn main() -> Io<()> {
  h()
  print("after")
}
"#;

    const ADR295_IO01: &str = r#"module core10_must_use_io_bare
fn main() -> Io<()> {
  host.read_text("missing.txt")
  print("after")
}
"#;

    const ADR295_IO02: &str = r#"module core10_must_use_io_let_underscore
fn main() -> Io<()> {
  let _: Result<Text, Int> = host.read_text("missing.txt")
  print("after")
}
"#;

    const ADR295_IO03: &str = r#"module core10_must_use_io_let_named
fn main() -> Io<()> {
  let r: Result<Text, Int> = host.read_text("missing.txt")
  print("after")
}
"#;

    const ADR295_IO04: &str = r#"module core10_must_use_io_let_underscore_named
fn main() -> Io<()> {
  let _r: Result<Text, Int> = host.read_text("missing.txt")
  print("after")
}
"#;

    const ADR295_IO05: &str = r#"module core10_must_use_io_write_bare
fn main() -> Io<()> {
  host.write_text("scratch.txt", "x")
  print("after")
}
"#;

    const ADR295_OUT01: &str = r#"module core10_must_use_await_result
async fn h() -> Result<Int, Int> {
  Ok(1)
}
async fn main() -> Io<()> {
  await h()
  print("after")
}
"#;

    // PREP r4 R4.4.2 / ADR-295 D4: PREC-18 (E0343 gana a un E0272 previo que sería el primer error del cuerpo).
    const ADR295_PREC18_BASE: &str = r#"module core10_must_use_prec_e0343_e0344
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  f(1)
  v[0] = 2
  let n: Int = f(2)?
  print(n)
}
"#;
    const ADR295_PREC18_C: &str = r#"module hoy_prec18c
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v[0] = 2
  f(1)
  let n: Int = f(2)?
  print(n)
}
"#;
    const ADR295_PREC18_B: &str = r#"module hoy_prec18b
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  let n: Int = f(2)?
  v[0] = 2
  f(1)
  print(n)
}
"#;
    const ADR295_PREC18_CTL: &str = r#"module hoy_prec18ctl
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v[0] = 2
  let n: Int = f(2)?
  print(n)
}
"#;

    #[test]
    fn adr286_s2_prec_bare_e0344_qmark() {
        // PREC-18: `v.push(0)` · `f(1)` · `v[0] = 2` · `let n: Int = f(2)?` · `print(n)` en `main`:
        // hoy E0344; tras S2 el E0272 de `f(1)` es el primer error del cuerpo y cede a E0343.
        adr295_expect(
            ADR295_PREC18_BASE,
            Some("E0343: question mark outside result fn"),
        );
        // PREC-18c: `v[0] = 2` · `f(1)` · `let n: Int = f(2)?` → el IA va primero: E0344.
        adr295_expect(ADR295_PREC18_C, Some("E0344"));
        // PREC-18b: `?` antes del IA → E0343.
        adr295_expect(
            ADR295_PREC18_B,
            Some("E0343: question mark outside result fn"),
        );
        // Control sin `f(1)`: E0344 en ambos mundos.
        adr295_expect(ADR295_PREC18_CTL, Some("E0344"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos01_bare_user_call_e0272() {
        adr295_expect(ADR295_POS01, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos02_let_underscore_e0272() {
        adr295_expect(ADR295_POS02, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos03_let_named_dead_e0272() {
        adr295_expect(ADR295_POS03, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos04_let_plain_dead_e0272() {
        adr295_expect(ADR295_POS04, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos05_let_transitive_dead_e0272() {
        adr295_expect(ADR295_POS05, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos06_vec_set_bare_e0272() {
        adr295_expect(ADR295_POS06, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos07_vec_insert_bare_e0272() {
        adr295_expect(ADR295_POS07, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos08_nested_if_e0272() {
        adr295_expect(ADR295_POS08, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos09_nested_while_e0272() {
        adr295_expect(ADR295_POS09, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos10_nested_match_arm_e0272() {
        adr295_expect(ADR295_POS10, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos11_result_fn_midbody_e0272() {
        adr295_expect(ADR295_POS11, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos12_async_main_e0272() {
        adr295_expect(ADR295_POS12, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos13_main_tail_e0272() {
        adr295_expect(ADR295_POS13, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos14_io_helper_e0272() {
        adr295_expect(ADR295_POS14, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos15_nested_if_let_body_e0272() {
        adr295_expect(ADR295_POS15, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos16_ident_stmt_not_use_e0272() {
        adr295_expect(ADR295_POS16, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos17_ok_literal_midbody_e0272() {
        adr295_expect(ADR295_POS17, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos18_midbody_compound_e0272() {
        adr295_expect(ADR295_POS18, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos19_int_helper_bare_e0272() {
        adr295_expect(ADR295_POS19, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos20_timeout_bare_e0272() {
        adr295_expect(ADR295_POS20, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos21_try_reserve_bare_e0272() {
        adr295_expect(ADR295_POS21, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_pos22_main_tail_match_arm_e0272() {
        adr295_expect(ADR295_POS22, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg01_qmark_use_ok() {
        adr295_expect(ADR295_NEG01, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg02_let_used_match_ok() {
        adr295_expect(ADR295_NEG02, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg03_underscore_read_ok() {
        adr295_expect(ADR295_NEG03, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg04_tail_call_return_ok() {
        adr295_expect(ADR295_NEG04, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg05_tail_match_arms_return_ok() {
        adr295_expect(ADR295_NEG05, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg06_bare_qmark_stmt_ok() {
        adr295_expect(ADR295_NEG06, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg07_passed_as_arg_ok() {
        adr295_expect(ADR295_NEG07, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg08_if_let_scrutinee_ok() {
        adr295_expect(ADR295_NEG08, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg09_ident_try_ok() {
        adr295_expect(ADR295_NEG09, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg10_binding_tail_return_ok() {
        adr295_expect(ADR295_NEG10, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg11_vec_index_assign_ok() {
        adr295_expect(ADR295_NEG11, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg12_option_unit_out_ok() {
        adr295_expect(ADR295_NEG12, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg13_async_await_ok() {
        adr295_expect(ADR295_NEG13, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg14_param_unused_ok() {
        adr295_expect(ADR295_NEG14, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg15_receiver_bare_stmt_ok() {
        adr295_expect(ADR295_NEG15, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg16_compound_index_assign_ok() {
        adr295_expect(ADR295_NEG16, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg17_nested_tail_two_levels_ok() {
        adr295_expect(ADR295_NEG17, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_neg18_tail_if_else_ok() {
        adr295_expect(ADR295_NEG18, None);

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec01_prec_bare_before_e0206() {
        adr295_expect(ADR295_PREC01, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec02_prec_e0206_before_bare() {
        adr295_expect(ADR295_PREC02, Some("E0206"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec03_prec_bare_before_e0343() {
        adr295_expect(
            ADR295_PREC03,
            Some("E0343: question mark outside result fn"),
        );

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec04_prec_e0343_before_bare() {
        adr295_expect(
            ADR295_PREC04,
            Some("E0343: question mark outside result fn"),
        );

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec05_prec_bare_before_e0344() {
        adr295_expect(ADR295_PREC05, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec06_prec_e0344_before_bare() {
        adr295_expect(
            ADR295_PREC06,
            Some("E0344: index assign outside result fn @157..165"),
        );

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec07_prec_bare_before_dead_sink() {
        adr295_expect(ADR295_PREC07, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec08_prec_int_helper_deferred() {
        adr295_expect(ADR295_PREC08, Some("E0006: non-main fn `h` body must be Expr binary/path/lit/call in this slice @82..156"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec09_prec_dead_let_before_e0206() {
        adr295_expect(ADR295_PREC09, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec10_prec_int_helper_tail_e0342() {
        adr295_expect(ADR295_PREC10, Some("E0342"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec11_prec_bare_before_e0319() {
        adr295_expect(ADR295_PREC11, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec12_prec_e0319_before_bare() {
        adr295_expect(ADR295_PREC12, Some("E0319"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec13_prec_e0007_decl_after() {
        adr295_expect(ADR295_PREC13, Some("E0007"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec14_prec_undeclared_stmt_before_bare() {
        adr295_expect(ADR295_PREC14, Some("E0347"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec15_prec_bare_before_undeclared() {
        adr295_expect(ADR295_PREC15, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec16_prec_undeclared_let_rhs_wins() {
        adr295_expect(ADR295_PREC16, Some("E0347"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec17_prec_arm_let_before_e0206() {
        adr295_expect(ADR295_PREC17, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec19_prec_undeclared_vs_e0343() {
        adr295_expect(
            ADR295_PREC19,
            Some("E0343: question mark outside result fn"),
        );

        assert_eq!((), ());
    }

    #[test]
    fn adr295_prec20_prec_async_bare_e0241() {
        adr295_expect(ADR295_PREC20, Some("E0241"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_io01_io_host_read_bare_e0340() {
        adr295_expect(ADR295_IO01, Some("E0340"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_io02_io_host_let_underscore_e0340() {
        adr295_expect(ADR295_IO02, Some("E0340"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_io03_io_host_let_named_dead_e0272() {
        adr295_expect(ADR295_IO03, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_io04_io_host_let_underscore_named_e0272() {
        adr295_expect(ADR295_IO04, Some(ADR286_E0272));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_io05_io_host_write_bare_e0340() {
        adr295_expect(ADR295_IO05, Some("E0340"));

        assert_eq!((), ());
    }

    #[test]
    fn adr295_out01_await_result_out_of_scope_ok() {
        adr295_expect(ADR295_OUT01, None);

        assert_eq!((), ());
    }

    // ADR-286 S2 (CORE-0.10-MUST-USE-20260927): S2-D1 — el error del RHS gana a P2(b).
    const ADR286_S2_RHS_XOR: &str = r#"module adr286_s2_rhs_error_wins
    fn main() -> Io<()> {
      let a: Result<Int, String> = Ok(1)
      let b: Result<Int, String> = Ok(2)
      let c: Result<Int, String> = a.xor(b)
      print(0)
    }
    "#;
    const ADR286_S2_RHS_OR_DEAD: &str = r#"module adr286_s2_rhs_control_valid
    fn main() -> Io<()> {
      let a: Result<Int, String> = Ok(1)
      let b: Result<Int, String> = Ok(2)
      let c: Result<Int, String> = a.or(b)
      print(0)
    }
    "#;
    const ADR286_S2_RHS_OR_USED: &str = r#"module adr286_s2_rhs_control_used
    fn main() -> Io<()> {
      let a: Result<Int, String> = Ok(1)
      let b: Result<Int, String> = Ok(2)
      let c: Result<Int, String> = a.or(b)
      match c {
        Ok(v) => { print(v) }
        Err(e) => {
          print("err")
          print(e)
        }
      }
    }
    "#;
    const ADR286_S2_RHS_ANCHOR: &str = r#"module adr286_s2_rhs_control_anchor
    fn main() -> Io<()> {
      let a: Result<Int, String> = Ok(1)
      let b: Result<Int, String> = Ok(2)
      let c: Result<Int, String> = a.or(b)
      let mut w: Vec<Int> = Vec::new()
      print(w.bogus())
    }
    "#;

    #[test]
    fn adr286_s2_rhs_error_wins() {
        // Patrón de ejemplos/f2/neg/e0206-result-xor.arita: el init falla → el `let` no pasa el
        // check → P2(b) no se evalúa → E0206 exacto, no E0272 (los 9 BLOCKER no se migran).
        assert_eq!(
            adr283_check_src(ADR286_S2_RHS_XOR),
            Err("E0206: method not in F2 std whitelist".to_string())
        );
        // Control: mismo `let c` con init válido y sin uso → P2(b), anclado en el `let`.
        assert_eq!(
            adr283_check_src(ADR286_S2_RHS_OR_DEAD),
            Err(ADR286_E0272.to_string())
        );
        // Control: `c` usado como scrutinee → aceptado.
        assert_eq!(adr283_check_src(ADR286_S2_RHS_OR_USED), Ok(()));
        // Control de anclaje: el `let c` sin uso va antes que un E0206 posterior → gana E0272.
        assert_eq!(
            adr283_check_src(ADR286_S2_RHS_ANCHOR),
            Err(ADR286_E0272.to_string())
        );
    }

    // ADR-286 S2: S2-D2 — receptor y argumento son siempre uso; solo el alias directo es recursivo.
    const ADR286_S2_RECV_OR_DEAD: &str = r#"module adr286_s2_receiver_is_use
    fn f(x: Int) -> Result<Int, Int> {
      Ok(x)
    }
    fn main() -> Io<()> {
      let a: Result<Int, Int> = f(1)
      let b: Result<Int, Int> = f(2)
      let _c: Result<Int, Int> = a.or(b)
      print(0)
    }
    "#;
    const ADR286_S2_RECV_OR_USED: &str = r#"module adr286_s2_receiver_control_used
    fn f(x: Int) -> Result<Int, Int> {
      Ok(x)
    }
    fn main() -> Io<()> {
      let a: Result<Int, Int> = f(1)
      let b: Result<Int, Int> = f(2)
      let _c: Result<Int, Int> = a.or(b)
      match _c {
        Ok(v) => { print(v) }
        Err(e) => {
          print("err")
          print(e)
        }
      }
    }
    "#;
    const ADR286_S2_RECV_ALIAS_DEAD: &str = r#"module adr286_s2_receiver_control_alias
    fn f(x: Int) -> Result<Int, Int> {
      Ok(x)
    }
    fn main() -> Io<()> {
      let r: Result<Int, Int> = f(1)
      let y: Result<Int, Int> = r
      print(0)
    }
    "#;
    const ADR286_S2_RECV_ARG_USE: &str = r#"module adr286_s2_receiver_control_arg
    fn f(x: Int) -> Result<Int, Int> {
      Ok(x)
    }
    fn k(r: Result<Int, Int>) -> Result<Int, Int> {
      Ok(0)
    }
    fn main() -> Io<()> {
      let r: Result<Int, Int> = f(1)
      match k(r) {
        Ok(v) => { print(v) }
        Err(e) => {
          print("err")
          print(e)
        }
      }
    }
    "#;

    #[test]
    fn adr286_s2_receiver_is_use() {
        // `a` receptor y `b` argumento de `a.or(b)` = uso; solo `_c` sin uso → E0272 (P2(b) en `_c`).
        assert_eq!(
            adr283_check_src(ADR286_S2_RECV_OR_DEAD),
            Err(ADR286_E0272.to_string())
        );
        // Control: la misma fn usando `_c` (match) compila → `a` y `b` no disparan E0272.
        assert_eq!(adr283_check_src(ADR286_S2_RECV_OR_USED), Ok(()));
        // Control: alias directo `let y: Result<Int, Int> = r` con `y` sin uso → E0272.
        assert_eq!(
            adr283_check_src(ADR286_S2_RECV_ALIAS_DEAD),
            Err(ADR286_E0272.to_string())
        );
        // Control: `r` pasado como argumento a un callee que lo ignora → compila (S2-D2, S2-D4).
        assert_eq!(adr283_check_src(ADR286_S2_RECV_ARG_USE), Ok(()));
    }

    // Q2 (ADR-295 Sello 3): los bloques `test` no son fns `Result`; su última sentencia `Result`
    // es descarte (E0272), igual que la cola de `main`. La gramática de `test` solo admite
    // `assert`/`unsafe`/`todo`…, así que el caso se construye a mano en HIR.
    fn adr295_q2_module(test_body: Vec<HirStmt>) -> HirModule {
        HirModule {
            name: "adr295_q2".into(),
            functions: vec![
                HirFunction {
                    name: "f".into(),
                    params: vec![HirParam {
                        name: "x".into(),
                        ty: HirType::Int,
                    }],
                    ret_ty: HirType::Result(Box::new(HirType::Int), Box::new(HirType::Int)),
                    body: vec![HirStmt::Expr(HirExpr::Ok(Box::new(HirExpr::Path(
                        HirPath {
                            segments: vec!["x".into()],
                        },
                    ))))],
                    is_async: false,
                    is_pub: false,
                },
                HirFunction {
                    name: "main".into(),
                    params: vec![],
                    ret_ty: HirType::IoUnit,
                    body: vec![HirStmt::Expr(HirExpr::Call(HirCall {
                        callee: "print".into(),
                        args: vec![HirExpr::LitStr("ok".into())],
                    }))],
                    is_async: false,
                    is_pub: false,
                },
            ],
            tests: vec![HirTest {
                name: "t".into(),
                body: test_body,
            }],
            records: vec![],
            enums: vec![],
            imports: vec![],
        }
    }

    fn adr295_q2_call_f() -> HirExpr {
        HirExpr::Call(HirCall {
            callee: "f".into(),
            args: vec![HirExpr::LitInt(1)],
        })
    }

    fn adr295_q2_assert() -> HirStmt {
        HirStmt::Assert {
            lhs: HirExpr::LitInt(1),
            rhs: HirExpr::LitInt(1),
        }
    }

    #[test]
    fn adr295_q2_test_block_result_tail_is_e0272() {
        // Cola de un `test` de tipo Result → descarte.
        let hir = adr295_q2_module(vec![adr295_q2_assert(), HirStmt::Expr(adr295_q2_call_f())]);
        assert_eq!(
            check(&hir).expect_err("test tail Result").to_string(),
            ADR286_E0272
        );
        // A mitad de bloque también.
        let hir = adr295_q2_module(vec![HirStmt::Expr(adr295_q2_call_f()), adr295_q2_assert()]);
        assert_eq!(
            check(&hir).expect_err("test mid Result").to_string(),
            ADR286_E0272
        );
        // Control: el valor se consume (`match` con brazos que lo leen) → compila.
        let hir = adr295_q2_module(vec![
            adr295_q2_assert(),
            HirStmt::Match {
                scrutinee: adr295_q2_call_f(),
                arms: vec![
                    HirMatchArm {
                        pat: HirPat::Ok("v".into()),
                        body: vec![HirStmt::Assert {
                            lhs: HirExpr::Path(HirPath {
                                segments: vec!["v".into()],
                            }),
                            rhs: HirExpr::LitInt(1),
                        }],
                    },
                    HirMatchArm {
                        pat: HirPat::Err("e".into()),
                        body: vec![HirStmt::Assert {
                            lhs: HirExpr::Path(HirPath {
                                segments: vec!["e".into()],
                            }),
                            rhs: HirExpr::LitInt(0),
                        }],
                    },
                ],
            },
        ]);
        assert!(check(&hir).is_ok(), "{:?}", check(&hir));
        // Control: un `test` solo con asserts sigue compilando.
        assert!(check(&adr295_q2_module(vec![adr295_q2_assert()])).is_ok());
    }

    // Los 10 puntos de cuerpo anidado (ADR-295 Sello 1): una llamada `Result` suelta en cada uno.
    const ADR295_NEST_N01_IF_THEN: &str = r#"module adr295_n01_if_then
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let c: Bool = true
  if c {
    f(1)
  }
  print("after")
}
"#;
    const ADR295_NEST_N02_IF_ELSE: &str = r#"module adr295_n02_if_else
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let c: Bool = true
  if c {
    print(1)
  } else {
    f(1)
  }
  print("after")
}
"#;
    const ADR295_NEST_N03_IFLET_THEN: &str = r#"module adr295_n03_iflet_then
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let o: Option<Int> = Some(1)
  if let Some(x) = o {
    f(x)
  } else {
    print(0)
  }
  print("after")
}
"#;
    const ADR295_NEST_N04_IFLET_ELSE: &str = r#"module adr295_n04_iflet_else
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let o: Option<Int> = Some(1)
  if let Some(x) = o {
    print(x)
  } else {
    f(1)
  }
  print("after")
}
"#;
    const ADR295_NEST_N05_WHILE: &str = r#"module adr295_n05_while
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut i: Int = 0
  while i < 1 {
    i = i + 1
    f(1)
  }
  print("after")
}
"#;
    const ADR295_NEST_N06_WHILELET: &str = r#"module adr295_n06_whilelet
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  while let Some(x) = v.pop() {
    f(x)
  }
  print("after")
}
"#;
    const ADR295_NEST_N07_MATCH_RESULT: &str = r#"module adr295_n07_match_result
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  match f(1) {
    Ok(v) => {
      f(v)
    }
    Err(e) => {
      print(e)
    }
  }
  print("after")
}
"#;
    const ADR295_NEST_N08_MATCH_OPTION: &str = r#"module adr295_n08_match_option
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let o: Option<Int> = Some(1)
  match o {
    Some(x) => {
      f(x)
    }
    None => {
      print(0)
      print(1)
    }
  }
  print("after")
}
"#;
    const ADR295_NEST_N09_MATCH_BOOL: &str = r#"module adr295_n09_match_bool
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let c: Bool = true
  match c {
    true => {
      f(1)
    }
    false => {
      print(0)
    }
  }
  print("after")
}
"#;
    const ADR295_NEST_N10_MATCH_ENUM: &str = r#"module adr295_n10_match_enum
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  match col {
    Color::Red => {
      f(1)
    }
    Color::Green => {
      print(2)
    }
  }
  print("after")
}
"#;

    #[test]
    fn adr295_ten_nested_body_points_bare_call_e0272() {
        let srcs: [&str; 10] = [
            ADR295_NEST_N01_IF_THEN,
            ADR295_NEST_N02_IF_ELSE,
            ADR295_NEST_N03_IFLET_THEN,
            ADR295_NEST_N04_IFLET_ELSE,
            ADR295_NEST_N05_WHILE,
            ADR295_NEST_N06_WHILELET,
            ADR295_NEST_N07_MATCH_RESULT,
            ADR295_NEST_N08_MATCH_OPTION,
            ADR295_NEST_N09_MATCH_BOOL,
            ADR295_NEST_N10_MATCH_ENUM,
        ];
        for src in srcs {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR286_E0272.to_string()),
                "{src}"
            );
        }
    }

    // A-1: las colas de if/else, if let/else y match (Result, Option, Bool, enum) de una fn `Result` son retorno.
    const ADR295_TAIL_T01_IF_ELSE: &str = r#"module adr295_t01_if_else
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  if c {
    f(1)
  } else {
    Err(2)
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T02_IFLET_ELSE: &str = r#"module adr295_t02_iflet_else
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  if let Some(x) = o {
    f(x)
  } else {
    Err(2)
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T03_MATCH_RESULT: &str = r#"module adr295_t03_match_result
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match f(1) {
    Ok(v) => {
      f(v)
    }
    Err(e) => {
      Err(e)
    }
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T04_MATCH_OPTION: &str = r#"module adr295_t04_match_option
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match o {
    Some(x) => {
      f(x)
    }
    None => {
      Err(0)
    }
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T05_MATCH_BOOL: &str = r#"module adr295_t05_match_bool
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match c {
    true => {
      f(1)
    }
    false => {
      Err(0)
    }
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T06_MATCH_ENUM: &str = r#"module adr295_t06_match_enum
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match col {
    Color::Red => {
      f(1)
    }
    Color::Green => {
      Err(2)
    }
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_TAIL_T07_NESTED_IF_IN_MATCH: &str = r#"module adr295_t07_nested_if_in_match
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match c {
    true => {
      if c {
        f(1)
      } else {
        Err(3)
      }
    }
    false => {
      Err(0)
    }
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    #[test]
    fn adr295_tail_propagates_through_every_branching_form_ok() {
        let srcs: [&str; 7] = [
            ADR295_TAIL_T01_IF_ELSE,
            ADR295_TAIL_T02_IFLET_ELSE,
            ADR295_TAIL_T03_MATCH_RESULT,
            ADR295_TAIL_T04_MATCH_OPTION,
            ADR295_TAIL_T05_MATCH_BOOL,
            ADR295_TAIL_T06_MATCH_ENUM,
            ADR295_TAIL_T07_NESTED_IF_IN_MATCH,
        ];
        for src in srcs {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
    }

    // A-2: compuesto que no está en cola, o cuerpo de bucle / `if` sin `else` en cola → descarte; y un `let` muerto en el brazo enum.
    const ADR295_NONTAIL_M01_IF_ELSE_NONTAIL: &str = r#"module adr295_m01_if_else_nontail
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  if c {
    f(1)
  } else {
    f(2)
  }
  Ok(0)
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_NONTAIL_M02_MATCH_ENUM_NONTAIL: &str = r#"module adr295_m02_match_enum_nontail
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match col {
    Color::Red => {
      f(1)
    }
    Color::Green => {
      f(2)
    }
  }
  Ok(0)
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_NONTAIL_M03_WHILE_LAST: &str = r#"module adr295_m03_while_last
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  let mut i: Int = 0
  while i < 1 {
    i = i + 1
    f(1)
  }
  Ok(0)
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_NONTAIL_M04_IF_NOELSE_TAIL: &str = r#"module adr295_m04_if_noelse_tail
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  if c {
    f(1)
  }
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_NONTAIL_M05_ENUM_ARM_DEAD_LET: &str = r#"module adr295_m05_enum_arm_dead_let
enum Color {
  Red,
  Green,
}
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool, o: Option<Int>, col: Color) -> Result<Int, Int> {
  match col {
    Color::Red => {
      let r: Result<Int, Int> = f(1)
      print(1)
    }
    Color::Green => {
      print(2)
    }
  }
  Ok(0)
}
fn main() -> Io<()> {
  let col: Color = Color::Red
  let o: Option<Int> = Some(1)
  match g(true, o, col) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    #[test]
    fn adr295_non_tail_compound_and_loop_tails_are_discards() {
        let srcs: [&str; 5] = [
            ADR295_NONTAIL_M01_IF_ELSE_NONTAIL,
            ADR295_NONTAIL_M02_MATCH_ENUM_NONTAIL,
            ADR295_NONTAIL_M03_WHILE_LAST,
            ADR295_NONTAIL_M04_IF_NOELSE_TAIL,
            ADR295_NONTAIL_M05_ENUM_ARM_DEAD_LET,
        ];
        for src in srcs {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR286_E0272.to_string()),
                "{src}"
            );
        }
    }

    // P4 / S2-D2: alias directo, redeclaración, sombra de patrón y `ident` de cola.
    const ADR295_LEX_ALIAS_USED: &str = r#"module adr295_alias_used
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  let y: Result<Int, Int> = r
  match y {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_LEX_IDENT_STMT_IN_RESULT_TAIL_NESTED: &str = r#"module adr295_ident_tail_arm
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool) -> Result<Int, Int> {
  if c {
    let ok: Result<Int, Int> = f(1)
    ok
  } else {
    Err(2)
  }
}
fn main() -> Io<()> {
  match g(true) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_LEX_USED_IN_LOOP: &str = r#"module adr295_used_in_loop
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  let mut i: Int = 0
  while i < 1 {
    match r {
      Ok(v) => { print(v) }
      Err(e) => {
        print("err")
        print(e)
      }
    }
    i = i + 1
  }
}
"#;
    const ADR295_LEX_IDENT_STMT_NONTAIL_ARM: &str = r#"module adr295_ident_nontail_arm
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn g(c: Bool) -> Result<Int, Int> {
  if c {
    let ok: Result<Int, Int> = f(1)
    ok
  } else {
    Err(2)
  }
  Ok(0)
}
fn main() -> Io<()> {
  match g(true) {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_LEX_PATTERN_SHADOW_DEAD: &str = r#"module adr295_pattern_shadow
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Int = 5
  let q: Result<Int, Int> = f(1)
  match f(2) {
    Ok(q) => { print(q) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;
    const ADR295_LEX_SHADOW_REDECL_DEAD: &str = r#"module adr295_shadow_redecl
fn f(x: Int) -> Result<Int, Int> {
  Ok(x)
}
fn main() -> Io<()> {
  let r: Result<Int, Int> = f(1)
  let r: Result<Int, Int> = f(2)
  match r {
    Ok(v) => { print(v) }
    Err(e) => {
      print("err")
      print(e)
    }
  }
}
"#;

    #[test]
    fn adr295_use_analysis_alias_shadow_and_ident_tail() {
        for src in [
            ADR295_LEX_ALIAS_USED,
            ADR295_LEX_IDENT_STMT_IN_RESULT_TAIL_NESTED,
            ADR295_LEX_USED_IN_LOOP,
        ] {
            assert_eq!(adr283_check_src(src), Ok(()), "{src}");
        }
        for src in [
            ADR295_LEX_IDENT_STMT_NONTAIL_ARM,
            ADR295_LEX_PATTERN_SHADOW_DEAD,
            ADR295_LEX_SHADOW_REDECL_DEAD,
        ] {
            assert_eq!(
                adr283_check_src(src),
                Err(ADR286_E0272.to_string()),
                "{src}"
            );
        }
    }
}
