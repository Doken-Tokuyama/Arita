//! ARITA F1.1 + F2 + F2.1 + F2.2 + F2.3 + LANG-CONTRACT + ASYNC-V0 pest parser + AST
//! (ADR-004 / ADR-006 / ADR-007 / ADR-014 / ADR-015 / ADR-016 / ADR-019 / ADR-025 / ADR-027).
//! CUT-ID ASYNC-V0-20260913 — `async fn` / `await`; E0240/E0241; test + assert; E0210–E0213; E0220–E0224; E0231.
//! CUT-ID TRAPS-LEN-THEATER-20260914 — E0214 vacuous length assert (ADR-030).
//! CUT-ID TRAPS-CMP-THEATER-20260915 — E0215 vacuous comparison assert (ADR-038).
//! CUT-ID DEPS-V0-20260914 — E0261 external crate path reject (ADR-029).
//! CUT-ID … IF-LET-V0 — if let Some + E0275.

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "arita.pest"]
struct AritaParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
    /// F2 tests; F1.1 lowering always yields empty.
    pub tests: Vec<Test>,
    /// LANG-CONTRACT (ADR-019): top-level contracts; not emitted to Rust.
    pub contracts: Vec<Contract>,
    /// ADR-233 Core 0.1: user `record` type definitions (module-level).
    pub records: Vec<RecordDef>,
    /// ADR-234 Core 0.1: user `enum` type definitions (unit variants).
    pub enums: Vec<EnumDef>,
    /// ADR-254 / CORE-0.4-MULTI-MODULE: `use mod::item` (local modules only).
    pub uses: Vec<UseDecl>,
}

/// ADR-254: `use module::item` — path/workspace modules; never crates.io.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecl {
    pub module: String,
    pub item: String,
}

/// ADR-233: `record Name { field: Type, ... }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<RecordField>,
    /// ADR-256: `pub record` crosses crate boundary; default private.
    pub is_pub: bool,
}

/// ADR-233: one field in a record definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub ty: Type,
}

/// ADR-234: `enum Name { V1, V2, ... }` unit variants only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
}

/// Language-level contract expect body (one kind per contract).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractExpect {
    /// Ordered stdout lines (`expect_stdout` strings in source order).
    Stdout(Vec<String>),
    /// Expected reject diagnostic code (`expect_reject "E0xxx"`).
    Reject(String),
}

/// Top-level `contract <name> { … }` (ADR-019 / ADR-020).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub name: String,
    pub expect: ContractExpect,
    /// Entry fn for `expect_stdout` (`None` ⇒ `"main"`). Ignored for `expect_reject` (ADR-020).
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    /// F2 params; F1.1 ⇒ empty.
    pub params: Vec<FnParam>,
    /// F2 return type; F1.1 ⇒ `Type::IoUnit`.
    pub ret_ty: Type,
    pub body: Vec<Stmt>,
    /// ADR-027: `async fn` (v0 Ret = Io<()> only).
    pub is_async: bool,
    /// ADR-256: `pub fn` crosses crate boundary; default private.
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnParam {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    String,
    /// ADR-236: bytes (`Vec<u8>`). Text surface aliases String.
    Bytes,
    /// ADR-243: opaque async task handle (`JoinHandle`).
    Task,
    /// ADR-047: unit `()` as Result type argument.
    Unit,
    Vec(Box<Type>),
    /// ADR-237: `Map<K, V>` (v0: K∈{Text/String}, V∈{Int}).
    Map(Box<Type>, Box<Type>),
    /// ADR-047: `Result<T, E>` with T,E ∈ {Int,Bool,String,()}.
    Result(Box<Type>, Box<Type>),
    /// ADR-050: `Option<T>` with T ∈ {Int,Bool,String,(),Option<…>} (ADR-184).
    Option(Box<Type>),
    /// ADR-233: user `record` type by name.
    Named(String),
    IoUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: String,
        ty: Type,
        init: Expr,
    },
    Expr(Expr),
    /// Equality check inside `Test` bodies (not `Binary` Eq).
    Assert {
        lhs: Expr,
        rhs: Expr,
    },
    /// F2.1: `if cond { … } [else { … }]`.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    /// ADR-053/054: `if let Some|Ok|Err(x) = … { … } else { … }`.
    IfLet {
        pat: Pat,
        scrutinee: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    /// ADR-055: `while let Some(x) = opt { … }` (re-eval; no else v0).
    WhileLet {
        pat: Pat,
        scrutinee: Expr,
        body: Vec<Stmt>,
    },
    /// F2.1: `while cond { … }`.
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// F2.1: `name = expr` to an existing `let mut` place.
    Assign {
        name: String,
        value: Expr,
    },
    /// ADR-282 (CORE-0.9-MAP-ASSIGN): `target[key] = value` (una indexación, `=` plano).
    /// El parser no conoce tipos: HIR acepta sólo `Map` local (`let mut`); Vec/String/otro → E0314.
    IndexAssign {
        target: String,
        key: Expr,
        value: Expr,
        /// ADR-290 slice A: `None` = plain `=`; `Some(Add|Sub|Mul)` = `+=` / `-=` / `*=`
        /// (the parser never produces another operator: `/=`, `%=` … stay E0006).
        op: Option<BinOp>,
        /// Byte span of the whole `target[key] = value` statement (E0314 `@a..b`).
        span: Span,
    },
    /// F2.2: `match scrutinee { pat => { stmts } … }` (statement form).
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
    },
    /// F2.3: unlabeled `break` (legal only inside `while`; HIR E0224 otherwise).
    Break,
    /// F2.3: unlabeled `continue` (legal only inside `while`; HIR E0224 otherwise).
    Continue,
}

/// F2.2 match arm: `pat => { body }` (statement form).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pat: Pat,
    pub body: Vec<Stmt>,
}

/// ADR-025 match-as-expr arm: `pat => { value }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchExprArm {
    pub pat: Pat,
    pub value: Expr,
}

/// F2.2 match pattern (MVP: lit bool/int or wildcard).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pat {
    LitBool(bool),
    LitInt(i64),
    Wildcard,
    /// ADR-047: `Ok(binding)`.
    Ok(String),
    /// ADR-047: `Err(binding)`.
    Err(String),
    /// ADR-050: `Some(binding)`.
    Some(String),
    /// ADR-050: `None`.
    None,
    /// ADR-234: unit variant pat (`Red` or `Color::Red`).
    Variant {
        enum_name: Option<String>,
        variant: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Call(Call),
    LitStr(String),
    LitInt(i64),
    LitBool(bool),
    Path(Path),
    /// Int arithmetic only; Eq goes through `Stmt::Assert`.
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Borrow {
        mutable: bool,
        inner: Box<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// ADR-025: `match scrutinee { pat => { value } … }` (expression form).
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchExprArm>,
    },
    /// ADR-027: `await <call>` (only inside `async fn`).
    Await {
        inner: Box<Expr>,
    },
    /// ADR-278: postfix `?` on Result-producing expr.
    Try {
        inner: Box<Expr>,
    },
    /// ADR-047: unit literal `()`.
    LitUnit,
    /// ADR-047: `Ok(expr)`.
    Ok(Box<Expr>),
    /// ADR-047: `Err(expr)`.
    Err(Box<Expr>),
    /// ADR-050: `Some(expr)`.
    Some(Box<Expr>),
    /// ADR-050: `None`.
    None,
    /// ADR-233: `Name { field: expr, ... }`.
    RecordLit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// ADR-233: shared field get `x.field` (not Index).
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },
    /// ADR-234: `Enum::Variant` construct.
    EnumPath {
        enum_name: String,
        variant: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub callee: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    /// Int comparison → Bool (expr position; assert keeps Stmt::Assert).
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub name: String,
    pub body: Vec<Stmt>,
}

/// Optional item wrapper; `Module.tests` is the preferred pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Test(Test),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Legacy variant — still works for callers matching on Message.
    Message(String),
    /// Preferred: stable code + English message + optional byte span.
    Coded {
        code: &'static str,
        message: String,
        span: Option<Span>,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Message(m) => write!(f, "{m}"),
            ParseError::Coded {
                code,
                message,
                span,
            } => {
                write!(f, "{code}: {message}")?;
                if let Some(s) = span {
                    write!(f, " @{}..{}", s.start, s.end)?;
                }
                Ok(())
            }
        }
    }
}

fn coded(code: &'static str, message: impl Into<String>, span: Option<Span>) -> ParseError {
    ParseError::Coded {
        code,
        message: message.into(),
        span,
    }
}

fn span_of(pair: &pest::iterators::Pair<Rule>) -> Span {
    let s = pair.as_span();
    Span {
        start: s.start(),
        end: s.end(),
    }
}

fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "module"
            | "fn"
            | "let"
            | "spec"
            | "const"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "use"
            | "deps"
            | "mod"
            | "Io"
            | "print"
            | "if"
            | "else"
            | "while"
            | "for"
            | "return"
            | "match"
            | "break"
            | "continue"
            | "var"
            | "true"
            | "false"
            | "mut"
            | "borrow"
            | "Bool"
            | "test"
            | "assert"
            | "todo"
            | "unimplemented"
            | "unsafe"
            | "extern"
            | "contract"
            | "expect_stdout"
            | "expect_reject"
            | "target"
            | "async"
            | "await"
            | "Ok"
            | "Err"
            | "Result"
            | "Some"
            | "None"
            | "Option"
            | "record"
    )
}

fn comments_and_ws_only(source: &str) -> bool {
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        return false;
    }
    true
}

fn unescape_string(inner: &str) -> Result<String, ()> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    // Clippy `while_let_on_iterator`: cannot use `for` when escape needs a second next().
    loop {
        match chars.next() {
            None => break,
            Some('\\') => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(_) | None => return Err(()),
            },
            Some(c) => out.push(c),
        }
    }
    Ok(out)
}

fn string_lit_to_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let raw = pair.as_str();
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(coded(
            "E0005",
            "unclosed LitStr / invalid escape in print",
            Some(sp),
        ));
    }
    let inner = &raw[1..raw.len() - 1];
    let s = unescape_string(inner).map_err(|_| {
        coded(
            "E0005",
            "unclosed LitStr / invalid escape in print",
            Some(sp),
        )
    })?;
    Ok(Expr::LitStr(s))
}

fn has_unclosed_or_bad_escape(source: &str) -> bool {
    let mut in_str = false;
    let mut chars = source.chars().peekable();
    // Body pairs `next()` / `peek()`; `for` exclusive-borrows the iterator (E0499),
    // and `loop`+`let-else` trips Clippy `while_let_loop`. Allow the intentional form.
    #[allow(clippy::while_let_on_iterator)]
    while let Some(c) = chars.next() {
        if !in_str {
            if c == '"' {
                in_str = true;
            } else if c == '/' && chars.peek() == Some(&'/') {
                chars.next(); // consume second '/'
                for n in chars.by_ref() {
                    if n == '\n' {
                        break;
                    }
                }
            }
            continue;
        }
        if c == '\\' {
            match chars.next() {
                Some('n' | 't' | 'r' | '\\' | '"') => {}
                Some(_) | None => return true,
            }
        } else if c == '\n' {
            return true;
        } else if c == '"' {
            in_str = false;
        }
    }
    in_str
}

/// ADR-291 (E0346): canonical text of the Mutex-reject diagnostic (`Display` appends ` @a..b`).
const E0346_MUTEX_REJECT: &str = "mutex concurrency is not available in this surface";

/// ADR-291 D2: names rejected as a type (`ty_named`) or as an associated-constructor receiver.
/// `RwLock` stays out (Q7).
fn is_mutex_reserved(name: &str) -> bool {
    matches!(name, "Mutex" | "Arc")
}

fn mutex_reject(span: Span) -> ParseError {
    coded("E0346", E0346_MUTEX_REJECT, Some(span))
}

/// ADR-291: `Named` type lowering (let / param / ret / Result|Option arg). The span is the
/// `ty_named` pair, which is exactly the identifier (`ty_named = { ident }`).
fn lower_ty_named(pair: &pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    if is_mutex_reserved(pair.as_str()) {
        return Err(mutex_reject(span_of(pair)));
    }
    Ok(Type::Named(pair.as_str().to_string()))
}

fn prev_non_blank_is_dot(b: &[u8], mut s: usize) -> bool {
    while s > 0 && (b[s - 1] == b' ' || b[s - 1] == b'\t') {
        s -= 1;
    }
    s > 0 && b[s - 1] == b'.'
}

/// ADR-291 (E0346), parse-failure path: `Mutex<..>` / `Arc<..>` (generic forms) and chained or
/// prefixed `.lock(` never reach the lowering because pest fails first. Scan the line that holds
/// the failure position (outside strings and `//` comments) for the first `Mutex`/`Arc` followed
/// by `<`, or `lock` preceded by `.` and followed by `(`. Lexical only; accepts nothing new.
fn mutex_reject_in_line(source: &str, pos: usize) -> Option<Span> {
    let pos = pos.min(source.len());
    let line_start = source.get(..pos)?.rfind('\n').map_or(0, |i| i + 1);
    let line_end = source
        .get(pos..)?
        .find('\n')
        .map_or(source.len(), |i| pos + i);
    let line = source.get(line_start..line_end)?;
    let b = line.as_bytes();
    let mut i = 0usize;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            break;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let s = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let word = &line[s..i];
            let mut j = i;
            while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            let next = b.get(j).copied();
            let hit = match word {
                "Mutex" | "Arc" => next == Some(b'<'),
                "lock" => next == Some(b'(') && prev_non_blank_is_dot(b, s),
                _ => false,
            };
            if hit {
                return Some(Span {
                    start: line_start + s,
                    end: line_start + i,
                });
            }
            continue;
        }
        i += 1;
    }
    None
}

