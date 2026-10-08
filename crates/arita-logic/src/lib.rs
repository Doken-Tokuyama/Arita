//! ARITA F3 positive-Datalog logic island (ADR-012 / CUT LOGIC-ENGINE-V0-20260913).
//! Own naive fixpoint engine. Tau/TML inspiration only — zero IDNI code/link/submodule.
//! Positive Datalog only: facts + Horn rules + queries. No negation/aggregation/temporal.

use pest::Parser;
use pest_derive::Parser;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "logic.pest"]
struct LogicParser;

// ─── AST ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Const(String),
    Var(String),
}

impl Term {
    pub fn from_ident(s: &str) -> Self {
        let first = s.chars().next().unwrap_or('_');
        if first.is_ascii_uppercase() {
            Term::Var(s.to_string())
        } else {
            Term::Const(s.to_string())
        }
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Term::Var(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Term::Const(s) | Term::Var(s) => s,
        }
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Atom {
    pub pred: String,
    pub args: Vec<Term>,
}

impl Atom {
    pub fn arity(&self) -> usize {
        self.args.len()
    }

    pub fn is_ground(&self) -> bool {
        self.args.iter().all(|t| !t.is_var())
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.pred)?;
        for (i, a) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{a}")?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HornRule {
    pub head: Atom,
    pub body: Vec<Atom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicModule {
    pub name: String,
    pub specs: Vec<Spec>,
    pub facts: Vec<Atom>,
    pub rules: Vec<HornRule>,
    pub queries: Vec<Atom>,
}

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicError {
    pub code: &'static str,
    pub message: String,
}

impl LogicError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for LogicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for LogicError {}

/// E0301 query failed · E0302 unsatisfiable · E0303 unknown pred/arity · E0304 construct not allowed
pub const E0301: &str = "E0301";
pub const E0302: &str = "E0302";
pub const E0303: &str = "E0303";
pub const E0304: &str = "E0304";

// ─── Parse ────────────────────────────────────────────────────────────────────

fn lower_term(pair: pest::iterators::Pair<Rule>) -> Term {
    // term → ident
    let inner = pair.into_inner().next().expect("term has ident");
    Term::from_ident(inner.as_str())
}

fn lower_atom(pair: pest::iterators::Pair<Rule>) -> Atom {
    let mut inner = pair.into_inner();
    let pred = inner.next().expect("atom pred").as_str().to_string();
    let mut args = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::term => args.push(lower_term(p)),
            other => unreachable!("unexpected in atom: {other:?}"),
        }
    }
    Atom { pred, args }
}

fn lower_fact(pair: pest::iterators::Pair<Rule>) -> Atom {
    let atom_pair = pair.into_inner().next().expect("fact atom");
    lower_atom(atom_pair)
}

fn lower_rule(pair: pest::iterators::Pair<Rule>) -> HornRule {
    let mut inner = pair.into_inner();
    let head = lower_atom(inner.next().expect("rule head"));
    let body_pair = inner.next().expect("rule body");
    let body: Vec<Atom> = body_pair.into_inner().map(lower_atom).collect();
    HornRule { head, body }
}

fn lower_query(pair: pest::iterators::Pair<Rule>) -> Atom {
    let atom_pair = pair.into_inner().next().expect("query atom");
    lower_atom(atom_pair)
}

fn lower_spec(pair: pest::iterators::Pair<Rule>) -> Spec {
    let name = pair
        .into_inner()
        .next()
        .expect("spec name")
        .as_str()
        .to_string();
    Spec { name }
}

/// Detect forbidden F1/F2 / non-v0 constructs for E0304.
fn classify_forbidden(src: &str) -> Option<LogicError> {
    // Line-oriented keyword scan (comments stripped roughly).
    for (i, line) in src.lines().enumerate() {
        let trimmed = line.split("//").next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }
        // leading keyword
        let first = trimmed.split_whitespace().next().unwrap_or("");
        match first {
            "fn" | "test" | "let" | "print" | "assert" => {
                return Some(LogicError::new(
                    E0304,
                    format!(
                        "logic construct not allowed in F3 v0: `{first}` (line {})",
                        i + 1
                    ),
                ));
            }
            "not" | "forall" | "exists" => {
                return Some(LogicError::new(
                    E0304,
                    format!(
                        "logic construct not allowed in F3 v0: `{first}` (line {})",
                        i + 1
                    ),
                ));
            }
            _ => {}
        }
        // negation / aggregation hints
        if trimmed.contains("\\+") || trimmed.contains(":- not ") {
            return Some(LogicError::new(
                E0304,
                format!("negation not allowed in F3 v0 (line {})", i + 1),
            ));
        }
        if trimmed.starts_with("main")
            || trimmed.contains("fn main")
            || (trimmed.starts_with("fn ") && trimmed.contains("main"))
        {
            return Some(LogicError::new(
                E0304,
                format!(
                    "logic construct not allowed in F3 v0: `main`/`fn` (line {})",
                    i + 1
                ),
            ));
        }
    }
    None
}