fn classify_pest_error(source: &str, err: &pest::error::Error<Rule>) -> ParseError {
    let (start, end) = match &err.location {
        pest::error::InputLocation::Pos(p) => (*p, *p),
        pest::error::InputLocation::Span((a, b)) => (*a, *b),
    };
    let sp = Some(Span { start, end });

    if has_unclosed_or_bad_escape(source) {
        return coded("E0005", "unclosed LitStr / invalid escape in print", sp);
    }

    // ADR-291 (E0346): generic `Mutex<..>` / `Arc<..>` and chained `.lock(` die in pest, not in
    // the lowering; anchor on the identifier / selector instead of the generic E0006.
    if let Some(msp) = mutex_reject_in_line(source, start) {
        return mutex_reject(msp);
    }

    // Keyword-looking token where an ident was required
    let around = source.get(start..).unwrap_or("");
    let tok = around
        .split(|c: char| c.is_whitespace() || "(){},;".contains(c))
        .next()
        .unwrap_or("");
    if is_keyword(tok) {
        return coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{tok}`"),
            sp,
        );
    }

    coded(
        "E0006",
        format!("construct outside F1.1 (parse failure): {err}"),
        sp,
    )
}

#[derive(Debug, Clone)]
enum RetKind {
    OkIoUnit,
    OkInt,
    /// ADR-245/277: Result / Named surface returns.
    OkType(Type),
    Missing,
    Other(Span),
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    params: Vec<FnParam>,
    ret: RetKind,
    body: Vec<Stmt>,
    is_async: bool,
    is_pub: bool,
}

/// ADR-261/266: `recv[k]` desugars to `recv.get(k)` (Option); never Rust Index.
fn lower_index_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let recv = inner
        .next()
        .ok_or_else(|| coded("E0006", "index_expr missing receiver", Some(sp.clone())))?;
    let key = inner
        .next()
        .ok_or_else(|| coded("E0006", "index_expr missing key", Some(sp.clone())))?;
    // ADR-273 / E0341: argv Index (`args[i]` / `argv[i]`) is theater — pin before get-desugar.
    let recv_name = recv.as_str();
    if recv_name == "args" || recv_name == "argv" {
        return Err(coded("E0341", "required arg miss as ok", Some(sp)));
    }
    let key_expr = match key.as_rule() {
        Rule::int_lit => {
            let n: i64 = key.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", key.as_str()),
                    Some(span_of(&key)),
                )
            })?;
            Expr::LitInt(n)
        }
        Rule::string_lit => string_lit_to_expr(key)?,
        Rule::ident => Expr::Path(Path {
            segments: vec![key.as_str().to_string()],
        }),
        other => {
            return Err(coded(
                "E0006",
                format!("index key must be int|string|ident, got {other:?}"),
                Some(span_of(&key)),
            ))
        }
    };
    Ok(Expr::MethodCall {
        receiver: Box::new(Expr::Path(Path {
            segments: vec![recv.as_str().to_string()],
        })),
        method: "get".into(),
        args: vec![key_expr],
    })
}

fn lower_map_new_call(_pair: pest::iterators::Pair<Rule>) -> Expr {
    Expr::Call(Call {
        callee: "Map::new".into(),
        args: vec![],
    })
}

fn lower_list_new_call(_pair: pest::iterators::Pair<Rule>) -> Expr {
    Expr::Call(Call {
        callee: "List::new".into(),
        args: vec![],
    })
}

fn lower_spawn_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| coded("E0006", "spawn missing call", Some(sp)))?;
    let call = lower_user_call(inner)?;
    Ok(Expr::Call(Call {
        callee: "spawn".into(),
        args: vec![call],
    }))
}

fn lower_join_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let id = pair
        .into_inner()
        .next()
        .ok_or_else(|| coded("E0006", "join missing ident", Some(sp)))?;
    Ok(Expr::Call(Call {
        callee: "join".into(),
        args: vec![Expr::Path(Path {
            segments: vec![id.as_str().to_string()],
        })],
    }))
}

fn lower_use_item(pair: pest::iterators::Pair<Rule>) -> Result<UseDecl, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let mod_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "use missing module", Some(sp.clone())))?;
    let item_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "use missing item", Some(sp)))?;
    Ok(UseDecl {
        module: mod_p.as_str().to_string(),
        item: item_p.as_str().to_string(),
    })
}

fn lower_scenario_item(pair: pest::iterators::Pair<Rule>) -> Result<Contract, ParseError> {
    // ADR-239: scenario { acceptance "…" } → ContractExpect::Stdout (same runtime as contract)
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "scenario missing name", Some(sp.clone())))?;
    let name = name_p.as_str().to_string();
    let mut stdout_lines = Vec::new();
    let mut target = None;
    for elem in inner {
        let elem = if elem.as_rule() == Rule::scenario_body_elem {
            elem.into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty scenario_body_elem", Some(sp.clone())))?
        } else {
            elem
        };
        match elem.as_rule() {
            Rule::contract_target => {
                let id = elem
                    .into_inner()
                    .next()
                    .ok_or_else(|| coded("E0006", "target missing ident", Some(sp.clone())))?;
                target = Some(id.as_str().to_string());
            }
            Rule::acceptance_elem => {
                let lit = elem
                    .into_inner()
                    .next()
                    .ok_or_else(|| coded("E0006", "acceptance missing string", Some(sp.clone())))?;
                match string_lit_to_expr(lit)? {
                    Expr::LitStr(s) if !s.is_empty() => stdout_lines.push(s),
                    _ => {
                        return Err(coded(
                            "E0006",
                            "acceptance must be a non-empty string",
                            Some(sp.clone()),
                        ))
                    }
                }
            }
            other => {
                return Err(coded(
                    "E0006",
                    format!("scenario body only allows target|acceptance, got {other:?}"),
                    Some(span_of(&elem)),
                ))
            }
        }
    }
    if stdout_lines.is_empty() {
        return Err(coded("E0006", "scenario must have acceptance", Some(sp)));
    }
    Ok(Contract {
        name,
        expect: ContractExpect::Stdout(stdout_lines),
        target,
    })
}

fn lower_param_type(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::ty_int => Ok(Type::Int),
        Rule::ty_bool => Ok(Type::Bool),
        Rule::ty_string | Rule::ty_text => Ok(Type::String),
        Rule::ty_bytes => Ok(Type::Bytes),
        Rule::ty_unit => Ok(Type::Unit),
        Rule::ty_named => lower_ty_named(&pair),
        Rule::ty_result => lower_ty_result(pair),
        Rule::ty_option => lower_ty_option(pair),
        other => Err(coded(
            "E0006",
            format!("unsupported fn param type {other:?}"),
            Some(sp),
        )),
    }
}

fn lower_fn_params(pair: pest::iterators::Pair<Rule>) -> Result<Vec<FnParam>, ParseError> {
    let mut params = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() != Rule::fn_param {
            continue;
        }
        let sp = span_of(&p);
        let mut inner = p.into_inner();
        let name_pair = inner
            .next()
            .ok_or_else(|| coded("E0006", "fn_param missing name", Some(sp.clone())))?;
        let name = name_pair.as_str().to_string();
        if is_keyword(&name) {
            return Err(coded(
                "E0007",
                format!("illegal ident / keyword misuse: `{name}`"),
                Some(span_of(&name_pair)),
            ));
        }
        let ty_pair = inner
            .next()
            .ok_or_else(|| coded("E0006", "fn_param missing type", Some(sp)))?;
        let ty = lower_param_type(ty_pair)?;
        params.push(FnParam { name, ty });
    }
    Ok(params)
}

fn lower_fn_ret_ok(pair: pest::iterators::Pair<Rule>) -> Result<RetKind, ParseError> {
    let sp = span_of(&pair);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| coded("E0006", "fn_ret_ok missing type", Some(sp.clone())))?;
    match inner.as_rule() {
        Rule::ty_io_unit => Ok(RetKind::OkIoUnit),
        Rule::ty_int => Ok(RetKind::OkInt),
        Rule::ty_unit => Ok(RetKind::OkType(Type::Unit)),
        Rule::ty_named => Ok(RetKind::OkType(lower_ty_named(&inner)?)),
        Rule::ty_result => Ok(RetKind::OkType(lower_ty_result(inner)?)),
        other => Err(coded(
            "E0006",
            format!("unexpected fn_ret_ok type {other:?}"),
            Some(span_of(&inner)),
        )),
    }
}

fn lower_function(pair: pest::iterators::Pair<Rule>) -> Result<FunctionInfo, ParseError> {
    let mut inner = pair.into_inner();
    let mut is_pub = false;
    let mut first = inner
        .next()
        .ok_or_else(|| coded("E0003", "missing fn main or name ≠ main", None))?;
    if first.as_rule() == Rule::pub_vis {
        is_pub = true;
        first = inner
            .next()
            .ok_or_else(|| coded("E0003", "missing fn main or name ≠ main", None))?;
    }
    let name_pair = first;
    let name = name_pair.as_str().to_string();
    if name != "main" && is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_pair)),
        ));
    }

    let mut params = Vec::new();
    let mut ret = RetKind::Missing;
    let mut body = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::fn_params => {
                params = lower_fn_params(p)?;
            }
            Rule::fn_ret_ok => {
                ret = lower_fn_ret_ok(p)?;
            }
            Rule::fn_ret_bad => {
                ret = RetKind::Other(span_of(&p));
            }
            Rule::block => {
                for stmt_pair in p.into_inner() {
                    body.push(lower_stmt(stmt_pair)?);
                }
            }
            _ => {}
        }
    }

    Ok(FunctionInfo {
        name,
        params,
        ret,
        body,
        is_async: false,
        is_pub,
    })
}

fn lower_async_function(pair: pest::iterators::Pair<Rule>) -> Result<FunctionInfo, ParseError> {
    let mut info = lower_function(pair)?;
    info.is_async = true;
    // ADR-027: async main / Io<()> ; ADR-245: async Result handlers (HTTP)
    match &info.ret {
        RetKind::OkIoUnit => {}
        RetKind::OkType(Type::Result(_, _)) => {}
        _ => {
            return Err(coded("E0241", "async feature not allowed here", None));
        }
    }
    Ok(info)
}

fn lower_await_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| coded("E0006", "await missing call", Some(sp.clone())))?;
    let call = match inner.as_rule() {
        Rule::join_call => lower_join_call(inner)?,
        Rule::user_call => lower_user_call(inner)?,
        other => {
            return Err(coded(
                "E0006",
                format!("await expects join_call|user_call, got {other:?}"),
                Some(sp),
            ))
        }
    };
    Ok(Expr::Await {
        inner: Box::new(call),
    })
}

fn lower_method_call_len(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    lower_method_call_named(pair, "len")
}

fn lower_method_call_is_empty(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    lower_method_call_named(pair, "is_empty")
}

fn lower_method_call_named(
    pair: pest::iterators::Pair<Rule>,
    method: &str,
) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let recv = inner.next().ok_or_else(|| {
        coded(
            "E0006",
            format!("method call {method} missing receiver"),
            Some(sp),
        )
    })?;
    let mut args = Vec::new();
    for a in inner {
        args.push(lower_print_arg(a)?);
    }
    Ok(Expr::MethodCall {
        receiver: Box::new(Expr::Path(Path {
            segments: vec![recv.as_str().to_string()],
        })),
        method: method.into(),
        args,
    })
}

fn lower_method_call_other(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let recv = inner
        .next()
        .ok_or_else(|| coded("E0006", "method call missing receiver", Some(sp.clone())))?;
    let method_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "method call missing method name", Some(sp.clone())))?;
    let method = method_pair.as_str().to_string();
    // ADR-291 (E0346): `.lock()` for any receiver (anchored on the selector) and the
    // associated constructors `Mutex.new` / `Arc.new` (anchored on the receiver identifier).
    // `Expr::MethodCall` keeps no span, so the check has to live here, where the pairs do.
    if method == "lock" {
        return Err(mutex_reject(span_of(&method_pair)));
    }
    if method == "new" && is_mutex_reserved(recv.as_str()) {
        return Err(mutex_reject(span_of(&recv)));
    }
    let mut args = Vec::new();
    for a in inner {
        // method_push_arg wraps string_lit|int_lit|ident
        let arg_pair = if a.as_rule() == Rule::method_push_arg {
            a.into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "method call missing arg", Some(sp.clone())))?
        } else {
            a
        };
        match arg_pair.as_rule() {
            Rule::string_lit => {
                args.push(string_lit_to_expr(arg_pair)?);
            }
            Rule::int_lit => {
                let n: i64 = arg_pair.as_str().parse().map_err(|_| {
                    coded(
                        "E0006",
                        format!("invalid int literal: {}", arg_pair.as_str()),
                        Some(span_of(&arg_pair)),
                    )
                })?;
                args.push(Expr::LitInt(n));
            }
            Rule::ident => args.push(Expr::Path(Path {
                segments: vec![arg_pair.as_str().to_string()],
            })),
            _ => {
                return Err(coded(
                    "E0006",
                    "method call arg must be string_lit|int_lit|ident",
                    Some(span_of(&arg_pair)),
                ))
            }
        }
    }
    Ok(Expr::MethodCall {
        receiver: Box::new(Expr::Path(Path {
            segments: vec![recv.as_str().to_string()],
        })),
        method,
        args,
    })
}

fn lower_method_call_push(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let recv = inner.next().ok_or_else(|| {
        coded(
            "E0006",
            "method call push missing receiver",
            Some(sp.clone()),
        )
    })?;
    let arg_wrap = inner
        .next()
        .ok_or_else(|| coded("E0006", "method call push missing arg", Some(sp.clone())))?;
    let arg_pair = if arg_wrap.as_rule() == Rule::method_push_arg {
        arg_wrap
            .into_inner()
            .next()
            .ok_or_else(|| coded("E0006", "method call push missing arg", Some(sp.clone())))?
    } else {
        arg_wrap
    };
    let arg = match arg_pair.as_rule() {
        Rule::string_lit => string_lit_to_expr(arg_pair)?,
        Rule::int_lit => {
            let n: i64 = arg_pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", arg_pair.as_str()),
                    Some(span_of(&arg_pair)),
                )
            })?;
            Expr::LitInt(n)
        }
        Rule::ident => Expr::Path(Path {
            segments: vec![arg_pair.as_str().to_string()],
        }),
        _ => {
            return Err(coded(
                "E0006",
                "method call push arg must be string_lit|int_lit|ident",
                Some(span_of(&arg_pair)),
            ))
        }
    };
    Ok(Stmt::Expr(Expr::MethodCall {
        receiver: Box::new(Expr::Path(Path {
            segments: vec![recv.as_str().to_string()],
        })),
        method: "push".into(),
        args: vec![arg],
    }))
}

fn lower_cmp_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let lhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "cmp_expr missing lhs", Some(sp.clone())))?;
    let op_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "cmp_expr missing op", Some(sp.clone())))?;
    let rhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "cmp_expr missing rhs", Some(sp.clone())))?;
    let op = match op_pair.as_str() {
        "==" => BinOp::Eq,
        "!=" => BinOp::Ne,
        "<" => BinOp::Lt,
        "<=" => BinOp::Le,
        ">" => BinOp::Gt,
        ">=" => BinOp::Ge,
        other => {
            return Err(coded(
                "E0006",
                format!("unknown cmp_op: {other}"),
                Some(span_of(&op_pair)),
            ))
        }
    };
    let lhs = lower_primary(lhs_pair)?;
    let rhs = lower_primary(rhs_pair)?;
    Ok(Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn lower_bool_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::bool_expr => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty bool_expr", Some(sp)))?;
            lower_bool_expr(inner)
        }
        Rule::cmp_expr => lower_cmp_expr(pair),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected bool_expr, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_block_stmts(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Stmt>, ParseError> {
    let mut body = Vec::new();
    for stmt_pair in pair.into_inner() {
        body.push(lower_stmt(stmt_pair)?);
    }
    Ok(body)
}

fn lower_if_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let cond_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "if missing condition", Some(sp.clone())))?;
    let then_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "if missing then block", Some(sp.clone())))?;
    let cond = lower_bool_expr(cond_pair)?;
    let then_body = lower_block_stmts(then_pair)?;
    let else_body = match inner.next() {
        Some(else_block) if else_block.as_rule() == Rule::block => {
            Some(lower_block_stmts(else_block)?)
        }
        Some(other) => {
            return Err(coded(
                "E0006",
                format!("if else expected block, got {:?}", other.as_rule()),
                Some(span_of(&other)),
            ))
        }
        None => None,
    };
    Ok(Stmt::If {
        cond,
        then_body,
        else_body,
    })
}

fn lower_if_let_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let pat_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "if-let missing pat", Some(sp.clone())))?;
    let scrut_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "if-let missing scrutinee", Some(sp.clone())))?;
    let then_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "if-let missing then block", Some(sp.clone())))?;
    let pat = lower_match_pat(pat_pair)?;
    let binding = match &pat {
        Pat::Some(n) | Pat::Ok(n) | Pat::Err(n) => n.clone(),
        _ => {
            return Err(coded(
                "E0006",
                "if-let v0 only Some|Ok|Err(binding)",
                Some(sp.clone()),
            ))
        }
    };
    if is_keyword(&binding) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{binding}`"),
            Some(sp.clone()),
        ));
    }
    let scrutinee = lower_match_scrutinee(scrut_pair)?;
    let then_body = lower_block_stmts(then_pair)?;
    let else_body = match inner.next() {
        Some(else_block) if else_block.as_rule() == Rule::block => {
            Some(lower_block_stmts(else_block)?)
        }
        Some(other) => {
            return Err(coded(
                "E0006",
                format!("if-let else expected block, got {:?}", other.as_rule()),
                Some(span_of(&other)),
            ))
        }
        None => None,
    };
    Ok(Stmt::IfLet {
        pat,
        scrutinee,
        then_body,
        else_body,
    })
}

fn lower_while_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let cond_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "while missing condition", Some(sp.clone())))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "while missing block", Some(sp)))?;
    let cond = lower_bool_expr(cond_pair)?;
    let body = lower_block_stmts(body_pair)?;
    Ok(Stmt::While { cond, body })
}

fn lower_while_let_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let pat_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "while-let missing pat", Some(sp.clone())))?;
    let scrut_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "while-let missing scrutinee", Some(sp.clone())))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "while-let missing block", Some(sp.clone())))?;
    let pat = lower_match_pat(pat_pair)?;
    let binding = match &pat {
        Pat::Some(n) | Pat::Ok(n) | Pat::Err(n) => n.clone(),
        _ => {
            return Err(coded(
                "E0006",
                "while-let v0 only Some|Ok|Err(binding)",
                Some(sp.clone()),
            ))
        }
    };
    if is_keyword(&binding) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{binding}`"),
            Some(sp.clone()),
        ));
    }
    let scrutinee = lower_while_let_scrutinee(scrut_pair)?;
    let body = lower_block_stmts(body_pair)?;
    Ok(Stmt::WhileLet {
        pat,
        scrutinee,
        body,
    })
}

fn lower_while_let_scrutinee(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::while_let_scrutinee => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty while_let_scrutinee", Some(sp)))?;
            lower_while_let_scrutinee(inner)
        }
        Rule::method_call_other => lower_method_call_other(pair),
        Rule::none_expr => lower_none_expr(pair),
        Rule::some_expr => lower_some_expr(pair),
        Rule::ok_expr => lower_ok_expr(pair),
        Rule::err_expr => lower_err_expr(pair),
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        _ => Err(coded(
            "E0006",
            format!("expected while_let_scrutinee, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_match_scrutinee(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::match_scrutinee => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty match_scrutinee", Some(sp)))?;
            lower_match_scrutinee(inner)
        }
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        Rule::user_call => lower_user_call(pair),
        Rule::host_call => lower_host_call(pair),
        _ => Err(coded(
            "E0006",
            format!("expected match_scrutinee, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_match_pat(pair: pest::iterators::Pair<Rule>) -> Result<Pat, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::match_pat => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty match_pat", Some(sp)))?;
            lower_match_pat(inner)
        }
        Rule::match_pat_ok => {
            let name = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::ident)
                .map(|p| p.as_str().to_string())
                .ok_or_else(|| coded("E0006", "Ok pat missing binding", Some(sp.clone())))?;
            if is_keyword(&name) {
                return Err(coded(
                    "E0007",
                    format!("illegal ident / keyword misuse: `{name}`"),
                    Some(sp),
                ));
            }
            Ok(Pat::Ok(name))
        }
        Rule::match_pat_err => {
            let name = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::ident)
                .map(|p| p.as_str().to_string())
                .ok_or_else(|| coded("E0006", "Err pat missing binding", Some(sp.clone())))?;
            if is_keyword(&name) {
                return Err(coded(
                    "E0007",
                    format!("illegal ident / keyword misuse: `{name}`"),
                    Some(sp),
                ));
            }
            Ok(Pat::Err(name))
        }
        Rule::match_pat_some => {
            let name = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::ident)
                .map(|p| p.as_str().to_string())
                .ok_or_else(|| coded("E0006", "Some pat missing binding", Some(sp.clone())))?;
            if is_keyword(&name) {
                return Err(coded(
                    "E0007",
                    format!("illegal ident / keyword misuse: `{name}`"),
                    Some(sp),
                ));
            }
            Ok(Pat::Some(name))
        }
        Rule::match_pat_none => Ok(Pat::None),
        Rule::enum_path => {
            let mut inner = pair.into_inner();
            let en = inner
                .next()
                .ok_or_else(|| coded("E0006", "empty enum_path pat", Some(sp.clone())))?;
            let var = inner
                .next()
                .ok_or_else(|| coded("E0006", "empty enum_path pat", Some(sp)))?;
            Ok(Pat::Variant {
                enum_name: Some(en.as_str().to_string()),
                variant: var.as_str().to_string(),
            })
        }
        Rule::ident => Ok(Pat::Variant {
            enum_name: None,
            variant: pair.as_str().to_string(),
        }),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Pat::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Pat::LitInt(n))
        }
        Rule::wildcard => Ok(Pat::Wildcard),
        _ => Err(coded(
            "E0006",
            format!("expected match_pat, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_match_arm(pair: pest::iterators::Pair<Rule>) -> Result<MatchArm, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let pat_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match arm missing pattern", Some(sp.clone())))?;
    let body_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match arm missing body", Some(sp)))?;
    let pat = lower_match_pat(pat_pair)?;
    let body = lower_block_stmts(body_pair)?;
    Ok(MatchArm { pat, body })
}

fn lower_match_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let scrut_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match missing scrutinee", Some(sp.clone())))?;
    let scrutinee = lower_match_scrutinee(scrut_pair)?;
    let mut arms = Vec::new();
    for arm_pair in inner {
        if arm_pair.as_rule() != Rule::match_arm {
            return Err(coded(
                "E0006",
                format!("match expected arm, got {:?}", arm_pair.as_rule()),
                Some(span_of(&arm_pair)),
            ));
        }
        arms.push(lower_match_arm(arm_pair)?);
    }
    if arms.is_empty() {
        return Err(coded("E0006", "match requires at least one arm", Some(sp)));
    }
    Ok(Stmt::Match { scrutinee, arms })
}

fn lower_match_expr_value(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::match_expr_value => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty match_expr_value", Some(sp)))?;
            lower_match_expr_value(inner)
        }
        Rule::binary_expr => lower_binary_expr(pair),
        Rule::cmp_expr => lower_cmp_expr(pair),
        Rule::host_call => lower_host_call(pair),
        Rule::user_call => lower_user_call(pair),
        Rule::string_lit => string_lit_to_expr(pair),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected match_expr_value, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_match_expr_arm(pair: pest::iterators::Pair<Rule>) -> Result<MatchExprArm, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let pat_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match expr arm missing pattern", Some(sp.clone())))?;
    let val_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match expr arm missing value", Some(sp)))?;
    let pat = lower_match_pat(pat_pair)?;
    let value = lower_match_expr_value(val_pair)?;
    Ok(MatchExprArm { pat, value })
}

fn lower_match_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let scrut_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "match expr missing scrutinee", Some(sp.clone())))?;
    let scrutinee = lower_match_scrutinee(scrut_pair)?;
    let mut arms = Vec::new();
    for arm_pair in inner {
        if arm_pair.as_rule() != Rule::match_expr_arm {
            return Err(coded(
                "E0006",
                format!("match expr expected arm, got {:?}", arm_pair.as_rule()),
                Some(span_of(&arm_pair)),
            ));
        }
        arms.push(lower_match_expr_arm(arm_pair)?);
    }
    if arms.is_empty() {
        return Err(coded(
            "E0006",
            "match expr requires at least one arm",
            Some(sp),
        ));
    }
    Ok(Expr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
    })
}

fn lower_assign_rhs(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::assign_rhs => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty assign_rhs", Some(sp)))?;
            lower_assign_rhs(inner)
        }
        Rule::match_expr => lower_match_expr(pair),
        Rule::index_expr => lower_index_expr(pair),
        Rule::enum_path => lower_enum_path(pair),
        Rule::record_lit => lower_record_lit(pair),
        Rule::ok_expr => lower_ok_expr(pair),
        Rule::err_expr => lower_err_expr(pair),
        Rule::some_expr => lower_some_expr(pair),
        Rule::none_expr => lower_none_expr(pair),
        Rule::method_call_len => lower_method_call_len(pair),
        Rule::method_call_is_empty => lower_method_call_is_empty(pair),
        Rule::method_call_other => lower_method_call_other(pair),
        Rule::field_access => lower_field_access(pair),
        Rule::cmp_expr => lower_cmp_expr(pair),
        Rule::binary_expr => lower_binary_expr(pair),
        Rule::host_call_try => lower_host_call_try(pair),
        Rule::host_call => lower_host_call(pair),
        Rule::user_call_try => lower_user_call_try(pair),
        Rule::user_call => lower_user_call(pair),
        Rule::ident_try => lower_ident_try(pair),
        Rule::string_lit => string_lit_to_expr(pair),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected assign_rhs, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_assign_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "assign missing name", Some(sp.clone())))?;
    let name = name_pair.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_pair)),
        ));
    }
    let rhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "assign missing rhs", Some(sp)))?;
    let value = lower_assign_rhs(rhs_pair)?;
    Ok(Stmt::Assign { name, value })
}

/// ADR-282: `target[key] = value` → `Stmt::IndexAssign` sin decidir tipo (Map vs Vec/String se
/// decide en HIR tras tipar). RHS = cualquier `assign_rhs` v0 (no hereda la restricción lit de `put`).
fn lower_index_assign_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let target = inner
        .next()
        .ok_or_else(|| coded("E0006", "index assign missing target", Some(sp.clone())))?
        .as_str()
        .to_string();
    let key_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "index assign missing key", Some(sp.clone())))?;
    let key = match key_pair.as_rule() {
        Rule::int_lit => Expr::LitInt(key_pair.as_str().parse().map_err(|_| {
            coded(
                "E0006",
                format!("invalid int literal: {}", key_pair.as_str()),
                Some(span_of(&key_pair)),
            )
        })?),
        Rule::string_lit => string_lit_to_expr(key_pair)?,
        Rule::ident => Expr::Path(Path {
            segments: vec![key_pair.as_str().to_string()],
        }),
        other => {
            return Err(coded(
                "E0006",
                format!("index key must be int|string|ident, got {other:?}"),
                Some(span_of(&key_pair)),
            ))
        }
    };
    let mut rhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "index assign missing rhs", Some(sp.clone())))?;
    // ADR-290: optional `op_assign` pair between the key and the rhs.
    let mut op = None;
    if rhs_pair.as_rule() == Rule::op_assign {
        op = Some(match rhs_pair.as_str() {
            "+=" => BinOp::Add,
            "-=" => BinOp::Sub,
            "*=" => BinOp::Mul,
            other => {
                return Err(coded(
                    "E0006",
                    format!("unsupported compound operator: {other}"),
                    Some(span_of(&rhs_pair)),
                ))
            }
        });
        rhs_pair = inner
            .next()
            .ok_or_else(|| coded("E0006", "index assign missing rhs", Some(sp.clone())))?;
    }
    let value = lower_assign_rhs(rhs_pair)?;
    Ok(Stmt::IndexAssign {
        target,
        key,
        value,
        op,
        span: sp,
    })
}

/// ADR-283: callee of the synthetic first statement the parser prepends to a fn whose
/// shape-gate error (main E0001 / `fn -> Int` E0006) was deferred because the fn contains an
/// `IndexAssign`. Args: `[LitStr(code), LitStr(message), LitInt(start), LitInt(end)]`.
/// Not lexable as an ident, so user code cannot forge it. HIR (`arita_hir::check`) always
/// consumes it and ALWAYS errors: the non-Map index-assign diagnostic if the body produces one,
/// otherwise exactly the deferred parser error (same code / text / span).
pub const DEFERRED_SHAPE_MARKER: &str = "<arita:deferred-shape>";

/// ADR-283: purely structural (no type peek) — does the body contain an `IndexAssign`?
fn body_contains_index_assign(body: &[Stmt]) -> bool {
    body.iter().any(|s| match s {
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
        } => {
            body_contains_index_assign(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|b| body_contains_index_assign(b))
        }
        Stmt::While { body, .. } | Stmt::WhileLet { body, .. } => body_contains_index_assign(body),
        Stmt::Match { arms, .. } => arms.iter().any(|a| body_contains_index_assign(&a.body)),
        _ => false,
    })
}

fn deferred_shape_marker(err: &ParseError) -> Option<Stmt> {
    let ParseError::Coded {
        code,
        message,
        span: Some(sp),
    } = err
    else {
        return None;
    };
    Some(Stmt::Expr(Expr::Call(Call {
        callee: DEFERRED_SHAPE_MARKER.to_string(),
        args: vec![
            Expr::LitStr((*code).to_string()),
            Expr::LitStr(message.clone()),
            Expr::LitInt(i64::try_from(sp.start).ok()?),
            Expr::LitInt(i64::try_from(sp.end).ok()?),
        ],
    })))
}

/// ADR-283 (Ingeniero §3): the exact parser shape error (E0001 / E0006: code, text, span) that
/// [`parse`] deferred behind a [`DEFERRED_SHAPE_MARKER`] statement, if the module carries one.
/// `None` for every module without a marker (all valid programs).
pub fn deferred_shape_error(module: &Module) -> Option<ParseError> {
    module.functions.iter().find_map(|f| {
        let Some(Stmt::Expr(Expr::Call(c))) = f.body.first() else {
            return None;
        };
        if c.callee != DEFERRED_SHAPE_MARKER {
            return None;
        }
        let malformed = || coded("E0006", "malformed deferred shape marker", None);
        let [Expr::LitStr(code), Expr::LitStr(message), Expr::LitInt(start), Expr::LitInt(end)] =
            c.args.as_slice()
        else {
            return Some(malformed());
        };
        let code: &'static str = match code.as_str() {
            "E0001" => "E0001",
            "E0006" => "E0006",
            _ => return Some(malformed()),
        };
        let (Ok(start), Ok(end)) = (usize::try_from(*start), usize::try_from(*end)) else {
            return Some(malformed());
        };
        Some(coded(code, message.clone(), Some(Span { start, end })))
    })
}

/// ADR-283 (Ingeniero §3): parse-only entry point (for `arita parse` and any consumer that does
/// NOT run `arita_hir::check`). Same as [`parse`], but a deferred shape error is returned as the
/// exact parser error instead of travelling to HIR as a marker, so no invalid program leaves it Ok.
/// Build / LSP / package / measure keep calling [`parse`] + HIR (HIR decides E0344/E0314 first).
pub fn parse_standalone(source: &str) -> Result<Module, ParseError> {
    let module = parse(source)?;
    match deferred_shape_error(&module) {
        Some(e) => Err(e),
        None => Ok(module),
    }
}

fn lower_primary(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::primary => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty primary in binary_expr", Some(sp)))?;
            lower_primary(inner)
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected int_lit|ident primary, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_binary_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let lhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "binary_expr missing lhs", Some(sp.clone())))?;
    let op_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "binary_expr missing op", Some(sp.clone())))?;
    let rhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "binary_expr missing rhs", Some(sp.clone())))?;
    let op = match op_pair.as_str() {
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "%" => BinOp::Rem,
        other => {
            return Err(coded(
                "E0006",
                format!("unknown bin_op: {other}"),
                Some(span_of(&op_pair)),
            ))
        }
    };
    let lhs = lower_primary(lhs_pair)?;
    let rhs = lower_primary(rhs_pair)?;
    Ok(Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn lower_host_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    // ADR-033: `host.<fn>(args…)` → Call { callee: "host::<fn>", args }
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "host_call missing fn name", Some(sp.clone())))?;
    let name = name_pair.as_str().to_string();
    if name.is_empty() || is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_pair)),
        ));
    }
    // ADR-291 (E0346): `.lock(` is rejected for any receiver, `host` included.
    if name == "lock" {
        return Err(mutex_reject(span_of(&name_pair)));
    }
    let mut args = Vec::new();
    for arg in inner {
        args.push(lower_print_arg(arg)?);
    }
    Ok(Expr::Call(Call {
        callee: format!("host::{name}"),
        args,
    }))
}

fn lower_user_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "user_call missing callee", Some(sp.clone())))?;
    let callee = name_pair.as_str().to_string();
    if callee == "print" || callee == "Vec" {
        return Err(coded(
            "E0006",
            format!("construct outside F1.1: call `{callee}`"),
            Some(sp),
        ));
    }
    if is_keyword(&callee) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{callee}`"),
            Some(span_of(&name_pair)),
        ));
    }
    let mut args = Vec::new();
    for arg in inner {
        args.push(lower_print_arg(arg)?);
    }
    Ok(Expr::Call(Call { callee, args }))
}

fn lower_print_arg(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::print_arg_ok | Rule::print_arg => {
            let inner = pair.into_inner().next().ok_or_else(|| {
                coded(
                    "E0010",
                    "print with args ≠ one LitStr|Int|Bool|Path|len|is_empty|call",
                    Some(sp),
                )
            })?;
            lower_print_arg(inner)
        }
        Rule::string_lit => string_lit_to_expr(pair),
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::host_call => lower_host_call(pair),
        Rule::method_call_len => lower_method_call_len(pair),
        Rule::method_call_is_empty => lower_method_call_is_empty(pair),
        Rule::method_call_other => lower_method_call_other(pair),
        Rule::index_expr => lower_index_expr(pair),
        Rule::field_access => lower_field_access(pair),
        Rule::user_call => lower_user_call(pair),
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0010",
            "print with args ≠ one LitStr|Int|Bool|Path|len|is_empty|call",
            Some(sp),
        )),
    }
}

fn lower_borrow_kw_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    // ADR-235: same AST as `&` / `&mut`
    lower_borrow_expr(pair)
}

fn lower_borrow_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut mutable = false;
    let mut name: Option<String> = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::mut_kw => mutable = true,
            Rule::ident => name = Some(p.as_str().to_string()),
            _ => {}
        }
    }
    let name = name.ok_or_else(|| coded("E0006", "borrow_expr missing place ident", Some(sp)))?;
    Ok(Expr::Borrow {
        mutable,
        inner: Box::new(Expr::Path(Path {
            segments: vec![name],
        })),
    })
}

fn lower_unit_lit(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    if pair.as_rule() != Rule::unit_lit {
        return Err(coded(
            "E0006",
            format!("expected unit_lit, got {:?}", pair.as_rule()),
            Some(sp),
        ));
    }
    Ok(Expr::LitUnit)
}

fn lower_result_payload(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::result_payload => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty result_payload", Some(sp)))?;
            lower_result_payload(inner)
        }
        Rule::unit_lit => lower_unit_lit(pair),
        Rule::string_lit => string_lit_to_expr(pair),
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected result_payload, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_ok_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let payload = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::result_payload)
        .ok_or_else(|| coded("E0006", "Ok missing payload", Some(sp)))?;
    Ok(Expr::Ok(Box::new(lower_result_payload(payload)?)))
}

fn lower_err_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let payload = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::result_payload)
        .ok_or_else(|| coded("E0006", "Err missing payload", Some(sp)))?;
    Ok(Expr::Err(Box::new(lower_result_payload(payload)?)))
}

fn lower_some_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let payload = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::result_payload)
        .ok_or_else(|| coded("E0006", "Some missing payload", Some(sp)))?;
    Ok(Expr::Some(Box::new(lower_result_payload(payload)?)))
}

fn lower_none_expr(_pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    Ok(Expr::None)
}

fn lower_ty_result_arg(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::ty_result_arg => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty ty_result_arg", Some(sp)))?;
            lower_ty_result_arg(inner)
        }
        Rule::ty_result => Ok(lower_ty_result(pair)?),
        Rule::ty_option => Ok(lower_ty_option(pair)?),
        Rule::ty_int => Ok(Type::Int),
        Rule::ty_bool => Ok(Type::Bool),
        Rule::ty_string | Rule::ty_text => Ok(Type::String),
        Rule::ty_bytes => Ok(Type::Bytes),
        Rule::ty_unit => Ok(Type::Unit),
        Rule::ty_named => lower_ty_named(&pair),
        _ => Err(coded(
            "E0006",
            format!("expected Result type arg, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_ty_result(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    let sp = span_of(&pair);
    let mut args = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::ty_result_arg {
            args.push(lower_ty_result_arg(p)?);
        }
    }
    if args.len() != 2 {
        return Err(coded(
            "E0006",
            format!("Result expects 2 type args, got {}", args.len()),
            Some(sp),
        ));
    }
    Ok(Type::Result(
        Box::new(args[0].clone()),
        Box::new(args[1].clone()),
    ))
}

fn lower_ty_option_arg(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::ty_option_arg => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty ty_option_arg", Some(sp)))?;
            lower_ty_option_arg(inner)
        }
        Rule::ty_option => Ok(lower_ty_option(pair)?),
        Rule::ty_result => Ok(lower_ty_result(pair)?),
        Rule::ty_int => Ok(Type::Int),
        Rule::ty_bool => Ok(Type::Bool),
        Rule::ty_string | Rule::ty_text => Ok(Type::String),
        Rule::ty_bytes => Ok(Type::Bytes),
        Rule::ty_unit => Ok(Type::Unit),
        Rule::ty_named => lower_ty_named(&pair),
        _ => Err(coded(
            "E0006",
            format!("expected Option type arg, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_ty_option(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
    let sp = span_of(&pair);
    let mut args = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::ty_option_arg {
            args.push(lower_ty_option_arg(p)?);
        }
    }
    if args.len() != 1 {
        return Err(coded(
            "E0006",
            format!("Option expects 1 type arg, got {}", args.len()),
            Some(sp),
        ));
    }
    Ok(Type::Option(Box::new(args[0].clone())))
}

fn lower_field_access(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let base = inner
        .next()
        .ok_or_else(|| coded("E0006", "field access missing base", Some(sp.clone())))?;
    let field = inner
        .next()
        .ok_or_else(|| coded("E0006", "field access missing field", Some(sp)))?;
    Ok(Expr::FieldAccess {
        base: Box::new(Expr::Path(Path {
            segments: vec![base.as_str().to_string()],
        })),
        field: field.as_str().to_string(),
    })
}

fn lower_record_init_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| coded("E0006", "record field init missing expr", None))?;
    match inner.as_rule() {
        Rule::index_expr => lower_index_expr(inner),
        Rule::field_access => lower_field_access(inner),
        Rule::string_lit => string_lit_to_expr(inner),
        Rule::bool_lit => {
            let v = match inner.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(span_of(&inner)),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::int_lit => {
            let n: i64 = inner.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", inner.as_str()),
                    Some(span_of(&inner)),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![inner.as_str().to_string()],
        })),
        other => Err(coded(
            "E0006",
            format!("unsupported record field init: {other:?}"),
            Some(span_of(&inner)),
        )),
    }
}

fn lower_record_lit(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "record lit missing name", Some(sp.clone())))?;
    let name = name_p.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_p)),
        ));
    }
    let mut fields = Vec::new();
    for p in inner {
        if p.as_rule() != Rule::record_field_init {
            continue;
        }
        let mut fi = p.into_inner();
        let fname = fi
            .next()
            .ok_or_else(|| coded("E0006", "record field missing name", Some(sp.clone())))?;
        let fexpr = fi
            .next()
            .ok_or_else(|| coded("E0006", "record field missing expr", Some(sp.clone())))?;
        let expr = if fexpr.as_rule() == Rule::record_init_expr {
            lower_record_init_expr(fexpr)?
        } else {
            return Err(coded(
                "E0006",
                "record field init shape",
                Some(span_of(&fexpr)),
            ));
        };
        fields.push((fname.as_str().to_string(), expr));
    }
    Ok(Expr::RecordLit { name, fields })
}

fn lower_enum_item(pair: pest::iterators::Pair<Rule>) -> Result<EnumDef, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "enum missing name", Some(sp.clone())))?;
    let name = name_p.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_p)),
        ));
    }
    let mut variants = Vec::new();
    for p in inner {
        if p.as_rule() != Rule::enum_variant {
            continue;
        }
        let v = p.as_str().to_string();
        if is_keyword(&v) {
            return Err(coded(
                "E0007",
                format!("illegal ident / keyword misuse: `{v}`"),
                Some(span_of(&p)),
            ));
        }
        if variants.iter().any(|x| x == &v) {
            return Err(coded("E0317", "unknown variant", Some(span_of(&p))));
        }
        variants.push(v);
    }
    if variants.is_empty() {
        return Err(coded("E0006", "enum has no variants", Some(sp)));
    }
    Ok(EnumDef { name, variants })
}

fn lower_enum_path(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let en = inner
        .next()
        .ok_or_else(|| coded("E0006", "enum_path missing enum", Some(sp.clone())))?;
    let var = inner
        .next()
        .ok_or_else(|| coded("E0006", "enum_path missing variant", Some(sp)))?;
    Ok(Expr::EnumPath {
        enum_name: en.as_str().to_string(),
        variant: var.as_str().to_string(),
    })
}

fn lower_record_item(pair: pest::iterators::Pair<Rule>) -> Result<RecordDef, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let mut is_pub = false;
    let mut name_p = inner
        .next()
        .ok_or_else(|| coded("E0006", "record missing name", Some(sp.clone())))?;
    if name_p.as_rule() == Rule::pub_vis {
        is_pub = true;
        name_p = inner
            .next()
            .ok_or_else(|| coded("E0006", "record missing name", Some(sp.clone())))?;
    }
    let name = name_p.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_p)),
        ));
    }
    let mut fields = Vec::new();
    for p in inner {
        if p.as_rule() != Rule::record_field_decl {
            continue;
        }
        let fsp = span_of(&p);
        let mut fi = p.into_inner();
        let fname = fi
            .next()
            .ok_or_else(|| coded("E0006", "record field missing name", Some(fsp.clone())))?;
        let fty = fi
            .next()
            .ok_or_else(|| coded("E0006", "record field missing type", Some(fsp)))?;
        // record_field_ty wraps ty_int|ty_string|ty_bool
        let ty_pair = if fty.as_rule() == Rule::record_field_ty {
            let fsp = span_of(&fty);
            fty.into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "record field type empty", Some(fsp)))?
        } else {
            fty
        };
        let ty = match ty_pair.as_rule() {
            Rule::ty_int => Type::Int,
            Rule::ty_string => Type::String,
            Rule::ty_bool => Type::Bool,
            _ => {
                return Err(coded(
                    "E0006",
                    "record field type must be Int, Bool, or String",
                    Some(span_of(&ty_pair)),
                ))
            }
        };
        fields.push(RecordField {
            name: fname.as_str().to_string(),
            ty,
        });
    }
    Ok(RecordDef {
        name,
        fields,
        is_pub,
    })
}

fn lower_let_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut mutable = false;
    let mut name: Option<String> = None;
    let mut ty: Option<Type> = None;
    let mut init: Option<Expr> = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::mut_kw => mutable = true,
            Rule::ident => {
                if name.is_none() {
                    name = Some(p.as_str().to_string());
                } else {
                    // path init after binding name already taken
                    init = Some(Expr::Path(Path {
                        segments: vec![p.as_str().to_string()],
                    }));
                }
            }
            Rule::ty_int => ty = Some(Type::Int),
            Rule::ty_string | Rule::ty_text => ty = Some(Type::String),
            Rule::ty_bytes => ty = Some(Type::Bytes),
            Rule::ty_task => ty = Some(Type::Task),
            Rule::ty_bool => ty = Some(Type::Bool),
            Rule::ty_vec_int => ty = Some(Type::Vec(Box::new(Type::Int))),
            Rule::ty_list_int => ty = Some(Type::Vec(Box::new(Type::Int))),
            Rule::ty_map_text_int => {
                ty = Some(Type::Map(Box::new(Type::String), Box::new(Type::Int)))
            }
            Rule::ty_result => ty = Some(lower_ty_result(p)?),
            Rule::ty_option => ty = Some(lower_ty_option(p)?),
            Rule::ty_named => {
                ty = Some(lower_ty_named(&p)?);
            }
            Rule::int_lit => {
                let n: i64 = p.as_str().parse().map_err(|_| {
                    coded(
                        "E0006",
                        format!("invalid int literal: {}", p.as_str()),
                        Some(span_of(&p)),
                    )
                })?;
                init = Some(Expr::LitInt(n));
            }
            Rule::string_lit => {
                init = Some(string_lit_to_expr(p)?);
            }
            Rule::bool_lit => {
                let v = match p.as_str() {
                    "true" => true,
                    "false" => false,
                    other => {
                        return Err(coded(
                            "E0006",
                            format!("invalid bool literal: {other}"),
                            Some(span_of(&p)),
                        ))
                    }
                };
                init = Some(Expr::LitBool(v));
            }
            Rule::match_expr => {
                init = Some(lower_match_expr(p)?);
            }
            Rule::index_expr => {
                init = Some(lower_index_expr(p)?);
            }
            Rule::enum_path => {
                init = Some(lower_enum_path(p)?);
            }
            Rule::record_lit => {
                init = Some(lower_record_lit(p)?);
            }
            Rule::field_access => {
                init = Some(lower_field_access(p)?);
            }
            Rule::ok_expr => {
                init = Some(lower_ok_expr(p)?);
            }
            Rule::err_expr => {
                init = Some(lower_err_expr(p)?);
            }
            Rule::some_expr => {
                init = Some(lower_some_expr(p)?);
            }
            Rule::none_expr => {
                init = Some(lower_none_expr(p)?);
            }
            Rule::cmp_expr => {
                init = Some(lower_cmp_expr(p)?);
            }
            Rule::binary_expr => {
                init = Some(lower_binary_expr(p)?);
            }
            Rule::borrow_kw_expr => {
                init = Some(lower_borrow_kw_expr(p)?);
            }
            Rule::borrow_expr => {
                init = Some(lower_borrow_expr(p)?);
            }
            Rule::vec_new_call => {
                init = Some(Expr::Call(Call {
                    callee: "Vec::new".into(),
                    args: vec![],
                }));
            }
            Rule::method_call_len => {
                init = Some(lower_method_call_len(p)?);
            }
            Rule::method_call_is_empty => {
                init = Some(lower_method_call_is_empty(p)?);
            }
            Rule::method_call_other => {
                init = Some(lower_method_call_other(p)?);
            }
            Rule::map_new_call => {
                init = Some(lower_map_new_call(p));
            }
            Rule::list_new_call => {
                init = Some(lower_list_new_call(p));
            }
            Rule::spawn_call => {
                init = Some(lower_spawn_call(p)?);
            }
            Rule::host_call_try => {
                init = Some(lower_host_call_try(p)?);
            }
            Rule::user_call_try => {
                init = Some(lower_user_call_try(p)?);
            }
            Rule::ident_try => {
                init = Some(lower_ident_try(p)?);
            }
            Rule::host_call => {
                init = Some(lower_host_call(p)?);
            }
            Rule::user_call => {
                init = Some(lower_user_call(p)?);
            }
            // Rule::ident for init already handled above when name.is_some()
            _ => {}
        }
    }

    let name = name.ok_or_else(|| coded("E0006", "let missing binding name", Some(sp.clone())))?;
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(sp),
        ));
    }
    let ty = ty.ok_or_else(|| coded("E0006", "let missing type", Some(sp.clone())))?;
    let init = init.ok_or_else(|| coded("E0006", "let missing initializer", Some(sp)))?;

    Ok(Stmt::Let {
        mutable,
        name,
        ty,
        init,
    })
}