/// Parse a logic-only `.arita` source into a [`LogicModule`].
/// Parse failures / forbidden constructs → E0303 / E0304 as appropriate.
pub fn parse_logic(src: &str) -> Result<LogicModule, LogicError> {
    if let Some(err) = classify_forbidden(src) {
        return Err(err);
    }

    let mut pairs = LogicParser::parse(Rule::logic_file, src).map_err(|e| {
        // Re-check forbidden if pest failed (e.g. `fn` in file).
        if let Some(err) = classify_forbidden(src) {
            return err;
        }
        let msg = e.to_string();
        // Unknown / malformed → E0304 (construct not allowed / outside surface)
        // unless it looks like an arity/predicate issue after partial parse.
        if msg.contains("expected") || msg.contains("expected one of") {
            LogicError::new(
                E0304,
                format!("logic construct not allowed in F3 v0 / parse failure: {msg}"),
            )
        } else {
            LogicError::new(E0304, format!("parse failure: {msg}"))
        }
    })?;

    let file = pairs.next().expect("logic_file");
    let mut name = String::new();
    let mut specs = Vec::new();
    let mut facts = Vec::new();
    let mut rules = Vec::new();
    let mut queries = Vec::new();

    for part in file.into_inner() {
        match part.as_rule() {
            Rule::module_header => {
                let id = part.into_inner().next().expect("module ident");
                name = id.as_str().to_string();
            }
            Rule::item => {
                let inner = part.into_inner().next().expect("item inner");
                match inner.as_rule() {
                    Rule::spec_item => specs.push(lower_spec(inner)),
                    Rule::fact_item => facts.push(lower_fact(inner)),
                    Rule::rule_item => rules.push(lower_rule(inner)),
                    Rule::query_item => queries.push(lower_query(inner)),
                    other => {
                        return Err(LogicError::new(E0304, format!("unexpected item {other:?}")));
                    }
                }
            }
            Rule::EOI => {}
            other => {
                return Err(LogicError::new(
                    E0304,
                    format!("unexpected top-level {other:?}"),
                ));
            }
        }
    }

    if name.is_empty() {
        return Err(LogicError::new(E0304, "missing module name"));
    }

    // Facts must be ground in v0.
    for f in &facts {
        if !f.is_ground() {
            return Err(LogicError::new(
                E0304,
                format!("fact must be ground in F3 v0: {f}"),
            ));
        }
    }

    Ok(LogicModule {
        name,
        specs,
        facts,
        rules,
        queries,
    })
}

// ─── Engine (naive positive Datalog fixpoint) ─────────────────────────────────

type Subst = HashMap<String, String>;

fn apply_term(t: &Term, subst: &Subst) -> Term {
    match t {
        Term::Const(c) => Term::Const(c.clone()),
        Term::Var(v) => {
            if let Some(c) = subst.get(v) {
                Term::Const(c.clone())
            } else {
                Term::Var(v.clone())
            }
        }
    }
}

fn apply_atom(a: &Atom, subst: &Subst) -> Atom {
    Atom {
        pred: a.pred.clone(),
        args: a.args.iter().map(|t| apply_term(t, subst)).collect(),
    }
}

/// Unify a pattern atom against a ground fact; extend subst. Returns None on fail.
fn unify(pattern: &Atom, fact: &Atom, subst: &Subst) -> Option<Subst> {
    if pattern.pred != fact.pred || pattern.arity() != fact.arity() {
        return None;
    }
    let mut s = subst.clone();
    for (p, f) in pattern.args.iter().zip(fact.args.iter()) {
        let f_c = match f {
            Term::Const(c) => c.clone(),
            Term::Var(_) => return None, // fact must be ground
        };
        match apply_term(p, &s) {
            Term::Const(c) => {
                if c != f_c {
                    return None;
                }
            }
            Term::Var(v) => {
                s.insert(v, f_c);
            }
        }
    }
    Some(s)
}