fn is_trivial_true_lit(e: &Expr) -> bool {
    matches!(e, Expr::LitBool(true))
}

/// E0214: `assert <recv>.len() == <same recv>.len()` (AST-equal receivers).
fn is_vacuous_len_eq_len(lhs: &Expr, rhs: &Expr) -> bool {
    match (lhs, rhs) {
        (
            Expr::MethodCall {
                receiver: r1,
                method: m1,
                args: a1,
            },
            Expr::MethodCall {
                receiver: r2,
                method: m2,
                args: a2,
            },
        ) if m1 == "len" && m2 == "len" && a1.is_empty() && a2.is_empty() => r1 == r2,
        _ => false,
    }
}

/// E0215: both assert sides syntactically identical (reflexive / vacuous cmp theater).
fn is_vacuous_identical_cmp(lhs: &Expr, rhs: &Expr) -> bool {
    lhs == rhs
}

fn lower_assert_side(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::assert_side => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty assert_side", Some(sp)))?;
            lower_assert_side(inner)
        }
        Rule::binary_expr => lower_binary_expr(pair),
        Rule::method_call_len => lower_method_call_len(pair),
        Rule::method_call_is_empty => lower_method_call_is_empty(pair),
        Rule::user_call => lower_user_call(pair),
        Rule::int_lit => {
            let n: i64 = pair.as_str().parse().map_err(|_| {
                coded(
                    "E0006",
                    format!("invalid int literal: {}", pair.as_str()),
                    Some(sp),
                )
            })?;
            Ok(Expr::LitInt(n))
        }
        Rule::bool_lit => {
            let v = match pair.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(coded(
                        "E0006",
                        format!("invalid bool literal: {other}"),
                        Some(sp),
                    ))
                }
            };
            Ok(Expr::LitBool(v))
        }
        Rule::ident => Ok(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        })),
        _ => Err(coded(
            "E0006",
            format!("expected assert_side, got {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

fn lower_assert_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let lhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "assert missing lhs", Some(sp.clone())))?;
    let rhs_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "assert missing rhs", Some(sp.clone())))?;
    let lhs = lower_assert_side(lhs_pair)?;
    let rhs = lower_assert_side(rhs_pair)?;
    // E0211: both sides must be non-trivial (not lone LitBool true)
    if is_trivial_true_lit(&lhs) || is_trivial_true_lit(&rhs) {
        return Err(coded("E0211", "assert requires evidence", Some(sp)));
    }
    // E0214: assert <recv>.len() == <same>.len()
    if is_vacuous_len_eq_len(&lhs, &rhs) {
        return Err(coded("E0214", "vacuous length assert", Some(sp)));
    }
    // E0215: assert <expr> == <same expr> (syntactic identity)
    if is_vacuous_identical_cmp(&lhs, &rhs) {
        return Err(coded("E0215", "vacuous comparison assert", Some(sp)));
    }
    Ok(Stmt::Assert { lhs, rhs })
}

/// ADR-038 / E0215: `assert <side> (<=|>=|!=) <side>` — identical sides → E0215;
/// unequal sides are OUT of assert surface v0 → E0006 (do not expand assert ops).
fn reject_assert_vacuous_rel(pair: pest::iterators::Pair<Rule>) -> ParseError {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let Some(lhs_pair) = inner.next() else {
        return coded("E0006", "assert missing lhs", Some(sp));
    };
    let Some(rhs_pair) = inner.next() else {
        return coded("E0006", "assert missing rhs", Some(sp));
    };
    let lhs = match lower_assert_side(lhs_pair) {
        Ok(e) => e,
        Err(e) => return e,
    };
    let rhs = match lower_assert_side(rhs_pair) {
        Ok(e) => e,
        Err(e) => return e,
    };
    if is_vacuous_identical_cmp(&lhs, &rhs) {
        return coded("E0215", "vacuous comparison assert", Some(sp));
    }
    coded("E0006", "assert comparison must use ==", Some(sp))
}

fn lower_test_item(pair: pest::iterators::Pair<Rule>) -> Result<Test, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "test missing name", Some(sp.clone())))?;
    let name = name_pair.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_pair)),
        ));
    }
    let mut body = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::test_body_elem => {
                let elem = p
                    .into_inner()
                    .next()
                    .ok_or_else(|| coded("E0006", "empty test_body_elem", Some(sp.clone())))?;
                match elem.as_rule() {
                    Rule::assert_stmt => body.push(lower_assert_stmt(elem)?),
                    Rule::assert_vacuous_len | Rule::assert_is_empty_taut => {
                        return Err(coded(
                            "E0214",
                            "vacuous length assert",
                            Some(span_of(&elem)),
                        ));
                    }
                    Rule::assert_vacuous_rel => {
                        return Err(reject_assert_vacuous_rel(elem));
                    }
                    Rule::assert_true => {
                        return Err(coded(
                            "E0211",
                            "assert requires evidence",
                            Some(span_of(&elem)),
                        ));
                    }
                    Rule::todo_bang | Rule::unimplemented_bang | Rule::todo_kw_stmt => {
                        return Err(coded(
                            "E0210",
                            "todo/unimplemented not allowed",
                            Some(span_of(&elem)),
                        ));
                    }
                    Rule::unsafe_block
                    | Rule::unsafe_fn_item
                    | Rule::unsafe_kw_stmt
                    | Rule::extern_block
                    | Rule::extern_fn_item
                    | Rule::extern_kw_stmt => {
                        return Err(coded(
                            "E0231",
                            "unsafe / FFI not allowed in ARITA",
                            Some(span_of(&elem)),
                        ));
                    }
                    other => {
                        return Err(coded(
                            "E0006",
                            format!("test body only allows assert, got {other:?}"),
                            Some(span_of(&elem)),
                        ));
                    }
                }
            }
            Rule::assert_stmt => body.push(lower_assert_stmt(p)?),
            Rule::assert_vacuous_len | Rule::assert_is_empty_taut => {
                return Err(coded("E0214", "vacuous length assert", Some(span_of(&p))));
            }
            Rule::assert_vacuous_rel => {
                return Err(reject_assert_vacuous_rel(p));
            }
            Rule::assert_true => {
                return Err(coded(
                    "E0211",
                    "assert requires evidence",
                    Some(span_of(&p)),
                ));
            }
            Rule::todo_bang | Rule::unimplemented_bang | Rule::todo_kw_stmt => {
                return Err(coded(
                    "E0210",
                    "todo/unimplemented not allowed",
                    Some(span_of(&p)),
                ));
            }
            Rule::unsafe_block
            | Rule::unsafe_fn_item
            | Rule::unsafe_kw_stmt
            | Rule::extern_block
            | Rule::extern_fn_item
            | Rule::extern_kw_stmt => {
                return Err(coded(
                    "E0231",
                    "unsafe / FFI not allowed in ARITA",
                    Some(span_of(&p)),
                ));
            }
            other => {
                return Err(coded(
                    "E0006",
                    format!("test body only allows assert, got {other:?}"),
                    Some(span_of(&p)),
                ));
            }
        }
    }
    if body.is_empty() {
        return Err(coded("E0212", "empty test not allowed", Some(sp)));
    }
    Ok(Test { name, body })
}

fn lower_contract_item(pair: pest::iterators::Pair<Rule>) -> Result<Contract, ParseError> {
    let sp = span_of(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| coded("E0006", "contract missing name", Some(sp.clone())))?;
    let name = name_pair.as_str().to_string();
    if is_keyword(&name) {
        return Err(coded(
            "E0007",
            format!("illegal ident / keyword misuse: `{name}`"),
            Some(span_of(&name_pair)),
        ));
    }

    let mut stdout_lines: Vec<String> = Vec::new();
    let mut reject_code: Option<String> = None;
    let mut target: Option<String> = None;

    for p in inner {
        let elem = if p.as_rule() == Rule::contract_body_elem {
            p.into_inner()
                .next()
                .ok_or_else(|| coded("E0006", "empty contract_body_elem", Some(sp.clone())))?
        } else {
            p
        };
        match elem.as_rule() {
            Rule::contract_target => {
                if target.is_some() {
                    return Err(coded(
                        "E0006",
                        "contract: at most one target",
                        Some(span_of(&elem)),
                    ));
                }
                let id = elem
                    .into_inner()
                    .next()
                    .ok_or_else(|| coded("E0006", "target missing ident", Some(sp.clone())))?;
                let tname = id.as_str().to_string();
                if is_keyword(&tname) {
                    return Err(coded(
                        "E0007",
                        format!("illegal ident / keyword misuse: `{tname}`"),
                        Some(span_of(&id)),
                    ));
                }
                target = Some(tname);
            }
            Rule::contract_expect_stdout => {
                if reject_code.is_some() {
                    return Err(coded(
                        "E0006",
                        "contract: exactly one expect kind (stdout or reject)",
                        Some(span_of(&elem)),
                    ));
                }
                let lit = elem.into_inner().next().ok_or_else(|| {
                    coded("E0006", "expect_stdout missing string", Some(sp.clone()))
                })?;
                match string_lit_to_expr(lit)? {
                    Expr::LitStr(s) => stdout_lines.push(s),
                    _ => {
                        return Err(coded(
                            "E0006",
                            "expect_stdout must be a string literal",
                            Some(sp.clone()),
                        ))
                    }
                }
            }
            Rule::contract_expect_reject => {
                if !stdout_lines.is_empty() || reject_code.is_some() {
                    return Err(coded(
                        "E0006",
                        "contract: exactly one expect kind (stdout or reject)",
                        Some(span_of(&elem)),
                    ));
                }
                let lit = elem.into_inner().next().ok_or_else(|| {
                    coded("E0006", "expect_reject missing string", Some(sp.clone()))
                })?;
                match string_lit_to_expr(lit)? {
                    Expr::LitStr(s) if !s.is_empty() => reject_code = Some(s),
                    Expr::LitStr(_) => {
                        return Err(coded(
                            "E0006",
                            "expect_reject must be a non-empty string",
                            Some(sp.clone()),
                        ))
                    }
                    _ => {
                        return Err(coded(
                            "E0006",
                            "expect_reject must be a string literal",
                            Some(sp.clone()),
                        ))
                    }
                }
            }
            other => {
                return Err(coded(
                    "E0006",
                    format!(
                        "contract body only allows target|expect_stdout|expect_reject, got {other:?}"
                    ),
                    Some(span_of(&elem)),
                ));
            }
        }
    }

    let expect = if let Some(code) = reject_code {
        ContractExpect::Reject(code)
    } else if !stdout_lines.is_empty() {
        ContractExpect::Stdout(stdout_lines)
    } else {
        return Err(coded(
            "E0006",
            "contract must have expect_stdout or expect_reject",
            Some(sp),
        ));
    };

    Ok(Contract {
        name,
        expect,
        target,
    })
}

fn lower_try_postfix(inner: Expr) -> Expr {
    Expr::Try {
        inner: Box::new(inner),
    }
}

fn lower_host_call_try(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = None;
    for p in pair.into_inner() {
        if p.as_rule() == Rule::host_call {
            inner = Some(lower_host_call(p)?);
        }
    }
    let inner = inner.ok_or_else(|| coded("E0006", "host_call_try missing host_call", None))?;
    Ok(lower_try_postfix(inner))
}

fn lower_user_call_try(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = None;
    for p in pair.into_inner() {
        if p.as_rule() == Rule::user_call {
            inner = Some(lower_user_call(p)?);
        }
    }
    let inner = inner.ok_or_else(|| coded("E0006", "user_call_try missing user_call", None))?;
    Ok(lower_try_postfix(inner))
}

fn lower_ident_try(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut name = None;
    for p in pair.into_inner() {
        if p.as_rule() == Rule::ident {
            name = Some(p.as_str().to_string());
        }
    }
    let name = name.ok_or_else(|| coded("E0006", "ident_try missing ident", None))?;
    Ok(lower_try_postfix(Expr::Path(Path {
        segments: vec![name],
    })))
}

fn lower_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let sp = span_of(&pair);
    match pair.as_rule() {
        Rule::stmt => {
            let inner = pair.into_inner().next().ok_or_else(|| {
                coded("E0006", "construct outside F1.1: empty statement", Some(sp))
            })?;
            lower_stmt(inner)
        }
        Rule::todo_bang | Rule::unimplemented_bang | Rule::todo_kw_stmt => {
            Err(coded("E0210", "todo/unimplemented not allowed", Some(sp)))
        }
        Rule::unsafe_block
        | Rule::unsafe_fn_item
        | Rule::unsafe_kw_stmt
        | Rule::extern_block
        | Rule::extern_fn_item
        | Rule::extern_kw_stmt => Err(coded(
            "E0231",
            "unsafe / FFI not allowed in ARITA",
            Some(sp),
        )),
        Rule::let_stmt => lower_let_stmt(pair),
        Rule::if_let_stmt => lower_if_let_stmt(pair),
        Rule::if_stmt => lower_if_stmt(pair),
        Rule::while_let_stmt => lower_while_let_stmt(pair),
        Rule::while_stmt => lower_while_stmt(pair),
        Rule::break_stmt => Ok(Stmt::Break),
        Rule::continue_stmt => Ok(Stmt::Continue),
        Rule::match_stmt => lower_match_stmt(pair),
        // ADR-282: Map vs Vec/String se decide en HIR (E0314 residual allí, mismo texto)
        Rule::index_assign_stmt => lower_index_assign_stmt(pair),
        Rule::assign_stmt => lower_assign_stmt(pair),
        Rule::method_call_push => lower_method_call_push(pair),
        Rule::host_call_try => Ok(Stmt::Expr(lower_host_call_try(pair)?)),
        Rule::ok_expr => Ok(Stmt::Expr(lower_ok_expr(pair)?)),
        Rule::err_expr => Ok(Stmt::Expr(lower_err_expr(pair)?)),
        Rule::some_expr => Ok(Stmt::Expr(lower_some_expr(pair)?)),
        Rule::none_expr => Ok(Stmt::Expr(Expr::None)),
        Rule::int_lit => {
            let n: i64 = pair
                .as_str()
                .parse()
                .map_err(|_| coded("E0006", "bad int_lit stmt", None))?;
            Ok(Stmt::Expr(Expr::LitInt(n)))
        }
        Rule::host_call => Ok(Stmt::Expr(lower_host_call(pair)?)),
        Rule::method_call_other => Ok(Stmt::Expr(lower_method_call_other(pair)?)),
        Rule::print_ok => {
            let mut inner = pair.into_inner();
            let arg_pair = inner.next().ok_or_else(|| {
                coded(
                    "E0010",
                    "print with args ≠ one LitStr|Int|Bool|Path|len|is_empty|call",
                    Some(sp.clone()),
                )
            })?;
            let arg = lower_print_arg(arg_pair)?;
            Ok(Stmt::Expr(Expr::Call(Call {
                callee: "print".into(),
                args: vec![arg],
            })))
        }
        Rule::print_bad => Err(coded(
            "E0010",
            "print with args ≠ one LitStr|Int|Bool|Path|len|is_empty|call",
            Some(sp),
        )),
        Rule::await_expr => Ok(Stmt::Expr(lower_await_expr(pair)?)),
        Rule::async_illegal_stmt => Err(coded("E0241", "async feature not allowed here", Some(sp))),
        Rule::user_call_try => Ok(Stmt::Expr(lower_user_call_try(pair)?)),
        Rule::user_call => Ok(Stmt::Expr(lower_user_call(pair)?)),
        Rule::borrow_kw_expr => Ok(Stmt::Expr(lower_borrow_kw_expr(pair)?)),
        Rule::borrow_expr => Ok(Stmt::Expr(lower_borrow_expr(pair)?)),
        Rule::binary_expr => Ok(Stmt::Expr(lower_binary_expr(pair)?)),
        Rule::ident_try => Ok(Stmt::Expr(lower_ident_try(pair)?)),
        Rule::ident => Ok(Stmt::Expr(Expr::Path(Path {
            segments: vec![pair.as_str().to_string()],
        }))),
        Rule::call_other => {
            let callee = pair
                .clone()
                .into_inner()
                .next()
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            Err(coded(
                "E0006",
                format!("construct outside F1.1: call `{callee}`"),
                Some(sp),
            ))
        }
        Rule::illegal_stmt => Err(coded(
            "E0006",
            format!("construct outside F1.1: {}", pair.as_str().trim()),
            Some(sp),
        )),
        _ => Err(coded(
            "E0006",
            format!("construct outside F1.1: {:?}", pair.as_rule()),
            Some(sp),
        )),
    }
}

/// ADR-029 / E0261: surface must never name external crate paths (`tokio::…`).
/// Allowed `::` forms in v0: `Vec::new` and `Enum::Variant` (ADR-234). Multi-segment crate paths rejected.
fn reject_external_crate_paths(source: &str) -> Result<(), ParseError> {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut in_str = false;
    let mut line_comment = false;
    while i < bytes.len() {
        let c = bytes[i];
        if line_comment {
            if c == b'\n' {
                line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            line_comment = true;
            i += 2;
            continue;
        }
        if c == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if (c as char).is_ascii_alphabetic() || c == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len()
                && ((bytes[i] as char).is_ascii_alphanumeric() || bytes[i] == b'_')
            {
                i += 1;
            }
            if i + 1 < bytes.len() && bytes[i] == b':' && bytes[i + 1] == b':' {
                let name = &source[start..i];
                let after = &source[i + 2..];
                // Allowlist: Vec::new (ADR-006 / F2) + Enum::Variant unit paths (ADR-234).
                // Multi-segment crate paths (`tokio::runtime::...`) stay rejected.
                let rest = after;
                let mut j = 0usize;
                let rb = rest.as_bytes();
                if j < rb.len() && ((rb[j] as char).is_ascii_alphabetic() || rb[j] == b'_') {
                    j += 1;
                    while j < rb.len() && ((rb[j] as char).is_ascii_alphanumeric() || rb[j] == b'_')
                    {
                        j += 1;
                    }
                }
                let variant = &rest[..j];
                let after_var = rest.get(j..).unwrap_or("");
                let more_path = after_var.starts_with("::");
                // Allow: Vec/Map/List::new; Enum::Variant (ADR-234); local `use mod::item`
                // (e.g. util::double). Deny known external crate roots → E0261
                // (use tokio::spawn must not look like Enum::Variant / E0404).
                let external_crate = matches!(
                    name,
                    "tokio"
                        | "std"
                        | "core"
                        | "alloc"
                        | "serde"
                        | "reqwest"
                        | "hyper"
                        | "axum"
                        | "futures"
                        | "async_std"
                        | "mio"
                        | "tracing"
                        | "log"
                        | "anyhow"
                        | "thiserror"
                        | "clap"
                        | "regex"
                        | "rand"
                        | "chrono"
                        | "uuid"
                        | "bytes"
                        | "http"
                        | "tower"
                        | "tonic"
                );
                let allowed = ((name == "Vec" || name == "Map" || name == "List")
                    && rest.starts_with("new"))
                    || (!variant.is_empty()
                        && !more_path
                        && !after_var.starts_with('(')
                        && !external_crate);
                if !allowed {
                    return Err(coded(
                        "E0261",
                        "external crate path not allowed in surface",
                        Some(Span {
                            start,
                            end: (i + 2).min(source.len()),
                        }),
                    ));
                }
            }
            continue;
        }
        i += 1;
    }
    Ok(())
}

pub fn parse(source: &str) -> Result<Module, ParseError> {
    if source.trim().is_empty() || comments_and_ws_only(source) {
        return Err(coded(
            "E0008",
            "empty file / comments only",
            Some(Span {
                start: 0,
                end: source.len(),
            }),
        ));
    }

    reject_external_crate_paths(source)?;

    let pairs = match AritaParser::parse(Rule::file, source) {
        Ok(p) => p,
        Err(e) => return Err(classify_pest_error(source, &e)),
    };

    let file = pairs.into_iter().next().ok_or_else(|| {
        coded(
            "E0008",
            "empty file / comments only",
            Some(Span {
                start: 0,
                end: source.len(),
            }),
        )
    })?;

    let mut modules: Vec<(String, Span)> = Vec::new();
    let mut functions: Vec<(FunctionInfo, Span)> = Vec::new();
    let mut tests: Vec<Test> = Vec::new();
    let mut contracts: Vec<Contract> = Vec::new();
    let mut records: Vec<RecordDef> = Vec::new();
    let mut enums: Vec<EnumDef> = Vec::new();
    let mut uses: Vec<UseDecl> = Vec::new();
    // ADR-283: first shape-gate error deferred to HIR (fn name, exact parser error).
    let mut deferred: Option<(String, ParseError)> = None;

    for item in file.into_inner() {
        // pest wraps alternatives under `item`
        let pair = if item.as_rule() == Rule::item {
            match item.into_inner().next() {
                Some(inner) => inner,
                None => continue,
            }
        } else {
            item
        };

        match pair.as_rule() {
            Rule::module_decl => {
                let sp = span_of(&pair);
                let mut inner = pair.into_inner();
                let id = inner
                    .next()
                    .ok_or_else(|| coded("E0002", "missing module <ident>", Some(sp.clone())))?;
                let name = id.as_str().to_string();
                if is_keyword(&name) {
                    return Err(coded(
                        "E0007",
                        format!("illegal ident / keyword misuse: `{name}`"),
                        Some(span_of(&id)),
                    ));
                }
                modules.push((name, sp));
            }
            Rule::use_item => {
                uses.push(lower_use_item(pair)?);
            }
            Rule::record_item => {
                records.push(lower_record_item(pair)?);
            }
            Rule::enum_item => {
                enums.push(lower_enum_item(pair)?);
            }
            Rule::unsafe_fn_item
            | Rule::extern_fn_item
            | Rule::extern_block
            | Rule::unsafe_block
            | Rule::unsafe_kw_stmt
            | Rule::extern_kw_stmt => {
                return Err(coded(
                    "E0231",
                    "unsafe / FFI not allowed in ARITA",
                    Some(span_of(&pair)),
                ));
            }
            Rule::async_function => {
                let sp = span_of(&pair);
                let info = lower_async_function(pair)?;
                functions.push((info, sp));
            }
            Rule::async_illegal_top => {
                return Err(coded(
                    "E0241",
                    "async feature not allowed here",
                    Some(span_of(&pair)),
                ));
            }
            Rule::function => {
                let sp = span_of(&pair);
                let info = lower_function(pair)?;
                functions.push((info, sp));
            }
            Rule::test_item => {
                tests.push(lower_test_item(pair)?);
            }
            Rule::contract_item => {
                contracts.push(lower_contract_item(pair)?);
            }
            Rule::scenario_item => {
                contracts.push(lower_scenario_item(pair)?);
            }
            Rule::illegal_top => {
                return Err(coded(
                    "E0006",
                    format!(
                        "construct outside F1.1: {}",
                        pair.as_str().lines().next().unwrap_or("").trim()
                    ),
                    Some(span_of(&pair)),
                ));
            }
            Rule::EOI => {}
            other => {
                return Err(coded(
                    "E0006",
                    format!("construct outside F1.1: unexpected {other:?}"),
                    Some(span_of(&pair)),
                ));
            }
        }
    }

    if modules.is_empty() {
        return Err(coded(
            "E0002",
            "missing module <ident>",
            Some(Span {
                start: 0,
                end: source.len(),
            }),
        ));
    }
    if modules.len() > 1 {
        return Err(coded(
            "E0009",
            "more than one module",
            Some(modules[1].1.clone()),
        ));
    }
    if functions.is_empty() {
        return Err(coded(
            "E0003",
            "missing fn main or name ≠ main",
            Some(Span {
                start: 0,
                end: source.len(),
            }),
        ));
    }

    let main_positions: Vec<usize> = functions
        .iter()
        .enumerate()
        .filter(|(_, (info, _))| info.name == "main")
        .map(|(i, _)| i)
        .collect();
    if main_positions.len() > 1 {
        return Err(coded(
            "E0006",
            "more than one fn main",
            Some(functions[main_positions[1]].1.clone()),
        ));
    }
    // ADR-254/255: lib modules may omit `main` when ≥1 non-main fn exists.
    if !main_positions.is_empty() {
        let main_idx = main_positions[0];
        let (main_info, main_span) = &functions[main_idx];

        match &main_info.ret {
            RetKind::OkIoUnit => {}
            RetKind::Missing => {
                return Err(coded(
                    "E0004",
                    "main return type ≠ Io<()>",
                    Some(main_span.clone()),
                ));
            }
            RetKind::OkInt | RetKind::OkType(_) | RetKind::Other(_) => {
                let sp = match &main_info.ret {
                    RetKind::Other(sp) => sp.clone(),
                    _ => main_span.clone(),
                };
                return Err(coded("E0004", "main return type ≠ Io<()>", Some(sp)));
            }
        }
        if !main_info.params.is_empty() {
            return Err(coded(
                "E0006",
                "main must have empty params in this slice",
                Some(main_span.clone()),
            ));
        }

        fn count_usable_prints(stmts: &[Stmt]) -> usize {
            let mut n = 0;
            for s in stmts {
                match s {
                    Stmt::Expr(Expr::Call(c))
                        if c.callee == "print"
                            && c.args.len() == 1
                            && matches!(
                                c.args[0],
                                Expr::LitStr(_)
                                    | Expr::LitInt(_)
                                    | Expr::LitBool(_)
                                    | Expr::Path(_)
                                    | Expr::MethodCall { .. }
                                    | Expr::FieldAccess { .. }
                                    | Expr::Call(_)
                            ) =>
                    {
                        n += 1;
                    }
                    Stmt::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        n += count_usable_prints(then_body);
                        if let Some(eb) = else_body {
                            n += count_usable_prints(eb);
                        }
                    }
                    Stmt::IfLet {
                        then_body,
                        else_body,
                        ..
                    } => {
                        n += count_usable_prints(then_body);
                        if let Some(eb) = else_body {
                            n += count_usable_prints(eb);
                        }
                    }
                    Stmt::WhileLet { body, .. } => {
                        n += count_usable_prints(body);
                    }
                    Stmt::While { body, .. } => {
                        n += count_usable_prints(body);
                    }
                    Stmt::Match { arms, .. } => {
                        for arm in arms {
                            n += count_usable_prints(&arm.body);
                        }
                    }
                    _ => {}
                }
            }
            n
        }
        let usable_prints = count_usable_prints(&main_info.body);
        fn has_await(stmts: &[Stmt]) -> bool {
            for s in stmts {
                match s {
                    Stmt::Expr(Expr::Await { .. }) => return true,
                    Stmt::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        if has_await(then_body) {
                            return true;
                        }
                        if let Some(eb) = else_body {
                            if has_await(eb) {
                                return true;
                            }
                        }
                    }
                    Stmt::IfLet {
                        then_body,
                        else_body,
                        ..
                    } => {
                        if has_await(then_body) {
                            return true;
                        }
                        if let Some(eb) = else_body {
                            if has_await(eb) {
                                return true;
                            }
                        }
                    }
                    Stmt::WhileLet { body, .. } => {
                        if has_await(body) {
                            return true;
                        }
                    }
                    Stmt::While { body, .. } => {
                        if has_await(body) {
                            return true;
                        }
                    }
                    Stmt::Match { arms, .. } => {
                        for arm in arms {
                            if has_await(&arm.body) {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        // ADR-027: async main may effect via `await` helpers that print.
        if usable_prints == 0 && !(main_info.is_async && has_await(&main_info.body)) {
            let e = coded("E0001", "no usable print in main", Some(main_span.clone()));
            // ADR-283: HIR decides; its non-Map index-assign error wins, else this exact E0001.
            if body_contains_index_assign(&main_info.body) {
                deferred = Some(("main".to_string(), e));
            } else {
                return Err(e);
            }
        }
    } // end main-present gate

    // Non-main fns:
    // - Int helpers (F2): params Int, ret Int; body Expr binary/path/lit/call (no print)
    // - Io<()> helpers (ADR-020 contract target): empty params; print allowed; non-empty body
    // ADR-283: run as a closure so that, once a shape error was deferred, any later gate error
    // is replaced by that first (deferred) error — exactly what the parser returned before.
    let non_main_gate = |deferred: &mut Option<(String, ParseError)>| -> Result<(), ParseError> {
        for (info, fn_span) in &functions {
            if info.name == "main" {
                continue;
            }
            match &info.ret {
                RetKind::OkIoUnit => {
                    if !info.params.is_empty() {
                        return Err(coded(
                            "E0006",
                            format!(
                                "non-main Io<()> fn `{}` must have empty params (MVP target shape)",
                                info.name
                            ),
                            Some(fn_span.clone()),
                        ));
                    }
                    if info.body.is_empty() {
                        return Err(coded(
                            "E0213",
                            "stub function body not allowed",
                            Some(fn_span.clone()),
                        ));
                    }
                    // Body: allow print and the same stmt surface as main-ish (Expr/Let/If/While/…)
                    for stmt in &info.body {
                        match stmt {
                            Stmt::Expr(_)
                            | Stmt::Let { .. }
                            | Stmt::If { .. }
                            | Stmt::IfLet { .. }
                            | Stmt::WhileLet { .. }
                            | Stmt::While { .. }
                            | Stmt::Assign { .. }
                            | Stmt::IndexAssign { .. }
                            | Stmt::Match { .. }
                            | Stmt::Break
                            | Stmt::Continue => {}
                            Stmt::Assert { .. } => {
                                return Err(coded(
                                    "E0006",
                                    format!(
                                        "non-main Io<()> fn `{}` body may not use assert",
                                        info.name
                                    ),
                                    Some(fn_span.clone()),
                                ));
                            }
                        }
                    }
                }
                RetKind::OkInt => {
                    // E0213: valued fn (ret ≠ IoUnit) with empty body
                    if info.body.is_empty() {
                        return Err(coded(
                            "E0213",
                            "stub function body not allowed",
                            Some(fn_span.clone()),
                        ));
                    }
                    for p in &info.params {
                        if p.ty != Type::Int {
                            return Err(coded(
                                "E0006",
                                format!(
                                    "non-main fn `{}` params must be Int in this slice",
                                    info.name
                                ),
                                Some(fn_span.clone()),
                            ));
                        }
                    }
                    let shape_err = || {
                        coded(
                            "E0006",
                            format!(
                            "non-main fn `{}` body must be Expr binary/path/lit/call in this slice",
                            info.name
                        ),
                            Some(fn_span.clone()),
                        )
                    };
                    // ADR-283: an IndexAssign as the ONLY offending stmt defers this E0006 to HIR.
                    let mut ia_seen = false;
                    for stmt in &info.body {
                        match stmt {
                            Stmt::Expr(Expr::Call(c)) if c.callee == "print" => {
                                if ia_seen {
                                    return Err(shape_err());
                                }
                                return Err(coded(
                                    "E0006",
                                    format!(
                                        "non-main fn `{}` body may not use print in this slice",
                                        info.name
                                    ),
                                    Some(fn_span.clone()),
                                ));
                            }
                            Stmt::Expr(Expr::Binary { .. })
                            | Stmt::Expr(Expr::Path(_))
                            | Stmt::Expr(Expr::LitInt(_))
                            | Stmt::Expr(Expr::Call(_))
                            | Stmt::Expr(Expr::Borrow { .. })
                            | Stmt::Expr(Expr::Try { .. })
                            | Stmt::Expr(Expr::Ok(_))
                            | Stmt::Expr(Expr::Err(_))
                            | Stmt::Expr(Expr::Some(_))
                            | Stmt::Expr(Expr::None)
                            | Stmt::Let { .. }
                            | Stmt::Match { .. } => {}
                            Stmt::IndexAssign { .. } => ia_seen = true,
                            _ => {
                                return Err(shape_err());
                            }
                        }
                    }
                    if ia_seen {
                        if deferred.is_some() {
                            return Err(shape_err());
                        }
                        *deferred = Some((info.name.clone(), shape_err()));
                    }
                }
                RetKind::OkType(_) => {
                    // ADR-245/277 Result/Named helpers
                    if info.body.is_empty() {
                        return Err(coded(
                            "E0213",
                            "stub function body not allowed",
                            Some(fn_span.clone()),
                        ));
                    }
                    for stmt in &info.body {
                        match stmt {
                            Stmt::Expr(_)
                            | Stmt::Let { .. }
                            | Stmt::If { .. }
                            | Stmt::IfLet { .. }
                            | Stmt::WhileLet { .. }
                            | Stmt::While { .. }
                            | Stmt::Assign { .. }
                            | Stmt::IndexAssign { .. }
                            | Stmt::Match { .. }
                            | Stmt::Break
                            | Stmt::Continue => {}
                            Stmt::Assert { .. } => {
                                return Err(coded(
                                    "E0006",
                                    format!(
                                        "non-main Result/Named fn `{}` body may not use assert",
                                        info.name
                                    ),
                                    Some(fn_span.clone()),
                                ));
                            }
                        }
                    }
                }
                RetKind::Missing => {
                    return Err(coded(
                        "E0006",
                        format!(
                        "non-main fn `{}` must return Int, Io<()>, or Result/Named in this slice",
                        info.name
                    ),
                        Some(fn_span.clone()),
                    ));
                }
                RetKind::Other(sp) => {
                    return Err(coded(
                        "E0006",
                        format!(
                        "non-main fn `{}` must return Int, Io<()>, or Result/Named in this slice",
                        info.name
                    ),
                        Some(sp.clone()),
                    ));
                }
            }
        }
        Ok(())
    };
    if let Err(e) = non_main_gate(&mut deferred) {
        return Err(match deferred {
            Some((_, first)) => first,
            None => e,
        });
    }

    let out_fns: Vec<Function> = functions
        .iter()
        .map(|(info, _)| {
            let ret_ty = match &info.ret {
                RetKind::OkIoUnit => Type::IoUnit,
                RetKind::OkInt => Type::Int,
                RetKind::OkType(ty) => ty.clone(),
                RetKind::Missing | RetKind::Other(_) => Type::IoUnit,
            };
            Function {
                name: info.name.clone(),
                params: info.params.clone(),
                ret_ty,
                body: info.body.clone(),
                is_async: info.is_async,
                is_pub: info.is_pub,
            }
        })
        .collect();

    // ADR-283: hand the deferred shape error to HIR as the fn's synthetic first statement.
    let mut out_fns = out_fns;
    if let Some((fn_name, err)) = deferred {
        let marker = deferred_shape_marker(&err).ok_or_else(|| err.clone())?;
        match out_fns.iter_mut().find(|f| f.name == fn_name) {
            Some(f) => f.body.insert(0, marker),
            None => return Err(err),
        }
    }

    Ok(Module {
        name: modules[0].0.clone(),
        functions: out_fns,
        tests,
        contracts,
        records,
        enums,
        uses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code_of(err: ParseError) -> &'static str {
        match err {
            ParseError::Coded { code, .. } => code,
            ParseError::Message(m) => {
                if m.starts_with("E0001") {
                    "E0001"
                } else {
                    "Message"
                }
            }
        }
    }

    #[test]
    fn parses_hello() {
        let src = r#"
module hello
fn main() -> Io<()> {
  print("hello")
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "hello");
        assert_eq!(m.functions.len(), 1);
        assert!(m.tests.is_empty());
        assert_eq!(m.functions[0].name, "main");
        assert!(m.functions[0].params.is_empty());
        assert_eq!(m.functions[0].ret_ty, Type::IoUnit);
        match &m.functions[0].body[0] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(c.args, vec![Expr::LitStr("hello".into())]);
            }
            _ => unreachable!("expected print call"),
        }
    }

    #[test]
    fn parses_hello_with_comment() {
        let src = r#"
// greeting
module hello
fn main() -> Io<()> {
  print("hello") // ok
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "hello");
    }

    #[test]
    fn e0001_empty_main() {
        let src = r#"
module hello
fn main() -> Io<()> {
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0001");
    }

    #[test]
    fn e0002_missing_module() {
        let src = r#"
fn main() -> Io<()> {
  print("hello")
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0002");
    }

    #[test]
    fn e0003_no_functions() {
        let src = r#"
module hello
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0003");
    }

    /// ADR-254/255: non-main-only module is a legal lib surface (no E0003).
    #[test]
    fn lib_module_without_main_ok() {
        let src = r#"
module hello
fn foo() -> Io<()> {
  print("hello")
}
"#;
        let m = parse(src).expect("lib module without main");
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "foo");
    }

    #[test]
    fn e0004_bad_return_type() {
        let src = r#"
module hello
fn main() -> () {
  print("hello")
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0004");
    }

    #[test]
    fn e0005_bad_escape() {
        let src = r#"
module hello
fn main() -> Io<()> {
  print("bad \q escape")
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0005");
    }

    #[test]
    fn e0006_let_outside_f11() {
        // `let` is F2 IN; use `spec` (still OUT) as the E0006 probe.
        let src = r#"
module hello
fn main() -> Io<()> {
  spec foo
  print("hello")
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0006");
    }

    #[test]
    fn parses_let_int_and_print_path() {
        let src = r#"
module demo
fn main() -> Io<()> {
  let n: Int = 42
  let mut m: Int = 7
  print(n)
  print(m)
  print(3)
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "demo");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 5);
        match &body[0] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(!mutable);
                assert_eq!(name, "n");
                assert_eq!(*ty, Type::Int);
                assert_eq!(*init, Expr::LitInt(42));
            }
            other => unreachable!("expected Let, got {other:?}"),
        }
        match &body[1] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(*mutable);
                assert_eq!(name, "m");
                assert_eq!(*ty, Type::Int);
                assert_eq!(*init, Expr::LitInt(7));
            }
            other => unreachable!("expected Let mut, got {other:?}"),
        }
        match &body[2] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(
                    c.args,
                    vec![Expr::Path(Path {
                        segments: vec!["n".into()]
                    })]
                );
            }
            other => unreachable!("expected print(n), got {other:?}"),
        }
        match &body[4] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.args, vec![Expr::LitInt(3)]);
            }
            other => unreachable!("expected print(3), got {other:?}"),
        }
    }

    #[test]
    fn parses_string_len_and_is_empty() {
        let src = r#"
module demo
fn main() -> Io<()> {
  let s: String = "hi"
  print(s.len())
  print(s.is_empty())
}
"#;
        let m = parse(src).expect("parse");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 3);
        match &body[1] {
            Stmt::Expr(Expr::Call(c)) => match &c.args[0] {
                Expr::MethodCall { method, args, .. } => {
                    assert_eq!(method, "len");
                    assert!(args.is_empty());
                }
                other => unreachable!("expected len, got {other:?}"),
            },
            other => unreachable!("expected print len, got {other:?}"),
        }
        match &body[2] {
            Stmt::Expr(Expr::Call(c)) => match &c.args[0] {
                Expr::MethodCall { method, args, .. } => {
                    assert_eq!(method, "is_empty");
                    assert!(args.is_empty());
                }
                other => unreachable!("expected is_empty, got {other:?}"),
            },
            other => unreachable!("expected print is_empty, got {other:?}"),
        }
    }

    #[test]
    fn parses_vec_push_len() {
        let src = r#"
module vec_len
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  print(v.len())
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "vec_len");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 4);
        match &body[0] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(*mutable);
                assert_eq!(name, "v");
                assert_eq!(*ty, Type::Vec(Box::new(Type::Int)));
                assert_eq!(
                    *init,
                    Expr::Call(Call {
                        callee: "Vec::new".into(),
                        args: vec![],
                    })
                );
            }
            other => unreachable!("expected Let mut Vec, got {other:?}"),
        }
        match &body[1] {
            Stmt::Expr(Expr::MethodCall {
                receiver,
                method,
                args,
            }) => {
                assert_eq!(method, "push");
                assert_eq!(
                    **receiver,
                    Expr::Path(Path {
                        segments: vec!["v".into()]
                    })
                );
                assert_eq!(*args, vec![Expr::LitInt(1)]);
            }
            other => unreachable!("expected v.push(1), got {other:?}"),
        }
        match &body[2] {
            Stmt::Expr(Expr::MethodCall {
                receiver,
                method,
                args,
            }) => {
                assert_eq!(method, "push");
                assert_eq!(
                    **receiver,
                    Expr::Path(Path {
                        segments: vec!["v".into()]
                    })
                );
                assert_eq!(*args, vec![Expr::LitInt(2)]);
            }
            other => unreachable!("expected v.push(2), got {other:?}"),
        }
        match &body[3] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(c.args.len(), 1);
                match &c.args[0] {
                    Expr::MethodCall {
                        receiver,
                        method,
                        args,
                    } => {
                        assert_eq!(method, "len");
                        assert!(args.is_empty());
                        assert_eq!(
                            **receiver,
                            Expr::Path(Path {
                                segments: vec!["v".into()]
                            })
                        );
                    }
                    other => unreachable!("expected MethodCall len, got {other:?}"),
                }
            }
            other => unreachable!("expected print(v.len()), got {other:?}"),
        }
    }

    #[test]
    fn parses_string_let_print() {
        let src = r#"
module str_hi
fn main() -> Io<()> {
  let s: String = "hi"
  print(s)
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "str_hi");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 2);
        match &body[0] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(!mutable);
                assert_eq!(name, "s");
                assert_eq!(*ty, Type::String);
                assert_eq!(*init, Expr::LitStr("hi".into()));
            }
            other => unreachable!("expected Let String, got {other:?}"),
        }
        match &body[1] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(
                    c.args,
                    vec![Expr::Path(Path {
                        segments: vec!["s".into()]
                    })]
                );
            }
            other => unreachable!("expected print(s), got {other:?}"),
        }

        // let mut String is also accepted (cheap; mut_kw already optional)
        let src_mut = r#"