/// Match body atoms against the fact set; yield all successful substitutions.
fn match_body(body: &[Atom], facts: &BTreeSet<Atom>) -> Vec<Subst> {
    let mut current: Vec<Subst> = vec![HashMap::new()];
    for atom in body {
        let mut next = Vec::new();
        for subst in &current {
            let pat = apply_atom(atom, subst);
            for fact in facts {
                if let Some(s2) = unify(&pat, fact, subst) {
                    next.push(s2);
                }
            }
        }
        current = next;
        if current.is_empty() {
            break;
        }
    }
    current
}

/// Collect declared predicate arities; error on conflict → E0303.
fn check_arities(module: &LogicModule) -> Result<BTreeMap<String, usize>, LogicError> {
    let mut arities: BTreeMap<String, usize> = BTreeMap::new();
    let mut observe = |atom: &Atom| -> Result<(), LogicError> {
        if let Some(&a) = arities.get(&atom.pred) {
            if a != atom.arity() {
                return Err(LogicError::new(
                    E0303,
                    format!(
                        "unknown predicate or arity: `{}` used with arity {} and {}",
                        atom.pred,
                        a,
                        atom.arity()
                    ),
                ));
            }
        } else {
            arities.insert(atom.pred.clone(), atom.arity());
        }
        Ok(())
    };
    for f in &module.facts {
        observe(f)?;
    }
    for r in &module.rules {
        observe(&r.head)?;
        for b in &r.body {
            observe(b)?;
        }
    }
    for q in &module.queries {
        // Query predicate must already be known (from facts/rules), else E0303.
        if let Some(&a) = arities.get(&q.pred) {
            if a != q.arity() {
                return Err(LogicError::new(
                    E0303,
                    format!(
                        "unknown predicate or arity: query `{}` arity {} ≠ declared {}",
                        q.pred,
                        q.arity(),
                        a
                    ),
                ));
            }
        } else {
            return Err(LogicError::new(
                E0303,
                format!("unknown predicate or arity: `{}`/{}", q.pred, q.arity()),
            ));
        }
    }
    Ok(arities)
}