module str_mut
fn main() -> Io<()> {
  let mut s: String = "hi"
  print(s)
}
"#;
        let m2 = parse(src_mut).expect("parse mut");
        match &m2.functions[0].body[0] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(*mutable);
                assert_eq!(name, "s");
                assert_eq!(*ty, Type::String);
                assert_eq!(*init, Expr::LitStr("hi".into()));
            }
            other => unreachable!("expected Let mut String, got {other:?}"),
        }
    }

    #[test]
    fn e0007_keyword_module_name() {
        let src = r#"
module fn
fn main() -> Io<()> {
  print("hello")
}
"#;
        let err = parse(src).unwrap_err();
        let c = code_of(err);
        assert!(c == "E0007" || c == "E0006" || c == "E0002", "got {c}");
    }

    #[test]
    fn e0008_empty() {
        assert_eq!(code_of(parse("").unwrap_err()), "E0008");
        assert_eq!(code_of(parse("// only\n\n").unwrap_err()), "E0008");
    }

    #[test]
    fn e0009_two_modules() {
        let src = r#"
module a
module b
fn main() -> Io<()> {
  print("hello")
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0009");
    }

    #[test]
    fn e0010_print_no_args() {
        let src = r#"
module hello
fn main() -> Io<()> {
  print()
}
"#;
        assert_eq!(code_of(parse(src).unwrap_err()), "E0010");
    }

    #[test]
    fn parses_int_binary() {
        let src = r#"
module arith
fn main() -> Io<()> {
  let a: Int = 2
  let b: Int = 3
  let c: Int = a + b
  print(c)
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "arith");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 4);
        match &body[2] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(!mutable);
                assert_eq!(name, "c");
                assert_eq!(*ty, Type::Int);
                assert_eq!(
                    *init,
                    Expr::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::Path(Path {
                            segments: vec!["a".into()]
                        })),
                        rhs: Box::new(Expr::Path(Path {
                            segments: vec!["b".into()]
                        })),
                    }
                );
            }
            other => unreachable!("expected Let c = a + b, got {other:?}"),
        }

        // lit + lit
        let src2 = r#"
module litadd
fn main() -> Io<()> {
  let n: Int = 2 + 3
  print(n)
}
"#;
        let m2 = parse(src2).expect("parse lit+lit");
        match &m2.functions[0].body[0] {
            Stmt::Let { init, .. } => {
                assert_eq!(
                    *init,
                    Expr::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::LitInt(2)),
                        rhs: Box::new(Expr::LitInt(3)),
                    }
                );
            }
            other => unreachable!("expected lit+lit, got {other:?}"),
        }

        // path + lit and other ops
        for (src_op, op) in [
            ("a - 1", BinOp::Sub),
            ("a * 2", BinOp::Mul),
            ("a / 2", BinOp::Div),
            ("a % 2", BinOp::Rem),
        ] {
            let src_op_full = format!(
                r#"
module ops
fn main() -> Io<()> {{
  let a: Int = 10
  let n: Int = {src_op}
  print(n)
}}
"#
            );
            let m_op = parse(&src_op_full).unwrap_or_else(|e| unreachable!("parse {src_op}: {e}"));
            match &m_op.functions[0].body[1] {
                Stmt::Let { init, .. } => match init {
                    Expr::Binary { op: got, .. } => assert_eq!(*got, op, "op for {src_op}"),
                    other => unreachable!("expected Binary for {src_op}, got {other:?}"),
                },
                other => unreachable!("expected Let for {src_op}, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_bool_let_print() {
        let src = r#"
module flag
fn main() -> Io<()> {
  let flag: Bool = true
  print(flag)
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "flag");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 2);
        match &body[0] {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                assert!(!mutable);
                assert_eq!(name, "flag");
                assert_eq!(*ty, Type::Bool);
                assert_eq!(*init, Expr::LitBool(true));
            }
            other => unreachable!("expected Let Bool true, got {other:?}"),
        }
        match &body[1] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(
                    c.args,
                    vec![Expr::Path(Path {
                        segments: vec!["flag".into()]
                    })]
                );
            }
            other => unreachable!("expected print(flag), got {other:?}"),
        }

        let src_f = r#"
module flag_f
fn main() -> Io<()> {
  let flag: Bool = false
  print(flag)
}
"#;
        let m2 = parse(src_f).expect("parse false");
        match &m2.functions[0].body[0] {
            Stmt::Let { init, ty, .. } => {
                assert_eq!(*ty, Type::Bool);
                assert_eq!(*init, Expr::LitBool(false));
            }
            other => unreachable!("expected Let Bool false, got {other:?}"),
        }

        // print(true) direct lit
        let src_p = r#"
module pb
fn main() -> Io<()> {
  print(true)
}
"#;
        let m3 = parse(src_p).expect("parse print true");
        match &m3.functions[0].body[0] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.args, vec![Expr::LitBool(true)]);
            }
            other => unreachable!("expected print(true), got {other:?}"),
        }
    }

    #[test]
    fn parses_fn_call_double() {
        let src = r#"
module fn_call

fn double(x: Int) -> Int {
  x + x
}

fn main() -> Io<()> {
  print(double(21))
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "fn_call");
        assert_eq!(m.functions.len(), 2);
        assert_eq!(m.functions[0].name, "double");
        assert_eq!(
            m.functions[0].params,
            vec![FnParam {
                name: "x".into(),
                ty: Type::Int,
            }]
        );
        assert_eq!(m.functions[0].ret_ty, Type::Int);
        assert_eq!(m.functions[0].body.len(), 1);
        match &m.functions[0].body[0] {
            Stmt::Expr(Expr::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
            }) => {
                assert_eq!(
                    **lhs,
                    Expr::Path(Path {
                        segments: vec!["x".into()]
                    })
                );
                assert_eq!(
                    **rhs,
                    Expr::Path(Path {
                        segments: vec!["x".into()]
                    })
                );
            }
            other => unreachable!("expected x + x, got {other:?}"),
        }
        assert_eq!(m.functions[1].name, "main");
        assert!(m.functions[1].params.is_empty());
        assert_eq!(m.functions[1].ret_ty, Type::IoUnit);
        match &m.functions[1].body[0] {
            Stmt::Expr(Expr::Call(c)) => {
                assert_eq!(c.callee, "print");
                assert_eq!(c.args.len(), 1);
                match &c.args[0] {
                    Expr::Call(inner) => {
                        assert_eq!(inner.callee, "double");
                        assert_eq!(inner.args, vec![Expr::LitInt(21)]);
                    }
                    other => unreachable!("expected double(21), got {other:?}"),
                }
            }
            other => unreachable!("expected print(double(21)), got {other:?}"),
        }
    }

    #[test]
    fn parses_assert_test() {
        let src = r#"
module assert_demo

fn main() -> Io<()> {
  print("ok")
}

test t {
  assert 2 + 2 == 4
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.name, "assert_demo");
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "main");
        assert_eq!(m.tests.len(), 1);
        assert_eq!(m.tests[0].name, "t");
        assert_eq!(m.tests[0].body.len(), 1);
        match &m.tests[0].body[0] {
            Stmt::Assert { lhs, rhs } => {
                assert_eq!(
                    *lhs,
                    Expr::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::LitInt(2)),
                        rhs: Box::new(Expr::LitInt(2)),
                    }
                );
                assert_eq!(*rhs, Expr::LitInt(4));
            }
            other => unreachable!("expected Assert, got {other:?}"),
        }
        // Assert stays in Module.tests, not in main body
        assert!(
            m.functions[0]
                .body
                .iter()
                .all(|s| !matches!(s, Stmt::Assert { .. })),
            "Assert must not appear in main"
        );

        let src2 = r#"
module assert_demo

fn double(x: Int) -> Int {
  x + x
}

fn main() -> Io<()> {
  print("ok")
}

test t {
  assert double(2) == 4
}
"#;
        let m2 = parse(src2).expect("parse call assert");
        assert_eq!(m2.tests.len(), 1);
        match &m2.tests[0].body[0] {
            Stmt::Assert { lhs, rhs } => {
                assert_eq!(
                    *lhs,
                    Expr::Call(Call {
                        callee: "double".into(),
                        args: vec![Expr::LitInt(2)],
                    })
                );
                assert_eq!(*rhs, Expr::LitInt(4));
            }
            other => unreachable!("expected Assert call, got {other:?}"),
        }
    }

    #[test]
    fn e0231_unsafe_block() {
        let src = r#"
module e0231_unsafe
fn main() -> Io<()> {
  unsafe {
    print("no")
  }
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0231");
        assert!(err.to_string().contains("unsafe / FFI not allowed"));
        let src2 = r#"
module e0231_extern
extern "C" {
  fn foo()
}
fn main() -> Io<()> {
  print("no")
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0231");
        let src3 = r#"
module e0231_ufn
unsafe fn main() -> Io<()> {
  print("no")
}
"#;
        assert_eq!(code_of(parse(src3).unwrap_err()), "E0231");
    }

    // ── ADR-291 MUTEX-REJECT (CORE-0.10-MUTEX-REJECT-20261003), Parser part ──
    // Sources live in `const` items outside the `#[test]` bodies (B-286-5 / VT006).
    // NOT VALIDATED: written without compiling or running (Lex gate, ADR-292).
    const ADR291_MSG: &str = "E0346: mutex concurrency is not available in this surface";

    const ADR291_LET_GENERIC: &str =
        "module t\nfn main() -> Io<()> {\n  let m: Mutex<Int> = 0\n  print(1)\n}\n";
    const ADR291_PARAM_ARC_GENERIC: &str = "module t\nfn f(a: Arc<Mutex<Int>>) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_RET_GENERIC: &str =
        "module t\nfn f() -> Mutex<Int> {\n  1\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_RESULT_ARG_GENERIC: &str = "module t\nfn f(r: Result<Mutex<Int>, Int>) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_LET_NAMED: &str =
        "module t\nfn main() -> Io<()> {\n  let m: Mutex = 0\n  print(1)\n}\n";
    const ADR291_PARAM_NAMED: &str = "module t\nfn f(m: Mutex) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_RET_NAMED: &str =
        "module t\nfn f() -> Arc {\n  1\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_RESULT_ARG_NAMED: &str = "module t\nfn f(r: Result<Mutex, Int>) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_OPTION_ARG_NAMED: &str = "module t\nfn f(o: Option<Arc>) -> Io<()> {\n  print(1)\n}\nfn main() -> Io<()> {\n  print(2)\n}\n";
    const ADR291_CTOR_MUTEX: &str =
        "module t\nfn main() -> Io<()> {\n  let m: Int = Mutex.new(0)\n  print(1)\n}\n";
    const ADR291_CTOR_ARC: &str =
        "module t\nfn main() -> Io<()> {\n  let a: Int = Arc.new(0)\n  print(1)\n}\n";
    const ADR291_LOCK_LET: &str = "module t\nfn main() -> Io<()> {\n  let n: Int = 0\n  let g: Int = n.lock()\n  print(1)\n}\n";
    const ADR291_LOCK_STMT: &str =
        "module t\nfn main() -> Io<()> {\n  let n: Int = 0\n  n.lock()\n  print(1)\n}\n";
    const ADR291_LOCK_HOST: &str =
        "module t\nfn main() -> Io<()> {\n  let g: Int = host.lock()\n  print(1)\n}\n";
    const ADR291_LOCK_CHAINED: &str =
        "module t\nfn main() -> Io<()> {\n  let n: Int = 0\n  n.lock().unwrap()\n  print(1)\n}\n";
    const ADR291_ORDER_CTOR_FIRST: &str = "module t\nfn main() -> Io<()> {\n  let m: Int = Mutex.new(0)\n  let n: Int = 0\n  let g: Int = n.lock()\n  print(1)\n}\n";
    const ADR291_ORDER_LOCK_FIRST: &str = "module t\nfn main() -> Io<()> {\n  let n: Int = 0\n  let g: Int = n.lock()\n  let m: Int = Mutex.new(0)\n  print(1)\n}\n";
    const ADR291_PEST_AFTER_LOWERING: &str = "module t\nfn main() -> Io<()> {\n  let n: Int = 0\n  n.lock()\n  let m: Mutex<Int> = 0\n  print(1)\n}\n";
    const ADR291_OK_WORDS: &str = "module t\n// Arc<Mutex<Int>> and n.lock() only in a comment\nfn main() -> Io<()> {\n  let lock: Int = 2\n  let mutex_count: Int = 1\n  print(\"Mutex<Int> n.lock() Arc.new\")\n  print(lock)\n}\n";
    const ADR291_RWLOCK: &str =
        "module t\nfn main() -> Io<()> {\n  let r: RwLock = 0\n  print(1)\n}\n";
    const ADR291_FREE_MUTEX_NEW: &str =
        "module t\nfn main() -> Io<()> {\n  let _m: Int = mutex_new(0)\n  print(1)\n}\n";
    const ADR291_SPEC_E0006: &str =
        "\nmodule hello\nfn main() -> Io<()> {\n  spec foo\n  print(\"hello\")\n}\n";

    /// Cases that die in pest (generic forms): (source, identifier the span must cover).
    const ADR291_GENERIC_CASES: [(&str, &str); 4] = [
        (ADR291_LET_GENERIC, "Mutex"),
        (ADR291_PARAM_ARC_GENERIC, "Arc"),
        (ADR291_RET_GENERIC, "Mutex"),
        (ADR291_RESULT_ARG_GENERIC, "Mutex"),
    ];

    /// Cases that reach the lowering as `Type::Named` (no `<..>`).
    const ADR291_NAMED_CASES: [(&str, &str); 5] = [
        (ADR291_LET_NAMED, "Mutex"),
        (ADR291_PARAM_NAMED, "Mutex"),
        (ADR291_RET_NAMED, "Arc"),
        (ADR291_RESULT_ARG_NAMED, "Mutex"),
        (ADR291_OPTION_ARG_NAMED, "Arc"),
    ];

    const ADR291_LOCK_CASES: [&str; 4] = [
        ADR291_LOCK_LET,
        ADR291_LOCK_STMT,
        ADR291_LOCK_HOST,
        ADR291_LOCK_CHAINED,
    ];

    /// `ADR291_MSG` + the byte span of the FIRST occurrence of `anchor` in `src`.
    fn adr291_want(src: &str, anchor: &str) -> String {
        let s = src.find(anchor).expect("anchor in source");
        format!("{ADR291_MSG} @{}..{}", s, s + anchor.len())
    }

    #[test]
    fn adr291_generic_forms_reject_at_identifier() {
        for (src, anchor) in ADR291_GENERIC_CASES {
            let err = parse(src).expect_err("Mutex<..> / Arc<..> must reject");
            assert_eq!(code_of(err.clone()), "E0346", "{src}");
            assert_eq!(err.to_string(), adr291_want(src, anchor), "{src}");
        }
    }

    #[test]
    fn adr291_named_type_forms_reject_at_identifier() {
        for (src, anchor) in ADR291_NAMED_CASES {
            let err = parse(src).expect_err("Named Mutex / Arc must reject");
            assert_eq!(err.to_string(), adr291_want(src, anchor), "{src}");
        }
    }

    #[test]
    fn adr291_constructor_anchors_on_receiver_identifier() {
        for (src, anchor) in [(ADR291_CTOR_MUTEX, "Mutex"), (ADR291_CTOR_ARC, "Arc")] {
            let err = parse(src).expect_err("Mutex.new / Arc.new must reject");
            assert_eq!(err.to_string(), adr291_want(src, anchor), "{src}");
        }
    }

    #[test]
    fn adr291_lock_any_receiver_anchors_on_selector() {
        for src in ADR291_LOCK_CASES {
            let err = parse(src).expect_err(".lock() must reject for any receiver");
            assert_eq!(code_of(err.clone()), "E0346", "{src}");
            assert_eq!(err.to_string(), adr291_want(src, "lock"), "{src}");
        }
    }

    #[test]
    fn adr291_source_order_inside_lowering() {
        let a = parse(ADR291_ORDER_CTOR_FIRST).expect_err("ctor first");
        assert_eq!(a.to_string(), adr291_want(ADR291_ORDER_CTOR_FIRST, "Mutex"));
        let b = parse(ADR291_ORDER_LOCK_FIRST).expect_err("lock first");
        assert_eq!(b.to_string(), adr291_want(ADR291_ORDER_LOCK_FIRST, "lock"));
    }

    #[test]
    fn adr291_pest_failure_wins_over_lowering_offender() {
        // Phase order, not source order: pest fails on the generic form (line 5) before any
        // lowering runs, so the `.lock()` on line 4 is never reached.
        let err = parse(ADR291_PEST_AFTER_LOWERING).expect_err("generic form fails in pest");
        assert_eq!(
            err.to_string(),
            adr291_want(ADR291_PEST_AFTER_LOWERING, "Mutex")
        );
    }

    #[test]
    fn adr291_does_not_widen_or_over_reject() {
        // Words inside comments, strings and longer identifiers are not reserved.
        assert!(
            parse(ADR291_OK_WORDS).is_ok(),
            "{:?}",
            parse(ADR291_OK_WORDS).err()
        );
        // Q7: RwLock stays out; `mutex_new(0)` is not an ADR-291 form (ADR-293 / E0347 owns it).
        assert!(
            parse(ADR291_RWLOCK).is_ok(),
            "{:?}",
            parse(ADR291_RWLOCK).err()
        );
        assert!(
            parse(ADR291_FREE_MUTEX_NEW).is_ok(),
            "{:?}",
            parse(ADR291_FREE_MUTEX_NEW).err()
        );
        // Unrelated parse failures keep their code.
        let e = parse(ADR291_SPEC_E0006).expect_err("spec is outside F1.1");
        assert_ne!(code_of(e.clone()), "E0346", "{e}");
        assert_eq!(code_of(e), "E0006");
    }

    #[test]
    fn e0261_external_crate_path() {
        let src = r#"
module e0261_crate
fn main() -> Io<()> {
  tokio::spawn()
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0261");
        assert!(
            err.to_string().contains("external crate path not allowed"),
            "err={err}"
        );
        // Vec::new remains legal
        let ok = r#"
module vec_ok
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  print(v.len())
}
"#;
        assert!(parse(ok).is_ok(), "Vec::new must remain legal");
    }

    #[test]
    fn e0261_rejects_use_tokio_spawn() {
        // ADR-029 / core04-mm-neg-crate: `use tokio::spawn` must be E0261, not Enum::Variant/E0404.
        let src = r#"
module e0261_use_tokio
use tokio::spawn
fn main() -> Io<()> {
  print(1)
}
"#;
        let err = parse(src).expect_err("use tokio::spawn must fail");
        assert_eq!(code_of(err.clone()), "E0261");
    }

    #[test]
    fn use_local_mod_item_ok_mm_happy() {
        // core04-mm-happy: local `use util::double` must parse (not E0261).
        let src = r#"
module mm_happy_use
use util::double
fn main() -> Io<()> {
  print(1)
}
"#;
        assert!(
            parse(src).is_ok(),
            "local use mod::item must remain legal: {:?}",
            parse(src).err()
        );
    }

    #[test]
    fn async_fn_result_ret_ok_adr245() {
        // ADR-245: async handlers may return Result (HTTP); not E0241.
        let src = r#"
module async_result_ok
async fn handle(req: HttpRequest) -> Result<HttpResponse, IoError> {
  let out: Result<HttpResponse, IoError> = Err(e)
  out
}
async fn main() -> Io<()> {
  print(1)
}
"#;
        match parse(src) {
            Ok(m) => {
                assert!(m.functions.iter().any(|f| f.name == "handle" && f.is_async));
            }
            Err(e) => {
                assert_ne!(
                    code_of(e.clone()),
                    "E0241",
                    "async Result must not be E0241: {e}"
                );
                unreachable!("unexpected parse err: {e}");
            }
        }
    }

    #[test]
    fn e0210_todo_bang() {
        // Load fixtures from ejemplos so Veyra anti-theater (VT001/VT002) does not
        // treat E0210 rejection oracles embedded in raw strings as production stubs.
        const TODO_BANG: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ejemplos/f2/neg/e0210-todo.arita"
        ));
        const UNIMPLEMENTED_BANG: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ejemplos/f2/neg/e0210-unimplemented.arita"
        ));
        const TODO_KW: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ejemplos/f2/neg/e0210-todo-kw.arita"
        ));

        let err = parse(TODO_BANG).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0210");
        assert!(err.to_string().contains("todo/unimplemented not allowed"));

        assert_eq!(code_of(parse(UNIMPLEMENTED_BANG).unwrap_err()), "E0210");
        assert_eq!(code_of(parse(TODO_KW).unwrap_err()), "E0210");
    }

    #[test]
    fn e0211_assert_true() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert true
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0211");
        assert!(err.to_string().contains("assert requires evidence"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert(true)
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0211");

        let src3 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert true == true
}
"#;
        assert_eq!(code_of(parse(src3).unwrap_err()), "E0211");
    }

    #[test]
    fn e0214_vacuous_len_ge_zero() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.len() >= 0
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0214");
        assert!(err.to_string().contains("vacuous length assert"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert s.len() > -1
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0214");
    }

    #[test]
    fn e0214_len_eq_len() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.len() == v.len()
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0214");
        assert!(err.to_string().contains("vacuous length assert"));

        // Non-vacuous: different receivers or real evidence — must parse
        let ok = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.len() == 3
}
"#;
        parse(ok).expect("non-vacuous len assert must parse");
    }

    #[test]
    fn e0214_is_empty_tautology() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.is_empty() || !v.is_empty()
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0214");
        assert!(err.to_string().contains("vacuous length assert"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert !s.is_empty() || s.is_empty()
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0214");
    }

    #[test]
    fn e0215_eq_self_lit_and_path() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert 1 == 1
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0215");
        assert!(err.to_string().contains("vacuous comparison assert"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert n == n
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0215");

        // Non-vacuous evidence must still parse
        let ok = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert 2 + 2 == 4
}
"#;
        parse(ok).expect("non-vacuous assert must parse");

        // E0211 not diluted: true == true stays E0211
        let t = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert true == true
}
"#;
        assert_eq!(code_of(parse(t).unwrap_err()), "E0211");

        // E0214 not diluted: len == len stays E0214
        let len = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.len() == v.len()
}
"#;
        assert_eq!(code_of(parse(len).unwrap_err()), "E0214");
    }

    #[test]
    fn e0215_le_ge_ne_self() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert n <= n
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0215");
        assert!(err.to_string().contains("vacuous comparison assert"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert 3 >= 3
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0215");

        let src3 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert x != x
}
"#;
        assert_eq!(code_of(parse(src3).unwrap_err()), "E0215");

        // Unequal rel form stays OUT of assert surface (E0006)
        let bad = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert a <= b
}
"#;
        assert_eq!(code_of(parse(bad).unwrap_err()), "E0006");

        // len() >= 0 still E0214 (dedicated rule wins)
        let len = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  assert v.len() >= 0
}
"#;
        assert_eq!(code_of(parse(len).unwrap_err()), "E0214");
    }

    #[test]
    fn e0212_empty_test() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t { }
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0212");
        assert!(err.to_string().contains("empty test not allowed"));

        let src2 = r#"
module demo
fn main() -> Io<()> {
  print("ok")
}
test t {
  // only comments
}
"#;
        assert_eq!(code_of(parse(src2).unwrap_err()), "E0212");
    }

    #[test]
    fn e0213_empty_valued_fn() {
        let src = r#"
module demo
fn f(x: Int) -> Int { }
fn main() -> Io<()> {
  print("ok")
}
"#;
        let err = parse(src).unwrap_err();
        assert_eq!(code_of(err.clone()), "E0213");
        assert!(err.to_string().contains("stub function body not allowed"));
    }

    #[test]
    fn parses_borrow_expr_in_let() {
        let src = r#"
module m
fn main() -> Io<()> {
  let mut x: Int = 1
  let a: Int = &mut x
  let b: Int = &x
  print(1)
}
"#;
        let m = parse(src).expect("parse borrow");
        let body = &m.functions[0].body;
        assert_eq!(body.len(), 4);
        match &body[1] {
            Stmt::Let { name, init, .. } => {
                assert_eq!(name, "a");
                assert_eq!(
                    *init,
                    Expr::Borrow {
                        mutable: true,
                        inner: Box::new(Expr::Path(Path {
                            segments: vec!["x".into()]
                        })),
                    }
                );
            }
            other => unreachable!("expected Let a = &mut x, got {other:?}"),
        }
        match &body[2] {
            Stmt::Let { name, init, .. } => {
                assert_eq!(name, "b");
                assert_eq!(
                    *init,
                    Expr::Borrow {
                        mutable: false,
                        inner: Box::new(Expr::Path(Path {
                            segments: vec!["x".into()]
                        })),
                    }
                );
            }
            other => unreachable!("expected Let b = &x, got {other:?}"),
        }
    }

    #[test]
    fn parses_if_true() {
        let src = r#"
module if_true
fn main() -> Io<()> {
  if true {
    print("yes")
  }
}
"#;
        let m = parse(src).expect("parse if");
        assert_eq!(m.name, "if_true");
        match &m.functions[0].body[0] {
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                assert_eq!(*cond, Expr::LitBool(true));
                assert!(else_body.is_none());
                assert_eq!(then_body.len(), 1);
            }
            other => unreachable!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn parses_if_else() {
        let src = r#"
module if_else
fn main() -> Io<()> {
  let flag: Bool = false
  if flag {
    print("yes")
  } else {
    print("no")
  }
}
"#;
        let m = parse(src).expect("parse if-else");
        match &m.functions[0].body[1] {
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                assert_eq!(
                    *cond,
                    Expr::Path(Path {
                        segments: vec!["flag".into()]
                    })
                );
                assert_eq!(then_body.len(), 1);
                let eb = else_body.as_ref().expect("else");
                assert_eq!(eb.len(), 1);
            }
            other => unreachable!("expected If-else, got {other:?}"),
        }
    }

    #[test]
    fn parses_while_count() {
        let src = r#"
module while_count
fn main() -> Io<()> {
  let mut i: Int = 0
  while i < 3 {
    print(i)
    i = i + 1
  }
}
"#;
        let m = parse(src).expect("parse while");
        assert_eq!(m.functions[0].body.len(), 2);
        match &m.functions[0].body[1] {
            Stmt::While { cond, body } => {
                assert_eq!(
                    *cond,
                    Expr::Binary {
                        op: BinOp::Lt,
                        lhs: Box::new(Expr::Path(Path {
                            segments: vec!["i".into()]
                        })),
                        rhs: Box::new(Expr::LitInt(3)),
                    }
                );
                assert_eq!(body.len(), 2);
                match &body[1] {
                    Stmt::Assign { name, value } => {
                        assert_eq!(name, "i");
                        assert_eq!(
                            *value,
                            Expr::Binary {
                                op: BinOp::Add,
                                lhs: Box::new(Expr::Path(Path {
                                    segments: vec!["i".into()]
                                })),
                                rhs: Box::new(Expr::LitInt(1)),
                            }
                        );
                    }
                    other => unreachable!("expected Assign, got {other:?}"),
                }
            }
            other => unreachable!("expected While, got {other:?}"),
        }
    }

    #[test]
    fn parses_cmp_in_let_bool() {
        let src = r#"
module cmp
fn main() -> Io<()> {
  let a: Int = 1
  let b: Bool = a < 2
  print(b)
}
"#;
        let m = parse(src).expect("parse cmp let");
        match &m.functions[0].body[1] {
            Stmt::Let { ty, init, .. } => {
                assert_eq!(*ty, Type::Bool);
                assert_eq!(
                    *init,
                    Expr::Binary {
                        op: BinOp::Lt,
                        lhs: Box::new(Expr::Path(Path {
                            segments: vec!["a".into()]
                        })),
                        rhs: Box::new(Expr::LitInt(2)),
                    }
                );
            }
            other => unreachable!("expected Let Bool cmp, got {other:?}"),
        }
    }

    #[test]
    fn parses_result_ok_match() {
        let src = r#"
module result_ok
fn main() -> Io<()> {
  let r: Result<Int, String> = Ok(42)
  match r {
    Ok(x) => { print(x) }
    Err(e) => { print(e) }
  }
}
"#;
        let m = parse(src).expect("parse Result ok");
        assert_eq!(m.name, "result_ok");
        let body = &m.functions[0].body;
        assert!(matches!(
            body[0],
            Stmt::Let {
                ty: Type::Result(_, _),
                ..
            }
        ));
        assert!(matches!(
            body[0],
            Stmt::Let {
                init: Expr::Ok(_),
                ..
            }
        ));
        assert!(matches!(
            &body[1],
            Stmt::Match { arms, .. }
                if matches!(arms[0].pat, Pat::Ok(_)) && matches!(arms[1].pat, Pat::Err(_))
        ));
    }

    #[test]
    fn parses_match_bool() {
        let src = r#"
module match_bool
fn main() -> Io<()> {
  let flag: Bool = true
  match flag {
    true => { print("yes") }
    false => { print("no") }
  }
}
"#;
        let m = parse(src).expect("parse match bool");
        assert_eq!(m.name, "match_bool");
        match &m.functions[0].body[1] {
            Stmt::Match { scrutinee, arms } => {
                assert_eq!(
                    *scrutinee,
                    Expr::Path(Path {
                        segments: vec!["flag".into()]
                    })
                );
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].pat, Pat::LitBool(true));
                assert_eq!(arms[1].pat, Pat::LitBool(false));
                assert_eq!(arms[0].body.len(), 1);
                assert_eq!(arms[1].body.len(), 1);
            }
            other => unreachable!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn parses_match_int() {
        let src = r#"
module match_int
fn main() -> Io<()> {
  let n: Int = 1
  match n {
    1 => { print("a") }
    _ => { print("b") }
  }
}
"#;
        let m = parse(src).expect("parse match int");
        match &m.functions[0].body[1] {
            Stmt::Match { scrutinee, arms } => {
                assert_eq!(
                    *scrutinee,
                    Expr::Path(Path {
                        segments: vec!["n".into()]
                    })
                );
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].pat, Pat::LitInt(1));
                assert_eq!(arms[1].pat, Pat::Wildcard);
            }
            other => unreachable!("expected Match int, got {other:?}"),
        }
    }

    #[test]
    fn parses_match_expr_bool_let() {
        let src = r#"
module match_expr_bool
fn main() -> Io<()> {
  let flag: Bool = true
  let x: Int = match flag {
    true => { 1 }
    false => { 0 }
  }
  print(x)
}
"#;
        let m = parse(src).expect("parse match expr bool");
        match &m.functions[0].body[1] {
            Stmt::Let { name, ty, init, .. } => {
                assert_eq!(name, "x");
                assert_eq!(*ty, Type::Int);
                match init {
                    Expr::Match { scrutinee, arms } => {
                        assert_eq!(
                            **scrutinee,
                            Expr::Path(Path {
                                segments: vec!["flag".into()]
                            })
                        );
                        assert_eq!(arms.len(), 2);
                        assert_eq!(arms[0].pat, Pat::LitBool(true));
                        assert_eq!(arms[0].value, Expr::LitInt(1));
                        assert_eq!(arms[1].pat, Pat::LitBool(false));
                        assert_eq!(arms[1].value, Expr::LitInt(0));
                    }
                    other => unreachable!("expected Expr::Match, got {other:?}"),
                }
            }
            other => unreachable!("expected Let with match expr, got {other:?}"),
        }
    }

    #[test]
    fn parses_match_expr_int_let() {
        let src = r#"
module match_expr_int
fn main() -> Io<()> {
  let n: Int = 1
  let x: Int = match n {
    1 => { 10 }
    _ => { 20 }
  }
  print(x)
}
"#;
        let m = parse(src).expect("parse match expr int");
        match &m.functions[0].body[1] {
            Stmt::Let { init, .. } => match init {
                Expr::Match { arms, .. } => {
                    assert_eq!(arms[0].pat, Pat::LitInt(1));
                    assert_eq!(arms[0].value, Expr::LitInt(10));
                    assert_eq!(arms[1].pat, Pat::Wildcard);
                    assert_eq!(arms[1].value, Expr::LitInt(20));
                }
                other => unreachable!("expected Expr::Match, got {other:?}"),
            },
            other => unreachable!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn parses_while_break() {
        let src = r#"
module while_break
fn main() -> Io<()> {
  let mut i: Int = 0
  while i < 10 {
    print(i)
    i = i + 1
    if i == 2 {
      break
    }
  }
}
"#;
        let m = parse(src).expect("parse while break");
        match &m.functions[0].body[1] {
            Stmt::While { body, .. } => {
                assert!(
                    matches!(body.last(), Some(Stmt::If { then_body, .. }) if matches!(then_body.first(), Some(Stmt::Break))),
                    "expected break inside if in while, got {body:?}"
                );
            }
            other => unreachable!("expected While, got {other:?}"),
        }
    }

    #[test]
    fn parses_while_continue() {
        let src = r#"
module while_continue
fn main() -> Io<()> {
  let mut j: Int = 0
  while j < 6 {
    let rem: Int = j % 2
    if rem == 1 {
      j = j + 1
      continue
    }
    print(j)
    j = j + 1
  }
}
"#;
        let m = parse(src).expect("parse while continue");
        match &m.functions[0].body[1] {
            Stmt::While { body, .. } => {
                let if_stmt = body.iter().find(|s| matches!(s, Stmt::If { .. }));
                match if_stmt {
                    Some(Stmt::If { then_body, .. }) => {
                        assert!(
                            then_body.iter().any(|s| matches!(s, Stmt::Continue)),
                            "expected continue in if then, got {then_body:?}"
                        );
                    }
                    other => unreachable!("expected If with continue, got {other:?}"),
                }
            }
            other => unreachable!("expected While, got {other:?}"),
        }
    }

    /// Types-only: construct F2 AST nodes without parsing F2 surface.
    #[test]
    fn f2_ast_types_constructible() {
        let let_stmt = Stmt::Let {
            mutable: false,
            name: "n".into(),
            ty: Type::Int,
            init: Expr::LitInt(1),
        };
        let assert_stmt = Stmt::Assert {
            lhs: Expr::Path(Path {
                segments: vec!["n".into()],
            }),
            rhs: Expr::LitInt(1),
        };
        let test = Test {
            name: "one_equals_one".into(),
            body: vec![let_stmt.clone(), assert_stmt],
        };
        let borrow = Expr::Borrow {
            mutable: true,
            inner: Box::new(Expr::Path(Path {
                segments: vec!["n".into()],
            })),
        };
        let binary = Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr::LitInt(1)),
            rhs: Box::new(Expr::LitInt(2)),
        };
        let method = Expr::MethodCall {
            receiver: Box::new(Expr::Path(Path {
                segments: vec!["xs".into()],
            })),
            method: "push".into(),
            args: vec![Expr::LitInt(3)],
        };
        let m = Module {
            name: "demo".into(),
            functions: vec![Function {
                name: "main".into(),
                params: vec![FnParam {
                    name: "x".into(),
                    ty: Type::Bool,
                }],
                ret_ty: Type::IoUnit,
                is_async: false,
                body: vec![
                    Stmt::Expr(Expr::LitBool(true)),
                    Stmt::Expr(borrow),
                    Stmt::Expr(binary),
                    Stmt::Expr(method),
                    let_stmt,
                    Stmt::If {
                        cond: Expr::LitBool(true),
                        then_body: vec![],
                        else_body: None,
                    },
                    Stmt::While {
                        cond: Expr::LitBool(false),
                        body: vec![],
                    },
                    Stmt::Assign {
                        name: "n".into(),
                        value: Expr::LitInt(2),
                    },
                    Stmt::Match {
                        scrutinee: Expr::LitBool(true),
                        arms: vec![
                            MatchArm {
                                pat: Pat::LitBool(true),
                                body: vec![],
                            },
                            MatchArm {
                                pat: Pat::Wildcard,
                                body: vec![],
                            },
                        ],
                    },
                    Stmt::Break,
                    Stmt::Continue,
                ],
                is_pub: false,
            }],
            tests: vec![test],
            contracts: vec![],
            records: vec![],
            enums: vec![],
            uses: vec![],
        };
        assert_eq!(m.tests.len(), 1);
        assert_eq!(m.functions[0].params[0].ty, Type::Bool);
        assert_eq!(
            Type::Vec(Box::new(Type::String)),
            Type::Vec(Box::new(Type::String))
        );
        let _item = Item::Test(m.tests[0].clone());
    }

    #[test]
    fn parse_lang_contract_stdout() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("hello")
}
contract hello_c {
  expect_stdout "hello"
}
"#;
        let m = parse(src).expect("parse lang contract");
        assert_eq!(m.contracts.len(), 1);
        assert_eq!(m.contracts[0].name, "hello_c");
        assert_eq!(
            m.contracts[0].expect,
            ContractExpect::Stdout(vec!["hello".into()])
        );
    }

    #[test]
    fn parse_lang_contract_target_helper() {
        let src = r#"
module demo
fn helper() -> Io<()> {
  print("from-helper")
}
fn main() -> Io<()> {
  print("from-main")
}
contract c {
  target helper
  expect_stdout "from-helper"
}
"#;
        let m = parse(src).expect("parse target contract");
        assert_eq!(m.contracts.len(), 1);
        assert_eq!(m.contracts[0].target.as_deref(), Some("helper"));
        assert_eq!(
            m.contracts[0].expect,
            ContractExpect::Stdout(vec!["from-helper".into()])
        );
        assert!(m
            .functions
            .iter()
            .any(|f| f.name == "helper" && f.ret_ty == Type::IoUnit));
    }

    #[test]
    fn parse_lang_contract_default_target_none() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("hello")
}
contract hello_c {
  expect_stdout "hello"
}
"#;
        let m = parse(src).expect("parse");
        assert_eq!(m.contracts[0].target, None);
    }

    #[test]
    fn parse_lang_contract_multi_stdout() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("a")
  print("b")
}
contract lines_c {
  expect_stdout "a"
  expect_stdout "b"
}
"#;
        let m = parse(src).expect("parse multi stdout");
        assert_eq!(
            m.contracts[0].expect,
            ContractExpect::Stdout(vec!["a".into(), "b".into()])
        );
    }

    #[test]
    fn parse_lang_contract_reject() {
        let src = r#"
module bad
fn main() -> Io<()> {
  if true {
    break
  }
  print("x")
}
contract bad_c {
  expect_reject "E0224"
}
"#;
        let m = parse(src).expect("parse reject contract (HIR not run)");
        assert_eq!(m.contracts.len(), 1);
        assert_eq!(
            m.contracts[0].expect,
            ContractExpect::Reject("E0224".into())
        );
    }

    #[test]
    fn parse_lang_contract_mixed_expects_fails() {
        let src = r#"
module demo
fn main() -> Io<()> {
  print("hello")
}
contract bad {
  expect_stdout "hello"
  expect_reject "E0224"
}
"#;
        let err = parse(src).expect_err("mixed expects");
        let s = err.to_string();
        assert!(
            s.contains("exactly one expect kind") || s.contains("E0006"),
            "got={s}"
        );
    }

    #[test]
    fn adr278_parses_ident_try_let() {
        let src = r#"
module t
fn load(x: Int) -> Result<Int, Int> {
  let r: Result<Int, Int> = Ok(x)
  let v: Int = r?
  Ok(v)
}
fn main() -> Io<()> { print(1) }
"#;
        let m = parse(src).expect("parse r?");
        let load = m.functions.iter().find(|f| f.name == "load").unwrap();
        assert!(matches!(
            &load.body[1],
            Stmt::Let {
                init: Expr::Try { inner },
                ..
            } if matches!(inner.as_ref(), Expr::Path(_))
        ));
    }

    #[test]
    fn adr278_parses_user_call_try_let() {
        let src = r#"
module t
fn load(x: Int) -> Result<Int, Int> { Ok(x) }
fn pipeline(x: Int) -> Result<Int, Int> {
  let a: Int = load(x)?
  Ok(a)
}
fn main() -> Io<()> { print(1) }
"#;
        let m = parse(src).expect("parse load()?");
        let pipe = m.functions.iter().find(|f| f.name == "pipeline").unwrap();
        assert!(matches!(
            &pipe.body[0],
            Stmt::Let {
                init: Expr::Try { inner },
                ..
            } if matches!(inner.as_ref(), Expr::Call(c) if c.callee == "load")
        ));
    }

    // ADR-282 (CORE-0.9-MAP-ASSIGN-20260926): parser → Stmt::IndexAssign (sin decidir tipo).
    fn adr282_main(body: &str) -> String {
        format!("module t\nfn main() -> Io<()> {{\n{body}\n  print(1)\n}}\n")
    }

    #[test]
    fn adr282_parses_map_index_assign_node() {
        let src = adr282_main("  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 7");
        let m = parse(&src).expect("m[k] = v must parse");
        let start = src.find("m[\"a\"] = 7").expect("stmt in src");
        assert_eq!(
            m.functions[0].body[1],
            Stmt::IndexAssign {
                target: "m".into(),
                key: Expr::LitStr("a".into()),
                value: Expr::LitInt(7),
                op: None,
                span: Span {
                    start,
                    end: start + "m[\"a\"] = 7".len(),
                },
            }
        );
    }

    #[test]
    fn adr282_index_assign_rhs_len_and_arith_parse() {
        let src = adr282_main(
            "  let mut m: Map<Text, Int> = Map::new()\n  let x: Int = 4\n  m[\"b\"] = m.len()\n  m[\"c\"] = x + 1",
        );
        let m = parse(&src).expect("assign_rhs len / arith must parse");
        let body = &m.functions[0].body;
        assert!(matches!(
            &body[2],
            Stmt::IndexAssign { value: Expr::MethodCall { method, .. }, .. } if method == "len"
        ));
        assert!(matches!(
            &body[3],
            Stmt::IndexAssign {
                value: Expr::Binary { .. },
                ..
            }
        ));
    }

    #[test]
    fn adr282_vec_index_assign_parses_type_decided_in_hir() {
        let src = adr282_main("  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2");
        let m = parse(&src).expect("parser does not decide Map vs Vec");
        assert!(matches!(
            &m.functions[0].body[1],
            Stmt::IndexAssign { target, key: Expr::LitInt(0), .. } if target == "v"
        ));
    }

    // ADR-283 (CORE-0.9-VEC-ASSIGN-20260926): peek `has_non_map_index_assign` retirado; el
    // parser no decide Map/Vec/String. Si main sin print contiene un IndexAssign, el E0001 se
    // difiere a HIR como marcador (mismo código/texto/span); sin IndexAssign, E0001 directo.
    /// Test helper (ADR-283 gate): byte offset one past the `}` closing the last item.
    fn adr283_close_last(src: &str) -> usize {
        src.rfind('}').expect("close") + 1
    }

    /// Test helper (ADR-283 gate): byte offset one past the `}` right before item `next`.
    fn adr283_close_before(src: &str, next: &str) -> usize {
        src.find(&format!("}}\n{next}")).expect("close") + 1
    }

    fn adr283_marker(code: &str, message: &str, start: usize, end: usize) -> Stmt {
        Stmt::Expr(Expr::Call(Call {
            callee: DEFERRED_SHAPE_MARKER.to_string(),
            args: vec![
                Expr::LitStr(code.to_string()),
                Expr::LitStr(message.to_string()),
                Expr::LitInt(start as i64),
                Expr::LitInt(end as i64),
            ],
        }))
    }

    #[test]
    fn adr283_no_print_main_with_index_assign_defers_exact_e0001() {
        for body in [
            "  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2",
            "  let mut s: String = \"abc\"\n  s[0] = 1",
            "  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1",
            "  let mut v: Vec<Int> = Vec::new()\n  if true {\n    v[0] = 2\n  }",
        ] {
            let src = format!("module t\nfn main() -> Io<()> {{\n{body}\n}}\n");
            let start = src.find("fn main").expect("main");
            let end = adr283_close_last(&src);
            let m = parse(&src).expect("E0001 deferred to HIR");
            assert_eq!(
                m.functions[0].body[0],
                adr283_marker("E0001", "no usable print in main", start, end),
                "{body}"
            );
        }
        // No IndexAssign → plain parser E0001 (unchanged).
        let src = "module t\nfn main() -> Io<()> {\n  let x: Int = 1\n}\n";
        match parse(src).expect_err("no print") {
            ParseError::Coded { code, message, .. } => {
                assert_eq!(code, "E0001");
                assert_eq!(message, "no usable print in main");
            }
            other => unreachable!("expected coded E0001, got {other:?}"),
        }
    }

    #[test]
    fn adr283_later_gate_error_after_deferral_returns_first_e0001() {
        // Before ADR-283 the parser stopped at main's E0001; a later non-main gate error must
        // not surface — the first (deferred) error is returned, same text and span.
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2\n}\nfn h(x: Text) -> Int {\n  1\n}\n";
        let start = src.find("fn main").expect("main");
        let end = adr283_close_before(src, "fn h");
        match parse(src).expect_err("first error wins") {
            ParseError::Coded {
                code,
                message,
                span,
            } => {
                assert_eq!(code, "E0001");
                assert_eq!(message, "no usable print in main");
                assert_eq!(span, Some(Span { start, end }));
            }
            other => unreachable!("expected coded E0001, got {other:?}"),
        }
    }

    #[test]
    fn adr283_vec_index_assign_reaches_ast_in_result_fn_nested() {
        let src = "module t\nfn fill(n: Int) -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = n\n  if n > 0 {\n    v[0] = n + 1\n  }\n  Ok(n)\n}\nfn main() -> Io<()> {\n  print(1)\n}\n";
        let m = parse(src).expect("Vec index assign in Result fn must parse");
        let f = &m.functions[0];
        let start = src.find("v[0] = n\n").expect("stmt in src");
        let want_span = Span {
            start,
            end: start + "v[0] = n".len(),
        };
        assert!(matches!(
            &f.body[2],
            Stmt::IndexAssign { target, key: Expr::LitInt(0), value: Expr::Path(_), span, .. }
                if target == "v" && *span == want_span
        ));
        assert!(matches!(
            &f.body[3],
            Stmt::If { then_body, .. }
                if matches!(&then_body[0], Stmt::IndexAssign { target, .. } if target == "v")
        ));
    }

    #[test]
    fn adr283_index_assign_in_int_fn_defers_exact_e0006() {
        let msg = "non-main fn `f` body must be Expr binary/path/lit/call in this slice";
        let src = "module t\nfn f(n: Int) -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = n\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
        let start = src.find("fn f").expect("fn f");
        let end = adr283_close_before(src, "fn main");
        let m = parse(src).expect("E0006 deferred to HIR");
        assert_eq!(
            m.functions[0].body[0],
            adr283_marker("E0006", msg, start, end)
        );
        // IndexAssign not the only offender → immediate E0006 with the first offender's text.
        let src2 = "module t\nfn f(n: Int) -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = n\n  print(n)\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
        match parse(src2).expect_err("other offender") {
            ParseError::Coded { code, message, .. } => {
                assert_eq!(code, "E0006");
                assert_eq!(message, msg);
            }
            other => unreachable!("expected coded E0006, got {other:?}"),
        }
    }

    #[test]
    fn adr282_compound_nested_field_stay_e0006() {
        for stmt in ["m[\"a\"][0] = 1", "m[\"a\"].f = 1"] {
            let src = adr282_main(&format!(
                "  let mut m: Map<Text, Int> = Map::new()\n  {stmt}"
            ));
            let e = parse(&src).expect_err("residual form must reject");
            assert_eq!(code_of(e), "E0006", "{stmt}");
        }
    }

    #[test]
    fn adr282_put_literal_args_rule_intact_e0006() {
        let src = adr282_main("  let mut m: Map<Text, Int> = Map::new()\n  m.put(\"b\", m.len())");
        let e = parse(&src).expect_err("put keeps lit/ident args");
        assert_eq!(code_of(e), "E0006");
    }

    // ADR-283 addendum Ingeniero §3: parse-only entry point + marker not forgeable.
    fn adr283_coded(e: ParseError) -> (&'static str, String, Option<Span>) {
        match e {
            ParseError::Coded {
                code,
                message,
                span,
            } => (code, message, span),
            ParseError::Message(m) => ("Message", m, None),
        }
    }

    fn adr283_has_marker(m: &Module) -> bool {
        fn walk(body: &[Stmt]) -> bool {
            body.iter().any(|s| match s {
                Stmt::Expr(Expr::Call(c)) => c.callee == DEFERRED_SHAPE_MARKER,
                Stmt::If {
                    then_body,
                    else_body,
                    ..
                }
                | Stmt::IfLet {
                    then_body,
                    else_body,
                    ..
                } => walk(then_body) || else_body.as_ref().is_some_and(|b| walk(b)),
                Stmt::While { body, .. } | Stmt::WhileLet { body, .. } => walk(body),
                Stmt::Match { arms, .. } => arms.iter().any(|a| walk(&a.body)),
                _ => false,
            })
        }
        m.functions.iter().any(|f| walk(&f.body))
    }

    #[test]
    fn adr283_parse_standalone_no_print_vec_is_exact_e0001() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n}\n";
        let start = src.find("fn main").expect("main");
        let end = adr283_close_last(src);
        let (code, message, span) = adr283_coded(parse_standalone(src).expect_err("standalone"));
        assert_eq!(code, "E0001");
        assert_eq!(message, "no usable print in main");
        assert_eq!(span, Some(Span { start, end }));
        // `parse` (build/HIR path) still hands the marker to HIR, and the helper sees it.
        let m = parse(src).expect("deferred for HIR");
        assert!(adr283_has_marker(&m));
        assert_eq!(
            deferred_shape_error(&m).map(adr283_coded),
            Some((
                "E0001",
                "no usable print in main".to_string(),
                Some(Span { start, end })
            ))
        );
    }

    #[test]
    fn adr283_parse_standalone_no_print_map_is_e0001() {
        let src = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n}\n";
        let start = src.find("fn main").expect("main");
        let end = adr283_close_last(src);
        assert_eq!(
            adr283_coded(parse_standalone(src).expect_err("standalone")),
            (
                "E0001",
                "no usable print in main".to_string(),
                Some(Span { start, end })
            )
        );
    }

    #[test]
    fn adr283_parse_standalone_int_fn_index_assign_is_exact_e0006() {
        let src = "module t\nfn f(n: Int) -> Int {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = n\n  n\n}\nfn main() -> Io<()> {\n  print(f(1))\n}\n";
        let start = src.find("fn f").expect("fn f");
        let end = adr283_close_before(src, "fn main");
        assert_eq!(
            adr283_coded(parse_standalone(src).expect_err("standalone")),
            (
                "E0006",
                "non-main fn `f` body must be Expr binary/path/lit/call in this slice".to_string(),
                Some(Span { start, end })
            )
        );
    }

    #[test]
    fn adr283_parse_standalone_valid_programs_ok_without_marker() {
        for src in [
            "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] = 1\n  print(1)\n}\n",
            "module t\nfn fill(n: Int) -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(0)\n  v[0] = n\n  Ok(n)\n}\nfn main() -> Io<()> {\n  print(1)\n}\n",
        ] {
            let a = parse_standalone(src);
            let b = parse(src);
            assert!(matches!(&a, Ok(m) if !adr283_has_marker(m)), "{src}");
            assert_eq!(a, b, "standalone == parse when nothing is deferred");
            assert!(matches!(&b, Ok(m) if deferred_shape_error(m).is_none()));
        }
    }

    #[test]
    fn adr283_marker_cannot_be_written_from_surface() {
        for stmt in [
            "<arita:deferred-shape>(\"E0001\", \"no usable print in main\", 0, 1)",
            "arita:deferred-shape(\"E0001\", \"x\", 0, 1)",
            "\"<arita:deferred-shape>\"(\"E0001\", \"x\", 0, 1)",
            "<arita:deferred-shape>",
            "let x: Int = <arita:deferred-shape>(\"E0001\", \"x\", 0, 1)",
        ] {
            for tail in ["\n  print(1)", ""] {
                let src = format!("module t\nfn main() -> Io<()> {{\n  {stmt}{tail}\n}}\n");
                let r = parse(&src);
                assert!(
                    !matches!(&r, Ok(m) if adr283_has_marker(m)),
                    "marker forged: {src}"
                );
                let (code, message, _) = adr283_coded(r.expect_err("forge attempt must not parse"));
                assert_eq!(code, "E0006", "{src}");
                assert!(
                    message.starts_with("construct outside F1.1 (parse failure)"),
                    "{src}: {message}"
                );
                assert!(matches!(
                    parse_standalone(&src).map_err(adr283_coded),
                    Err(("E0006", _, _))
                ));
            }
        }
        // The marker text as a string literal is just data: parses, no marker stmt.
        let src = "module t\nfn main() -> Io<()> {\n  print(\"<arita:deferred-shape>\")\n}\n";
        let m = parse(src);
        assert!(matches!(&m, Ok(m) if !adr283_has_marker(m) && deferred_shape_error(m).is_none()));
    }

    // ADR-290 slice A (IndexMut compound): `v[i] op= rhs`, op ∈ {+=, -=, *=}. Sources live in
    // `const` items outside the `#[test]` bodies.
    const ADR290_ADD: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += 2\n  print(1)\n}\n";
    const ADR290_SUB: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] -= 2\n  print(1)\n}\n";
    const ADR290_MUL: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] *= 2\n  print(1)\n}\n";
    const ADR290_PLAIN: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] = 2\n  print(1)\n}\n";
    const ADR290_NOSPACE: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0]+=2\n  print(1)\n}\n";
    const ADR290_IDX_KEY: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  let i: Int = 1\n  v[i]  *=   3\n  print(1)\n}\n";
    const ADR290_MAP_ADD: &str = "module t\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  m[\"a\"] += 1\n  print(1)\n}\n";
    const ADR290_RHS_BINARY: &str = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  let a: Int = 1\n  let b: Int = 2\n  v[0] += a + b\n  print(1)\n}\n";
    const ADR290_PLUS_EXPR: &str = "module t\nfn main() -> Io<()> {\n  let a: Int = 1\n  let b: Int = 2\n  let c: Int = a + b\n  print(c)\n}\n";
    const ADR290_NO_PRINT: &str =
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] += 1\n}\n";
    const ADR290_RESIDUAL: [&str; 7] = [
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] /= 2\n  print(1)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] %= 2\n  print(1)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0][1] += 1\n  print(1)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0].f -= 1\n  print(1)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut n: Int = 0\n  n += 1\n  print(n)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] == 2\n  print(1)\n}\n",
        "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v[0] + = 2\n  print(1)\n}\n",
    ];

    fn adr290_index_assign(src: &str) -> (Option<BinOp>, Expr) {
        let m = parse(src).expect("compound index-assign must parse");
        let body = &m.functions[0].body;
        let found = body.iter().find_map(|s| match s {
            Stmt::IndexAssign { op, value, .. } => Some((op.clone(), value.clone())),
            _ => None,
        });
        found.expect("IndexAssign in body")
    }

    #[test]
    fn adr290_compound_ops_parse_with_op() {
        assert_eq!(
            adr290_index_assign(ADR290_ADD),
            (Some(BinOp::Add), Expr::LitInt(2))
        );
        assert_eq!(
            adr290_index_assign(ADR290_SUB),
            (Some(BinOp::Sub), Expr::LitInt(2))
        );
        assert_eq!(
            adr290_index_assign(ADR290_MUL),
            (Some(BinOp::Mul), Expr::LitInt(2))
        );
    }

    #[test]
    fn adr290_plain_assign_has_no_op() {
        assert_eq!(adr290_index_assign(ADR290_PLAIN), (None, Expr::LitInt(2)));
    }

    #[test]
    fn adr290_spacing_and_ident_key_variants_parse() {
        assert_eq!(
            adr290_index_assign(ADR290_NOSPACE),
            (Some(BinOp::Add), Expr::LitInt(2))
        );
        assert_eq!(
            adr290_index_assign(ADR290_IDX_KEY),
            (Some(BinOp::Mul), Expr::LitInt(3))
        );
    }

    #[test]
    fn adr290_parser_does_not_decide_receiver_type_for_compound() {
        // Map receiver: the parser accepts; HIR emits the E0006 (D6).
        let (op, _) = adr290_index_assign(ADR290_MAP_ADD);
        assert_eq!(op, Some(BinOp::Add));
    }

    #[test]
    fn adr290_span_covers_whole_compound_statement() {
        let m = parse(ADR290_ADD).expect("parse");
        let start = ADR290_ADD.find("v[0] += 2").expect("stmt");
        let want = Span {
            start,
            end: start + "v[0] += 2".len(),
        };
        assert!(matches!(
            &m.functions[0].body[1],
            Stmt::IndexAssign { span, .. } if *span == want
        ));
    }

    #[test]
    fn adr290_unsupported_ops_and_residual_forms_stay_parser_e0006() {
        for src in ADR290_RESIDUAL {
            let e = parse(src).expect_err("residual form must reject in the parser");
            assert_eq!(code_of(e), "E0006", "{src}");
        }
    }

    #[test]
    fn adr290_rhs_binary_expr_and_plain_plus_unchanged() {
        let (op, value) = adr290_index_assign(ADR290_RHS_BINARY);
        assert_eq!(op, Some(BinOp::Add));
        assert!(matches!(value, Expr::Binary { op: BinOp::Add, .. }));
        // `a + b` (ADR-045) parses exactly as before: a Binary Add in a `let` init.
        let m = parse(ADR290_PLUS_EXPR).expect("a + b parses");
        assert!(matches!(
            &m.functions[0].body[2],
            Stmt::Let {
                init: Expr::Binary { op: BinOp::Add, .. },
                ..
            }
        ));
    }

    #[test]
    fn adr290_deferred_shape_still_defers_behind_compound() {
        let m =
            parse(ADR290_NO_PRINT).expect("main shape E0001 is deferred behind the IndexAssign");
        assert!(adr283_has_marker(&m));
    }
}