/// Compute least Herbrand model (naive fixpoint) for positive Datalog.
pub fn fixpoint(module: &LogicModule) -> Result<BTreeSet<Atom>, LogicError> {
    let _arities = check_arities(module)?;
    let mut facts: BTreeSet<Atom> = module.facts.iter().cloned().collect();
    loop {
        let mut added = false;
        for rule in &module.rules {
            for subst in match_body(&rule.body, &facts) {
                let derived = apply_atom(&rule.head, &subst);
                if !derived.is_ground() {
                    // Unsafe rule (head var not bound) — reject as E0304
                    return Err(LogicError::new(
                        E0304,
                        format!("unsafe rule (non-ground head after body): {derived}"),
                    ));
                }
                if facts.insert(derived) {
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }
    Ok(facts)
}

/// Does the (possibly open) query atom hold against the derived fact set?
fn query_holds(query: &Atom, facts: &BTreeSet<Atom>) -> bool {
    if query.is_ground() {
        return facts.contains(query);
    }
    // Existential: any fact that unifies.
    for fact in facts {
        if unify(query, fact, &HashMap::new()).is_some() {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    pub query: Atom,
    pub sat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalResult {
    pub module_name: String,
    pub derived: BTreeSet<Atom>,
    pub queries: Vec<QueryResult>,
    /// True iff every query is sat. Empty queries → false (skip ≠ ok).
    pub all_sat: bool,
}

/// Run the engine: fixpoint + evaluate all queries.
/// Empty query list is **not** success (anti-theater: skip ≠ accepted).
pub fn run_queries(module: &LogicModule) -> Result<EvalResult, LogicError> {
    let derived = fixpoint(module)?;
    let mut qresults = Vec::new();
    for q in &module.queries {
        let sat = query_holds(q, &derived);
        qresults.push(QueryResult {
            query: q.clone(),
            sat,
        });
    }
    let all_sat = !qresults.is_empty() && qresults.iter().all(|r| r.sat);
    Ok(EvalResult {
        module_name: module.name.clone(),
        derived,
        queries: qresults,
        all_sat,
    })
}

/// Evaluate: if any query fails → Err(E0301). Empty queries → Err (skip ≠ ok).
/// Pure positive Datalog has no contradiction; E0302 reserved for future.
pub fn eval_module(module: &LogicModule) -> Result<EvalResult, LogicError> {
    let result = run_queries(module)?;
    if result.queries.is_empty() {
        return Err(LogicError::new(
            E0301,
            "query failed: no queries in module (skip ≠ ok)",
        ));
    }
    for qr in &result.queries {
        if !qr.sat {
            return Err(LogicError::new(
                E0301,
                format!("query failed: {}", qr.query),
            ));
        }
    }
    Ok(result)
}

/// Parse + eval a source string. On query fail returns Err with E0301.
pub fn eval_source(src: &str) -> Result<EvalResult, LogicError> {
    let module = parse_logic(src)?;
    eval_module(&module)
}

/// Read file, parse, eval.
pub fn eval_file(path: impl AsRef<Path>) -> Result<EvalResult, LogicError> {
    let path = path.as_ref();
    let src = fs::read_to_string(path)
        .map_err(|e| LogicError::new(E0304, format!("cannot read {}: {e}", path.display())))?;
    eval_source(&src)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const PATH_OK: &str = r#"
module demo_logic
spec Reachability { }
fact edge(a, b)
fact edge(b, c)
rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)
query path(a, c)
"#;

    const PATH_FAIL: &str = r#"
module demo_logic
spec Reachability { }
fact edge(a, b)
fact edge(b, c)
rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)
query path(c, a)
"#;

    const PATH_BOTH: &str = r#"
module demo_logic
spec Reachability { }
fact edge(a, b)
fact edge(b, c)
rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)
query path(a, c)
query path(c, a)
"#;

    #[test]
    fn parse_path_example() {
        let m = parse_logic(PATH_OK).expect("parse ok");
        assert_eq!(m.name, "demo_logic");
        assert_eq!(m.specs.len(), 1);
        assert_eq!(m.specs[0].name, "Reachability");
        assert_eq!(m.facts.len(), 2);
        assert_eq!(m.rules.len(), 2);
        assert_eq!(m.queries.len(), 1);
        assert_eq!(m.queries[0].pred, "path");
    }

    #[test]
    fn path_ok_sat() {
        let m = parse_logic(PATH_OK).unwrap();
        let r = eval_module(&m).expect("path(a,c) must succeed");
        assert!(r.all_sat);
        assert!(r.queries[0].sat);
        // Derived should contain path(a,c), path(a,b), path(b,c)
        let has = |pred: &str, a: &str, b: &str| {
            r.derived.contains(&Atom {
                pred: pred.into(),
                args: vec![Term::Const(a.into()), Term::Const(b.into())],
            })
        };
        assert!(has("path", "a", "c"));
        assert!(has("path", "a", "b"));
        assert!(has("path", "b", "c"));
        assert!(has("edge", "a", "b"));
    }

    #[test]
    fn path_fail_e0301() {
        let m = parse_logic(PATH_FAIL).unwrap();
        let err = eval_module(&m).expect_err("path(c,a) must fail");
        assert_eq!(err.code, E0301);
        assert!(err.message.contains("path(c, a)") || err.message.contains("path(c,a)"));
    }

    #[test]
    fn path_both_fails_on_second() {
        let m = parse_logic(PATH_BOTH).unwrap();
        let err = eval_module(&m).expect_err("mixed queries → fail");
        assert_eq!(err.code, E0301);
    }

    #[test]
    fn run_queries_reports_per_query() {
        let m = parse_logic(PATH_BOTH).unwrap();
        let r = run_queries(&m).unwrap();
        assert_eq!(r.queries.len(), 2);
        assert!(r.queries[0].sat);
        assert!(!r.queries[1].sat);
        assert!(!r.all_sat);
    }

    #[test]
    fn forbidden_fn_e0304() {
        let src = r#"
module bad
fn main() -> Io<()> {
  print("no")
}
"#;
        let err = parse_logic(src).expect_err("fn forbidden");
        assert_eq!(err.code, E0304);
    }

    #[test]
    fn unknown_pred_e0303() {
        let src = r#"
module u
fact edge(a, b)
query missing(a)
"#;
        let m = parse_logic(src).unwrap();
        let err = eval_module(&m).expect_err("unknown pred");
        assert_eq!(err.code, E0303);
    }

    #[test]
    fn arity_mismatch_e0303() {
        let src = r#"
module u
fact edge(a, b)
fact edge(a, b, c)
query edge(a, b)
"#;
        let m = parse_logic(src).unwrap();
        let err = fixpoint(&m).expect_err("arity");
        assert_eq!(err.code, E0303);
    }

    #[test]
    fn empty_queries_not_ok() {
        let src = r#"
module empty_q
fact edge(a, b)
"#;
        let m = parse_logic(src).unwrap();
        let err = eval_module(&m).expect_err("no queries");
        assert_eq!(err.code, E0301);
    }

    #[test]
    fn eval_source_ok() {
        let r = eval_source(PATH_OK).unwrap();
        assert!(r.all_sat);
    }
}
