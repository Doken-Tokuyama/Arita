//! `arita measure` v0 — real rustc oracles + required clippy workspace oracle +
//! E0xxx negative oracles (E0201/E0202/E021x incl. E0214–E0217/E0220–E0227/E0240–E0242/E0270/E0272; accepted iff parse_lower_check fails with matching code) +
//! F3 logic oracles (real arita-logic engine; not rustc) +
//! CONTRACT-V0 JSON contract oracles (`arita contract` / ADR-017) +
//! ATTEST-V0 source_sha256 oracles (`arita attest verify` / ADR-018) +
//! LANG-CONTRACT language-level `contract { }` oracles (ADR-019) +
//! CONTRACT-TARGET-FN `target` entry-rewrite oracles (ADR-020) +
//! MIRI-ORACLE workspace lib tests (ADR-021; missing → inconclusive).
//! PERF-V0 oracles (ADR-028): release run + Cargo.toml opt-level evidence + E0250 CLI neg.
//! DEPS-V0 oracles (ADR-029): tokio bridge E2E + E0260 unknown crate + E0261 crate path.
//! HOST-BORDER-HELLO oracles (ADR-035/033b): host.mark() bridge E2E → stdout host-ok.
//! BOOTSTRAP-V0/02/03 oracles (ADR-037/042/057): dual-oracle pure fns (`bootstrap-01` fact5→120,
//! `bootstrap-02` sum5→15, `bootstrap-03` fib6→8, `bootstrap-04` fib7→13, `bootstrap-05` fib8→21, `bootstrap-06` fib9→34, `bootstrap-07` fib10→55, `bootstrap-08` fib11→89, `bootstrap-09` fib12→144, `bootstrap-10` fib13→233, `bootstrap-11` fib14→377, `bootstrap-12` fib15→610, `bootstrap-13` fib16→987, `bootstrap-14` fib17→1597, `bootstrap-15` fib18→2584, `bootstrap-16` fib19→4181, `bootstrap-17` fib20→6765, `bootstrap-18` fib21→10946, `bootstrap-19` fib22→17711, `bootstrap-20` fib23→28657, `bootstrap-21` fib24→46368, `bootstrap-22` fib25→75025, `bootstrap-23` fib26→121393, `bootstrap-24` fib27→196418, `bootstrap-25` fib28→317811, `bootstrap-26` fib29→514229, `bootstrap-27` fib30→832040, `bootstrap-28` fib31→1346269, `bootstrap-29` fib32→2178309, `bootstrap-30` fib33→3524578, `bootstrap-31` fib34→5702887, `bootstrap-32` fib35→9227465, `bootstrap-33` fib36→14930352, `bootstrap-34` fib37→24157817, `bootstrap-35` fib38→39088169, `bootstrap-36` fib39→63245986, `bootstrap-37` fib40→102334155, `bootstrap-38` fib41→165580141, `bootstrap-39` fib42→267914296, `bootstrap-40` fib43→433494437, `bootstrap-41` fib44→701408733, `bootstrap-42` fib45→1134903170, `bootstrap-43` fib46→1836311903, `bootstrap-44` fib47→2971215073, `bootstrap-45` fib48→4807526976, `bootstrap-46` fib49→7778742049, `bootstrap-47` fib50→12586269025, `bootstrap-48` fib51→20365011074, `bootstrap-49` fib52→32951280099, `bootstrap-50` fib53→53316291173, `bootstrap-51` fib54→86267571272, `bootstrap-52` fib55→139583862445, `bootstrap-53` fib56→225851433717, `bootstrap-54` fib57→365435296162, `bootstrap-55` fib58→591286729879, `bootstrap-56` fib59→956722026041); Rust reference in `bootstrap` module (`cargo test`); no CLI bootstrap-check.
//! TARGETS-V0 oracles (ADR-034): gated cross `--target` build; missing toolchain/linker →
//! inconclusive (ids `target-*`); gated inconclusive does **not** tumble overall (≠ fake PASS).
//! Anti-theater: skip/missing/timeout ≠ PASS; verdicts only rejected|inconclusive|accepted.

use crate::build;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const MEASURE_VERSION: &str = "0.1.0-f1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Rejected,
    Inconclusive,
    Accepted,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Rejected => "rejected",
            Verdict::Inconclusive => "inconclusive",
            Verdict::Accepted => "accepted",
        }
    }
}

#[derive(Debug)]
pub struct OracleResult {
    pub id: String,
    pub path: String,
    pub verdict: Verdict,
    pub detail: String,
}

/// Declared ejemplos oracles (from ejemplos/README.md table; embedded, not scraped).
pub struct EjemploOracle {
    pub id: &'static str,
    pub path: &'static str,
    pub expected: &'static [&'static str],
    /// When true, accept only if build+run stdout matches AND `arita test` PASS.
    pub require_test: bool,
}

pub const EJEMPLO_ORACLES: &[EjemploOracle] = &[
    EjemploOracle {
        id: "01-hello",
        path: "ejemplos/01-hello.arita",
        expected: &["hello"],
        require_test: false,
    },
    EjemploOracle {
        id: "02-hello-lines",
        path: "ejemplos/02-hello-lines.arita",
        expected: &["line one", "line two", "line three"],
        require_test: false,
    },
    EjemploOracle {
        id: "03-greet",
        path: "ejemplos/03-greet.arita",
        expected: &["hola ARITA"],
        require_test: false,
    },
    EjemploOracle {
        id: "04-multi-print",
        path: "ejemplos/04-multi-print.arita",
        expected: &["one", "two", "three"],
        require_test: false,
    },
    EjemploOracle {
        id: "05-with-comments",
        path: "ejemplos/05-with-comments.arita",
        expected: &["alpha", "beta"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-01-let-int",
        path: "ejemplos/f2/01-let-int.arita",
        expected: &["42", "7"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-02-vec-len",
        path: "ejemplos/f2/02-vec-len.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-03-string",
        path: "ejemplos/f2/03-string.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-04-int-arith",
        path: "ejemplos/f2/04-int-arith.arita",
        expected: &["5"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-05-bool",
        path: "ejemplos/f2/05-bool.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-06-fn-call",
        path: "ejemplos/f2/06-fn-call.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-07-assert",
        path: "ejemplos/f2/07-assert.arita",
        expected: &["ok"],
        require_test: true,
    },
    EjemploOracle {
        id: "f2-08-string-len",
        path: "ejemplos/f2/08-string-len.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "f2-09-is-empty",
        path: "ejemplos/f2/09-is-empty.arita",
        expected: &["true", "false", "false"],
        require_test: false,
    },
    EjemploOracle {
        id: "f21-01-if-true",
        path: "ejemplos/f2.1/01-if-true.arita",
        expected: &["yes"],
        require_test: false,
    },
    EjemploOracle {
        id: "f21-02-if-else",
        path: "ejemplos/f2.1/02-if-else.arita",
        expected: &["no"],
        require_test: false,
    },
    EjemploOracle {
        id: "f21-03-while-count",
        path: "ejemplos/f2.1/03-while-count.arita",
        expected: &["0", "1", "2"],
        require_test: false,
    },
    EjemploOracle {
        id: "f21-04-cmp-if",
        path: "ejemplos/f2.1/04-cmp-if.arita",
        expected: &["yes"],
        require_test: false,
    },
    EjemploOracle {
        id: "f21-05-cmp-let",
        path: "ejemplos/f2.1/05-cmp-let.arita",
        expected: &["eq"],
        require_test: false,
    },
    EjemploOracle {
        id: "f22-01-match-bool",
        path: "ejemplos/f2.2/01-match-bool.arita",
        expected: &["yes"],
        require_test: false,
    },
    EjemploOracle {
        id: "f22-02-match-int",
        path: "ejemplos/f2.2/02-match-int.arita",
        expected: &["a"],
        require_test: false,
    },
    EjemploOracle {
        id: "f22-03-match-bool-wild",
        path: "ejemplos/f2.2/03-match-bool-wild.arita",
        expected: &["other"],
        require_test: false,
    },
    EjemploOracle {
        id: "f22-04-match-expr-bool",
        path: "ejemplos/f2.2/04-match-expr-bool.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "f22-05-match-expr-int",
        path: "ejemplos/f2.2/05-match-expr-int.arita",
        expected: &["10"],
        require_test: false,
    },
    EjemploOracle {
        id: "f23-01-while-break",
        path: "ejemplos/f2.3/01-while-break.arita",
        expected: &["0", "1"],
        require_test: false,
    },
    EjemploOracle {
        id: "f23-02-while-continue",
        path: "ejemplos/f2.3/02-while-continue.arita",
        expected: &["0", "2", "4"],
        require_test: false,
    },
    EjemploOracle {
        id: "async-01-await-greet",
        path: "ejemplos/async/01-await-greet.arita",
        expected: &["hi", "done"],
        require_test: false,
    },
    EjemploOracle {
        id: "async-02-async-print",
        path: "ejemplos/async/02-async-print.arita",
        expected: &["ok"],
        require_test: false,
    },
    // ADR-242 / CORE-0.2-ASYNC-SURFACE-20260920
    EjemploOracle {
        id: "core02-async-text",
        path: "ejemplos/core02/01-async-text.arita",
        expected: &["hi"],
        require_test: false,
    },
    // ADR-243 / CORE-0.2-TASK-SPAWN-20260920
    EjemploOracle {
        id: "core02-spawn-join",
        path: "ejemplos/core02/02-spawn-join.arita",
        expected: &["w", "done"],
        require_test: false,
    },
    // ADR-244 / CORE-0.2-TIMEOUT-CANCEL-20260920
    EjemploOracle {
        id: "core02-timeout-ok-fast",
        path: "ejemplos/core02/01-timeout-ok-fast.arita",
        expected: &["fast", "ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-timeout-err-elapsed",
        path: "ejemplos/core02/02-timeout-err-elapsed.arita",
        expected: &["timeout"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-cancel-cooperative",
        path: "ejemplos/core02/03-cancel-cooperative.arita",
        expected: &["cancelled-ok"],
        require_test: false,
    },
    // ADR-246 / CORE-0.2-SERVICE-POLICY-20260920
    EjemploOracle {
        id: "core02-policy-defaults",
        path: "ejemplos/core02/http/05-policy-defaults.arita",
        expected: &["1048576", "65536", "100", "30000", "policy-defaults-ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-policy-set-ok",
        path: "ejemplos/core02/http/06-policy-set.arita",
        expected: &["4096", "8192", "50", "5000", "policy-set-ok"],
        require_test: false,
    },
    // ADR-247 / CORE-0.2-SCENARIO-HTTP-20260920
    EjemploOracle {
        id: "core02-scen-health",
        path: "ejemplos/core02/http/scenarios/01-scen-health.arita",
        expected: &["19871", "ready", "200", "ok", "scen-health-done"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-scen-echo",
        path: "ejemplos/core02/http/scenarios/02-scen-echo.arita",
        expected: &["19872", "ready", "200", "hello-echo", "scen-echo-done"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-scen-body-cap",
        path: "ejemplos/core02/http/scenarios/03-scen-body-cap.arita",
        expected: &[
            "19873",
            "ready",
            "413",
            "body too large",
            "scen-body-cap-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-scen-shutdown",
        path: "ejemplos/core02/http/scenarios/04-scen-shutdown.arita",
        expected: &["19874", "ready", "200", "scen-shutdown-ok"],
        require_test: false,
    },
    // ADR-248 / CORE-0.2-REF-HTTP-DAEMON-20260920
    EjemploOracle {
        id: "core02-ref-health",
        path: "ejemplos/core02/ref-http-json/scen-health.arita",
        expected: &["19911", "ready", "200", "{\"ok\":true}", "ref-health-done"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-ref-echo",
        path: "ejemplos/core02/ref-http-json/scen-echo.arita",
        expected: &["19912", "ready", "200", "ref-echo-payload", "ref-echo-done"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-ref-body-cap",
        path: "ejemplos/core02/ref-http-json/scen-body-cap.arita",
        expected: &["19913", "ready", "413", "ref-body-cap-done"],
        require_test: false,
    },
    EjemploOracle {
        id: "core02-ref-shutdown",
        path: "ejemplos/core02/ref-http-json/scen-shutdown.arita",
        expected: &["19914", "ready", "200", "ref-shutdown-ok"],
        require_test: false,
    },
    // ADR-249 / CORE-0.3-CLIENT-COMPOSE-20260920
    EjemploOracle {
        id: "core03-compose-smoke-roundtrip",
        path: "ejemplos/core03/client-compose/03-compose-smoke.arita",
        expected: &[
            "19921",
            "upstream-up",
            "19920",
            "gateway-up",
            "200",
            "compose-ping",
            "timeout-ok",
            "compose-timeout-ok",
            "compose-smoke-done",
        ],
        require_test: false,
    },
    // ADR-250 / CORE-0.3-ERROR-PROP-20260920
    EjemploOracle {
        id: "core03-err-upstream-down",
        path: "ejemplos/core03/error-prop/03-err-down-smoke.arita",
        expected: &[
            "19930",
            "gateway-up",
            "502",
            "upstream_unavailable",
            "err-down-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-err-upstream-4xx",
        path: "ejemplos/core03/error-prop/04-err-4xx-smoke.arita",
        expected: &[
            "upstream-404-up",
            "gateway-up",
            "404",
            "nope",
            "err-4xx-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-err-upstream-5xx",
        path: "ejemplos/core03/error-prop/05-err-5xx-smoke.arita",
        expected: &[
            "upstream-503-up",
            "gateway-up",
            "503",
            "busy",
            "err-5xx-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-err-timeout",
        path: "ejemplos/core03/error-prop/06-err-timeout-smoke.arita",
        expected: &["gateway-up", "504", "upstream_timeout", "err-timeout-done"],
        require_test: false,
    },
    // ADR-251 / CORE-0.3-SEQ-PIPELINE-20260920
    // neg-core03-pipe-parallel = DOC OUT (no wire / skip≠PASS)
    EjemploOracle {
        id: "core03-pipe-happy",
        path: "ejemplos/core03/seq-pipeline/01-pipe-happy-smoke.arita",
        expected: &[
            "step-a-up",
            "step-b-up",
            "pipe-gw-up",
            "200",
            "pipe-ping",
            "pipe-happy-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-pipe-fail-a",
        path: "ejemplos/core03/seq-pipeline/02-pipe-fail-a-smoke.arita",
        expected: &[
            "step-b-up",
            "pipe-gw-up",
            "502",
            "upstream_unavailable",
            "pipe-fail-a-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-pipe-fail-b",
        path: "ejemplos/core03/seq-pipeline/03-pipe-fail-b-smoke.arita",
        expected: &[
            "step-a-up",
            "pipe-gw-up",
            "502",
            "upstream_unavailable",
            "pipe-fail-b-done",
        ],
        require_test: false,
    },
    // ADR-252 / CORE-0.3-SCENARIO-COMPOSE-20260920
    EjemploOracle {
        id: "core03-scen-compose-happy",
        path: "ejemplos/core03/scenario-compose/01-scen-happy.arita",
        expected: &[
            "19951",
            "scen-a-up",
            "19952",
            "scen-b-up",
            "19950",
            "scen-gw-up",
            "200",
            "scen-compose-ping",
            "scen-compose-happy-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-scen-compose-fail-a",
        path: "ejemplos/core03/scenario-compose/02-scen-fail-a.arita",
        expected: &[
            "scen-b-up",
            "scen-gw-up",
            "502",
            "upstream_unavailable",
            "scen-compose-fail-a-done",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-scen-compose-fail-b",
        path: "ejemplos/core03/scenario-compose/03-scen-fail-b.arita",
        expected: &[
            "scen-a-up",
            "scen-gw-up",
            "502",
            "upstream_unavailable",
            "scen-compose-fail-b-done",
        ],
        require_test: false,
    },
    // ADR-253 / CORE-0.3-REF-COMPOSE-20260920
    EjemploOracle {
        id: "core03-ref-health",
        path: "ejemplos/core03/ref-http-compose/scen-health.arita",
        expected: &[
            "200",
            "{\"ok\":true,\"service\":\"arita-ref-http-compose\"}",
        ],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-ref-pipeline-happy",
        path: "ejemplos/core03/ref-http-compose/scen-pipeline-happy.arita",
        expected: &["200", "ref-compose-ping"],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-ref-fail-a",
        path: "ejemplos/core03/ref-http-compose/scen-fail-a.arita",
        expected: &["502", "upstream_unavailable"],
        require_test: false,
    },
    EjemploOracle {
        id: "core03-ref-fail-b",
        path: "ejemplos/core03/ref-http-compose/scen-fail-b.arita",
        expected: &["502", "upstream_unavailable"],
        require_test: false,
    },
    // ADR-254 / CORE-0.4-MULTI-MODULE-20260920
    EjemploOracle {
        id: "core04-mm-happy",
        path: "ejemplos/core04/multi-module/main.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-01",
        path: "ejemplos/bootstrap/01-fact.arita",
        expected: &["120"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-02",
        path: "ejemplos/bootstrap/02-sum.arita",
        expected: &["15"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-03",
        path: "ejemplos/bootstrap/03-fib.arita",
        expected: &["8"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-04",
        path: "ejemplos/bootstrap/04-fib7.arita",
        expected: &["13"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-05",
        path: "ejemplos/bootstrap/05-fib8.arita",
        expected: &["21"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-06",
        path: "ejemplos/bootstrap/06-fib9.arita",
        expected: &["34"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-07",
        path: "ejemplos/bootstrap/07-fib10.arita",
        expected: &["55"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-08",
        path: "ejemplos/bootstrap/08-fib11.arita",
        expected: &["89"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-09",
        path: "ejemplos/bootstrap/09-fib12.arita",
        expected: &["144"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-10",
        path: "ejemplos/bootstrap/10-fib13.arita",
        expected: &["233"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-11",
        path: "ejemplos/bootstrap/11-fib14.arita",
        expected: &["377"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-12",
        path: "ejemplos/bootstrap/12-fib15.arita",
        expected: &["610"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-13",
        path: "ejemplos/bootstrap/13-fib16.arita",
        expected: &["987"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-14",
        path: "ejemplos/bootstrap/14-fib17.arita",
        expected: &["1597"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-15",
        path: "ejemplos/bootstrap/15-fib18.arita",
        expected: &["2584"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-16",
        path: "ejemplos/bootstrap/16-fib19.arita",
        expected: &["4181"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-17",
        path: "ejemplos/bootstrap/17-fib20.arita",
        expected: &["6765"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-18",
        path: "ejemplos/bootstrap/18-fib21.arita",
        expected: &["10946"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-19",
        path: "ejemplos/bootstrap/19-fib22.arita",
        expected: &["17711"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-20",
        path: "ejemplos/bootstrap/20-fib23.arita",
        expected: &["28657"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-21",
        path: "ejemplos/bootstrap/21-fib24.arita",
        expected: &["46368"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-22",
        path: "ejemplos/bootstrap/22-fib25.arita",
        expected: &["75025"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-23",
        path: "ejemplos/bootstrap/23-fib26.arita",
        expected: &["121393"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-24",
        path: "ejemplos/bootstrap/24-fib27.arita",
        expected: &["196418"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-25",
        path: "ejemplos/bootstrap/25-fib28.arita",
        expected: &["317811"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-26",
        path: "ejemplos/bootstrap/26-fib29.arita",
        expected: &["514229"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-27",
        path: "ejemplos/bootstrap/27-fib30.arita",
        expected: &["832040"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-28",
        path: "ejemplos/bootstrap/28-fib31.arita",
        expected: &["1346269"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-29",
        path: "ejemplos/bootstrap/29-fib32.arita",
        expected: &["2178309"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-30",
        path: "ejemplos/bootstrap/30-fib33.arita",
        expected: &["3524578"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-31",
        path: "ejemplos/bootstrap/31-fib34.arita",
        expected: &["5702887"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-32",
        path: "ejemplos/bootstrap/32-fib35.arita",
        expected: &["9227465"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-33",
        path: "ejemplos/bootstrap/33-fib36.arita",
        expected: &["14930352"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-34",
        path: "ejemplos/bootstrap/34-fib37.arita",
        expected: &["24157817"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-35",
        path: "ejemplos/bootstrap/35-fib38.arita",
        expected: &["39088169"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-36",
        path: "ejemplos/bootstrap/36-fib39.arita",
        expected: &["63245986"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-37",
        path: "ejemplos/bootstrap/37-fib40.arita",
        expected: &["102334155"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-38",
        path: "ejemplos/bootstrap/38-fib41.arita",
        expected: &["165580141"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-39",
        path: "ejemplos/bootstrap/39-fib42.arita",
        expected: &["267914296"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-40",
        path: "ejemplos/bootstrap/40-fib43.arita",
        expected: &["433494437"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-41",
        path: "ejemplos/bootstrap/41-fib44.arita",
        expected: &["701408733"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-42",
        path: "ejemplos/bootstrap/42-fib45.arita",
        expected: &["1134903170"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-43",
        path: "ejemplos/bootstrap/43-fib46.arita",
        expected: &["1836311903"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-44",
        path: "ejemplos/bootstrap/44-fib47.arita",
        expected: &["2971215073"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-45",
        path: "ejemplos/bootstrap/45-fib48.arita",
        expected: &["4807526976"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-46",
        path: "ejemplos/bootstrap/46-fib49.arita",
        expected: &["7778742049"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-47",
        path: "ejemplos/bootstrap/47-fib50.arita",
        expected: &["12586269025"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-48",
        path: "ejemplos/bootstrap/48-fib51.arita",
        expected: &["20365011074"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-49",
        path: "ejemplos/bootstrap/49-fib52.arita",
        expected: &["32951280099"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-50",
        path: "ejemplos/bootstrap/50-fib53.arita",
        expected: &["53316291173"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-51",
        path: "ejemplos/bootstrap/51-fib54.arita",
        expected: &["86267571272"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-52",
        path: "ejemplos/bootstrap/52-fib55.arita",
        expected: &["139583862445"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-53",
        path: "ejemplos/bootstrap/53-fib56.arita",
        expected: &["225851433717"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-54",
        path: "ejemplos/bootstrap/54-fib57.arita",
        expected: &["365435296162"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-55",
        path: "ejemplos/bootstrap/55-fib58.arita",
        expected: &["591286729879"],
        require_test: false,
    },
    EjemploOracle {
        id: "bootstrap-56",
        path: "ejemplos/bootstrap/56-fib59.arita",
        expected: &["956722026041"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-extend",
        path: "ejemplos/f2/267-vec-extend.arita",
        expected: &["3", "2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-extend-empty",
        path: "ejemplos/f2/268-vec-extend-empty.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "result-01",
        path: "ejemplos/result/01-ok.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "result-02",
        path: "ejemplos/result/02-err.arita",
        expected: &["fail"],
        require_test: false,
    },
    EjemploOracle {
        id: "result-swallow-ok",
        path: "ejemplos/result/03-swallow-ok.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clear-vec",
        path: "ejemplos/f2/11-clear-vec.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clear-string",
        path: "ejemplos/f2/12-clear-string.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "option-01",
        path: "ejemplos/option/01-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "option-02",
        path: "ejemplos/option/02-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "option-swallow-ok",
        path: "ejemplos/option/03-swallow-ok.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-pop-some",
        path: "ejemplos/f2/13-pop-some.arita",
        expected: &["9"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-pop-none",
        path: "ejemplos/f2/14-pop-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clone-string",
        path: "ejemplos/f2/15-clone-string.arita",
        expected: &["hi", "hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clone-vec",
        path: "ejemplos/f2/16-clone-vec.arita",
        expected: &["0", "1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-push-str-lit",
        path: "ejemplos/f2/17-push-str-lit.arita",
        expected: &["hi!"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-push-str-string",
        path: "ejemplos/f2/18-push-str-string.arita",
        expected: &["hithere"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-contains-true",
        path: "ejemplos/f2/19-contains-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-contains-false",
        path: "ejemplos/f2/20-contains-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-starts-with-true",
        path: "ejemplos/f2/21-starts-with-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-starts-with-false",
        path: "ejemplos/f2/22-starts-with-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ends-with-true",
        path: "ejemplos/f2/23-ends-with-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ends-with-false",
        path: "ejemplos/f2/24-ends-with-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-spaces",
        path: "ejemplos/f2/25-trim-spaces.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-noop",
        path: "ejemplos/f2/26-trim-noop.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-replace-hit",
        path: "ejemplos/f2/27-replace-hit.arita",
        expected: &["hexxo"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-replace-miss",
        path: "ejemplos/f2/28-replace-miss.arita",
        expected: &["hello"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-to-uppercase",
        path: "ejemplos/f2/29-to-uppercase.arita",
        expected: &["HI"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-to-lowercase",
        path: "ejemplos/f2/30-to-lowercase.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-parse-int-ok",
        path: "ejemplos/f2/31-parse-int-ok.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-parse-int-err",
        path: "ejemplos/f2/32-parse-int-err.arita",
        expected: &["invalid integer"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-int-to-string",
        path: "ejemplos/f2/33-int-to-string.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-int-to-string-roundtrip",
        path: "ejemplos/f2/34-int-to-string-roundtrip.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-abs-pos",
        path: "ejemplos/f2/35-abs-pos.arita",
        expected: &["5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-abs-neg",
        path: "ejemplos/f2/36-abs-neg.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-bool-to-string-true",
        path: "ejemplos/f2/37-bool-to-string-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-bool-to-string-false",
        path: "ejemplos/f2/38-bool-to-string-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-min",
        path: "ejemplos/f2/39-min.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-max",
        path: "ejemplos/f2/40-max.arita",
        expected: &["5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clamp-in",
        path: "ejemplos/f2/41-clamp-in.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clamp-low",
        path: "ejemplos/f2/42-clamp-low.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-clamp-high",
        path: "ejemplos/f2/43-clamp-high.arita",
        expected: &["5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-start",
        path: "ejemplos/f2/44-trim-start.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-end",
        path: "ejemplos/f2/45-trim-end.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-strip-prefix-some",
        path: "ejemplos/f2/46-strip-prefix-some.arita",
        expected: &["llo"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-strip-prefix-none",
        path: "ejemplos/f2/47-strip-prefix-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-strip-suffix-some",
        path: "ejemplos/f2/48-strip-suffix-some.arita",
        expected: &["hel"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-repeat",
        path: "ejemplos/f2/49-repeat.arita",
        expected: &["ababab"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-repeat-zero",
        path: "ejemplos/f2/50-repeat-zero.arita",
        expected: &[""],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-repeat",
        path: "ejemplos/f2/51-vec-repeat.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-repeat-zero",
        path: "ejemplos/f2/52-vec-repeat-zero.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-pow",
        path: "ejemplos/f2/53-pow.arita",
        expected: &["1024"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-pow-zero-exp",
        path: "ejemplos/f2/54-pow-zero-exp.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-add-some",
        path: "ejemplos/f2/55-checked-add-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-add-none",
        path: "ejemplos/f2/56-checked-add-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-sub-some",
        path: "ejemplos/f2/57-checked-sub-some.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-sub-none",
        path: "ejemplos/f2/58-checked-sub-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-mul-some",
        path: "ejemplos/f2/59-checked-mul-some.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-mul-none",
        path: "ejemplos/f2/60-checked-mul-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-some",
        path: "ejemplos/f2/61-checked-div-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-none-zero",
        path: "ejemplos/f2/62-checked-div-none-zero.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-none-overflow",
        path: "ejemplos/f2/63-checked-div-none-overflow.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-rem-some",
        path: "ejemplos/f2/64-checked-rem-some.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-rem-none-zero",
        path: "ejemplos/f2/65-checked-rem-none-zero.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-rem-none-overflow",
        path: "ejemplos/f2/66-checked-rem-none-overflow.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-get-some",
        path: "ejemplos/f2/67-vec-get-some.arita",
        expected: &["9"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-get-none-oob",
        path: "ejemplos/f2/68-vec-get-none-oob.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-get-none-neg",
        path: "ejemplos/f2/69-vec-get-none-neg.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-first-some",
        path: "ejemplos/f2/70-vec-first-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-first-none",
        path: "ejemplos/f2/71-vec-first-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-last-some",
        path: "ejemplos/f2/72-vec-last-some.arita",
        expected: &["9"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-contains-true",
        path: "ejemplos/f2/73-vec-contains-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-contains-false",
        path: "ejemplos/f2/74-vec-contains-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-remove-some",
        path: "ejemplos/f2/75-vec-remove-some.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-remove-none",
        path: "ejemplos/f2/76-vec-remove-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-add-normal",
        path: "ejemplos/f2/77-saturating-add-normal.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-add-max",
        path: "ejemplos/f2/78-saturating-add-max.arita",
        expected: &["9223372036854775807"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-sub-normal",
        path: "ejemplos/f2/79-saturating-sub-normal.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-sub-min",
        path: "ejemplos/f2/80-saturating-sub-min.arita",
        expected: &["-9223372036854775808"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-mul-normal",
        path: "ejemplos/f2/81-saturating-mul-normal.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-mul-max",
        path: "ejemplos/f2/82-saturating-mul-max.arita",
        expected: &["9223372036854775807"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-unwrap-or-option-some",
        path: "ejemplos/f2/83-unwrap-or-option-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-unwrap-or-option-none",
        path: "ejemplos/f2/84-unwrap-or-option-none.arita",
        expected: &["99"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-unwrap-or-result-ok",
        path: "ejemplos/f2/85-unwrap-or-result-ok.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-unwrap-or-result-err",
        path: "ejemplos/f2/86-unwrap-or-result-err.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-some-true",
        path: "ejemplos/f2/87-is-some-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-none-true",
        path: "ejemplos/f2/88-is-none-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-ok-true",
        path: "ejemplos/f2/89-is-ok-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-err-true",
        path: "ejemplos/f2/90-is-err-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-add-normal",
        path: "ejemplos/f2/91-wrapping-add-normal.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-add-overflow",
        path: "ejemplos/f2/92-wrapping-add-overflow.arita",
        expected: &["-9223372036854775808"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-sub-normal",
        path: "ejemplos/f2/93-wrapping-sub-normal.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-sub-underflow",
        path: "ejemplos/f2/94-wrapping-sub-underflow.arita",
        expected: &["9223372036854775807"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-mul-normal",
        path: "ejemplos/f2/95-wrapping-mul-normal.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-mul-overflow",
        path: "ejemplos/f2/96-wrapping-mul-overflow.arita",
        expected: &["-2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-add-overflow-match",
        path: "ejemplos/f2/97-checked-add-overflow-match.arita",
        expected: &["overflow"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-add-overflow-assert",
        path: "ejemplos/f2/98-wrapping-add-overflow-assert.arita",
        expected: &["-9223372036854775808"],
        require_test: true,
    },
    EjemploOracle {
        id: "std-get-oob-iflet-miss",
        path: "ejemplos/f2/99-get-oob-iflet-miss.arita",
        expected: &["miss"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-signum-pos",
        path: "ejemplos/f2/100-signum-pos.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-signum-neg",
        path: "ejemplos/f2/101-signum-neg.arita",
        expected: &["-1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-signum-zero",
        path: "ejemplos/f2/102-signum-zero.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-positive-true",
        path: "ejemplos/f2/103-is-positive-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-negative-true",
        path: "ejemplos/f2/104-is-negative-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-sign-zero-false",
        path: "ejemplos/f2/105-is-sign-zero-false.arita",
        expected: &["false", "false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-neg-normal",
        path: "ejemplos/f2/106-wrapping-neg-normal.arita",
        expected: &["-5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-neg-min",
        path: "ejemplos/f2/107-wrapping-neg-min.arita",
        expected: &["-9223372036854775808"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-neg-min",
        path: "ejemplos/f2/108-saturating-neg-min.arita",
        expected: &["9223372036854775807"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-neg-some",
        path: "ejemplos/f2/109-checked-neg-some.arita",
        expected: &["-5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-neg-none",
        path: "ejemplos/f2/110-checked-neg-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-div-euclid",
        path: "ejemplos/f2/111-div-euclid.arita",
        expected: &["-2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-rem-euclid",
        path: "ejemplos/f2/112-rem-euclid.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-euclid-some",
        path: "ejemplos/f2/113-checked-div-euclid-some.arita",
        expected: &["-2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-euclid-none-zero",
        path: "ejemplos/f2/114-checked-div-euclid-none-zero.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-div-euclid-none-min",
        path: "ejemplos/f2/115-checked-div-euclid-none-min.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-find-some",
        path: "ejemplos/f2/116-find-some.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-find-none",
        path: "ejemplos/f2/117-find-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-rfind-some",
        path: "ejemplos/f2/118-rfind-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-swap-remove-some",
        path: "ejemplos/f2/119-swap-remove-some.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-swap-remove-none",
        path: "ejemplos/f2/120-swap-remove-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-midpoint-even",
        path: "ejemplos/f2/121-midpoint-even.arita",
        expected: &["5"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-midpoint-neg",
        path: "ejemplos/f2/122-midpoint-neg.arita",
        expected: &["-1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-multiple-of-true",
        path: "ejemplos/f2/123-is-multiple-of-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-multiple-of-false",
        path: "ejemplos/f2/124-is-multiple-of-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-multiple-of-zero-div",
        path: "ejemplos/f2/125-is-multiple-of-zero-div.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-truncate-shrink",
        path: "ejemplos/f2/126-truncate-shrink.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-truncate-noop",
        path: "ejemplos/f2/127-truncate-noop.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-resize-grow",
        path: "ejemplos/f2/128-resize-grow.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-resize-shrink",
        path: "ejemplos/f2/129-resize-shrink.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog2-some",
        path: "ejemplos/f2/130-ilog2-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog2-one",
        path: "ejemplos/f2/131-ilog2-one.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog2-none",
        path: "ejemplos/f2/132-ilog2-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog10-some",
        path: "ejemplos/f2/133-ilog10-some.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog10-one",
        path: "ejemplos/f2/134-ilog10-one.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog10-none",
        path: "ejemplos/f2/135-ilog10-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-count-ones-zero",
        path: "ejemplos/f2/136-count-ones-zero.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-count-ones-seven",
        path: "ejemplos/f2/137-count-ones-seven.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-count-zeros-zero",
        path: "ejemplos/f2/140-count-zeros-zero.arita",
        expected: &["64"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-count-zeros-neg1",
        path: "ejemplos/f2/141-count-zeros-neg1.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-leading-zeros-one",
        path: "ejemplos/f2/142-leading-zeros-one.arita",
        expected: &["63"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trailing-zeros-eight",
        path: "ejemplos/f2/143-trailing-zeros-eight.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-leading-zeros-zero",
        path: "ejemplos/f2/144-leading-zeros-zero.arita",
        expected: &["64"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-reverse-bits-one",
        path: "ejemplos/f2/145-reverse-bits-one.arita",
        expected: &["-9223372036854775808"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-reverse-bits-zero",
        path: "ejemplos/f2/146-reverse-bits-zero.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-rotate-left",
        path: "ejemplos/f2/147-rotate-left.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-rotate-right",
        path: "ejemplos/f2/148-rotate-right.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-swap-bytes-roundtrip",
        path: "ejemplos/f2/149-swap-bytes-roundtrip.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-swap-bytes-one",
        path: "ejemplos/f2/150-swap-bytes-one.arita",
        expected: &["72057594037927936"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-to-be-le-rel",
        path: "ejemplos/f2/151-to-be-le-rel.arita",
        expected: &["ok"],
        require_test: true,
    },
    EjemploOracle {
        id: "std-to-le-be-rel",
        path: "ejemplos/f2/152-to-le-be-rel.arita",
        expected: &["ok"],
        require_test: true,
    },
    EjemploOracle {
        id: "std-from-be-roundtrip",
        path: "ejemplos/f2/153-from-be-roundtrip.arita",
        expected: &["ok"],
        require_test: true,
    },
    EjemploOracle {
        id: "std-from-le-roundtrip",
        path: "ejemplos/f2/154-from-le-roundtrip.arita",
        expected: &["ok"],
        require_test: true,
    },
    EjemploOracle {
        id: "std-string-is-empty-true",
        path: "ejemplos/f2/155-string-is-empty-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-is-empty-false",
        path: "ejemplos/f2/156-string-is-empty-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-is-ascii-true",
        path: "ejemplos/f2/157-string-is-ascii-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-is-ascii-false",
        path: "ejemplos/f2/158-string-is-ascii-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-to-ascii-lower",
        path: "ejemplos/f2/159-to-ascii-lower.arita",
        expected: &["abc"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-to-ascii-upper",
        path: "ejemplos/f2/160-to-ascii-upper.arita",
        expected: &["ABC"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-eq-ignore-ascii-true",
        path: "ejemplos/f2/161-eq-ignore-ascii-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-eq-ignore-ascii-false",
        path: "ejemplos/f2/162-eq-ignore-ascii-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-char-boundary-true",
        path: "ejemplos/f2/163-is-char-boundary-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-char-boundary-false",
        path: "ejemplos/f2/164-is-char-boundary-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-floor-char-boundary",
        path: "ejemplos/f2/165-floor-char-boundary.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ceil-char-boundary",
        path: "ejemplos/f2/166-ceil-char-boundary.arita",
        expected: &["4"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-ascii",
        path: "ejemplos/f2/167-trim-ascii.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-ascii-start",
        path: "ejemplos/f2/168-trim-ascii-start.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trim-ascii-end",
        path: "ejemplos/f2/169-trim-ascii-end.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-make-ascii-lowercase",
        path: "ejemplos/f2/170-make-ascii-lower.arita",
        expected: &["abc"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-make-ascii-uppercase",
        path: "ejemplos/f2/171-make-ascii-upper.arita",
        expected: &["ABC"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-shl-some",
        path: "ejemplos/f2/172-checked-shl-some.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-shl-none",
        path: "ejemplos/f2/173-checked-shl-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-shr-some",
        path: "ejemplos/f2/174-checked-shr-some.arita",
        expected: &["4"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-shl",
        path: "ejemplos/f2/175-wrapping-shl.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-shr",
        path: "ejemplos/f2/176-wrapping-shr.arita",
        expected: &["4"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-shl-wide",
        path: "ejemplos/f2/177-wrapping-shl-wide.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-capacity-string-empty",
        path: "ejemplos/f2/178-capacity-string-empty.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-capacity-vec-empty",
        path: "ejemplos/f2/179-capacity-vec-empty.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-capacity-vec-after-push",
        path: "ejemplos/f2/180-capacity-vec-after-push.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-reserve-string",
        path: "ejemplos/f2/181-reserve-string.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-reserve-vec",
        path: "ejemplos/f2/182-reserve-vec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-shrink-to-fit-string",
        path: "ejemplos/f2/183-shrink-to-fit-string.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-shrink-to-fit-vec",
        path: "ejemplos/f2/184-shrink-to-fit-vec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-swap-ok",
        path: "ejemplos/f2/185-vec-swap-ok.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-swap-oob-noop",
        path: "ejemplos/f2/186-vec-swap-oob-noop.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-truncate-ascii",
        path: "ejemplos/f2/187-string-truncate-ascii.arita",
        expected: &["he"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-truncate-utf8-mid",
        path: "ejemplos/f2/188-string-truncate-utf8-mid.arita",
        expected: &["ni"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-string-split-off",
        path: "ejemplos/f2/189-string-split-off.arita",
        expected: &["llo"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-split-off",
        path: "ejemplos/f2/190-vec-split-off.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-split-off-oob-empty",
        path: "ejemplos/f2/191-split-off-oob-empty.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-bool-then-some-true",
        path: "ejemplos/f2/192-bool-then-some-true.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-bool-then-some-false",
        path: "ejemplos/f2/193-bool-then-some-false.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-fill-overwrite",
        path: "ejemplos/f2/194-vec-fill-overwrite.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-fill-empty",
        path: "ejemplos/f2/195-vec-fill-empty.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-take-some",
        path: "ejemplos/f2/196-option-take-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-take-none",
        path: "ejemplos/f2/197-option-take-none.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-replace",
        path: "ejemplos/f2/198-option-replace.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-rotate-left",
        path: "ejemplos/f2/199-vec-rotate-left.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-rotate-right",
        path: "ejemplos/f2/200-vec-rotate-right.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-or",
        path: "ejemplos/f2/201-option-or.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-and",
        path: "ejemplos/f2/202-option-and.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-xor",
        path: "ejemplos/f2/203-option-xor.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-or",
        path: "ejemplos/f2/204-result-or.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-and",
        path: "ejemplos/f2/205-result-and.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-dedup-runs",
        path: "ejemplos/f2/206-vec-dedup-runs.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-dedup-nonconsec",
        path: "ejemplos/f2/207-vec-dedup-nonconsec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-int-abs-diff",
        path: "ejemplos/f2/208-int-abs-diff.arita",
        expected: &["8"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-int-abs-diff-minmax",
        path: "ejemplos/f2/209-int-abs-diff-minmax.arita",
        expected: &["-1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-sort-unstable-int",
        path: "ejemplos/f2/210-vec-sort-unstable-int.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-sort-unstable-rev",
        path: "ejemplos/f2/211-vec-sort-unstable-rev.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-reverse",
        path: "ejemplos/f2/212-vec-reverse.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-reverse-empty",
        path: "ejemplos/f2/213-vec-reverse-empty.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-flatten-some-some",
        path: "ejemplos/f2/214-option-flatten-some-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-flatten-some-none",
        path: "ejemplos/f2/215-option-flatten-some-none.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-option-flatten-none",
        path: "ejemplos/f2/216-option-flatten-none.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-is-sorted-true",
        path: "ejemplos/f2/217-vec-is-sorted-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-is-sorted-false",
        path: "ejemplos/f2/218-vec-is-sorted-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-is-sorted-empty",
        path: "ejemplos/f2/219-vec-is-sorted-empty.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-div-ceil",
        path: "ejemplos/f2/220-div-ceil.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-div-floor",
        path: "ejemplos/f2/221-div-floor.arita",
        expected: &["-3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-binary-search-ok",
        path: "ejemplos/f2/222-vec-binary-search-ok.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-binary-search-err",
        path: "ejemplos/f2/223-vec-binary-search-err.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-next-multiple-of",
        path: "ejemplos/f2/224-next-multiple-of.arita",
        expected: &["9"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-next-multiple-of-some",
        path: "ejemplos/f2/225-checked-next-multiple-of-some.arita",
        expected: &["9"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-next-multiple-of-none",
        path: "ejemplos/f2/226-checked-next-multiple-of-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-previous-multiple-of",
        path: "ejemplos/f2/227-previous-multiple-of.arita",
        expected: &["6"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-previous-multiple-of-some",
        path: "ejemplos/f2/228-checked-previous-multiple-of-some.arita",
        expected: &["6"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-previous-multiple-of-none",
        path: "ejemplos/f2/229-checked-previous-multiple-of-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-starts-with-true",
        path: "ejemplos/f2/230-vec-starts-with-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-starts-with-false",
        path: "ejemplos/f2/231-vec-starts-with-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-ends-with-true",
        path: "ejemplos/f2/232-vec-ends-with-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-flatten-ok-ok",
        path: "ejemplos/f2/233-result-flatten-ok-ok.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-flatten-ok-err",
        path: "ejemplos/f2/234-result-flatten-ok-err.arita",
        expected: &["x"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-flatten-err",
        path: "ejemplos/f2/235-result-flatten-err.arita",
        expected: &["outer"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-transpose-option-ok",
        path: "ejemplos/f2/236-transpose-option-ok.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-transpose-option-err",
        path: "ejemplos/f2/237-transpose-option-err.arita",
        expected: &["x"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-transpose-option-none",
        path: "ejemplos/f2/238-transpose-option-none.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-transpose-result-some",
        path: "ejemplos/f2/239-transpose-result-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-transpose-result-none",
        path: "ejemplos/f2/240-transpose-result-none.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ok-or-some",
        path: "ejemplos/f2/241-ok-or-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ok-or-none",
        path: "ejemplos/f2/242-ok-or-none.arita",
        expected: &["x"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-ok-some",
        path: "ejemplos/f2/243-result-ok-some.arita",
        expected: &["7"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-result-err-some",
        path: "ejemplos/f2/244-result-err-some.arita",
        expected: &["x"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-try-reserve-vec",
        path: "ejemplos/f2/245-try-reserve-vec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-try-reserve-string",
        path: "ejemplos/f2/246-try-reserve-string.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-try-reserve-exact-vec",
        path: "ejemplos/f2/247-try-reserve-exact-vec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-try-reserve-exact-string",
        path: "ejemplos/f2/248-try-reserve-exact-string.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-shrink-to-vec",
        path: "ejemplos/f2/249-shrink-to-vec.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-shrink-to-string",
        path: "ejemplos/f2/250-shrink-to-string.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-div-normal",
        path: "ejemplos/f2/251-saturating-div-normal.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-div-min",
        path: "ejemplos/f2/252-saturating-div-min.arita",
        expected: &["9223372036854775807"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-div-normal",
        path: "ejemplos/f2/253-wrapping-div-normal.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-div-min",
        path: "ejemplos/f2/254-wrapping-div-min.arita",
        expected: &["-9223372036854775808"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-wrapping-rem-normal",
        path: "ejemplos/f2/255-wrapping-rem-normal.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-rem-normal",
        path: "ejemplos/f2/256-saturating-rem-normal.arita",
        expected: &["1"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-saturating-rem-min",
        path: "ejemplos/f2/257-saturating-rem-min.arita",
        expected: &["0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-leading-ones-neg1",
        path: "ejemplos/f2/258-leading-ones-neg1.arita",
        expected: &["64"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-trailing-ones-seven",
        path: "ejemplos/f2/259-trailing-ones-seven.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-power-of-two-true",
        path: "ejemplos/f2/260-is-power-of-two-true.arita",
        expected: &["true"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-is-power-of-two-false",
        path: "ejemplos/f2/261-is-power-of-two-false.arita",
        expected: &["false"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-next-pow2-some",
        path: "ejemplos/f2/262-checked-next-pow2-some.arita",
        expected: &["8"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-next-pow2-nonpos",
        path: "ejemplos/f2/263-checked-next-pow2-nonpos.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-checked-next-pow2-overflow",
        path: "ejemplos/f2/264-checked-next-pow2-overflow.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-append",
        path: "ejemplos/f2/265-vec-append.arita",
        expected: &["3", "0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-vec-append-empty",
        path: "ejemplos/f2/266-vec-append-empty.arita",
        expected: &["1"],
        require_test: false,
    },
    // ADR-233 / CORE-0.1-RECORD-20260919
    EjemploOracle {
        id: "core01-record-construct-access",
        path: "ejemplos/core01/01-record-construct-access.arita",
        expected: &["3", "7"],
        require_test: false,
    },
    // ADR-234 / CORE-0.1-ENUM-MATCH-20260919
    EjemploOracle {
        id: "core01-enum-match",
        path: "ejemplos/core01/02-enum-match.arita",
        expected: &["1"],
        require_test: false,
    },
    // ADR-235 / CORE-0.1-BORROW-SURFACE-20260919
    EjemploOracle {
        id: "core01-borrow-shared",
        path: "ejemplos/core01/03-borrow-shared.arita",
        expected: &["7"],
        require_test: false,
    },
    // ADR-236 / CORE-0.1-TEXT-BYTES-20260919
    EjemploOracle {
        id: "core01-text-lit",
        path: "ejemplos/core01/04-text-lit.arita",
        expected: &["hi"],
        require_test: false,
    },
    EjemploOracle {
        id: "core01-bytes-as-bytes",
        path: "ejemplos/core01/05-bytes-as-bytes.arita",
        expected: &["2"],
        require_test: false,
    },
    // ADR-237 / CORE-0.1-LIST-MAP-NAMES-20260919
    EjemploOracle {
        id: "core01-list-alias",
        path: "ejemplos/core01/06-list-alias.arita",
        expected: &["2"],
        require_test: false,
    },
    EjemploOracle {
        id: "core01-map-put-get",
        path: "ejemplos/core01/07-map-put-get.arita",
        expected: &["7"],
        require_test: false,
    },
    // ADR-238 / CORE-0.1-FILES-JSON-CLI-20260919
    EjemploOracle {
        id: "core01-io-read-text",
        path: "ejemplos/core01/io/08-read-text.arita",
        expected: &["hello-io"],
        require_test: false,
    },
    EjemploOracle {
        id: "core01-io-json-get-int",
        path: "ejemplos/core01/io/09-json-get-int.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "core01-io-cli-arg0",
        path: "ejemplos/core01/io/10-cli-arg0.arita",
        expected: &["has-arg0"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-find-none-branch-ok",
        path: "ejemplos/f2/138-find-none-branch-ok.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "std-ilog-none-branch-ok",
        path: "ejemplos/f2/139-ilog-none-branch-ok.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "iflet-01-some",
        path: "ejemplos/iflet/01-some.arita",
        expected: &["3"],
        require_test: false,
    },
    EjemploOracle {
        id: "iflet-02-none",
        path: "ejemplos/iflet/02-none.arita",
        expected: &["none"],
        require_test: false,
    },
    EjemploOracle {
        id: "iflet-result-01-ok",
        path: "ejemplos/iflet/result-01-ok.arita",
        expected: &["42"],
        require_test: false,
    },
    EjemploOracle {
        id: "iflet-result-02-err",
        path: "ejemplos/iflet/result-02-err.arita",
        expected: &["fail"],
        require_test: false,
    },
    EjemploOracle {
        id: "whilelet-01-pop-drain",
        path: "ejemplos/whilelet/01-pop-drain.arita",
        expected: &["3", "2", "1"],
        require_test: false,
    },
    EjemploOracle {
        id: "whilelet-02-none",
        path: "ejemplos/whilelet/02-none.arita",
        expected: &["ok"],
        require_test: false,
    },
    EjemploOracle {
        id: "whilelet-result-01-ok-once",
        path: "ejemplos/whilelet/result-01-ok-once.arita",
        expected: &["1", "after"],
        require_test: false,
    },
    EjemploOracle {
        id: "whilelet-result-02-err-skip",
        path: "ejemplos/whilelet/result-02-err-skip.arita",
        expected: &["after"],
        require_test: false,
    },
    EjemploOracle {
        id: "whilelet-result-03-err-pat",
        path: "ejemplos/whilelet/result-03-err-pat.arita",
        expected: &["7", "after"],
        require_test: false,
    },
];

/// Negative (anti-theater) oracles: measure **accepted** iff parse_lower_check fails with the expected E0xxx
/// (E0201/E0202 HIR check, or E021x parse). Accidental success or wrong code → **rejected**. Missing file → **inconclusive**.
pub struct NegOracle {
    pub id: &'static str,
    pub path: &'static str,
    /// Stable diagnostic code that must appear in the error (e.g. "E0201", "E0210").
    pub code: &'static str,
}

pub const NEG_ORACLES: &[NegOracle] = &[
    NegOracle {
        id: "neg-e0210-todo",
        path: "ejemplos/f2/neg/e0210-todo.arita",
        code: "E0210",
    },
    NegOracle {
        id: "neg-e0211-assert-true",
        path: "ejemplos/f2/neg/e0211-assert-true.arita",
        code: "E0211",
    },
    NegOracle {
        id: "neg-e0214-len-ge-zero",
        path: "ejemplos/f2/neg/e0214-len-ge-zero.arita",
        code: "E0214",
    },
    NegOracle {
        id: "neg-e0214-len-eq-len",
        path: "ejemplos/f2/neg/e0214-len-eq-len.arita",
        code: "E0214",
    },
    NegOracle {
        id: "neg-e0214-is-empty-taut",
        path: "ejemplos/f2/neg/e0214-is-empty-taut.arita",
        code: "E0214",
    },
    NegOracle {
        id: "neg-e0215-eq-self",
        path: "ejemplos/f2/neg/e0215-eq-self.arita",
        code: "E0215",
    },
    NegOracle {
        id: "neg-e0215-le-self",
        path: "ejemplos/f2/neg/e0215-le-self.arita",
        code: "E0215",
    },
    NegOracle {
        id: "neg-e0216-div0",
        path: "ejemplos/f2/neg/e0216-div0.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0217-add-overflow",
        path: "ejemplos/f2/neg/e0217-add-overflow.arita",
        code: "E0217",
    },
    NegOracle {
        id: "neg-e0217-sub-overflow",
        path: "ejemplos/f2/neg/e0217-lit-sub-overflow.arita",
        code: "E0217",
    },
    NegOracle {
        id: "neg-e0217-mul-overflow",
        path: "ejemplos/f2/neg/e0217-lit-mul-overflow.arita",
        code: "E0217",
    },
    NegOracle {
        id: "neg-e0212-empty-test",
        path: "ejemplos/f2/neg/e0212-empty-test.arita",
        code: "E0212",
    },
    NegOracle {
        id: "neg-e0213-empty-fn",
        path: "ejemplos/f2/neg/e0213-empty-fn.arita",
        code: "E0213",
    },
    NegOracle {
        id: "neg-e0201-use-after-move",
        path: "ejemplos/f2/neg/e0201-use-after-move.arita",
        code: "E0201",
    },
    NegOracle {
        id: "neg-e0202-double-mut",
        path: "ejemplos/f2/neg/e0202-double-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0220-bad-cond",
        path: "ejemplos/f2.1/neg/e0220-bad-cond.arita",
        code: "E0220",
    },
    NegOracle {
        id: "neg-e0226-while-false",
        path: "ejemplos/f2.1/neg/e0226-while-false.arita",
        code: "E0226",
    },
    NegOracle {
        id: "neg-e0227-if-false",
        path: "ejemplos/f2.1/neg/e0227-if-false.arita",
        code: "E0227",
    },
    NegOracle {
        id: "neg-e0221-bool-nonex",
        path: "ejemplos/f2.2/neg/e0221-bool-nonex.arita",
        code: "E0221",
    },
    NegOracle {
        id: "neg-e0222-int-no-wild",
        path: "ejemplos/f2.2/neg/e0222-int-no-wild.arita",
        code: "E0222",
    },
    NegOracle {
        id: "neg-e0223-pat-mismatch",
        path: "ejemplos/f2.2/neg/e0223-pat-mismatch.arita",
        code: "E0223",
    },
    NegOracle {
        id: "neg-e0225-match-bool-same",
        path: "ejemplos/f2.2/neg/e0225-match-bool-same.arita",
        code: "E0225",
    },
    NegOracle {
        id: "neg-e0225-match-int-same",
        path: "ejemplos/f2.2/neg/e0225-match-int-same.arita",
        code: "E0225",
    },
    NegOracle {
        id: "neg-e0224-break-outside",
        path: "ejemplos/f2.3/neg/e0224-break-outside.arita",
        code: "E0224",
    },
    NegOracle {
        id: "neg-e0231-unsafe",
        path: "ejemplos/f2/neg/e0231-unsafe.arita",
        code: "E0231",
    },
    NegOracle {
        id: "neg-e0206-bad-method",
        path: "ejemplos/f2/neg/e0206-bad-method.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-clone-bad",
        path: "ejemplos/f2/neg/e0206-clone-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0203-starts-with-vec-elem",
        path: "ejemplos/f2/neg/e0206-starts-with-vec.arita",
        code: "E0203",
    },
    NegOracle {
        id: "neg-e0203-ends-with-vec-elem",
        path: "ejemplos/f2/neg/e0206-ends-with-vec.arita",
        code: "E0203",
    },
    NegOracle {
        id: "neg-e0206-trim-vec",
        path: "ejemplos/f2/neg/e0206-trim-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-replace-vec",
        path: "ejemplos/f2/neg/e0206-replace-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-case-vec",
        path: "ejemplos/f2/neg/e0206-case-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-parse-int-vec",
        path: "ejemplos/f2/neg/e0206-parse-int-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-to-string-vec",
        path: "ejemplos/f2/neg/e0206-to-string-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-min-string",
        path: "ejemplos/f2/neg/e0206-min-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0279-clamp-range",
        path: "ejemplos/f2/neg/e0279-clamp-range.arita",
        code: "E0279",
    },
    NegOracle {
        id: "neg-e0206-trim-start-vec",
        path: "ejemplos/f2/neg/e0206-trim-start-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-strip-vec",
        path: "ejemplos/f2/neg/e0206-strip-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-add-string",
        path: "ejemplos/f2/neg/e0206-checked-add-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-sub-string",
        path: "ejemplos/f2/neg/e0206-checked-sub-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-mul-string",
        path: "ejemplos/f2/neg/e0206-checked-mul-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-div-string",
        path: "ejemplos/f2/neg/e0206-checked-div-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-rem-string",
        path: "ejemplos/f2/neg/e0206-checked-rem-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-get-string",
        path: "ejemplos/f2/neg/e0206-get-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-first-string",
        path: "ejemplos/f2/neg/e0206-first-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-contains-int",
        path: "ejemplos/f2/neg/e0206-contains-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-saturating-add-string",
        path: "ejemplos/f2/neg/e0206-saturating-add-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-saturating-sub-string",
        path: "ejemplos/f2/neg/e0206-saturating-sub-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-saturating-mul-string",
        path: "ejemplos/f2/neg/e0206-saturating-mul-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-unwrap-or-int",
        path: "ejemplos/f2/neg/e0206-unwrap-or-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-some-int",
        path: "ejemplos/f2/neg/e0206-is-some-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-wrapping-add-string",
        path: "ejemplos/f2/neg/e0206-wrapping-add-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-wrapping-sub-string",
        path: "ejemplos/f2/neg/e0206-wrapping-sub-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-wrapping-mul-string",
        path: "ejemplos/f2/neg/e0206-wrapping-mul-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0285-checked-add-unwrap-or-0",
        path: "ejemplos/f2/neg/e0285-checked-add-unwrap-or-0.arita",
        code: "E0285",
    },
    NegOracle {
        id: "neg-e0285-checked-mul-unwrap-or-0",
        path: "ejemplos/f2/neg/e0285-checked-mul-unwrap-or-0.arita",
        code: "E0285",
    },
    NegOracle {
        id: "neg-e0286-wrapping-add-zero",
        path: "ejemplos/f2/neg/e0286-wrapping-add-zero.arita",
        code: "E0286",
    },
    NegOracle {
        id: "neg-e0286-saturating-mul-one",
        path: "ejemplos/f2/neg/e0286-saturating-mul-one.arita",
        code: "E0286",
    },
    NegOracle {
        id: "neg-e0287-get-oob-unwrap-or-0",
        path: "ejemplos/f2/neg/e0287-get-oob-unwrap-or-0.arita",
        code: "E0287",
    },
    NegOracle {
        id: "neg-e0287-get-empty-unwrap-or-0",
        path: "ejemplos/f2/neg/e0287-get-empty-unwrap-or-0.arita",
        code: "E0287",
    },
    NegOracle {
        id: "neg-e0206-signum-string",
        path: "ejemplos/f2/neg/e0206-signum-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-positive-string",
        path: "ejemplos/f2/neg/e0206-is-positive-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-wrapping-neg-string",
        path: "ejemplos/f2/neg/e0206-wrapping-neg-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-neg-string",
        path: "ejemplos/f2/neg/e0206-checked-neg-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-div-euclid-zero",
        path: "ejemplos/f2/neg/e0216-div-euclid-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-div-euclid-string",
        path: "ejemplos/f2/neg/e0206-div-euclid-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-div-euclid-string",
        path: "ejemplos/f2/neg/e0206-checked-div-euclid-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-find-vec",
        path: "ejemplos/f2/neg/e0206-find-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-swap-remove-not-mut",
        path: "ejemplos/f2/neg/e0202-swap-remove-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-midpoint-string",
        path: "ejemplos/f2/neg/e0206-midpoint-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-multiple-of-string",
        path: "ejemplos/f2/neg/e0206-is-multiple-of-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0288-truncate-neg",
        path: "ejemplos/f2/neg/e0288-truncate-neg.arita",
        code: "E0288",
    },
    NegOracle {
        id: "neg-e0202-truncate-not-mut",
        path: "ejemplos/f2/neg/e0202-truncate-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0288-resize-neg",
        path: "ejemplos/f2/neg/e0288-resize-neg.arita",
        code: "E0288",
    },
    NegOracle {
        id: "neg-e0202-resize-not-mut",
        path: "ejemplos/f2/neg/e0202-resize-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-ilog2-string",
        path: "ejemplos/f2/neg/e0206-ilog2-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-ilog10-string",
        path: "ejemplos/f2/neg/e0206-ilog10-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-count-ones-string",
        path: "ejemplos/f2/neg/e0206-count-ones-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-count-zeros-string",
        path: "ejemplos/f2/neg/e0206-count-zeros-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-leading-zeros-string",
        path: "ejemplos/f2/neg/e0206-leading-zeros-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-reverse-bits-string",
        path: "ejemplos/f2/neg/e0206-reverse-bits-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0291-rotate-neg",
        path: "ejemplos/f2/neg/e0291-rotate-neg.arita",
        code: "E0291",
    },
    NegOracle {
        id: "neg-e0206-rotate-string",
        path: "ejemplos/f2/neg/e0206-rotate-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-swap-bytes-string",
        path: "ejemplos/f2/neg/e0206-swap-bytes-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-to-be-string",
        path: "ejemplos/f2/neg/e0206-to-be-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-from-be-string",
        path: "ejemplos/f2/neg/e0206-from-be-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-empty-int",
        path: "ejemplos/f2/neg/e0206-is-empty-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-ascii-int",
        path: "ejemplos/f2/neg/e0206-is-ascii-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-to-ascii-lower-vec",
        path: "ejemplos/f2/neg/e0206-to-ascii-lower-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-eq-ignore-ascii-vec",
        path: "ejemplos/f2/neg/e0206-eq-ignore-ascii-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0292-char-boundary-neg",
        path: "ejemplos/f2/neg/e0292-char-boundary-neg.arita",
        code: "E0292",
    },
    NegOracle {
        id: "neg-e0206-char-boundary-vec",
        path: "ejemplos/f2/neg/e0206-char-boundary-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0292-floor-char-boundary-neg",
        path: "ejemplos/f2/neg/e0292-floor-char-boundary-neg.arita",
        code: "E0292",
    },
    NegOracle {
        id: "neg-e0206-floor-char-boundary-vec",
        path: "ejemplos/f2/neg/e0206-floor-char-boundary-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-trim-ascii-vec",
        path: "ejemplos/f2/neg/e0206-trim-ascii-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-make-ascii-vec",
        path: "ejemplos/f2/neg/e0206-make-ascii-vec.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-make-ascii-not-mut",
        path: "ejemplos/f2/neg/e0202-make-ascii-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0293-checked-shl-neg",
        path: "ejemplos/f2/neg/e0293-checked-shl-neg.arita",
        code: "E0293",
    },
    NegOracle {
        id: "neg-e0206-checked-shl-string",
        path: "ejemplos/f2/neg/e0206-checked-shl-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0293-wrapping-shl-neg",
        path: "ejemplos/f2/neg/e0293-wrapping-shl-neg.arita",
        code: "E0293",
    },
    NegOracle {
        id: "neg-e0206-wrapping-shl-string",
        path: "ejemplos/f2/neg/e0206-wrapping-shl-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-capacity-int",
        path: "ejemplos/f2/neg/e0206-capacity-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0294-reserve-neg",
        path: "ejemplos/f2/neg/e0294-reserve-neg.arita",
        code: "E0294",
    },
    NegOracle {
        id: "neg-e0202-reserve-not-mut",
        path: "ejemplos/f2/neg/e0202-reserve-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-reserve-int",
        path: "ejemplos/f2/neg/e0206-reserve-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-shrink-to-fit-not-mut",
        path: "ejemplos/f2/neg/e0202-shrink-to-fit-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-shrink-to-fit-int",
        path: "ejemplos/f2/neg/e0206-shrink-to-fit-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0295-swap-neg",
        path: "ejemplos/f2/neg/e0295-swap-neg.arita",
        code: "E0295",
    },
    NegOracle {
        id: "neg-e0202-swap-not-mut",
        path: "ejemplos/f2/neg/e0202-swap-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-swap-int",
        path: "ejemplos/f2/neg/e0206-swap-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0288-string-truncate-neg",
        path: "ejemplos/f2/neg/e0288-string-truncate-neg.arita",
        code: "E0288",
    },
    NegOracle {
        id: "neg-e0202-string-truncate-not-mut",
        path: "ejemplos/f2/neg/e0202-string-truncate-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-string-truncate-int",
        path: "ejemplos/f2/neg/e0206-string-truncate-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0296-split-off-neg",
        path: "ejemplos/f2/neg/e0296-split-off-neg.arita",
        code: "E0296",
    },
    NegOracle {
        id: "neg-e0202-split-off-not-mut",
        path: "ejemplos/f2/neg/e0202-split-off-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-then-some-int",
        path: "ejemplos/f2/neg/e0206-then-some-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-fill-not-mut",
        path: "ejemplos/f2/neg/e0202-fill-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-fill-int",
        path: "ejemplos/f2/neg/e0206-fill-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-option-take-not-mut",
        path: "ejemplos/f2/neg/e0202-option-take-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-option-take-int",
        path: "ejemplos/f2/neg/e0206-option-take-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0297-vec-rotate-neg",
        path: "ejemplos/f2/neg/e0297-vec-rotate-neg.arita",
        code: "E0297",
    },
    NegOracle {
        id: "neg-e0202-vec-rotate-not-mut",
        path: "ejemplos/f2/neg/e0202-vec-rotate-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-option-or-int",
        path: "ejemplos/f2/neg/e0206-option-or-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-result-xor",
        path: "ejemplos/f2/neg/e0206-result-xor.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-dedup-not-mut",
        path: "ejemplos/f2/neg/e0202-dedup-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-dedup-string",
        path: "ejemplos/f2/neg/e0206-dedup-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-abs-diff-string",
        path: "ejemplos/f2/neg/e0206-abs-diff-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-sort-unstable-not-mut",
        path: "ejemplos/f2/neg/e0202-sort-unstable-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-sort-unstable-string",
        path: "ejemplos/f2/neg/e0206-sort-unstable-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-reverse-not-mut",
        path: "ejemplos/f2/neg/e0202-reverse-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-reverse-string",
        path: "ejemplos/f2/neg/e0206-reverse-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-flatten-int",
        path: "ejemplos/f2/neg/e0206-flatten-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-sorted-string",
        path: "ejemplos/f2/neg/e0206-is-sorted-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-div-ceil-zero",
        path: "ejemplos/f2/neg/e0216-div-ceil-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-div-ceil-string",
        path: "ejemplos/f2/neg/e0206-div-ceil-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-binary-search-string",
        path: "ejemplos/f2/neg/e0206-binary-search-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-next-multiple-of-zero",
        path: "ejemplos/f2/neg/e0216-next-multiple-of-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-next-multiple-of-string",
        path: "ejemplos/f2/neg/e0206-next-multiple-of-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-previous-multiple-of-zero",
        path: "ejemplos/f2/neg/e0216-previous-multiple-of-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-previous-multiple-of-string",
        path: "ejemplos/f2/neg/e0206-previous-multiple-of-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-vec-starts-with-int",
        path: "ejemplos/f2/neg/e0206-vec-starts-with-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-result-flatten-non-nested",
        path: "ejemplos/f2/neg/e0206-result-flatten-non-nested.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-transpose-int",
        path: "ejemplos/f2/neg/e0206-transpose-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-ok-or-int",
        path: "ejemplos/f2/neg/e0206-ok-or-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0294-try-reserve-neg",
        path: "ejemplos/f2/neg/e0294-try-reserve-neg.arita",
        code: "E0294",
    },
    NegOracle {
        id: "neg-e0202-try-reserve-not-mut",
        path: "ejemplos/f2/neg/e0202-try-reserve-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0294-try-reserve-exact-neg",
        path: "ejemplos/f2/neg/e0294-try-reserve-exact-neg.arita",
        code: "E0294",
    },
    NegOracle {
        id: "neg-e0202-try-reserve-exact-not-mut",
        path: "ejemplos/f2/neg/e0202-try-reserve-exact-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0298-shrink-to-neg",
        path: "ejemplos/f2/neg/e0298-shrink-to-neg.arita",
        code: "E0298",
    },
    NegOracle {
        id: "neg-e0202-shrink-to-not-mut",
        path: "ejemplos/f2/neg/e0202-shrink-to-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0216-saturating-div-zero",
        path: "ejemplos/f2/neg/e0216-saturating-div-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-saturating-div-string",
        path: "ejemplos/f2/neg/e0206-saturating-div-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-wrapping-div-zero",
        path: "ejemplos/f2/neg/e0216-wrapping-div-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-wrapping-div-string",
        path: "ejemplos/f2/neg/e0206-wrapping-div-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0216-saturating-rem-zero",
        path: "ejemplos/f2/neg/e0216-saturating-rem-zero.arita",
        code: "E0216",
    },
    NegOracle {
        id: "neg-e0206-saturating-rem-string",
        path: "ejemplos/f2/neg/e0206-saturating-rem-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-leading-ones-string",
        path: "ejemplos/f2/neg/e0206-leading-ones-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-is-power-of-two-string",
        path: "ejemplos/f2/neg/e0206-is-power-of-two-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-checked-next-pow2-string",
        path: "ejemplos/f2/neg/e0206-checked-next-pow2-string.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-append-arg-not-mut",
        path: "ejemplos/f2/neg/e0202-append-arg-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-append-int",
        path: "ejemplos/f2/neg/e0206-append-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0202-extend-not-mut",
        path: "ejemplos/f2/neg/e0202-extend-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0206-extend-int",
        path: "ejemplos/f2/neg/e0206-extend-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0310-index-vec",
        path: "ejemplos/f2/neg/e0310-index-vec.arita",
        // ADR-283 M1: Vec `v[i] = x` in main → E0344 (was E0314); exact span pinned in
        // `adr283_m1_vec_negs_smoke`.
        code: "E0344",
    },
    NegOracle {
        id: "neg-e0310-index-string",
        path: "ejemplos/f2/neg/e0310-index-string.arita",
        code: "E0314",
    },
    // ADR-233 / CORE-0.1-RECORD-20260919
    NegOracle {
        id: "neg-e0315-missing-field",
        path: "ejemplos/core01/neg/e0315-missing-field.arita",
        code: "E0315",
    },
    NegOracle {
        id: "neg-e0314-unknown-field",
        path: "ejemplos/core01/neg/e0314-unknown-field.arita",
        code: "E0314",
    },
    NegOracle {
        id: "neg-e0316-nonexhaustive-enum",
        path: "ejemplos/core01/neg/e0316-nonexhaustive-enum.arita",
        code: "E0316",
    },
    NegOracle {
        id: "neg-e0317-unknown-variant",
        path: "ejemplos/core01/neg/e0317-unknown-variant.arita",
        code: "E0317",
    },
    NegOracle {
        id: "neg-e0202-borrow-double-mut",
        path: "ejemplos/core01/neg/e0202-borrow-double-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0318-borrow-escape-return",
        path: "ejemplos/core01/neg/e0318-borrow-escape-return.arita",
        code: "E0318",
    },
    NegOracle {
        id: "neg-e0206-as-bytes-on-int",
        path: "ejemplos/core01/neg/e0206-as-bytes-on-int.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0206-map-insert-banned",
        path: "ejemplos/core01/neg/e0206-map-insert-banned.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-e0289-find-unwrap-or-0",
        path: "ejemplos/f2/neg/e0289-find-unwrap-or-0.arita",
        code: "E0289",
    },
    NegOracle {
        id: "neg-e0289-rfind-unwrap-or-0",
        path: "ejemplos/f2/neg/e0289-rfind-unwrap-or-0.arita",
        code: "E0289",
    },
    NegOracle {
        id: "neg-e0290-ilog2-unwrap-or-0",
        path: "ejemplos/f2/neg/e0290-ilog2-unwrap-or-0.arita",
        code: "E0290",
    },
    NegOracle {
        id: "neg-e0290-ilog10-unwrap-or-0",
        path: "ejemplos/f2/neg/e0290-ilog10-unwrap-or-0.arita",
        code: "E0290",
    },
    NegOracle {
        id: "neg-e0280-repeat-neg",
        path: "ejemplos/f2/neg/e0280-repeat-neg.arita",
        code: "E0280",
    },
    NegOracle {
        id: "neg-e0280-vec-repeat-neg",
        path: "ejemplos/f2/neg/e0280-vec-repeat-neg.arita",
        code: "E0280",
    },
    NegOracle {
        id: "neg-e0281-pow-neg-exp",
        path: "ejemplos/f2/neg/e0281-pow-neg-exp.arita",
        code: "E0281",
    },
    NegOracle {
        id: "neg-e0283-pow-overflow",
        path: "ejemplos/f2/neg/e0283-pow-overflow.arita",
        code: "E0283",
    },
    NegOracle {
        id: "neg-e0278-abs-min",
        path: "ejemplos/f2/neg/e0278-abs-min.arita",
        code: "E0278",
    },
    NegOracle {
        id: "neg-e0240-await-outside",
        path: "ejemplos/async/neg/e0240-await-outside.arita",
        code: "E0240",
    },
    NegOracle {
        id: "neg-e0240-spawn-outside",
        path: "ejemplos/core02/neg/e0240-spawn-outside.arita",
        code: "E0240",
    },
    NegOracle {
        id: "neg-hang-theater",
        path: "ejemplos/core02/neg/hang-theater.arita",
        code: "E0320",
    },
    NegOracle {
        id: "neg-e0206-http-listen-freefn",
        path: "ejemplos/core02/http/neg/01-listen-freefn.arita",
        code: "E0206",
    },
    NegOracle {
        id: "neg-core02-policy-neg-int",
        path: "ejemplos/core02/http/neg/02-policy-neg-int.arita",
        code: "E0323",
    },
    NegOracle {
        id: "neg-core02-policy-zero-body",
        path: "ejemplos/core02/http/neg/03-policy-zero-body.arita",
        code: "E0323",
    },
    NegOracle {
        id: "neg-core02-scen-reqwest",
        path: "ejemplos/core02/http/neg/04-scen-reqwest.arita",
        code: "E0321",
    },
    NegOracle {
        id: "neg-core03-compose-reqwest",
        path: "ejemplos/core03/client-compose/neg/01-reqwest-surface.arita",
        code: "E0321",
    },
    NegOracle {
        id: "neg-core03-err-unwrap",
        path: "ejemplos/core03/error-prop/neg/01-unwrap.arita",
        code: "E0272",
    },
    NegOracle {
        id: "neg-core03-scen-reqwest",
        path: "ejemplos/core03/scenario-compose/neg/01-reqwest.arita",
        code: "E0321",
    },
    NegOracle {
        id: "neg-core03-ref-reqwest",
        path: "ejemplos/core03/ref-http-compose/neg/01-reqwest.arita",
        code: "E0321",
    },
    NegOracle {
        id: "core04-mm-neg-crate",
        path: "ejemplos/core04/multi-module/neg/use-crate.arita",
        code: "E0261",
    },
    NegOracle {
        id: "neg-e0241-async-illegal",
        path: "ejemplos/async/neg/e0241-async-illegal.arita",
        code: "E0241",
    },
    NegOracle {
        id: "neg-e0242-borrow-across-await",
        path: "ejemplos/async/neg/e0242-borrow-across-await.arita",
        code: "E0242",
    },
    NegOracle {
        id: "deps-neg-e0261",
        path: "ejemplos/deps/neg/e0261-crate-path.arita",
        code: "E0261",
    },
    NegOracle {
        id: "neg-e0270",
        path: "ejemplos/result/neg/e0270-nonexhaustive.arita",
        code: "E0270",
    },
    NegOracle {
        id: "neg-e0272-err-default-lit",
        path: "ejemplos/f2/neg/e0272-err-default-lit.arita",
        code: "E0272",
    },
    NegOracle {
        id: "neg-e0272-err-underscore-empty",
        path: "ejemplos/f2/neg/e0272-err-underscore-empty.arita",
        code: "E0272",
    },
    NegOracle {
        id: "neg-e0272-err-unused-print-ok",
        path: "ejemplos/f2/neg/e0272-err-unused-print-ok.arita",
        code: "E0272",
    },
    NegOracle {
        id: "neg-e0202-clear-not-mut",
        path: "ejemplos/f2/neg/e0202-clear-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0273",
        path: "ejemplos/option/neg/e0273-nonexhaustive.arita",
        code: "E0273",
    },
    NegOracle {
        id: "neg-e0274",
        path: "ejemplos/option/neg/e0274-none-default-lit.arita",
        code: "E0274",
    },
    NegOracle {
        id: "neg-e0274-print-ok",
        path: "ejemplos/option/neg/e0274-none-print-ok.arita",
        code: "E0274",
    },
    NegOracle {
        id: "neg-e0202-pop-not-mut",
        path: "ejemplos/f2/neg/e0202-pop-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0202-remove-not-mut",
        path: "ejemplos/f2/neg/e0202-remove-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0202-push-str",
        path: "ejemplos/f2/neg/e0202-push-str-not-mut.arita",
        code: "E0202",
    },
    NegOracle {
        id: "neg-e0275",
        path: "ejemplos/iflet/neg/e0275-no-else.arita",
        code: "E0275",
    },
    NegOracle {
        id: "neg-e0275-result",
        path: "ejemplos/iflet/neg/e0275-result-no-else.arita",
        code: "E0275",
    },
    NegOracle {
        id: "neg-e0277",
        path: "ejemplos/whilelet/neg/e0277-lit-none.arita",
        code: "E0277",
    },
    NegOracle {
        id: "neg-e0277-empty-body",
        path: "ejemplos/whilelet/neg/e0277-empty-body.arita",
        code: "E0277",
    },
    NegOracle {
        id: "neg-e0282-ok-on-err",
        path: "ejemplos/whilelet/neg/e0282-ok-on-err.arita",
        code: "E0282",
    },
    NegOracle {
        id: "neg-e0282-err-on-ok",
        path: "ejemplos/whilelet/neg/e0282-err-on-ok.arita",
        code: "E0282",
    },
];

/// F3 logic oracles using the real positive-Datalog engine (`arita-logic`), not rustc.
/// `expect_sat = true` → accepted iff eval succeeds (all queries sat).
/// `expect_sat = false` → accepted iff engine rejects with `code` (e.g. E0301).
/// Missing file → inconclusive (never accepted). Skip/empty queries → never accepted.
pub struct LogicOracle {
    pub id: &'static str,
    pub path: &'static str,
    /// When true, measure accepted iff `eval_file` Ok (all queries sat).
    /// When false, accepted iff `eval_file` Err containing `code`.
    pub expect_sat: bool,
    /// Expected diagnostic when `expect_sat` is false (e.g. "E0301").
    pub code: &'static str,
}

pub const LOGIC_ORACLES: &[LogicOracle] = &[
    LogicOracle {
        id: "f3-01-path-ok",
        path: "ejemplos/f3/01-path-ok.arita",
        expect_sat: true,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-02-path-fail",
        path: "ejemplos/f3/02-path-fail.arita",
        expect_sat: false,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-03-ancestor-ok",
        path: "ejemplos/f3/03-ancestor-ok.arita",
        expect_sat: true,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-04-sibling-fail",
        path: "ejemplos/f3/04-sibling-fail.arita",
        expect_sat: false,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-05-edge-ok",
        path: "ejemplos/f3/05-edge-ok.arita",
        expect_sat: true,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-06-unknown-pred",
        path: "ejemplos/f3/06-unknown-pred.arita",
        expect_sat: false,
        code: "E0303",
    },
    LogicOracle {
        id: "f3-07-forbidden-fn",
        path: "ejemplos/f3/07-forbidden-fn.arita",
        expect_sat: false,
        code: "E0304",
    },
    LogicOracle {
        id: "f3-08-arity-mismatch",
        path: "ejemplos/f3/08-arity-mismatch.arita",
        expect_sat: false,
        code: "E0303",
    },
    LogicOracle {
        id: "f3-09-twohop-ok",
        path: "ejemplos/f3/09-twohop-ok.arita",
        expect_sat: true,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-10-cycle-reach-ok",
        path: "ejemplos/f3/10-cycle-reach-ok.arita",
        expect_sat: true,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-11-multi-query-fail",
        path: "ejemplos/f3/11-multi-query-fail.arita",
        expect_sat: false,
        code: "E0301",
    },
    LogicOracle {
        id: "f3-12-join-miss-fail",
        path: "ejemplos/f3/12-join-miss-fail.arita",
        expect_sat: false,
        code: "E0301",
    },
];

/// Locate workspace root: walk from start (and CARGO_MANIFEST_DIR) for Cargo.toml with [workspace].
pub fn find_workspace_root() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(m) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(m);
        candidates.push(p.clone());
        if let Some(parent) = p.parent() {
            candidates.push(parent.to_path_buf());
            if let Some(gp) = parent.parent() {
                candidates.push(gp.to_path_buf());
            }
        }
    }
    for start in candidates {
        let mut cur = start;
        loop {
            let cargo = cur.join("Cargo.toml");
            if cargo.is_file() {
                if let Ok(text) = fs::read_to_string(&cargo) {
                    if text.contains("[workspace]") {
                        return Some(cur);
                    }
                }
            }
            if !cur.pop() {
                break;
            }
        }
    }
    None
}

/// Normalize stdout: split on newlines; drop a single trailing empty line from final newline.
pub fn normalize_stdout(stdout: &str) -> Vec<String> {
    let mut lines: Vec<String> = stdout.split('\n').map(|s| s.to_string()).collect();
    if lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

/// ADR-034: oracles whose id starts with `target-` are **gated**. Their
/// `Inconclusive` (missing rustup target / linker) must not tumble overall to
/// inconclusive — and must never be invented as PASS. A gated `Rejected` still
/// rejects the suite (real failure when toolchain is present).
pub fn is_gated_oracle(id: &str) -> bool {
    id.starts_with("target-")
}

pub fn combine_verdicts(results: &[OracleResult]) -> Verdict {
    let mut any_rejected = false;
    let mut any_inconclusive = false;
    for r in results {
        match r.verdict {
            Verdict::Rejected => any_rejected = true,
            Verdict::Inconclusive => {
                if !is_gated_oracle(&r.id) {
                    any_inconclusive = true;
                }
            }
            Verdict::Accepted => {}
        }
    }
    if any_rejected {
        Verdict::Rejected
    } else if any_inconclusive {
        Verdict::Inconclusive
    } else {
        Verdict::Accepted
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn format_measure_json(overall: Verdict, oracles: &[OracleResult]) -> String {
    let mut buf = String::new();
    buf.push_str("{\n");
    buf.push_str("  \"tool\": \"arita measure\",\n");
    buf.push_str(&format!("  \"version\": \"{MEASURE_VERSION}\",\n"));
    buf.push_str(&format!("  \"verdict\": \"{}\",\n", overall.as_str()));
    buf.push_str("  \"oracles\": [\n");
    for (i, o) in oracles.iter().enumerate() {
        let comma = if i + 1 < oracles.len() { "," } else { "" };
        buf.push_str(&format!(
            "    {{\"id\": \"{}\", \"path\": \"{}\", \"verdict\": \"{}\", \"detail\": \"{}\"}}{comma}\n",
            json_escape(&o.id),
            json_escape(&o.path),
            o.verdict.as_str(),
            json_escape(&o.detail)
        ));
    }
    buf.push_str("  ]\n");
    buf.push('}');
    buf
}

/// Required clippy workspace oracle. Missing clippy tooling → inconclusive (never accepted).
pub fn run_clippy_oracle(root: &Path) -> OracleResult {
    let path_label =
        "cargo clippy -p arita-syntax -p arita-codegen -p arita-hir -p arita-logic -p arita-cli -- -D warnings";
    let output = Command::new("cargo")
        .args([
            "clippy",
            "-p",
            "arita-syntax",
            "-p",
            "arita-codegen",
            "-p",
            "arita-hir",
            "-p",
            "arita-logic",
            "-p",
            "arita-cli",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(root)
        .output();

    match output {
        Err(e) => OracleResult {
            id: "clippy-workspace".into(),
            path: path_label.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("cannot spawn cargo clippy: {e}"),
        },
        Ok(out) => {
            let status = out.status;
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let combined = format!("{stdout}\n{stderr}");
            // Clippy not installed / not a cargo subcommand → cannot run → inconclusive
            let missing = !status.success()
                && (combined.contains("no such command: `clippy`")
                    || combined.contains("no such command: \"clippy\"")
                    || combined.contains("is not installed")
                    || combined.contains("clippy-preview"));
            if status.success() {
                OracleResult {
                    id: "clippy-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("clippy ok (exit {})", status.code().unwrap_or(0)),
                }
            } else if missing {
                OracleResult {
                    id: "clippy-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Inconclusive,
                    detail: "clippy not available (cannot run); never treated as accepted".into(),
                }
            } else {
                // Prefer rustc/clippy `error:` lines so measure detail is actionable.
                let err_lines: Vec<&str> = combined
                    .lines()
                    .filter(|l| {
                        let t = l.trim_start();
                        t.starts_with("error") || t.starts_with("-->") || t.starts_with('|')
                    })
                    .collect();
                let focused = if err_lines.is_empty() {
                    combined.clone()
                } else {
                    err_lines.join("\n")
                };
                let snip: String = focused.chars().take(900).collect();
                OracleResult {
                    id: "clippy-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "clippy failed (exit {}): {}",
                        status.code().unwrap_or(-1),
                        snip.replace('\n', " ")
                    ),
                }
            }
        }
    }
}

/// Required Miri workspace oracle (lib crates). Missing nightly/miri → inconclusive (never accepted).
/// Scope: syntax/codegen/hir/logic `--lib` only — arita-cli tests spawn host binaries (unsupported under Miri).
pub fn run_miri_oracle(root: &Path) -> OracleResult {
    let path_label =
        "cargo +nightly miri test -p arita-syntax -p arita-codegen -p arita-hir -p arita-logic --lib";
    let output = Command::new("cargo")
        .args([
            "+nightly",
            "miri",
            "test",
            "-p",
            "arita-syntax",
            "-p",
            "arita-codegen",
            "-p",
            "arita-hir",
            "-p",
            "arita-logic",
            "--lib",
        ])
        .current_dir(root)
        .env("MIRIFLAGS", "-Zmiri-disable-isolation")
        .output();

    match output {
        Err(e) => OracleResult {
            id: "miri-workspace".into(),
            path: path_label.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("cannot spawn cargo miri: {e}"),
        },
        Ok(out) => {
            let status = out.status;
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let combined = format!("{stdout}\n{stderr}");
            let missing = !status.success()
                && (combined.contains("no such command: `miri`")
                    || combined.contains("no such command: \"miri\"")
                    || combined.contains("no such command: `+nightly`")
                    || combined.contains("toolchain 'nightly")
                    || combined.contains("toolchain \"nightly")
                    || combined.contains("is not installed")
                    || combined.contains("not installed for the toolchain")
                    || combined.contains("Unknown binary 'miri'")
                    || combined.contains("the `miri` component")
                    || (combined.contains("can't find crate for `std`")
                        && combined.to_lowercase().contains("nightly")));
            if status.success() {
                OracleResult {
                    id: "miri-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("miri ok (exit {})", status.code().unwrap_or(0)),
                }
            } else if missing {
                OracleResult {
                    id: "miri-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Inconclusive,
                    detail: "miri/nightly not available (cannot run); never treated as accepted"
                        .into(),
                }
            } else {
                let err_lines: Vec<&str> = combined
                    .lines()
                    .filter(|l| {
                        let t = l.trim_start();
                        t.starts_with("error")
                            || t.starts_with("-->")
                            || t.contains("undefined behavior")
                            || t.contains("UNDEFINED BEHAVIOR")
                            || t.starts_with("FAIL")
                            || t.starts_with("test result:")
                    })
                    .collect();
                let focused = if err_lines.is_empty() {
                    combined.clone()
                } else {
                    err_lines.join("\n")
                };
                let snip: String = focused.chars().take(900).collect();
                OracleResult {
                    id: "miri-workspace".into(),
                    path: path_label.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "miri failed (exit {}): {}",
                        status.code().unwrap_or(-1),
                        snip.replace('\n', " ")
                    ),
                }
            }
        }
    }
}

/// ADR-028: `perf-01-release-run` — E2E build+run under `--profile release`, fixed stdout.
pub fn run_perf_release_run_oracle(root: &Path) -> OracleResult {
    let id = "perf-01-release-run";
    let rel = "ejemplos/perf/01-release-run.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    let out = match crate::build_with_profile(path_str, arita_codegen::BuildProfile::Release) {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("release build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&out.bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", out.bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected = vec!["release-ok".to_string()];
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "release build+run ok, stdout match".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// True iff `Cargo.toml` has `[profile.release]` with `opt-level = 3` (ADR-028 evidence).
pub fn cargo_toml_has_release_opt_level_3(toml: &str) -> bool {
    let mut in_release = false;
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
            return true;
        }
    }
    false
}

/// ADR-028: `perf-02-release-optlevel` — generated Cargo.toml contains `opt-level = 3`.
pub fn run_perf_optlevel_oracle(root: &Path) -> OracleResult {
    let id = "perf-02-release-optlevel";
    let rel = "ejemplos/perf/01-release-run.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    let out = match crate::build_with_profile(path_str, arita_codegen::BuildProfile::Release) {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("release build failed: {e}"),
            };
        }
    };
    let Some(cargo_toml_path) = out.cargo_toml.as_ref() else {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "release build produced no Cargo.toml (theater/rustc-only path)".into(),
        };
    };
    let content = match fs::read_to_string(cargo_toml_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: cargo_toml_path.display().to_string(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read generated Cargo.toml: {e}"),
            };
        }
    };
    if cargo_toml_has_release_opt_level_3(&content) {
        OracleResult {
            id: id.into(),
            path: cargo_toml_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "generated Cargo.toml has [profile.release] opt-level = 3".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: cargo_toml_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "generated Cargo.toml missing opt-level = 3 under [profile.release]: {}",
                content.chars().take(400).collect::<String>()
            ),
        }
    }
}

/// ADR-028: `perf-neg-e0250` — CLI `--profile fantasma` → E0250 `unknown build profile`.
pub fn run_perf_e0250_oracle(root: &Path) -> OracleResult {
    let id = "perf-neg-e0250";
    let rel = "ejemplos/perf/neg/e0250-bad-profile.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing neg oracle file: {}", full.display()),
        };
    }
    // Same path as `arita build --profile fantasma <file>` (CLI-only profile).
    let args = vec![
        "--profile".to_string(),
        "fantasma".to_string(),
        full.to_str().unwrap_or(rel).to_string(),
    ];
    match crate::parse_build_args(&args) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0250 for --profile fantasma, but parse_build_args succeeded".into(),
        },
        Err(msg) => {
            if msg.contains("E0250") && msg.contains("unknown build profile") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("CLI rejected with E0250: {msg}"),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("failed without expected E0250 unknown build profile: {msg}"),
                }
            }
        }
    }
}

/// ADR-029: `deps-01-tokio-bridge` — E2E build+run with `[deps] tokio` arita.toml; fixed stdout.
pub fn run_deps_tokio_bridge_oracle(root: &Path) -> OracleResult {
    let id = "deps-01-tokio-bridge";
    let rel = "ejemplos/deps/01-tokio-bridge.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let toml = full
        .parent()
        .map(|d| d.join("arita.toml"))
        .filter(|p| p.is_file());
    if toml.is_none() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "missing ejemplos/deps/arita.toml beside oracle (ADR-029 manifesto)".into(),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    let out = match crate::build_with_profile(path_str, arita_codegen::BuildProfile::Debug) {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("deps tokio bridge build failed: {e}"),
            };
        }
    };
    // Evidence: generated Cargo.toml must carry pinned tokio when [deps] declared.
    if let Some(cargo_toml_path) = out.cargo_toml.as_ref() {
        match fs::read_to_string(cargo_toml_path) {
            Ok(content) => {
                if !content.contains("tokio") || !content.contains("=1.53.1") {
                    return OracleResult {
                        id: id.into(),
                        path: cargo_toml_path.display().to_string(),
                        verdict: Verdict::Rejected,
                        detail: format!(
                            "generated Cargo.toml missing tokio pin from whitelist: {}",
                            content.chars().take(400).collect::<String>()
                        ),
                    };
                }
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: cargo_toml_path.display().to_string(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("cannot read generated Cargo.toml: {e}"),
                };
            }
        }
    } else {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "deps build produced no Cargo.toml (tokio bridge requires Cargo emit)".into(),
        };
    }
    let run = match Command::new(&out.bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", out.bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected = vec!["hi".to_string(), "deps-ok".to_string()];
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "deps tokio bridge E2E ok, stdout match + Cargo.toml pin".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-029: `deps-neg-e0260` — unknown crate in arita.toml `[deps]` → E0260.
pub fn run_deps_e0260_oracle(root: &Path) -> OracleResult {
    let id = "deps-neg-e0260";
    let rel = "ejemplos/deps/neg/e0260-unknown-crate/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing neg oracle file: {}", full.display()),
        };
    }
    let toml = full
        .parent()
        .map(|d| d.join("arita.toml"))
        .filter(|p| p.is_file());
    if toml.is_none() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: "missing sibling arita.toml for E0260 oracle".into(),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    match crate::build_with_profile(path_str, arita_codegen::BuildProfile::Debug) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0260 for unknown [deps] crate, but build succeeded".into(),
        },
        Err(msg) => {
            if msg.contains("E0260") && msg.contains("dependency not in ARITA whitelist") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("build rejected with E0260: {msg}"),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without expected E0260 dependency not in ARITA whitelist: {msg}"
                    ),
                }
            }
        }
    }
}

/// ADR-241 / EVIDENCE-R0R2: codegen production must not inject unwrap/expect/panic!.
pub fn run_emit_ban_r0r2_oracle(root: &Path) -> OracleResult {
    let id = "emit-ban-r0r2";
    let path_label = "crates/arita-codegen (evidence_r0r2_emit_ban)";
    let output = Command::new("cargo")
        .args([
            "test",
            "-p",
            "arita-codegen",
            "evidence_r0r2_emit_ban",
            "--lib",
        ])
        .current_dir(root)
        .output();
    match output {
        Err(e) => OracleResult {
            id: id.into(),
            path: path_label.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("cannot spawn cargo test: {e}"),
        },
        Ok(o) => {
            let status = o.status;
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);
            if status.success() {
                OracleResult {
                    id: id.into(),
                    path: path_label.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "cargo test evidence_r0r2_emit_ban ok (exit {})",
                        status.code().unwrap_or(0)
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: path_label.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "emit-ban R0/R2 failed (exit {}): {}{}",
                        status.code().unwrap_or(-1),
                        stdout.chars().take(400).collect::<String>(),
                        stderr.chars().take(400).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-035 / 033b: `host-01-bridge-hello` — E2E `[host-bridges]` + `host.mark()` → stdout `host-ok`.
pub fn run_host_bridge_hello_oracle(root: &Path) -> OracleResult {
    let id = "host-01-bridge-hello";
    let rel = "ejemplos/host/01-bridge-hello.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let toml = full
        .parent()
        .map(|d| d.join("arita.toml"))
        .filter(|p| p.is_file());
    if toml.is_none() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "missing ejemplos/host/arita.toml beside oracle (ADR-035 [host-bridges])"
                .into(),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    let out = match crate::build_with_profile(path_str, arita_codegen::BuildProfile::Debug) {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("host bridge hello build failed: {e}"),
            };
        }
    };
    // Evidence: generated Cargo.toml must path-dep the host crate.
    if let Some(cargo_toml_path) = out.cargo_toml.as_ref() {
        match fs::read_to_string(cargo_toml_path) {
            Ok(content) => {
                if !content.contains("arita-host-demo") || !content.contains("path") {
                    return OracleResult {
                        id: id.into(),
                        path: cargo_toml_path.display().to_string(),
                        verdict: Verdict::Rejected,
                        detail: format!(
                            "generated Cargo.toml missing host path dep: {}",
                            content.chars().take(400).collect::<String>()
                        ),
                    };
                }
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: cargo_toml_path.display().to_string(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("cannot read generated Cargo.toml: {e}"),
                };
            }
        }
    } else {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "host bridge build produced no Cargo.toml (requires Cargo emit)".into(),
        };
    }
    let run = match Command::new(&out.bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", out.bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected = vec!["host-ok".to_string()];
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "host bridge hello E2E ok, stdout host-ok + Cargo.toml path dep".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-034 pin triples (Windows gnu default; Linux aarch64). msvc OUT v0.
pub const TARGET_WIN_GNU: &str = "x86_64-pc-windows-gnu";
pub const TARGET_LINUX_ARM64: &str = "aarch64-unknown-linux-gnu";

/// Shared gated cross-target build oracle (compile only; no run — foreign binary).
/// Missing rustup target / linker → **inconclusive** (never PASS, never fake reject).
pub fn run_target_build_oracle(root: &Path, id: &str, triple: &str) -> OracleResult {
    let rel = "ejemplos/01-hello.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    if !crate::rustup_target_installed(triple) {
        return OracleResult {
            id: id.into(),
            path: format!("--target {triple}"),
            verdict: Verdict::Inconclusive,
            detail: format!(
                "rustup target `{triple}` not installed (gated; run `rustup target add {triple}`); never treated as accepted"
            ),
        };
    }
    let path_str = full.to_str().unwrap_or(rel);
    match crate::build_with_entry_profile_target(
        path_str,
        "main",
        arita_codegen::BuildProfile::Debug,
        Some(triple),
    ) {
        Ok(out) => {
            if out.bin.is_file() {
                OracleResult {
                    id: id.into(),
                    path: format!("--target {triple}"),
                    verdict: Verdict::Accepted,
                    detail: format!("cross build ok for `{triple}` → {}", out.bin.display()),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: format!("--target {triple}"),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "cargo reported ok but artifact missing: {}",
                        out.bin.display()
                    ),
                }
            }
        }
        Err(e) => {
            if crate::cross_toolchain_unavailable(&e) {
                OracleResult {
                    id: id.into(),
                    path: format!("--target {triple}"),
                    verdict: Verdict::Inconclusive,
                    detail: format!(
                        "cross toolchain/linker unavailable for `{triple}` (gated): {}",
                        e.chars().take(300).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: format!("--target {triple}"),
                    verdict: Verdict::Rejected,
                    detail: format!("cross build failed for `{triple}`: {e}"),
                }
            }
        }
    }
}

pub fn run_target_win_gnu_oracle(root: &Path) -> OracleResult {
    run_target_build_oracle(root, "target-win-gnu", TARGET_WIN_GNU)
}

pub fn run_target_linux_arm64_oracle(root: &Path) -> OracleResult {
    run_target_build_oracle(root, "target-linux-arm64", TARGET_LINUX_ARM64)
}

/// Build+run one ejemplo oracle via the same `build` → rustc pipeline as `arita build`.
pub fn run_ejemplo_oracle(root: &Path, oracle: &EjemploOracle) -> OracleResult {
    let full = root.join(oracle.path);
    if !full.is_file() {
        return OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }

    let bin = match build(full.to_str().unwrap_or(oracle.path)) {
        Ok(p) => p,
        Err(e) => {
            // Build/parse/rustc failure is a definitive reject (E0100 etc.)
            return OracleResult {
                id: oracle.id.into(),
                path: oracle.path.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };

    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: oracle.id.into(),
                path: oracle.path.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };

    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }

    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = oracle.expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        if oracle.require_test {
            let test_path = full.to_str().unwrap_or(oracle.path);
            match crate::run_tests_report_to_stderr(test_path) {
                Ok(()) => OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Accepted,
                    detail: "build+run ok; arita test PASS".into(),
                },
                Err(e) => {
                    // skip ≠ PASS → rejected; spawn-impossible → inconclusive; never accept
                    let spawn_impossible =
                        e.contains("failed to spawn") || e.contains("cannot spawn");
                    if spawn_impossible {
                        OracleResult {
                            id: oracle.id.into(),
                            path: oracle.path.into(),
                            verdict: Verdict::Inconclusive,
                            detail: format!("test tooling cannot spawn: {e}"),
                        }
                    } else {
                        OracleResult {
                            id: oracle.id.into(),
                            path: oracle.path.into(),
                            verdict: Verdict::Rejected,
                            detail: format!("arita test failed: {e}"),
                        }
                    }
                }
            }
        } else {
            OracleResult {
                id: oracle.id.into(),
                path: oracle.path.into(),
                verdict: Verdict::Accepted,
                detail: "build ok, exit 0, stdout match".into(),
            }
        }
    } else {
        OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// Run one negative oracle: accepted iff parse_lower_check fails containing the expected E0xxx code.
pub fn run_neg_oracle(root: &Path, oracle: &NegOracle) -> OracleResult {
    let full = root.join(oracle.path);
    if !full.is_file() {
        return OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing neg oracle file: {}", full.display()),
        };
    }

    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: oracle.id.into(),
                path: oracle.path.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read neg oracle: {e}"),
            };
        }
    };

    // Same gate as CLI `arita parse` / `parse_only`: parse → lower → HIR check.
    // Catches E021x (parse) and E020x (check).
    match crate::parse_lower_check(&src) {
        Ok(_) => OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected fail with {}, but succeeded", oracle.code),
        },
        Err(msg) => {
            // Same text the CLI prints to stderr (`parse_only` → eprintln).
            if msg.contains(oracle.code) {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("rejected with {}: {}", oracle.code, msg),
                }
            } else {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("failed without expected {}: {}", oracle.code, msg),
                }
            }
        }
    }
}

/// Run one F3 logic oracle via `arita_logic::eval_file` (real engine, not rustc).
pub fn run_logic_oracle(root: &Path, oracle: &LogicOracle) -> OracleResult {
    let full = root.join(oracle.path);
    if !full.is_file() {
        return OracleResult {
            id: oracle.id.into(),
            path: oracle.path.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing logic oracle file: {}", full.display()),
        };
    }

    let path_str = full.to_str().unwrap_or(oracle.path);
    match arita_logic::eval_file(path_str) {
        Ok(r) if oracle.expect_sat => {
            if r.all_sat {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("logic ok: {} quer(y/ies) sat", r.queries.len()),
                }
            } else {
                // Empty or partial — never accepted
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Rejected,
                    detail: "logic eval Ok but not all_sat (skip ≠ accepted)".into(),
                }
            }
        }
        Ok(_) => {
            // expect_sat == false but engine succeeded → measure oracle rejected
            OracleResult {
                id: oracle.id.into(),
                path: oracle.path.into(),
                verdict: Verdict::Rejected,
                detail: format!("expected fail with {}, but logic succeeded", oracle.code),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if oracle.expect_sat {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("logic failed (expected sat): {msg}"),
                }
            } else if msg.contains(oracle.code) {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("rejected with {}: {msg}", oracle.code),
                }
            } else {
                OracleResult {
                    id: oracle.id.into(),
                    path: oracle.path.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("failed without expected {}: {msg}", oracle.code),
                }
            }
        }
    }
}

/// ADR-245 typed surface: HttpServer.bind + port + shutdown (ephemeral port).
fn run_core02_http_health_oracle(root: &Path) -> OracleResult {
    let id = "core02-http-health";
    let rel = "ejemplos/core02/http/01-health.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary: {e}"),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got.len() != 2 || got.get(1).map(String::as_str) != Some("http-health-ok") {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected=[<port>,\"http-health-ok\"]"),
        };
    }
    let port_ok = got[0].parse::<i64>().map(|n| n > 0).unwrap_or(false);
    if !port_ok {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("ephemeral port not positive: {:?}", got[0]),
        };
    }
    OracleResult {
        id: id.into(),
        path: rel.into(),
        verdict: Verdict::Accepted,
        detail: format!("HttpServer.bind smoke ok, port={}, http-health-ok", got[0]),
    }
}

/// ADR-245 — HttpServer.bind + .serve(handler) + ephemeral curl GET /health.
fn run_core02_http_serve_client_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core02-http-serve-client";
    let rel = "ejemplos/core02/http/02-serve-health.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn serve binary: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing child stdout pipe".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        Some(Err(e)) => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read port line: {e}"),
            };
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF before port line".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("missing ready line after port={port_line}"),
            };
        }
    };
    if ready != "ready" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected ready, got {ready:?} (port={port_line})"),
        };
    }
    let port: i64 = match port_line.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("bad ephemeral port: {port_line:?}"),
            };
        }
    };
    let url = format!("http://127.0.0.1:{port}/health");
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut body: Option<String> = None;
    let mut curl_err = String::new();
    while Instant::now() < deadline {
        match Command::new("curl").args(["-sS", "-m", "2", &url]).output() {
            Ok(o) if o.status.success() => {
                body = Some(String::from_utf8_lossy(&o.stdout).trim().to_string());
                if body.as_deref() == Some("ok") {
                    break;
                }
                curl_err = format!("body={body:?}");
            }
            Ok(o) => {
                curl_err = format!(
                    "curl exit {:?} stderr={}",
                    o.status.code(),
                    String::from_utf8_lossy(&o.stderr)
                        .chars()
                        .take(120)
                        .collect::<String>()
                );
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("cannot spawn curl: {e}"),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    if body.as_deref() == Some("ok") {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("serve+client ok port={port} GET /health → ok"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("GET /health failed after serve ready: {curl_err}"),
        }
    }
}

/// ADR-246 — set_policy max_body_bytes=16 + serve; POST 17 bytes must not return should-not-see.
fn run_core02_policy_body_cap_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core02-policy-body-cap";
    let rel = "ejemplos/core02/http/07-policy-body-cap.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing stdout pipe".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF/err before port".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("missing body-cap-ready after port={port_line}"),
            };
        }
    };
    if ready != "body-cap-ready" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected body-cap-ready, got {ready:?}"),
        };
    }
    let port: i64 = match port_line.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("bad port {port_line:?}"),
            };
        }
    };
    let url = format!("http://127.0.0.1:{port}/");
    let oversized = "x".repeat(17);
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last = String::new();
    let mut status_ok = false;
    while Instant::now() < deadline {
        match Command::new("curl")
            .args([
                "-sS",
                "-m",
                "2",
                "-o",
                "/tmp/arita-policy-body.out",
                "-w",
                "%{http_code}",
                "-X",
                "POST",
                "-d",
                &oversized,
                &url,
            ])
            .output()
        {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let body =
                    std::fs::read_to_string("/tmp/arita-policy-body.out").unwrap_or_default();
                last = format!("http={code} body={body:?}");
                // accept if not 200 and body is not should-not-see
                if code != "200" && !body.contains("should-not-see") {
                    status_ok = true;
                    break;
                }
                if code == "200" && body.contains("should-not-see") {
                    status_ok = false;
                    break;
                }
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("curl spawn: {e}"),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    if status_ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("policy body-cap ok port={port} POST17 → {last}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("body-cap failed: {last}"),
        }
    }
}

/// ADR-246 — low max_header_*; many headers must reject (not should-not-see).
fn run_core02_policy_header_cap_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core02-policy-header-cap";
    let rel = "ejemplos/core02/http/08-policy-header-cap.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing stdout pipe".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF before port".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("missing header-cap-ready after {port_line}"),
            };
        }
    };
    if ready != "header-cap-ready" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected header-cap-ready, got {ready:?}"),
        };
    }
    let port: i64 = match port_line.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("bad port {port_line:?}"),
            };
        }
    };
    let url = format!("http://127.0.0.1:{port}/");
    // count=2 allows Host + one more typically; send many custom headers to trip cap
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last = String::new();
    let mut ok = false;
    while Instant::now() < deadline {
        let mut cmd = Command::new("curl");
        cmd.args([
            "-sS",
            "-m",
            "2",
            "-o",
            "/tmp/arita-policy-hdr.out",
            "-w",
            "%{http_code}",
            &url,
        ]);
        for i in 0..8 {
            cmd.arg("-H").arg(format!("X-Arita-H{i}: value-{i}-pad"));
        }
        match cmd.output() {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let body = std::fs::read_to_string("/tmp/arita-policy-hdr.out").unwrap_or_default();
                last = format!("http={code} body={body:?}");
                if code != "200" && !body.contains("should-not-see") {
                    ok = true;
                    break;
                }
                if code == "200" && body.contains("should-not-see") {
                    ok = false;
                    break;
                }
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("curl: {e}"),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    if ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("policy header-cap ok port={port} {last}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("header-cap failed: {last}"),
        }
    }
}

/// ADR-245 deferred echo — POST body roundtrip via ok_text.
fn run_core02_http_echo_text_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core02-http-echo-text";
    let rel = "ejemplos/core02/http/03-echo-text.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing stdout".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF before port".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "missing ready".into(),
            };
        }
    };
    if ready != "ready" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected ready got {ready:?}"),
        };
    }
    let port: i64 = match port_line.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("bad port {port_line:?}"),
            };
        }
    };
    let url = format!("http://127.0.0.1:{port}/");
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last = String::new();
    let mut echo_ok = false;
    while Instant::now() < deadline {
        match Command::new("curl")
            .args(["-sS", "-m", "2", "-X", "POST", "-d", "hello-echo", &url])
            .output()
        {
            Ok(o) if o.status.success() => {
                let body = String::from_utf8_lossy(&o.stdout).trim().to_string();
                last = format!("body={body:?}");
                if body == "hello-echo" {
                    echo_ok = true;
                    break;
                }
            }
            Ok(o) => {
                last = format!(
                    "curl {:?} {}",
                    o.status.code(),
                    String::from_utf8_lossy(&o.stderr)
                        .chars()
                        .take(80)
                        .collect::<String>()
                );
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("curl: {e}"),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    if echo_ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("echo ok port={port}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("echo failed: {last}"),
        }
    }
}

/// ADR-245 deferred body-cap — POST 1048577 must not return should-not-see.
fn run_core02_http_body_cap_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core02-http-body-cap";
    let rel = "ejemplos/core02/http/04-body-cap.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing stdout".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF before port".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "missing ready".into(),
            };
        }
    };
    if ready != "ready" && ready != "body-cap-ready" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected ready, got {ready:?}"),
        };
    }
    let port: i64 = match port_line.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("bad port {port_line:?}"),
            };
        }
    };
    let url = format!("http://127.0.0.1:{port}/");
    let oversized = "y".repeat(1_048_577);
    let tmp = root.join("target/arita-measure-bodycap.bin");
    let _ = std::fs::write(&tmp, &oversized);
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut last = String::new();
    let mut ok = false;
    while Instant::now() < deadline {
        match Command::new("curl")
            .args([
                "-sS",
                "-m",
                "5",
                "-o",
                "/tmp/arita-http-bodycap.out",
                "-w",
                "%{http_code}",
                "-X",
                "POST",
                "--data-binary",
                &format!("@{}", tmp.display()),
                &url,
            ])
            .output()
        {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let body =
                    std::fs::read_to_string("/tmp/arita-http-bodycap.out").unwrap_or_default();
                last = format!("http={code} body_len={}", body.len());
                if code != "200" && !body.contains("should-not-see") {
                    ok = true;
                    break;
                }
                if code == "200" && body.contains("should-not-see") {
                    ok = false;
                    break;
                }
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("curl: {e}"),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    if ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("http body-cap ok port={port} {last}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("http body-cap failed: {last}"),
        }
    }
}

/// ADR-265 / CORE-0.6-SET-FALLIBLE-20260926 — in-bounds set → Ok + value via get (stdout ok).
fn run_core06_set_ok_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-set-ok",
            path: "ejemplos/core06/set-fallible/01-ok.arita",
            expected: &["ok"],
            require_test: false,
        },
    )
}

/// ADR-265 — OOB set → Err(0); stdout err; no panic.
fn run_core06_set_oob_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-set-oob",
            path: "ejemplos/core06/set-fallible/02-err-oob.arita",
            expected: &["err"],
            require_test: false,
        },
    )
}

/// ADR-265 / ADR-270 — lit i < 0 → E0319.
fn run_neg_core06_set_neg_lit_oracle(root: &Path) -> OracleResult {
    let id = "neg-core06-set-neg-lit";
    let rel = "ejemplos/core06/set-fallible/neg/01-neg-lit.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0319, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0319") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0319: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0319: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-265 — emit uses __arita_vec_set; no IndexMut / panic assign in main path.
fn run_core06_set_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core06-set-emit-ban";
    let rel = "ejemplos/core06/set-fallible/01-ok.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("core06_set_ok") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core06_set_ok*.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    let has_helper = txt.contains("fn __arita_vec_set");
    let has_call = txt.contains("__arita_vec_set(&mut");
    let has_index_mut = txt.contains("IndexMut") || txt.contains("std::ops::Index");
    let after_helper = txt.split("fn __arita_vec_set").nth(1).unwrap_or("");
    let after_fn = after_helper.split("fn main").nth(1).unwrap_or("");
    // Ban bare IndexMut-style assign in main: `] =` not via helper call path
    let bad_main_assign = after_fn.contains("] =") || after_fn.contains("]=");
    if has_helper && has_call && !has_index_mut && !bad_main_assign {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has __arita_vec_set; no IndexMut / panic assign in main".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail helper={has_helper} call={has_call} index_mut={has_index_mut} bad_main_assign={bad_main_assign}"
            ),
        }
    }
}

/// ADR-265 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @175..183` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core06_index_mut_assign_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core06-index-mut-assign",
        "ejemplos/core06/set-fallible/neg/02-index-mut-assign.arita",
        "175..183",
    )
}

/// ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): the 7 Vec-in-main negatives (was E0314) must fail
/// with the EXACT E0344 message + span; E0314 anywhere in the diag → Rejected (no vacuous pass).
fn adr283_m1_vec_assign_e0344_oracle(root: &Path, id: &str, rel: &str, span: &str) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let expected = format!("E0344: index assign outside result fn @{span}");
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected `{expected}`, but build succeeded"),
        },
        Err(e) => {
            let first = e.lines().next().unwrap_or("").trim();
            if first == expected && !e.contains("E0314") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("rejected with E0344 (M1 ADR-283): {first}"),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "expected exact `{expected}`, got: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-266 / CORE-0.6-MAP-INDEX-20260926 — m[k] hit → Some (stdout 7).
fn run_core06_map_index_some_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-map-index-some",
            path: "ejemplos/core06/map-index/01-some.arita",
            expected: &["7"],
            require_test: false,
        },
    )
}

/// ADR-266 — miss → None (stdout none; no panic).
fn run_core06_map_index_none_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-map-index-none",
            path: "ejemplos/core06/map-index/02-none.arita",
            expected: &["none"],
            require_test: false,
        },
    )
}

/// ADR-266 — m[k] ≡ m.get(k).
fn run_core06_map_index_eq_get_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-map-index-eq-get",
            path: "ejemplos/core06/map-index/03-eq-get.arita",
            expected: &["eq"],
            require_test: false,
        },
    )
}

/// ADR-266 — emit uses get path; no Index/IndexMut / panic `}[`.
fn run_core06_map_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core06-map-emit-ban";
    let rel = "ejemplos/core06/map-index/01-some.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("core06_map_index_some") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core06_map_index_some*.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    let has_get = txt.contains(".get(");
    let has_index_trait = txt.contains("IndexMut") || txt.contains("std::ops::Index");
    let panic_index = "}[";
    let has_panic_index = txt.contains(panic_index);
    if has_get && !has_index_trait && !has_panic_index {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit uses get path; no Index/IndexMut panic".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail get={has_get} index_trait={has_index_trait} panic_index={has_panic_index}"
            ),
        }
    }
}

/// ADR-266 → ADR-282 (CORE-0.9-MAP-ASSIGN-20260926) M1 retarget: `m[k] = v` is legal on `let mut`
/// (ADR-282 slice 1), so the neg now pins the non-mut binding → EXACT E0202 `borrow conflict`
/// (≡ `m.put` on `let m`). Was E0314 (IndexMut HOLD). Same id, still neg, still counted.
/// Neighbour codes E0314 / E0205 / E0001 are rejected explicitly (no vacuous pass).
fn run_neg_core06_map_index_mut_oracle(root: &Path) -> OracleResult {
    let id = "neg-core06-map-index-mut";
    let rel = "ejemplos/core06/map-index/neg/01-index-mut.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0202, but build succeeded".into(),
        },
        Err(e) => {
            let neighbour = e.contains("E0314") || e.contains("E0205") || e.contains("E0001");
            if e.contains("E0202") && !neighbour {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0202: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without exact E0202: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-267 — CLI/JSON + set Ok + Map m[k] Some → stdout gp-happy.
fn run_core06_scen_gp_happy_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-scen-gp-happy",
            path: "ejemplos/core06/scenario-gp/01-happy.arita",
            expected: &["gp-happy"],
            require_test: false,
        },
    )
}

/// ADR-267 — set OOB Err(0) + Map miss None → stdout gp-oob (no panic).
fn run_core06_scen_gp_oob_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core06-scen-gp-oob",
            path: "ejemplos/core06/scenario-gp/02-oob.arita",
            expected: &["gp-oob"],
            require_test: false,
        },
    )
}

/// ADR-267 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @149..157` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core06_scen_gp_index_mut_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core06-scen-gp-index-mut",
        "ejemplos/core06/scenario-gp/neg/01-index-mut.arita",
        "149..157",
    )
}

/// ADR-267 — lit i < 0 set → E0319 (wired per ADR-267 §2; also covered by 265).
fn run_neg_core06_scen_gp_set_neg_oracle(root: &Path) -> OracleResult {
    let id = "neg-core06-scen-gp-set-neg";
    let rel = "ejemplos/core06/scenario-gp/neg/02-set-neg.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0319, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0319") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0319: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0319: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-267 — happy scenario builds green (measure build).
fn run_core06_scen_gp_build_oracle(root: &Path) -> OracleResult {
    let id = "core06-scen-gp-build";
    let rel = "ejemplos/core06/scenario-gp/01-happy.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(bin) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("build ok → {}", bin.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("build failed: {}", e.chars().take(200).collect::<String>()),
        },
    }
}

/// ADR-268 — workspace lib+bin build green for arita-ref-gp.
fn run_core06_ref_gp_build_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-build";
    let rel = "ejemplos/core06/ref-gp/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace build ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-268 — CLI happy: lib apply_patch/peek_key (set Ok + Map Some) → gp-ref-happy.
fn run_core06_ref_gp_cli_happy_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-cli-happy";
    let rel = "ejemplos/core06/ref-gp/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["gp-ref-happy".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("CLI→lib apply_patch/peek_key → gp-ref-happy (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"gp-ref-happy\"]"),
        }
    }
}

/// ADR-268 — CLI oob: set Err + Map miss None → gp-ref-oob (no panic).
fn run_core06_ref_gp_cli_oob_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-cli-oob";
    let rel = "ejemplos/core06/ref-gp/edge/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["gp-ref-oob".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("oob path stable → gp-ref-oob (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"gp-ref-oob\"]"),
        }
    }
}

/// ADR-268 — ≥2 scenarios PASS (happy + oob CLIs; aligned ADR-267).
fn run_core06_ref_gp_scenario_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-scenario";
    let happy = root.join("ejemplos/core06/ref-gp/bin/main.arita");
    let oob = root.join("ejemplos/core06/ref-gp/edge/bin/main.arita");
    let scen_gp = root.join("ejemplos/core06/scenario-gp/01-happy.arita");
    if !(happy.is_file() && oob.is_file()) {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core06/ref-gp/bin".into(),
            verdict: Verdict::Rejected,
            detail: "need ≥2 acceptance bins (main+edge)".into(),
        };
    }
    let scen_ok = scen_gp.is_file();
    let h = run_core06_ref_gp_cli_happy_oracle(root);
    let e = run_core06_ref_gp_cli_oob_oracle(root);
    if h.verdict == Verdict::Accepted && e.verdict == Verdict::Accepted {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core06/ref-gp".into(),
            verdict: Verdict::Accepted,
            detail: format!("≥2 scenarios PASS (happy+oob); scenario-gp present={scen_ok}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core06/ref-gp".into(),
            verdict: Verdict::Rejected,
            detail: format!("scenario fail happy={:?} oob={:?}", h.verdict, e.verdict),
        }
    }
}

/// ADR-268 — evidence JSON + stable sha256; measure_pass:true after Core 0.6 CLOSED.
fn run_core06_ref_gp_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-evidence";
    let ev_rel = "ejemplos/core06/ref-gp/evidence.json";
    let pairs = [
        ("\"lib_sha256\"", "ejemplos/core06/ref-gp/lib/lib.arita"),
        ("\"bin_sha256\"", "ejemplos/core06/ref-gp/bin/main.arita"),
        (
            "\"edge_sha256\"",
            "ejemplos/core06/ref-gp/edge/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    if !ev_raw.contains("\"cut_id\": \"CORE-0.6-REF-GP-20260926\"") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must pin cut_id CORE-0.6-REF-GP-20260926".into(),
        };
    }
    if !ev_raw.contains("\"measure_pass\": true") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must keep measure_pass:true after Core 0.6 CLOSED".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        Ok(rest[..q2].to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence lib+bin+edge sha256 match; measure_pass:true".into(),
    }
}

/// ADR-268 — emit grep: __arita_vec_set + .get(; no IndexMut / panic set/`[]`.
fn run_core06_ref_gp_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core06-ref-gp-emit-ban";
    let rel = "ejemplos/core06/ref-gp/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let mut rs_files: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&crates_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("bin_") {
                let p = e.path().join("src").join("main.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
            if name.starts_with("lib_") {
                let p = e.path().join("src").join("lib.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
        }
    }
    if rs_files.is_empty() {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no emitted crates/*.rs".into(),
        };
    }
    let mut combined = String::new();
    for p in &rs_files {
        match fs::read_to_string(p) {
            Ok(s) => {
                combined.push_str(&s);
                combined.push('\n');
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: p.display().to_string(),
                    verdict: Verdict::Rejected,
                    detail: format!("read emit: {e}"),
                };
            }
        }
    }
    let has_helper = combined.contains("fn __arita_vec_set");
    let has_call = combined.contains("__arita_vec_set(&mut");
    let has_get = combined.contains(".get(");
    let has_index_trait = combined.contains("IndexMut") || combined.contains("std::ops::Index");
    let has_panic_index = combined.contains("}[");
    let has_bare_index_assign = combined.contains("] =") || combined.contains("]=");
    if has_helper
        && has_call
        && has_get
        && !has_index_trait
        && !has_panic_index
        && !has_bare_index_assign
    {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "emit has __arita_vec_set + get; no IndexMut / panic set/[]".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail helper={has_helper} call={has_call} get={has_get} index_trait={has_index_trait} panic_index={has_panic_index} bare_assign={has_bare_index_assign}"
            ),
        }
    }
}

/// ADR-268 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @148..156` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core06_ref_gp_index_mut_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core06-ref-gp-index-mut",
        "ejemplos/core06/ref-gp-neg/01-index-mut.arita",
        "148..156",
    )
}

/// ADR-268 — lit set(-1,…) → E0319 (local; also covered by 265/267).
fn run_neg_core06_ref_gp_set_neg_oracle(root: &Path) -> OracleResult {
    let id = "neg-core06-ref-gp-set-neg";
    let rel = "ejemplos/core06/ref-gp-neg/02-set-neg.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0319, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0319") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0319: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0319: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-272 / CORE-0.7-IO-PARSE-H1-20260926 — read Ok → fixture stdout.
/// Runs binary with `current_dir(root)` so relative host.read_text paths resolve.
fn run_core07_io_h1_read_ok_oracle(root: &Path) -> OracleResult {
    run_core07_io_h1_ejemplo(
        root,
        "core07-io-h1-read-ok",
        "ejemplos/core07/io-parse-h1/01-read-ok.arita",
        &["hello-io-h1"],
    )
}

/// ADR-272 — missing file → Err arm prints read_failed; no panic.
fn run_core07_io_h1_read_err_oracle(root: &Path) -> OracleResult {
    run_core07_io_h1_ejemplo(
        root,
        "core07-io-h1-read-err",
        "ejemplos/core07/io-parse-h1/02-read-err.arita",
        &["read_failed"],
    )
}

fn run_core07_io_h1_ejemplo(root: &Path, id: &str, rel: &str, expected: &[&str]) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "build ok, exit 0, stdout match (cwd=root)".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-272 — discard host.read_text Result → E0340.
fn run_neg_core07_io_h1_discard_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-io-h1-discard";
    let rel = "ejemplos/core07/io-parse-h1/neg/01-discard.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0340, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0340") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0340: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0340: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-272 — unwrap_or lit vacío tras read → E0340.
fn run_neg_core07_io_h1_default_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-io-h1-default";
    let rel = "ejemplos/core07/io-parse-h1/neg/02-default.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0340, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0340") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0340: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0340: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-272 / ADR-241 — emit path has zero unwrap/expect/panic in user main.
fn run_core07_io_h1_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core07-io-h1-emit-ban";
    let rel = "ejemplos/core07/io-parse-h1/01-read-ok.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            // Host-bridge Cargo emit: arita_host_core07_io_h1_read_ok_<hash>/src/main.rs
            if name.contains("core07_io_h1_read_ok") {
                let cand = e.path().join("src").join("main.rs");
                if cand.is_file() {
                    rs_path = Some(cand);
                    break;
                }
            }
            // Direct .rs emit (non-host path)
            if name.starts_with("core07_io_h1_read_ok") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core07_io_h1_read_ok src/main.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    core07_io_h1_emit_ban_check(id, &rs_path, &txt)
}

fn core07_io_h1_emit_ban_check(id: &str, rs_path: &Path, txt: &str) -> OracleResult {
    // ADR-241: no .unwrap() / .expect( / panic! in emitted user path
    let has_unwrap = txt.contains(".unwrap()");
    let has_expect = txt.contains(".expect(");
    let has_panic = txt.contains(concat!("panic!", "("));
    if !has_unwrap && !has_expect && !has_panic {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has zero unwrap/expect/panic".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic}"
            ),
        }
    }
}

/// ADR-273 / CORE-0.7-CLI-ARGV-H2-20260926 — cli_arg Some → stdout (argv[1]).
fn run_core07_io_h2_arg_ok_oracle(root: &Path) -> OracleResult {
    run_core07_io_h2_ejemplo_with_args(
        root,
        "core07-io-h2-arg-ok",
        "ejemplos/core07/cli-argv-h2/01-arg-ok.arita",
        &["hello-arg"],
        &["hello-arg"],
    )
}

/// ADR-273 — required miss → None arm prints usage; no panic.
fn run_core07_io_h2_arg_miss_oracle(root: &Path) -> OracleResult {
    run_core07_io_h2_ejemplo_with_args(
        root,
        "core07-io-h2-arg-miss",
        "ejemplos/core07/cli-argv-h2/02-arg-miss.arita",
        &[],
        &["usage"],
    )
}

/// ADR-273 — required json_get_int None → missing_n.
fn run_core07_io_h2_json_miss_oracle(root: &Path) -> OracleResult {
    run_core07_io_h2_ejemplo_with_args(
        root,
        "core07-io-h2-json-miss",
        "ejemplos/core07/cli-argv-h2/03-json-miss.arita",
        &[],
        &["missing_n"],
    )
}

fn run_core07_io_h2_ejemplo_with_args(
    root: &Path,
    id: &str,
    rel: &str,
    bin_args: &[&str],
    expected: &[&str],
) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).args(bin_args).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("build ok, exit 0, stdout match (cwd=root, args={bin_args:?})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-273 — unwrap_or lit vacío tras cli_arg → E0341.
fn run_neg_core07_io_h2_default_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-io-h2-default";
    let rel = "ejemplos/core07/cli-argv-h2/neg/01-default.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0341, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0341") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0341: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0341: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-273 — args[i] / Index argv → E0341.
fn run_neg_core07_io_h2_index_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-io-h2-index";
    let rel = "ejemplos/core07/cli-argv-h2/neg/02-index.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0341, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0341") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0341: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0341: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-273 / ADR-241 — emit: zero unwrap/expect/panic + zero Index/argv[] theater.
fn run_core07_io_h2_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core07-io-h2-emit-ban";
    let rel = "ejemplos/core07/cli-argv-h2/01-arg-ok.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.contains("core07_io_h2_arg_ok") {
                let cand = e.path().join("src").join("main.rs");
                if cand.is_file() {
                    rs_path = Some(cand);
                    break;
                }
            }
            if name.starts_with("core07_io_h2_arg_ok") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core07_io_h2_arg_ok src/main.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    core07_io_h2_emit_ban_check(id, &rs_path, &txt)
}

fn core07_io_h2_emit_ban_check(id: &str, rs_path: &Path, txt: &str) -> OracleResult {
    let has_unwrap = txt.contains(".unwrap()");
    let has_expect = txt.contains(".expect(");
    let has_panic = txt.contains(concat!("panic!", "("));
    let has_index_trait = txt.contains("IndexMut") || txt.contains("std::ops::Index");
    // Ban argv Index panic path / args[i] theater in emit
    let has_args_index = txt.contains("args[") || txt.contains("argv[");
    if !has_unwrap && !has_expect && !has_panic && !has_index_trait && !has_args_index {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has zero unwrap/expect/panic/Index/args[]".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic} index={has_index_trait} args_idx={has_args_index}"
            ),
        }
    }
}

/// ADR-274 / CORE-0.7-SCENARIO-IO-20260926 — cli_arg Some + read Ok + json_get Some → io-happy.
fn run_core07_scen_io_happy_oracle(root: &Path) -> OracleResult {
    run_core07_io_h2_ejemplo_with_args(
        root,
        "core07-scen-io-happy",
        "ejemplos/core07/scenario-io/01-happy.arita",
        &["ejemplos/core07/scenario-io/fixture.json"],
        &["io-happy"],
    )
}

/// ADR-274 — read Err (missing path) → io-fail; no panic.
fn run_core07_scen_io_fail_oracle(root: &Path) -> OracleResult {
    run_core07_io_h2_ejemplo_with_args(
        root,
        "core07-scen-io-fail",
        "ejemplos/core07/scenario-io/02-fail.arita",
        &["ejemplos/core07/scenario-io/no-such-file.json"],
        &["io-fail"],
    )
}

/// ADR-274 — scenario-context discard Result → E0340.
fn run_neg_core07_scen_io_h1_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-scen-io-h1";
    let rel = "ejemplos/core07/scenario-io/neg/01-h1.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0340, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0340") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0340: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0340: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-274 — scenario-context unwrap_or tras cli_arg required → E0341.
fn run_neg_core07_scen_io_h2_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-scen-io-h2";
    let rel = "ejemplos/core07/scenario-io/neg/02-h2.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0341, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0341") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0341: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0341: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-274 — happy scenario builds green (measure build).
fn run_core07_scen_io_build_oracle(root: &Path) -> OracleResult {
    let id = "core07-scen-io-build";
    let rel = "ejemplos/core07/scenario-io/01-happy.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(bin) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("build ok → {}", bin.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("build failed: {}", e.chars().take(200).collect::<String>()),
        },
    }
}

/// ADR-275 / CORE-0.7-REF-IO-20260926 — workspace lib+bin build verde.
fn run_core07_ref_io_build_oracle(root: &Path) -> OracleResult {
    let id = "core07-ref-io-build";
    let rel = "ejemplos/core07/ref-io/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace build ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-275 — CLI happy: cargo run with fixture path → io-ref-happy via lib.
fn run_core07_ref_io_cli_happy_oracle(root: &Path) -> OracleResult {
    run_core07_ref_io_cli(
        root,
        "core07-ref-io-cli-happy",
        "ejemplos/core07/ref-io/bin/main.arita",
        &["ejemplos/core07/ref-io/fixture.json"],
        &["io-ref-happy"],
    )
}

/// ADR-275 — CLI fail: miss path → io-ref-fail (no panic).
fn run_core07_ref_io_cli_fail_oracle(root: &Path) -> OracleResult {
    run_core07_ref_io_cli(
        root,
        "core07-ref-io-cli-fail",
        "ejemplos/core07/ref-io/edge/bin/main.arita",
        &["ejemplos/core07/ref-io/no-such-file.json"],
        // ADR-286 §3 fila 12 / L124: fichero ausente → read_text Err(0) → load_missing → Err(0).
        &["io-ref-fail", "0"],
    )
}

/// ADR-286 §3 L127: `expected` is the full stdout, one entry per line (multi-line since the
/// dead-sink migration prints label + payload); compared as an exact vector, never a prefix.
fn run_core07_ref_io_cli(
    root: &Path,
    id: &str,
    rel: &str,
    bin_args: &[&str],
    expected: &[&str],
) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .arg("--")
        .args(bin_args)
        .current_dir(root);
    let run = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let want: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == want {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("CLI→lib → {want:?} (cargo run -p {bin_pkg} args={bin_args:?})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected={want:?}"),
        }
    }
}

/// ADR-275 — ≥2 scenarios PASS (happy + fail CLIs; aligned ADR-274).
fn run_core07_ref_io_scenario_oracle(root: &Path) -> OracleResult {
    let id = "core07-ref-io-scenario";
    let happy = root.join("ejemplos/core07/ref-io/bin/main.arita");
    let fail = root.join("ejemplos/core07/ref-io/edge/bin/main.arita");
    let scen_io = root.join("ejemplos/core07/scenario-io/01-happy.arita");
    if !(happy.is_file() && fail.is_file()) {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core07/ref-io/bin".into(),
            verdict: Verdict::Rejected,
            detail: "need ≥2 acceptance bins (main+edge)".into(),
        };
    }
    let scen_ok = scen_io.is_file();
    let h = run_core07_ref_io_cli_happy_oracle(root);
    let e = run_core07_ref_io_cli_fail_oracle(root);
    if h.verdict == Verdict::Accepted && e.verdict == Verdict::Accepted {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core07/ref-io".into(),
            verdict: Verdict::Accepted,
            detail: format!("≥2 scenarios PASS (happy+fail); scenario-io present={scen_ok}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core07/ref-io".into(),
            verdict: Verdict::Rejected,
            detail: format!("scenario fail happy={:?} fail={:?}", h.verdict, e.verdict),
        }
    }
}

/// ADR-275 — evidence JSON + stable sha256; measure_pass:true after Core 0.7 CLOSED.
fn run_core07_ref_io_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core07-ref-io-evidence";
    let ev_rel = "ejemplos/core07/ref-io/evidence.json";
    let pairs = [
        ("\"lib_sha256\"", "ejemplos/core07/ref-io/lib/lib.arita"),
        ("\"bin_sha256\"", "ejemplos/core07/ref-io/bin/main.arita"),
        (
            "\"edge_sha256\"",
            "ejemplos/core07/ref-io/edge/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    if !ev_raw.contains("\"cut_id\": \"CORE-0.7-REF-IO-20260926\"") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must pin cut_id CORE-0.7-REF-IO-20260926".into(),
        };
    }
    if !ev_raw.contains("\"measure_pass\": true") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must keep measure_pass:true after Core 0.7 CLOSED".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        Ok(rest[..q2].to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence lib+bin+edge sha256 match; measure_pass:true".into(),
    }
}

/// ADR-275 / ADR-241 — emit grep: cero unwrap/expect/panic / argv Index.
fn run_core07_ref_io_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core07-ref-io-emit-ban";
    let rel = "ejemplos/core07/ref-io/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let mut rs_files: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&crates_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("bin_") {
                let p = e.path().join("src").join("main.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
            if name.starts_with("lib_") {
                let p = e.path().join("src").join("lib.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
        }
    }
    if rs_files.is_empty() {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no emitted crates/*.rs".into(),
        };
    }
    let mut combined = String::new();
    for p in &rs_files {
        match fs::read_to_string(p) {
            Ok(s) => {
                combined.push_str(&s);
                combined.push('\n');
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: p.display().to_string(),
                    verdict: Verdict::Rejected,
                    detail: format!("read emit: {e}"),
                };
            }
        }
    }
    let has_unwrap = combined.contains(".unwrap()");
    let has_expect = combined.contains(".expect(");
    let has_panic = combined.contains(concat!("panic!", "("));
    let has_index_trait = combined.contains("IndexMut") || combined.contains("std::ops::Index");
    let has_args_index = combined.contains("args[") || combined.contains("argv[");
    if !has_unwrap && !has_expect && !has_panic && !has_index_trait && !has_args_index {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "emit has zero unwrap/expect/panic/Index/args[]".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic} index={has_index_trait} args_idx={has_args_index}"
            ),
        }
    }
}

/// ADR-275 — local discard Result → E0340 (also covered by 272/274).
fn run_neg_core07_ref_io_h1_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-ref-io-h1";
    let rel = "ejemplos/core07/ref-io-neg/01-h1.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0340, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0340") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0340: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0340: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-275 — local unwrap_or after cli_arg → E0341 (also covered by 273/274).
fn run_neg_core07_ref_io_h2_oracle(root: &Path) -> OracleResult {
    let id = "neg-core07-ref-io-h2";
    let rel = "ejemplos/core07/ref-io-neg/02-h2.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0341, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0341") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0341: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0341: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-277 / CORE-0.8-FN-RESULT-20260926 — helper fn→Result Ok path → fixture stdout.
fn run_core08_fn_result_ok_oracle(root: &Path) -> OracleResult {
    run_core08_fn_result_ejemplo(
        root,
        "core08-fn-result-ok",
        "ejemplos/core08/fn-result/01-ok.arita",
        &["hello-fn-result"],
    )
}

/// ADR-277 — helper Err → caller match fail determinista (no panic).
fn run_core08_fn_result_err_oracle(root: &Path) -> OracleResult {
    run_core08_fn_result_ejemplo(
        root,
        "core08-fn-result-err",
        "ejemplos/core08/fn-result/02-err.arita",
        // ADR-286 §3 fila 2 / L121: brazo Err migrado imprime etiqueta + payload (Err(0) de read_text).
        &["load_failed", "0"],
    )
}

/// ADR-277 — fn main() -> Io<()> still OK calling helper Result via match.
fn run_core08_fn_result_main_io_oracle(root: &Path) -> OracleResult {
    run_core08_fn_result_ejemplo(
        root,
        "core08-fn-result-main-io",
        "ejemplos/core08/fn-result/03-main-io.arita",
        &["main-io-ok"],
    )
}

fn run_core08_fn_result_ejemplo(
    root: &Path,
    id: &str,
    rel: &str,
    expected: &[&str],
) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "build ok, exit 0, stdout match (cwd=root)".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-277 — bad return theater → E0342 (Parser soft pin; Codegen owns ejemplo).
fn run_neg_core08_fn_result_theater_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-fn-result-theater";
    let rel = "ejemplos/core08/fn-result/neg/01-theater.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0342, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0342") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0342: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0342: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-277 — invent surface unwrap/expect → reject (gate E0206; ≠ E0291 reopen).
fn run_neg_core08_fn_result_unwrap_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-fn-result-unwrap";
    let rel = "ejemplos/core08/fn-result/neg/02-unwrap.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected reject (E0206 unwrap invent), but build succeeded".into(),
        },
        Err(e) => {
            // Documented gate: E0206 whitelist (≠ E0291). Also accept explicit expect invent.
            if e.contains("E0206") || e.contains("E0342") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected invent unwrap/expect: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0206/E0342: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-277 / ADR-241 — emit path has zero unwrap/expect/panic in user load+main.
fn run_core08_fn_result_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core08-fn-result-emit-ban";
    let rel = "ejemplos/core08/fn-result/01-ok.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.contains("core08_fn_result_ok") {
                let cand = e.path().join("src").join("main.rs");
                if cand.is_file() {
                    rs_path = Some(cand);
                    break;
                }
            }
            if name.starts_with("core08_fn_result_ok") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core08_fn_result_ok src/main.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    core08_fn_result_emit_ban_check(id, &rs_path, &txt)
}

fn core08_fn_result_emit_ban_check(id: &str, rs_path: &Path, txt: &str) -> OracleResult {
    let has_unwrap = txt.contains(".unwrap()");
    let has_expect = txt.contains(".expect(");
    let has_panic = txt.contains(concat!("panic!", "("));
    let has_result_sig = txt.contains("fn load(") && txt.contains("-> Result<");
    if !has_unwrap && !has_expect && !has_panic && has_result_sig {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has Result sig + zero unwrap/expect/panic".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic} result_sig={has_result_sig}"
            ),
        }
    }
}

/// ADR-278 / CORE-0.8-QMARK-20260926 — ? chain in fn→Result Ok → fixture stdout.
fn run_core08_qmark_chain_oracle(root: &Path) -> OracleResult {
    run_core08_qmark_ejemplo(
        root,
        "core08-qmark-chain",
        "ejemplos/core08/qmark/01-chain.arita",
        &["hello-qmark"],
    )
}

/// ADR-278 — first ? Err → early return → main match fail path.
fn run_core08_qmark_err_oracle(root: &Path) -> OracleResult {
    run_core08_qmark_ejemplo(
        root,
        "core08-qmark-err",
        "ejemplos/core08/qmark/02-err.arita",
        // ADR-286 §3 fila 5 / L122.
        &["qmark_failed", "0"],
    )
}

/// ADR-278 — main Io match-convert sin ? → PASS.
fn run_core08_qmark_main_io_match_oracle(root: &Path) -> OracleResult {
    run_core08_qmark_ejemplo(
        root,
        "core08-qmark-main-io-match",
        "ejemplos/core08/qmark/03-main-io-match.arita",
        &["main-io-qmark-ok"],
    )
}

fn run_core08_qmark_ejemplo(root: &Path, id: &str, rel: &str, expected: &[&str]) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "build ok, exit 0, stdout match (cwd=root)".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-278 — ? inside fn → Io<()> / non-Result → E0343.
fn run_neg_core08_qmark_outside_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-qmark-outside";
    let rel = "ejemplos/core08/qmark/neg/01-outside.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0343, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0343") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0343: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0343: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-278 — ? on Option → E0203 (no invent E0xxx). Hardened for ADR-286 §2.3 L92 / §3 fila 7
/// (Ingeniero decision 12:18): EXACT single diagnostic, head = canonical ADR-006 L108 text.
const CORE08_QMARK_OPTION_HEAD: &str = "E0203: type mismatch";

fn run_neg_core08_qmark_option_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-qmark-option";
    let rel = "ejemplos/core08/qmark/neg/02-option.arita";
    match core09_expect_exact(root, rel, "E0203") {
        Ok(e) => {
            let head = core09_diag_head(&e);
            let codes = core09_diag_codes(&e);
            if head == CORE08_QMARK_OPTION_HEAD && codes.len() == 1 {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Accepted,
                    format!("rejected with exact E0203: {head} (1 diagnostic)"),
                )
            } else {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!(
                        "E0203 but not exact/unique: head={head:?} codes={codes:?} (want {CORE08_QMARK_OPTION_HEAD:?}, 1 diagnostic)"
                    ),
                )
            }
        }
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// ADR-278 / ADR-241 — emit has ? (or match early-return) + zero unwrap/expect/panic.
fn run_core08_qmark_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core08-qmark-emit-ban";
    let rel = "ejemplos/core08/qmark/01-chain.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.contains("core08_qmark_chain") {
                let cand = e.path().join("src").join("main.rs");
                if cand.is_file() {
                    rs_path = Some(cand);
                    break;
                }
            }
            if name.starts_with("core08_qmark_chain") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core08_qmark_chain src/main.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    core08_qmark_emit_ban_check(id, &rs_path, &txt)
}

fn core08_qmark_emit_ban_check(id: &str, rs_path: &Path, txt: &str) -> OracleResult {
    let has_unwrap = txt.contains(".unwrap()");
    let has_expect = txt.contains(".expect(");
    let has_panic = txt.contains(concat!("panic!", "("));
    let has_qmark = txt.contains("?;") || txt.contains(")?");
    let has_match_early = txt.contains("return Err(");
    let has_prop = has_qmark || has_match_early;
    let has_result_sig = txt.contains("fn load(") && txt.contains("-> Result<");
    if !has_unwrap && !has_expect && !has_panic && has_prop && has_result_sig {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has Result sig + ?/match-early + zero unwrap/expect/panic".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic} prop={has_prop} result_sig={has_result_sig}"
            ),
        }
    }
}

/// ADR-279 / CORE-0.8-SCENARIO-ERRPROP-20260926 — happy ? chain Ok → fixture stdout.
fn run_core08_scen_errprop_happy_oracle(root: &Path) -> OracleResult {
    run_core08_scen_errprop_ejemplo(
        root,
        "core08-scen-errprop-happy",
        "ejemplos/core08/scenario-errprop/01-happy.arita",
        &["errprop-happy"],
    )
}

/// ADR-279 — Err via ? early-return → main match fail path (no panic).
fn run_core08_scen_errprop_err_oracle(root: &Path) -> OracleResult {
    run_core08_scen_errprop_ejemplo(
        root,
        "core08-scen-errprop-err",
        "ejemplos/core08/scenario-errprop/02-err.arita",
        // ADR-286 §3 fila 9 / L123.
        &["errprop-err", "0"],
    )
}

/// ADR-279 — host 238 write_text/read_text Result + ? inside helper fn→Result (≠ E0340).
fn run_core08_scen_errprop_host238_oracle(root: &Path) -> OracleResult {
    run_core08_scen_errprop_ejemplo(
        root,
        "core08-scen-errprop-host238",
        "ejemplos/core08/scenario-errprop/03-host238.arita",
        &["errprop-host238"],
    )
}

fn run_core08_scen_errprop_ejemplo(
    root: &Path,
    id: &str,
    rel: &str,
    expected: &[&str],
) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }
    let stdout = String::from_utf8_lossy(&run.stdout);
    let got = normalize_stdout(&stdout);
    let expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == expected {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "build ok, exit 0, stdout match (cwd=root)".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

/// ADR-279 — ? inside fn → Io<()> (scenario context) → E0343.
fn run_neg_core08_scen_qmark_outside_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-scen-qmark-outside";
    let rel = "ejemplos/core08/scenario-errprop/neg/01-outside.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0343, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0343") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0343: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0343: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-279 — invent is_ok+unwrap theater → reject (gate E0206; ≠ E0291 reopen).
fn run_neg_core08_scen_unwrap_theater_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-scen-unwrap-theater";
    let rel = "ejemplos/core08/scenario-errprop/neg/02-unwrap-theater.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected reject (E0206 unwrap invent), but build succeeded".into(),
        },
        Err(e) => {
            // Documented gate: E0206 whitelist (≠ E0291). Mirror ADR-277 neg-unwrap.
            if e.contains("E0206") || e.contains("E0342") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected invent unwrap/expect theater: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0206/E0342: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-279 — measure build verde on happy fixture.
fn run_core08_scen_errprop_build_oracle(root: &Path) -> OracleResult {
    let id = "core08-scen-errprop-build";
    let rel = "ejemplos/core08/scenario-errprop/01-happy.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(bin) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("build ok → {}", bin.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("build failed: {}", e.chars().take(200).collect::<String>()),
        },
    }
}

/// ADR-280 / CORE-0.8-REF-ERRPROP-20260926 — workspace lib+bin build verde.
fn run_core08_ref_errprop_build_oracle(root: &Path) -> OracleResult {
    let id = "core08-ref-errprop-build";
    let rel = "ejemplos/core08/ref-errprop/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace build ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-280 — CLI happy: cargo run + fixture → errprop-ref-happy via lib ? chain + main match.
fn run_core08_ref_errprop_cli_happy_oracle(root: &Path) -> OracleResult {
    run_core08_ref_errprop_cli(
        root,
        "core08-ref-errprop-cli-happy",
        "ejemplos/core08/ref-errprop/bin/main.arita",
        &["ejemplos/core08/ref-errprop/fixture.json"],
        &["errprop-ref-happy"],
    )
}

/// ADR-280 — CLI fail: no arg / missing path → errprop-ref-fail (no panic).
fn run_core08_ref_errprop_cli_fail_oracle(root: &Path) -> OracleResult {
    run_core08_ref_errprop_cli(
        root,
        "core08-ref-errprop-cli-fail",
        "ejemplos/core08/ref-errprop/edge/bin/main.arita",
        &[],
        // ADR-286 §3 fila 13 / L125: sin arg → cli_arg(1) None → Err(2).
        &["errprop-ref-fail", "2"],
    )
}

/// ADR-286 §3 L127: `expected` is the full stdout, one entry per line (multi-line since the
/// dead-sink migration prints label + payload); compared as an exact vector, never a prefix.
fn run_core08_ref_errprop_cli(
    root: &Path,
    id: &str,
    rel: &str,
    bin_args: &[&str],
    expected: &[&str],
) -> OracleResult {
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .arg("--")
        .args(bin_args)
        .current_dir(root);
    let run = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let want: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == want {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("CLI→lib → {want:?} (cargo run -p {bin_pkg} args={bin_args:?})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected={want:?}"),
        }
    }
}

/// ADR-280 — ≥2 scenarios PASS (ref_errprop_happy + ref_errprop_err; aligned ADR-279 / REF-IO).
fn run_core08_ref_errprop_scenario_oracle(root: &Path) -> OracleResult {
    let id = "core08-ref-errprop-scenario";
    let happy = root.join("ejemplos/core08/ref-errprop/bin/main.arita");
    let fail = root.join("ejemplos/core08/ref-errprop/edge/bin/main.arita");
    let scen_prev = root.join("ejemplos/core08/scenario-errprop/01-happy.arita");
    if !(happy.is_file() && fail.is_file()) {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop/bin".into(),
            verdict: Verdict::Rejected,
            detail: "need ≥2 acceptance bins (main+edge); skip ≠ PASS".into(),
        };
    }
    let scen_ok = scen_prev.is_file();
    let h = run_core08_ref_errprop_cli_happy_oracle(root);
    let e = run_core08_ref_errprop_cli_fail_oracle(root);
    if h.verdict == Verdict::Accepted && e.verdict == Verdict::Accepted {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop".into(),
            verdict: Verdict::Accepted,
            detail: format!(
                "≥2 scenarios PASS (ref_errprop_happy+ref_errprop_err); scenario-errprop present={scen_ok}"
            ),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop".into(),
            verdict: Verdict::Rejected,
            detail: format!("scenario fail happy={:?} fail={:?}", h.verdict, e.verdict),
        }
    }
}

/// ADR-280 — evidence JSON + stable sha256; measure_pass:true after Core 0.8 CLOSED.
fn run_core08_ref_errprop_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core08-ref-errprop-evidence";
    let ev_rel = "ejemplos/core08/ref-errprop/evidence.json";
    let pairs = [
        (
            "\"lib_sha256\"",
            "ejemplos/core08/ref-errprop/lib/lib.arita",
        ),
        (
            "\"bin_sha256\"",
            "ejemplos/core08/ref-errprop/bin/main.arita",
        ),
        (
            "\"edge_sha256\"",
            "ejemplos/core08/ref-errprop/edge/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    if !ev_raw.contains("\"cut_id\": \"CORE-0.8-REF-ERRPROP-20260926\"") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must pin cut_id CORE-0.8-REF-ERRPROP-20260926".into(),
        };
    }
    if !ev_raw.contains("\"measure_pass\": true") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must keep measure_pass:true after Core 0.8 CLOSED".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        Ok(rest[..q2].to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence lib+bin+edge sha256 match; measure_pass:true".into(),
    }
}

/// ADR-280 / ADR-241 — emit grep: cero unwrap/expect/panic; cero ? in main Io (happy+edge).
fn run_core08_ref_errprop_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core08-ref-errprop-emit-ban";
    let bins = [
        "ejemplos/core08/ref-errprop/bin/main.arita",
        "ejemplos/core08/ref-errprop/edge/bin/main.arita",
    ];
    let mut main_rs: Vec<PathBuf> = Vec::new();
    let mut all_rs: Vec<PathBuf> = Vec::new();
    for rel in bins {
        let full = root.join(rel);
        if !full.is_file() {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("missing: {}", full.display()),
            };
        }
        let out_root = match crate::package::build_package_workspace(
            &full,
            arita_codegen::BuildProfile::Debug,
        ) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
        let crates_dir = out_root.join("crates");
        if let Ok(rd) = fs::read_dir(&crates_dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("bin_") {
                    let p = e.path().join("src").join("main.rs");
                    if p.is_file() {
                        main_rs.push(p.clone());
                        all_rs.push(p);
                    }
                }
                if name.starts_with("lib_") {
                    let p = e.path().join("src").join("lib.rs");
                    if p.is_file() {
                        all_rs.push(p);
                    }
                }
            }
        }
    }
    if main_rs.is_empty() || all_rs.is_empty() {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop".into(),
            verdict: Verdict::Rejected,
            detail: "no emitted crates main.rs/lib.rs (happy+edge)".into(),
        };
    }
    let mut combined = String::new();
    for p in &all_rs {
        match fs::read_to_string(p) {
            Ok(s) => {
                combined.push_str(&s);
                combined.push('\n');
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: p.display().to_string(),
                    verdict: Verdict::Rejected,
                    detail: format!("read emit: {e}"),
                };
            }
        }
    }
    let has_unwrap = combined.contains(".unwrap()");
    let has_expect = combined.contains(".expect(");
    let has_panic = combined.contains(concat!("panic!", "("));
    let mut main_has_qmark = false;
    for p in &main_rs {
        let txt = match fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: p.display().to_string(),
                    verdict: Verdict::Rejected,
                    detail: format!("read main emit: {e}"),
                };
            }
        };
        if txt.contains("?;") || txt.contains(")?") || txt.contains(" ?") {
            main_has_qmark = true;
            break;
        }
    }
    if !has_unwrap && !has_expect && !has_panic && !main_has_qmark {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop".into(),
            verdict: Verdict::Accepted,
            detail: "emit happy+edge: zero unwrap/expect/panic; zero ? in main Io".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core08/ref-errprop".into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail unwrap={has_unwrap} expect={has_expect} panic={has_panic} main_qmark={main_has_qmark}"
            ),
        }
    }
}

/// ADR-280 — ? outside Result fn → E0343 (reuse ADR-278 qmark/neg fixture; ADR §2 allows).
fn run_neg_core08_ref_qmark_outside_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-ref-qmark-outside";
    let rel = "ejemplos/core08/qmark/neg/01-outside.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!(
                "missing reuse path (need Codegen FIX local neg/): {}",
                full.display()
            ),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0343, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0343") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0343 (reuse 278): {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0343: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-280 — bad Result return theater → E0342 (reuse ADR-277 fn-result/neg; ADR §2 allows).
fn run_neg_core08_ref_fn_theater_oracle(root: &Path) -> OracleResult {
    let id = "neg-core08-ref-fn-theater";
    let rel = "ejemplos/core08/fn-result/neg/01-theater.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!(
                "missing reuse path (need Codegen FIX local neg/): {}",
                full.display()
            ),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0342, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0342") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0342 (reuse 277): {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0342: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

// ── ADR-282 MAP-ASSIGN (Core 0.9 slice 1) — CUT CORE-0.9-MAP-ASSIGN-20260926 ─────────────
// `m[k] = v` on Map ≡ `m.put(k, v)` (IndexAssign node running put's checks). Every neg pins an
// EXACT code (all `E\d{4}` tokens in the diag must equal it; never "any error", never OR-codes).
// Paired negs also require the `put` twin to give the same exact code and the same diag head.

/// ADR-282 positives #1–#5 (sugar programs) — emit-ban + emit-clippy inputs.
const CORE09_MAP_ASSIGN_POSITIVES: &[&str] = &[
    "ejemplos/core09/map-assign/01-insert.arita",
    "ejemplos/core09/map-assign/02-overwrite.arita",
    "ejemplos/core09/map-assign/03-eq-put.arita",
    "ejemplos/core09/map-assign/04-fn-result.arita",
    "ejemplos/core09/map-assign/05-self-ref.arita",
];

/// All `E\d{4}` codes mentioned in a diag (exact-code discipline).
fn core09_diag_codes(e: &str) -> Vec<String> {
    let b = e.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 5 <= b.len() {
        let boundary_before = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        let boundary_after = i + 5 == b.len() || !b[i + 5].is_ascii_alphanumeric();
        if b[i] == b'E'
            && b[i + 1..i + 5].iter().all(|c| c.is_ascii_digit())
            && boundary_before
            && boundary_after
        {
            out.push(e[i..i + 5].to_string());
            i += 5;
        } else {
            i += 1;
        }
    }
    out
}

/// Diag head (first line, span tail ` --> …` stripped) for sugar-vs-put text equality.
fn core09_diag_head(e: &str) -> String {
    let first = e.lines().next().unwrap_or("");
    let cut = first.find(" -->").map(|p| &first[..p]).unwrap_or(first);
    cut.trim().trim_end_matches(':').trim().to_string()
}

/// Occurrences of `\w+\[[^\]]+\]\s*=[^=]` (index-assign shape) in `txt`.
fn core09_count_index_assign(txt: &str) -> usize {
    let b = txt.as_bytes();
    let mut n = 0;
    for i in 1..b.len() {
        if b[i] != b'[' {
            continue;
        }
        let p = b[i - 1];
        if !(p.is_ascii_alphanumeric() || p == b'_') {
            continue;
        }
        let mut j = i + 1;
        while j < b.len() && b[j] != b']' {
            j += 1;
        }
        if j >= b.len() || j == i + 1 {
            continue;
        }
        let mut k = j + 1;
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k + 1 < b.len() && b[k] == b'=' && b[k + 1] != b'=' {
            n += 1;
        }
    }
    n
}

/// Source without `//` comment lines (fixture comments quote `m["a"]`).
fn core09_code_lines(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `let _ = <recv>.insert(` present (ADR-237 `put` emit shape).
fn core09_has_let_discard_insert(emit: &str) -> bool {
    let pat = "let _ = ";
    let mut from = 0;
    while let Some(p) = emit[from..].find(pat) {
        let rest = &emit[from + p + pat.len()..];
        let recv_len = rest
            .bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
            .count();
        if recv_len > 0 && rest[recv_len..].starts_with(".insert(") {
            return true;
        }
        from += p + pat.len();
    }
    false
}

/// Build `rel`; Ok(diag) iff the build fails with EXACTLY `code`.
fn core09_expect_exact(root: &Path, rel: &str, code: &str) -> Result<String, (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => Err((
            Verdict::Rejected,
            format!("{rel}: expected exact {code}, but build succeeded"),
        )),
        Err(e) => {
            let codes = core09_diag_codes(&e);
            if !codes.is_empty() && codes.iter().all(|c| c == code) {
                Ok(e)
            } else {
                Err((
                    Verdict::Rejected,
                    format!(
                        "{rel}: expected exact {code}, got codes={codes:?}: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                ))
            }
        }
    }
}

/// Build + run one program; returns (stdout lines, emitted Rust next to the binary).
fn core09_build_run_emit(
    root: &Path,
    rel: &str,
) -> Result<(Vec<String>, String), (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    let bin = build(full.to_str().unwrap_or(rel)).map_err(|e| {
        (
            Verdict::Rejected,
            format!(
                "{rel}: build failed: {}",
                e.chars().take(200).collect::<String>()
            ),
        )
    })?;
    let rs = PathBuf::from(format!("{}.rs", bin.display()));
    let emit = fs::read_to_string(&rs).map_err(|e| {
        (
            Verdict::Rejected,
            format!("{rel}: read emit {}: {e}", rs.display()),
        )
    })?;
    let run = Command::new(&bin).output().map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("{rel}: cannot spawn {}: {e}", bin.display()),
        )
    })?;
    if !run.status.success() {
        return Err((
            Verdict::Rejected,
            format!("{rel}: non-zero exit {}", run.status.code().unwrap_or(-1)),
        ));
    }
    Ok((
        normalize_stdout(&String::from_utf8_lossy(&run.stdout)),
        emit,
    ))
}

/// Paired neg runner: each (sugar, optional put twin) must fail with EXACTLY `code`; with a twin,
/// the diag heads must be identical (≡ `put`, Ingeniero 19:58).
fn run_core09_neg_paired(
    root: &Path,
    id: &str,
    pairs: &[(&str, Option<&str>)],
    code: &str,
) -> OracleResult {
    let label = pairs
        .first()
        .map(|p| p.0)
        .unwrap_or("ejemplos/core09/map-assign/neg");
    let mut notes = Vec::new();
    for (sugar, twin) in pairs {
        let es = match core09_expect_exact(root, sugar, code) {
            Ok(e) => e,
            Err((verdict, detail)) => {
                return OracleResult {
                    id: id.into(),
                    path: (*sugar).into(),
                    verdict,
                    detail,
                };
            }
        };
        let hs = core09_diag_head(&es);
        if let Some(twin) = twin {
            let et = match core09_expect_exact(root, twin, code) {
                Ok(e) => e,
                Err((verdict, detail)) => {
                    return OracleResult {
                        id: id.into(),
                        path: (*twin).into(),
                        verdict,
                        detail: format!("put twin: {detail}"),
                    };
                }
            };
            let ht = core09_diag_head(&et);
            if hs != ht {
                return OracleResult {
                    id: id.into(),
                    path: (*sugar).into(),
                    verdict: Verdict::Rejected,
                    detail: format!("sugar vs put diag differ: sugar={hs:?} put={ht:?}"),
                };
            }
            notes.push(format!("{sugar} ≡ put twin → {hs}"));
        } else {
            notes.push(format!("{sugar} → {hs}"));
        }
    }
    OracleResult {
        id: id.into(),
        path: label.into(),
        verdict: Verdict::Accepted,
        detail: format!("rejected with exact {code}: {}", notes.join("; ")),
    }
}

/// ADR-282 #1 — `m["a"] = 7` new key → insert; read back Some(7) → stdout 7.
fn run_core09_map_assign_insert_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-map-assign-insert",
            path: "ejemplos/core09/map-assign/01-insert.arita",
            expected: &["7"],
            require_test: false,
        },
    )
}

/// ADR-282 #2 — `m["a"]=7; m["a"]=9` → overwrite (old value dropped); len 1 → stdout 9 / 1.
fn run_core09_map_assign_overwrite_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-map-assign-overwrite",
            path: "ejemplos/core09/map-assign/02-overwrite.arita",
            expected: &["9", "1"],
            require_test: false,
        },
    )
}

/// ADR-282 #3 — sugar ≡ put twin: same stdout (9 / 2) + same diag (both build ok) + `insert(` in
/// both emits. NOT exact emit equality (sugar emits bare insert with temporaries; put keeps
/// `let _ =`).
fn run_core09_map_assign_eq_put_oracle(root: &Path) -> OracleResult {
    let id = "core09-map-assign-eq-put";
    let sugar = "ejemplos/core09/map-assign/03-eq-put.arita";
    let twin = "ejemplos/core09/map-assign/03-eq-put-twin.arita";
    let (so, se) = match core09_build_run_emit(root, sugar) {
        Ok(v) => v,
        Err((verdict, detail)) => {
            return OracleResult {
                id: id.into(),
                path: sugar.into(),
                verdict,
                detail,
            };
        }
    };
    let (to, te) = match core09_build_run_emit(root, twin) {
        Ok(v) => v,
        Err((verdict, detail)) => {
            return OracleResult {
                id: id.into(),
                path: twin.into(),
                verdict,
                detail: format!("put twin: {detail}"),
            };
        }
    };
    let expected: Vec<String> = vec!["9".into(), "2".into()];
    let sugar_insert = se.contains("insert(");
    let twin_insert = te.contains("insert(");
    if so == to && so == expected && sugar_insert && twin_insert {
        OracleResult {
            id: id.into(),
            path: sugar.into(),
            verdict: Verdict::Accepted,
            detail: format!(
                "sugar ≡ put twin: stdout {so:?} both; diag none both (build ok); insert( in both emits"
            ),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: sugar.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "eq-put fail sugar_stdout={so:?} put_stdout={to:?} expected={expected:?} sugar_insert={sugar_insert} put_insert={twin_insert}"
            ),
        }
    }
}

/// ADR-282 #4 — `m[k] = v` inside `fn -> Result<Int, Int>` with `?` (ADR-278) → Ok → stdout 6.
fn run_core09_map_assign_fn_result_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-map-assign-fn-result",
            path: "ejemplos/core09/map-assign/04-fn-result.arita",
            expected: &["6"],
            require_test: false,
        },
    )
}

/// ADR-282 #5 (Ingeniero, corrección A6) — `m["b"] = m.len()` POSITIVE: build ok (rustc, no
/// E0502) + stdout 1 (len evaluated before the insert). No put twin (put rhs is literal-only).
fn run_core09_map_assign_self_ref_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-map-assign-self-ref",
            path: "ejemplos/core09/map-assign/05-self-ref.arita",
            expected: &["1"],
            require_test: false,
        },
    )
}

/// ADR-282 #6 — non-mut `let m` + `m["a"] = 1` → EXACT E0202, same diag as the put twin.
fn run_neg_core09_map_assign_non_mut_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-map-assign-non-mut",
        &[(
            "ejemplos/core09/map-assign/neg/01-non-mut.arita",
            Some("ejemplos/core09/map-assign/neg/01-non-mut-put-twin.arita"),
        )],
        "E0202",
    )
}

/// ADR-282 #7 — `let mut m`; `r = borrow m`; assign; `r` used after → EXACT E0202 (+ put twin).
fn run_neg_core09_map_assign_shared_loan_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-map-assign-shared-loan",
        &[(
            "ejemplos/core09/map-assign/neg/02-shared-loan.arita",
            Some("ejemplos/core09/map-assign/neg/02-shared-loan-put-twin.arita"),
        )],
        "E0202",
    )
}

/// ADR-282 #8 — live loan (`r` read after the assign) → EXACT E0202 (+ put twin).
fn run_neg_core09_map_assign_live_loan_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-map-assign-live-loan",
        &[(
            "ejemplos/core09/map-assign/neg/03-live-loan.arita",
            Some("ejemplos/core09/map-assign/neg/03-live-loan-put-twin.arita"),
        )],
        "E0202",
    )
}

/// ADR-282 #9 (A3) — `m["a"] += 1`: no new grammar in 0.9 → EXACT E0006 (not E0314).
fn run_neg_core09_map_assign_compound_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-map-assign-compound",
        &[("ejemplos/core09/map-assign/neg/04-compound.arita", None)],
        "E0006",
    )
}

/// ADR-282 #10 (A4, Ingeniero 20:15-4) — invent `o.unwrap()` on the Option bound from `m["a"]`
/// → ONLY E0206 (never E0206||E0342). Fixture must bind first (chained = E0006) + carry the note.
fn run_neg_core09_map_assign_unwrap_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-map-assign-unwrap";
    let rel = "ejemplos/core09/map-assign/neg/05-unwrap.arita";
    let full = root.join(rel);
    if let Ok(src) = fs::read_to_string(&full) {
        let note = "// Se liga primero: m[\"a\"].unwrap() da E0006 hoy, no E0206.";
        let chained = core09_code_lines(&src).contains("].unwrap(");
        if !src.contains(note) || chained {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!(
                    "fixture shape: note_present={} chained_unwrap={chained} (Ingeniero 20:15-4)",
                    src.contains(note)
                ),
            };
        }
    }
    run_core09_neg_paired(root, id, &[(rel, None)], "E0206")
}

/// ADR-282 #11 / ADR-241 — emit of #1–#5: insert( per assign/put site; zero Rust index-assign;
/// zero IndexMut / std::ops::Index; zero unwrap/expect/panic.
fn run_core09_map_assign_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core09-map-assign-emit-ban";
    let mut notes = Vec::new();
    for rel in CORE09_MAP_ASSIGN_POSITIVES {
        let src = match fs::read_to_string(root.join(rel)) {
            Ok(s) => core09_code_lines(&s),
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: (*rel).into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("missing: {rel}: {e}"),
                };
            }
        };
        let (_, emit) = match core09_build_run_emit(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => {
                return OracleResult {
                    id: id.into(),
                    path: (*rel).into(),
                    verdict,
                    detail,
                };
            }
        };
        let sugar_sites = core09_count_index_assign(&src);
        let sites = sugar_sites + src.matches(".put(").count();
        let inserts = emit.matches("insert(").count();
        let rust_idx_assign = core09_count_index_assign(&emit);
        let index_trait = emit.contains("IndexMut") || emit.contains("std::ops::Index");
        let unwrap = emit.contains(".unwrap()");
        let expect = emit.contains(".expect(");
        let panic = emit.contains(concat!("panic!", "("));
        if sugar_sites == 0
            || inserts < sites
            || rust_idx_assign > 0
            || index_trait
            || unwrap
            || expect
            || panic
        {
            return OracleResult {
                id: id.into(),
                path: (*rel).into(),
                verdict: Verdict::Rejected,
                detail: format!(
                    "emit-ban fail sugar_sites={sugar_sites} sites={sites} insert={inserts} rust_idx_assign={rust_idx_assign} index_trait={index_trait} unwrap={unwrap} expect={expect} panic={panic}"
                ),
            };
        }
        notes.push(format!("{rel} insert={inserts}/{sites}"));
    }
    OracleResult {
        id: id.into(),
        path: "ejemplos/core09/map-assign".into(),
        verdict: Verdict::Accepted,
        detail: format!(
            "emit #1–#5: insert( per site; zero idx-assign/IndexMut/Index/unwrap/expect/panic ({})",
            notes.join(", ")
        ),
    }
}

/// ADR-282 #12 (Ingeniero A1) — `cargo clippy -- -D warnings` on the emitted Rust of #1–#5
/// (scratch crate per program, edition 2021). Missing clippy → Inconclusive (never PASS).
fn run_core09_map_assign_emit_clippy_oracle(root: &Path) -> OracleResult {
    let id = "core09-map-assign-emit-clippy";
    let scratch = root
        .join("target")
        .join("arita-out")
        .join("core09_map_assign_clippy");
    let target_dir = scratch.join("target");
    let mut checked = Vec::new();
    for rel in CORE09_MAP_ASSIGN_POSITIVES {
        let (_, emit) = match core09_build_run_emit(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => {
                return OracleResult {
                    id: id.into(),
                    path: (*rel).into(),
                    verdict,
                    detail,
                };
            }
        };
        let stem = Path::new(rel)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("prog")
            .replace('-', "_");
        let krate = format!("core09_clippy_{stem}");
        let dir = scratch.join(&krate);
        let manifest = dir.join("Cargo.toml");
        let toml = format!(
            "[package]\nname = \"{krate}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[workspace]\n"
        );
        let io = fs::create_dir_all(dir.join("src"))
            .and_then(|_| fs::write(&manifest, toml))
            .and_then(|_| fs::write(dir.join("src").join("main.rs"), &emit));
        if let Err(e) = io {
            return OracleResult {
                id: id.into(),
                path: dir.display().to_string(),
                verdict: Verdict::Inconclusive,
                detail: format!("IO error preparing clippy crate: {e}"),
            };
        }
        let out = Command::new("cargo")
            .args(["clippy", "--quiet", "--manifest-path"])
            .arg(&manifest)
            .arg("--target-dir")
            .arg(&target_dir)
            .args(["--", "-D", "warnings"])
            .current_dir(root)
            .output();
        let out = match out {
            Ok(o) => o,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: (*rel).into(),
                    verdict: Verdict::Inconclusive,
                    detail: format!("cannot spawn cargo clippy: {e}"),
                };
            }
        };
        if !out.status.success() {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let missing = combined.contains("no such command: `clippy`")
                || combined.contains("no such command: \"clippy\"")
                || combined.contains("is not installed")
                || combined.contains("clippy-preview");
            if missing {
                return OracleResult {
                    id: id.into(),
                    path: (*rel).into(),
                    verdict: Verdict::Inconclusive,
                    detail: "clippy not available (cannot run); never treated as accepted".into(),
                };
            }
            let err_lines: Vec<&str> = combined
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    t.starts_with("error") || t.starts_with("-->")
                })
                .collect();
            return OracleResult {
                id: id.into(),
                path: (*rel).into(),
                verdict: Verdict::Rejected,
                detail: format!(
                    "clippy -D warnings failed on emit of {rel} (exit {}): {}",
                    out.status.code().unwrap_or(-1),
                    err_lines.join(" ").chars().take(600).collect::<String>()
                ),
            };
        }
        checked.push(krate);
    }
    OracleResult {
        id: id.into(),
        path: "ejemplos/core09/map-assign".into(),
        verdict: Verdict::Accepted,
        detail: format!(
            "cargo clippy -D warnings clean on emitted Rust of {} positives ({})",
            checked.len(),
            checked.join(", ")
        ),
    }
}

/// ADR-282 pareado P3 (Ingeniero 20:15-1, CONFIRMED) — wrong key `m[1] = 2` and wrong value
/// `m["a"] = "x"` on Map<Text, Int> → EXACT E0203, each paired with its put twin.
fn run_neg_core09_map_assign_type_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-map-assign-type",
        &[
            (
                "ejemplos/core09/map-assign/neg/06-type.arita",
                Some("ejemplos/core09/map-assign/neg/06-type-put-twin.arita"),
            ),
            (
                "ejemplos/core09/map-assign/neg/07-value-type.arita",
                Some("ejemplos/core09/map-assign/neg/07-value-type-put-twin.arita"),
            ),
        ],
        "E0203",
    )
}

/// ADR-282 regresión ADR-237 (Ingeniero 20:15-2, CONFIRMED) — CONTENT check (never md5): emit of
/// `m.put` keeps `let _ = <recv>.insert(`, has no `__arita_mk` / `__arita_mv`; stdout stays 7.
fn run_core09_map_assign_put_emit_unchanged_oracle(root: &Path) -> OracleResult {
    let id = "core09-map-assign-put-emit-unchanged";
    let rel = "ejemplos/core01/07-map-put-get.arita";
    let (out, emit) = match core09_build_run_emit(root, rel) {
        Ok(v) => v,
        Err((verdict, detail)) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict,
                detail,
            };
        }
    };
    let let_insert = core09_has_let_discard_insert(&emit);
    let mk = emit.contains("__arita_mk");
    let mv = emit.contains("__arita_mv");
    let stdout_ok = out == ["7".to_string()];
    if let_insert && !mk && !mv && stdout_ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "put emit keeps `let _ = <recv>.insert(`; no __arita_mk/__arita_mv; stdout 7"
                .into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "put emit regression let_insert={let_insert} __arita_mk={mk} __arita_mv={mv} stdout={out:?}"
            ),
        }
    }
}

// ── ADR-283 VEC-ASSIGN (Core 0.9 slice 2) — CUT CORE-0.9-VEC-ASSIGN-20260926 ─────────────
// `v[i] = x` on Vec/List ≡ `v.set(i, x)?`, legal only inside a sync `fn … -> Result<_, Int>`.
// Wired by Measure (Orquestador GO 23:49 B1–B3; Ingeniero addendum 23:50 B4). Every neg pins the
// EXACT code from the ADR-283 TEXT (never the observed output; never "any error"/OR-codes).
// Spans / lines are derived from the fixture source itself (byte offsets of the statement).

/// ADR-283 positives using the sugar (emit-ban / emit-no-as-usize inputs).
const CORE09_VEC_ASSIGN_POSITIVES: &[&str] = &[
    "ejemplos/core09/vec-assign/01-set.arita",
    "ejemplos/core09/vec-assign/02-overwrite.arita",
    "ejemplos/core09/vec-assign/03-eq-set.arita",
    "ejemplos/core09/vec-assign/04-fn-result.arita",
    "ejemplos/core09/vec-assign/05-self-ref.arita",
    "ejemplos/core09/vec-assign/06-list.arita",
    "ejemplos/core09/vec-assign/08-qmark-chain.arita",
];

/// Byte span `a..b` of the first source line whose trimmed text is exactly `stmt`.
fn core09_vec_stmt_span(src: &str, stmt: &str) -> Option<String> {
    let mut off = 0usize;
    for line in src.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if body.trim() == stmt {
            let lead = body.len() - body.trim_start().len();
            let a = off + lead;
            return Some(format!("{}..{}", a, a + stmt.len()));
        }
        off += line.len();
    }
    None
}

/// 1-based line of the first source line whose trimmed text is exactly `stmt`.
fn core09_vec_stmt_line(src: &str, stmt: &str) -> Option<usize> {
    src.lines().position(|l| l.trim() == stmt).map(|p| p + 1)
}

/// Body of `fn <name><…` in emitted Rust (up to the first `\n}` at column 0).
fn core09_vec_helper_body<'a>(emit: &'a str, name: &str) -> Option<&'a str> {
    let start = emit.find(&format!("fn {name}<"))?;
    let rest = &emit[start..];
    let end = rest.find("\n}")?;
    Some(&rest[..end + 2])
}

fn core09_vec_fail(id: &str, path: &str, verdict: Verdict, detail: String) -> OracleResult {
    OracleResult {
        id: id.into(),
        path: path.into(),
        verdict,
        detail,
    }
}

/// E0344 neg: first diag line must be EXACTLY `E0344: index assign outside result fn @<span>`,
/// span = byte span of `stmt` in the fixture; every `E\d{4}` in the diag must be E0344.
fn run_core09_vec_neg_e0344(root: &Path, id: &str, rel: &str, stmt: &str) -> OracleResult {
    let src = match fs::read_to_string(root.join(rel)) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing: {rel}: {e}"),
            )
        }
    };
    let Some(span) = core09_vec_stmt_span(&src, stmt) else {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!("fixture shape: statement `{stmt}` not found"),
        );
    };
    let expected = format!("E0344: index assign outside result fn @{span}");
    match core09_expect_exact(root, rel, "E0344") {
        Ok(e) => {
            let first = e.lines().next().unwrap_or("").trim();
            if first == expected {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Accepted,
                    format!("rejected with exact E0344: {first}"),
                )
            } else {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!("expected `{expected}`, got first line `{first}`"),
                )
            }
        }
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// E0006 negs (no grammar in 0.9): each fixture EXACT E0006 and the diag points at the line of
/// the offending statement (`--> <line>:`).
fn run_core09_vec_neg_e0006(root: &Path, id: &str, cases: &[(&str, &str)]) -> OracleResult {
    let mut notes = Vec::new();
    for (rel, stmt) in cases {
        let src = match fs::read_to_string(root.join(rel)) {
            Ok(s) => s,
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        };
        let Some(line) = core09_vec_stmt_line(&src, stmt) else {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: statement `{stmt}` not found"),
            );
        };
        let e = match core09_expect_exact(root, rel, "E0006") {
            Ok(e) => e,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        if !e.contains(&format!("--> {line}:")) {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!(
                    "{rel}: exact E0006 but not at line {line} of `{stmt}`: {}",
                    e.chars().take(200).collect::<String>()
                ),
            );
        }
        notes.push(format!("{rel} `{stmt}` → E0006 @line {line}"));
    }
    core09_vec_fail(
        id,
        cases
            .first()
            .map(|c| c.0)
            .unwrap_or("ejemplos/core09/vec-assign/neg"),
        Verdict::Accepted,
        format!("rejected with exact E0006: {}", notes.join("; ")),
    )
}

/// ADR-283 core09-vec-assign-ok — in-bounds `v[1] = 9` in fn→Result → stdout 9.
fn run_core09_vec_assign_ok_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-ok",
            path: "ejemplos/core09/vec-assign/01-set.arita",
            expected: &["9"],
            require_test: false,
        },
    )
}

/// ADR-283 — overwrite: last value wins, len unchanged → stdout 5 / 2.
fn run_core09_vec_assign_overwrite_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-overwrite",
            path: "ejemplos/core09/vec-assign/02-overwrite.arita",
            expected: &["5", "2"],
            require_test: false,
        },
    )
}

/// ADR-283 core09-vec-assign-eq-set — sugar ≡ `v.set(i, x)` + `?` twin: same stdout (6 / 0,
/// incl. OOB → Err(0)), same diag (both build ok), `__arita_vec_set(` in both emits (not exact
/// emit equality).
fn run_core09_vec_assign_eq_set_oracle(root: &Path) -> OracleResult {
    let id = "core09-vec-assign-eq-set";
    let sugar = "ejemplos/core09/vec-assign/03-eq-set.arita";
    let twin = "ejemplos/core09/vec-assign/03-eq-set-twin.arita";
    let (so, se) = match core09_build_run_emit(root, sugar) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(id, sugar, verdict, detail),
    };
    let (to, te) = match core09_build_run_emit(root, twin) {
        Ok(v) => v,
        Err((verdict, detail)) => {
            return core09_vec_fail(id, twin, verdict, format!("set twin: {detail}"))
        }
    };
    let expected: Vec<String> = vec!["6".into(), "0".into()];
    let sugar_set = se.contains("__arita_vec_set(");
    let twin_set = te.contains("__arita_vec_set(");
    if so == to && so == expected && sugar_set && twin_set {
        core09_vec_fail(
            id,
            sugar,
            Verdict::Accepted,
            format!("sugar ≡ set twin: stdout {so:?} both; diag none both (build ok); __arita_vec_set( in both emits"),
        )
    } else {
        core09_vec_fail(
            id,
            sugar,
            Verdict::Rejected,
            format!(
                "eq-set fail sugar_stdout={so:?} set_stdout={to:?} expected={expected:?} sugar_set={sugar_set} twin_set={twin_set}"
            ),
        )
    }
}

/// ADR-283 — nested while/if/match inside fn→Result; OOB → Err(0) propagated → stdout 1 / 0.
fn run_core09_vec_assign_fn_result_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-fn-result",
            path: "ejemplos/core09/vec-assign/04-fn-result.arita",
            expected: &["1", "0"],
            require_test: false,
        },
    )
}

/// ADR-283 core09-vec-assign-self-ref (R2) — `v[0] = v.len()` POSITIVE → build ok + stdout 3.
fn run_core09_vec_assign_self_ref_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-self-ref",
            path: "ejemplos/core09/vec-assign/05-self-ref.arita",
            expected: &["3"],
            require_test: false,
        },
    )
}

/// ADR-283 — List<Int> receiver (≡ Vec, ADR-237) → stdout 4.
fn run_core09_vec_assign_list_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-list",
            path: "ejemplos/core09/vec-assign/06-list.arita",
            expected: &["4"],
            require_test: false,
        },
    )
}

/// Ingeniero regression (ADR-282 intact): Map-assign inside `if` → build ok, stdout 2 / 7.
fn run_core09_vec_assign_map_in_if_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-map-in-if",
            path: "ejemplos/core09/vec-assign/07-map-in-if.arita",
            expected: &["2", "7"],
            require_test: false,
        },
    )
}

/// ADR-283 §2 core09-vec-assign-qmark-chain (B4, Measure fixture) — `v[i] = x` + `?` in the same
/// fn: chain(0) → assign ok, boom(3)? → Err(3) → "3"; chain(4) → OOB Err(0) before boom → "0".
fn run_core09_vec_assign_qmark_chain_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core09-vec-assign-qmark-chain",
            path: "ejemplos/core09/vec-assign/08-qmark-chain.arita",
            expected: &["3", "0"],
            require_test: false,
        },
    )
}

/// ADR-283 §2 neg-core09-vec-assign-outside-main — `v[0] = 2` in `fn main() -> Io<()>` → E0344.
fn run_neg_core09_vec_assign_outside_main_oracle(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0344(
        root,
        "neg-core09-vec-assign-outside-main",
        "ejemplos/core09/vec-assign/neg/01-main.arita",
        "v[0] = 2",
    )
}

/// ADR-283 §0 Fuera de ámbito — `v[0] = 2` in helper `fn h() -> Io<()>` → E0344.
fn run_neg_core09_vec_assign_outside_io_helper_oracle(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0344(
        root,
        "neg-core09-vec-assign-outside-io-helper",
        "ejemplos/core09/vec-assign/neg/02-io-helper.arita",
        "v[0] = 2",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-outside-fn (B4, Measure fixture) — fn no-Result → E0344.
fn run_neg_core09_vec_assign_outside_fn_oracle(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0344(
        root,
        "neg-core09-vec-assign-outside-fn",
        "ejemplos/core09/vec-assign/neg/09-outside-fn.arita",
        "v[0] = 2",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-neg-lit — `v[-1] = n` in fn→Result → EXACT E0319.
fn run_neg_core09_vec_assign_neg_lit_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-vec-assign-neg-lit",
        &[("ejemplos/core09/vec-assign/neg/03-neg-lit.arita", None)],
        "E0319",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-neg-lit-vs-set (Ingeniero §1) — pinned pair:
/// `v[-1] = "x"` → EXACT E0319 AND `v.set(-1, "x")` → EXACT E0203 (265/270 not reopened).
fn run_neg_core09_vec_assign_neg_lit_vs_set_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-vec-assign-neg-lit-vs-set";
    let sugar = "ejemplos/core09/vec-assign/neg/04-neg-lit-text.arita";
    let twin = "ejemplos/core09/vec-assign/neg/04-neg-lit-text-set-twin.arita";
    for (rel, needle) in [(sugar, "v[-1] = \"x\""), (twin, "v.set(-1, \"x\")")] {
        match fs::read_to_string(root.join(rel)) {
            Ok(s) if core09_code_lines(&s).contains(needle) => {}
            Ok(_) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!("fixture shape: `{needle}` not in code lines"),
                )
            }
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        }
    }
    let es = match core09_expect_exact(root, sugar, "E0319") {
        Ok(e) => e,
        Err((verdict, detail)) => {
            return core09_vec_fail(id, sugar, verdict, format!("sugar: {detail}"))
        }
    };
    let et = match core09_expect_exact(root, twin, "E0203") {
        Ok(e) => e,
        Err((verdict, detail)) => {
            return core09_vec_fail(id, twin, verdict, format!("set twin: {detail}"))
        }
    };
    core09_vec_fail(
        id,
        sugar,
        Verdict::Accepted,
        format!(
            "pair pinned: v[-1] = \"x\" → exact E0319 ({}); v.set(-1, \"x\") → exact E0203 ({})",
            core09_diag_head(&es),
            core09_diag_head(&et)
        ),
    )
}

/// ADR-283 §2 neg-core09-vec-assign-compound — nested `v[0][1] = 1` and field `v[0].f = 1` →
/// EXACT E0006 each, at the offending line. ADR-290 IM-6: `v[0] += 1` (`05-compound.arita`) is now
/// VALID Rust-emitting Arita (compound index-assign on Vec<Int> in `fn -> Result`), so that case is
/// RETIRED from this oracle (the fixture file itself is NOT touched; the positive is pinned by
/// `core10-index-mut-migrated-05`). Migration is explained by the ADR-290 freeze addendum.
const VEC_ASSIGN_COMPOUND_NEG: [(&str, &str); 2] = [
    (
        "ejemplos/core09/vec-assign/neg/06-nested.arita",
        "v[0][1] = 1",
    ),
    (
        "ejemplos/core09/vec-assign/neg/07-field.arita",
        "v[0].f = 1",
    ),
];

fn run_neg_core09_vec_assign_compound_oracle(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0006(
        root,
        "neg-core09-vec-assign-compound",
        &VEC_ASSIGN_COMPOUND_NEG,
    )
}

/// ADR-283 §2 neg-core09-vec-assign-err-type (B4) — fn → Result<Int, Text> → EXACT E0203.
fn run_neg_core09_vec_assign_err_type_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-vec-assign-err-type",
        &[("ejemplos/core09/vec-assign/neg/10-err-type.arita", None)],
        "E0203",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-non-mut (B4) — non-mut binding → EXACT E0202 (= v.set).
fn run_neg_core09_vec_assign_non_mut_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-vec-assign-non-mut",
        &[("ejemplos/core09/vec-assign/neg/11-non-mut.arita", None)],
        "E0202",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-unwrap (B4) — bound `r = v.set(…)` + `r.unwrap()` → EXACT
/// E0206 (same code as the existing invent-unwrap negs). Fixture must bind first (no chaining).
fn run_neg_core09_vec_assign_unwrap_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-vec-assign-unwrap";
    let rel = "ejemplos/core09/vec-assign/neg/12-unwrap.arita";
    if let Ok(src) = fs::read_to_string(root.join(rel)) {
        let code = core09_code_lines(&src);
        let bound = code.contains("let r: Result<(), Int> = v.set(") && code.contains("r.unwrap()");
        let chained = code.contains(").unwrap(");
        if !bound || chained {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: bound_unwrap={bound} chained_unwrap={chained}"),
            );
        }
    }
    run_core09_neg_paired(root, id, &[(rel, None)], "E0206")
}

/// ADR-283 §2 neg-core09-vec-assign-bad-return (B4) — mutates then yields `0` → EXACT E0342.
fn run_neg_core09_vec_assign_bad_return_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-vec-assign-bad-return",
        &[("ejemplos/core09/vec-assign/neg/13-bad-return.arita", None)],
        "E0342",
    )
}

/// ADR-283 §2 neg-core09-vec-assign-err-swallow (B4, pin 23:50) — caller `Err(_) => <lit>` over a
/// fn→Result using `v[i] = x` → EXACT E0272 (same code as neg-e0272-err-default-lit).
fn run_neg_core09_vec_assign_err_swallow_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-vec-assign-err-swallow",
        &[("ejemplos/core09/vec-assign/neg/14-err-swallow.arita", None)],
        "E0272",
    )
}

/// Source-order guard for the qmark-chain pair: line of `first` must precede line of `second`.
fn core09_vec_order_ok(src: &str, first: &str, second: &str) -> bool {
    match (
        core09_vec_stmt_line(src, first),
        core09_vec_stmt_line(src, second),
    ) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

/// ADR-283 §2 neg-core09-vec-assign-qmark-chain (B4, pin 23:50) — non-Result fn, `v[0] = 2`
/// BEFORE `let n: Int = g()?` → EXACT E0344 + span (first offender in source order wins).
fn run_neg_core09_vec_assign_qmark_chain_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-vec-assign-qmark-chain";
    let rel = "ejemplos/core09/vec-assign/neg/15-qmark-chain.arita";
    if let Ok(src) = fs::read_to_string(root.join(rel)) {
        if !core09_vec_order_ok(&src, "v[0] = 2", "let n: Int = g()?") {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                "fixture shape: `v[0] = 2` must precede `let n: Int = g()?`".into(),
            );
        }
    }
    run_core09_vec_neg_e0344(root, id, rel, "v[0] = 2")
}

/// ADR-283 §2 + Ingeniero addendum 23:58 neg-core09-vec-assign-qmark-chain-rev — inverse order:
/// `let n: Int = g()?` BEFORE `v[0] = 2` in a non-Result fn → EXACT E0343 (an impl where E0344
/// always wins must fail here).
fn run_neg_core09_vec_assign_qmark_chain_rev_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-vec-assign-qmark-chain-rev";
    let rel = "ejemplos/core09/vec-assign/neg/16-qmark-chain-rev.arita";
    if let Ok(src) = fs::read_to_string(root.join(rel)) {
        if !core09_vec_order_ok(&src, "let n: Int = g()?", "v[0] = 2") {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                "fixture shape: `let n: Int = g()?` must precede `v[0] = 2`".into(),
            );
        }
    }
    run_core09_neg_paired(root, id, &[(rel, None)], "E0343")
}

/// ADR-283 §2 core09-vec-assign-emit-ban — emit of the sugar positives: one
/// `__arita_vec_set(&mut …)?` per sugar site (never discarded: no `let _ =`, `.ok()`,
/// `unwrap_or`); zero Rust index-assign; zero IndexMut / std::ops::Index; zero unwrap/expect/panic.
fn run_core09_vec_assign_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core09-vec-assign-emit-ban";
    let mut notes = Vec::new();
    for rel in CORE09_VEC_ASSIGN_POSITIVES {
        let src = match fs::read_to_string(root.join(rel)) {
            Ok(s) => core09_code_lines(&s),
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        };
        let (_, emit) = match core09_build_run_emit(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        let sites = core09_count_index_assign(&src);
        let set_lines: Vec<&str> = emit
            .lines()
            .filter(|l| l.contains("__arita_vec_set(&mut "))
            .collect();
        let propagated = set_lines
            .iter()
            .filter(|l| l.contains(")?;") && !l.contains(".ok()") && !l.contains("unwrap_or"))
            .count();
        let discard = emit.contains("let _ = __arita_vec_set");
        let rust_idx_assign = core09_count_index_assign(&emit);
        let index_trait = emit.contains("IndexMut") || emit.contains("std::ops::Index");
        let unwrap = emit.contains(".unwrap()");
        let expect = emit.contains(".expect(");
        let panic = emit.contains(concat!("panic!", "("));
        if sites == 0
            || propagated < sites
            || set_lines.len() != propagated
            || discard
            || rust_idx_assign > 0
            || index_trait
            || unwrap
            || expect
            || panic
        {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!(
                    "emit-ban fail sites={sites} set_calls={} propagated={propagated} discard={discard} rust_idx_assign={rust_idx_assign} index_trait={index_trait} unwrap={unwrap} expect={expect} panic={panic}",
                    set_lines.len()
                ),
            );
        }
        notes.push(format!("{rel} set?={propagated}/{sites}"));
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/vec-assign",
        Verdict::Accepted,
        format!(
            "emit: __arita_vec_set(&mut …)? per site; zero discard/idx-assign/IndexMut/Index/unwrap/expect/panic ({})",
            notes.join(", ")
        ),
    )
}

/// ADR-283 R1 core09-vec-assign-emit-no-as-usize — zero `as usize` in the `v[i] = x` lowering
/// (`__arita_vi` lines) and in the bodies of `__arita_vec_set` / `__arita_vec_insert`; both helpers
/// present (non-vacuous: insert via core05/insert/01-ok) and use `usize::try_from(i)`.
fn run_core09_vec_assign_emit_no_as_usize_oracle(root: &Path) -> OracleResult {
    let id = "core09-vec-assign-emit-no-as-usize";
    let mut notes = Vec::new();
    for rel in CORE09_VEC_ASSIGN_POSITIVES {
        let src = match fs::read_to_string(root.join(rel)) {
            Ok(s) => core09_code_lines(&s),
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        };
        let (_, emit) = match core09_build_run_emit(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        let sites = core09_count_index_assign(&src);
        let lowering: Vec<&str> = emit.lines().filter(|l| l.contains("__arita_vi")).collect();
        let lowering_as = lowering.iter().filter(|l| l.contains("as usize")).count();
        let helper = core09_vec_helper_body(&emit, "__arita_vec_set");
        let (h_present, h_as, h_try) = match helper {
            Some(b) => (
                true,
                b.matches("as usize").count(),
                b.contains("usize::try_from(i)"),
            ),
            None => (false, 0, false),
        };
        if sites == 0
            || lowering.len() < sites
            || lowering_as > 0
            || !h_present
            || h_as > 0
            || !h_try
        {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!(
                    "no-as-usize fail sites={sites} lowering_lines={} lowering_as_usize={lowering_as} set_helper={h_present} helper_as_usize={h_as} helper_try_from={h_try}",
                    lowering.len()
                ),
            );
        }
        notes.push(format!("{rel} lowering={}", lowering.len()));
    }
    let ins = "ejemplos/core05/insert/01-ok.arita";
    let (_, emit) = match core09_build_run_emit(root, ins) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(id, ins, verdict, detail),
    };
    match core09_vec_helper_body(&emit, "__arita_vec_insert") {
        Some(b) if !b.contains("as usize") && b.contains("usize::try_from(i)") => {}
        Some(b) => {
            return core09_vec_fail(
                id,
                ins,
                Verdict::Rejected,
                format!(
                    "insert helper as_usize={} try_from={}",
                    b.matches("as usize").count(),
                    b.contains("usize::try_from(i)")
                ),
            )
        }
        None => {
            return core09_vec_fail(
                id,
                ins,
                Verdict::Rejected,
                "insert helper absent from emit (vacuous)".into(),
            )
        }
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/vec-assign",
        Verdict::Accepted,
        format!(
            "zero `as usize` in lowering + __arita_vec_set/__arita_vec_insert bodies; usize::try_from(i) in both ({}; insert via {ins})",
            notes.join(", ")
        ),
    )
}

/// ADR-283 §2.1 core09-vec-assign-emit-no-deferred-shape — the `<arita:deferred-shape>` marker
/// never reaches emitted Rust (positives + eq-set twin + map-in-if); each emit must be a real
/// program (`fn main(` + `__arita_vec_set(` or Map `insert(`) so the ban is not vacuous.
fn run_core09_vec_assign_emit_no_deferred_shape_oracle(root: &Path) -> OracleResult {
    let id = "core09-vec-assign-emit-no-deferred-shape";
    let mut checked = 0usize;
    let extra = [
        "ejemplos/core09/vec-assign/03-eq-set-twin.arita",
        "ejemplos/core09/vec-assign/07-map-in-if.arita",
    ];
    for rel in CORE09_VEC_ASSIGN_POSITIVES.iter().chain(extra.iter()) {
        let (_, emit) = match core09_build_run_emit(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        let marker = emit.contains("arita:deferred-shape") || emit.contains("deferred-shape");
        let real = emit.contains("fn main(")
            && (emit.contains("__arita_vec_set(") || emit.contains("insert("));
        if marker || !real {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("deferred-shape ban fail marker={marker} real_program={real}"),
            );
        }
        checked += 1;
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/vec-assign",
        Verdict::Accepted,
        format!("zero `arita:deferred-shape` in {checked} emits (each with fn main + set/insert)"),
    )
}

// ── ADR-284 SCENARIO-MUT (Core 0.9 slice 3) — CUT CORE-0.9-SCENARIO-MUT-20260926 ────────────
// Scenarios Map+Vec end-to-end (ADR-284 §2 L64–L80, sha256 ac80c71c…). Wired by Measure (Orquestador GO 05:19;
// criteria approved by the Ingeniero 05:27). Negs pin EXACT codes: L74 E0344 · L75 E0202 ·
// L76 E0206 (unwrap-theater, approved 05:27; E0291 or any other code → rejected). Positives are
// Codegen programs: exact full stdout + exit 0 + no `panicked` in stderr + the program's own
// `scenario { acceptance }` accepted by the contract harness. Until the READY gate opens every
// positive-derived oracle is forced to Inconclusive (the probe verdict is kept in the detail).
// Skip ≠ PASS; a missing fixture is Inconclusive, never Accepted. No Result-discard oracle (L82).

/// Codegen positives gate (ADR-284 §2 positives + build + emit-ban). Opened by Measure on the
/// Orquestador GO 05:31 with ADR-284 sha256 ac80c71c… (pins L24 / L70–L73 / L76 / L80 / L98).
const CORE09_SCEN_MUT_POSITIVES_READY: bool = true;

/// ADR-284 L76 `neg-core09-scen-unwrap-theater`: EXACT code + message (Ingeniero OK 05:27).
const CORE09_SCEN_MUT_UNWRAP_THEATER_CODE: &str = "E0206";
const CORE09_SCEN_MUT_UNWRAP_THEATER_HEAD: &str = "E0206: method not in F2 std whitelist";

/// (id, program, exact full stdout). Stdout derived by Measure from ADR-284 §0/§2 semantics
/// (282 insert/overwrite + R2, 283 OOB → `Err(0)`, 278 `?` early-return) traced over the source;
/// approved pins 05:27. Trace: DOC/reviews/MEASURE_PREP_ADR284_SCENARIO_MUT_20260927.md §3.
const CORE09_SCEN_MUT_POSITIVES: &[(&str, &str, &[&str])] = &[
    (
        "core09-scen-mut-happy",
        "ejemplos/core09/scenario-mut/01-mut-happy.arita",
        &["2", "9", "2", "mut-happy"],
    ),
    (
        "core09-scen-mut-err",
        "ejemplos/core09/scenario-mut/02-mut-err.arita",
        &["0", "mut-err"],
    ),
    (
        "core09-scen-mut-chain",
        "ejemplos/core09/scenario-mut/03-mut-chain.arita",
        &["40", "err", "77", "err", "0"],
    ),
    (
        "core09-scen-mut-map-main",
        "ejemplos/core09/scenario-mut/04-mut-map-main.arita",
        &["3", "1", "7", "3"],
    ),
];

/// Pre-READY gate: positive-derived results are Inconclusive (probe verdict kept in detail).
fn core09_scen_mut_gate(r: OracleResult) -> OracleResult {
    if CORE09_SCEN_MUT_POSITIVES_READY {
        return r;
    }
    OracleResult {
        verdict: Verdict::Inconclusive,
        detail: format!(
            "not READY_FOR_MEASURE (Codegen positives gate) — probe verdict={:?}: {}",
            r.verdict, r.detail
        ),
        ..r
    }
}

/// ADR-284 L72 ordering for 03-mut-chain: full(1) Ok(40) · chain(1) Vec ok → post stage runs →
/// control tripwire Err(77) at the 3rd line · chain(5) Vec OOB → Err(0) early-return → the post
/// stage must NOT run (no 77 after the control, and not Err(1) from a skipped early-return).
fn core09_scen_mut_chain_order(got: &[String]) -> Result<(), String> {
    if got.len() != 5 {
        return Err(format!("chain: expected 5 stdout lines, got {got:?}"));
    }
    if got[0] != "40" {
        return Err(format!("chain: full(1) must print 40, got {:?}", got[0]));
    }
    if got[1] != "err" || got[2] != "77" {
        return Err(format!(
            "chain: chain(1) (Vec ok) must reach the post stage → err/77, got {:?}",
            &got[1..3]
        ));
    }
    if got[3..].iter().any(|l| l == "77") {
        return Err(format!(
            "chain: post-stage tripwire 77 observed AFTER Vec OOB (effects after Err ran): {:?}",
            &got[3..]
        ));
    }
    if got[3] != "err" || got[4] != "0" {
        return Err(format!(
            "chain: chain(5) (Vec OOB) must early-return Err(0) → err/0, got {:?}",
            &got[3..]
        ));
    }
    Ok(())
}

/// Build + run one positive: (stdout lines, stderr, emitted Rust). Non-zero exit → Rejected.
fn core09_scen_mut_build_run(
    root: &Path,
    rel: &str,
) -> Result<(Vec<String>, String, String), (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    let bin = build(full.to_str().unwrap_or(rel)).map_err(|e| {
        (
            Verdict::Rejected,
            format!(
                "{rel}: build failed: {}",
                e.chars().take(200).collect::<String>()
            ),
        )
    })?;
    let rs = PathBuf::from(format!("{}.rs", bin.display()));
    let emit = fs::read_to_string(&rs).map_err(|e| {
        (
            Verdict::Rejected,
            format!("{rel}: read emit {}: {e}", rs.display()),
        )
    })?;
    let run = Command::new(&bin).output().map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("{rel}: cannot spawn {}: {e}", bin.display()),
        )
    })?;
    let stderr = String::from_utf8_lossy(&run.stderr).into_owned();
    if !run.status.success() {
        return Err((
            Verdict::Rejected,
            format!(
                "{rel}: non-zero exit {} stderr={:?}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(160).collect::<String>()
            ),
        ));
    }
    Ok((
        normalize_stdout(&String::from_utf8_lossy(&run.stdout)),
        stderr,
        emit,
    ))
}

/// Positive probe (ungated): build + run exit 0 + no `panicked` in stderr + EXACT full stdout
/// (+ chain ordering) + the program's `scenario { acceptance }` accepted by the contract harness
/// (ADR-284 §0 Harness L20; §2 L80).
fn core09_scen_mut_positive_probe(
    root: &Path,
    id: &str,
    rel: &str,
    expected: &[&str],
) -> OracleResult {
    let (got, stderr, _emit) = match core09_scen_mut_build_run(root, rel) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
    };
    if stderr.contains("panicked") {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "stderr contains `panicked`: {}",
                stderr.chars().take(200).collect::<String>()
            ),
        );
    }
    let want: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if id == "core09-scen-mut-chain" {
        if let Err(why) = core09_scen_mut_chain_order(&got) {
            return core09_vec_fail(id, rel, Verdict::Rejected, why);
        }
    }
    if got != want {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!("stdout mismatch: got={got:?} expected={want:?}"),
        );
    }
    let scen = crate::contract::run_lang_contract_file(root, &root.join(rel));
    if scen.verdict != Verdict::Accepted {
        return core09_vec_fail(
            id,
            rel,
            scen.verdict,
            format!(
                "stdout ok {want:?} but scenario acceptance {:?}: {}",
                scen.verdict,
                scen.detail.chars().take(240).collect::<String>()
            ),
        );
    }
    core09_vec_fail(
        id,
        rel,
        Verdict::Accepted,
        format!("build ok, exit 0, no panic, exact stdout {want:?}; scenario acceptance ok"),
    )
}

fn run_core09_scen_mut_positive(root: &Path, i: usize) -> OracleResult {
    let (id, rel, exp) = CORE09_SCEN_MUT_POSITIVES[i];
    core09_scen_mut_gate(core09_scen_mut_positive_probe(root, id, rel, exp))
}

/// ADR-284 L70 core09-scen-mut-happy.
fn run_core09_scen_mut_happy_oracle(root: &Path) -> OracleResult {
    run_core09_scen_mut_positive(root, 0)
}

/// ADR-284 L71 (+§0 L24) core09-scen-mut-err — Vec OOB → Err(0) propagated → main Err branch, fixed stdout,
/// exit 0, no panic.
fn run_core09_scen_mut_err_oracle(root: &Path) -> OracleResult {
    run_core09_scen_mut_positive(root, 1)
}

/// ADR-284 L72 core09-scen-mut-chain — `?` chain Map ok + Vec OOB → early-return; later effects
/// do not happen (exact full stdout + `core09_scen_mut_chain_order`).
fn run_core09_scen_mut_chain_oracle(root: &Path) -> OracleResult {
    run_core09_scen_mut_positive(root, 2)
}

/// ADR-284 L73 core09-scen-mut-map-main — `m[k] = v` directly in main `Io<()>` → OK (total).
fn run_core09_scen_mut_map_main_oracle(root: &Path) -> OracleResult {
    run_core09_scen_mut_positive(root, 3)
}

/// ADR-284 L74 neg-core09-scen-vec-assign-outside — `v[i] = x` in main Io → EXACT E0344 + span.
fn run_neg_core09_scen_vec_assign_outside_oracle(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0344(
        root,
        "neg-core09-scen-vec-assign-outside",
        "ejemplos/core09/scenario-mut/neg/01-vec-assign-outside.arita",
        "v[0] = 9",
    )
}

/// ADR-284 L75 neg-core09-scen-map-assign-non-mut — `m[k] = v` on non-mut binding → EXACT E0202.
fn run_neg_core09_scen_map_assign_non_mut_oracle(root: &Path) -> OracleResult {
    run_core09_neg_paired(
        root,
        "neg-core09-scen-map-assign-non-mut",
        &[(
            "ejemplos/core09/scenario-mut/neg/02-map-assign-non-mut.arita",
            None,
        )],
        "E0202",
    )
}

/// ADR-284 L76 neg-core09-scen-unwrap-theater — `let r: Result<(), Int> = v.set(i, x)` then
/// `r.unwrap()` inside helper `fn -> Result` → EXACT E0206. Builds → rejected; E0291 (reopen) or
/// any other code → rejected (every diag code must be E0206).
fn run_neg_core09_scen_unwrap_theater_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-scen-unwrap-theater";
    let rel = "ejemplos/core09/scenario-mut/neg/03-unwrap-theater.arita";
    match core09_expect_exact(root, rel, CORE09_SCEN_MUT_UNWRAP_THEATER_CODE) {
        Ok(e) => {
            if core09_diag_codes(&e).iter().any(|c| c == "E0291") {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!(
                        "E0291 reopen forbidden (ADR-284 L76): {}",
                        core09_diag_head(&e)
                    ),
                );
            }
            let head = core09_diag_head(&e);
            if head != CORE09_SCEN_MUT_UNWRAP_THEATER_HEAD {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!("expected `{CORE09_SCEN_MUT_UNWRAP_THEATER_HEAD}`, got `{head}`"),
                );
            }
            core09_vec_fail(
                id,
                rel,
                Verdict::Accepted,
                format!(
                    "rejected with exact {CORE09_SCEN_MUT_UNWRAP_THEATER_CODE}: {}",
                    core09_diag_head(&e)
                ),
            )
        }
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// ADR-284 L77 core09-scen-mut-emit-ban probe (ungated) — emit of the 4 positives: zero Rust
/// index-assign `x[..] =`, zero IndexMut / std::ops::Index, zero unwrap/expect/panic; every Vec
/// site lowers to `__arita_vec_set(&mut …)?` (≥1 per program, never discarded).
fn core09_scen_mut_emit_ban_probe(root: &Path) -> OracleResult {
    let id = "core09-scen-mut-emit-ban";
    let mut notes = Vec::new();
    for (_, rel, _) in CORE09_SCEN_MUT_POSITIVES {
        let (_, _, emit) = match core09_scen_mut_build_run(root, rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        let set_lines: Vec<&str> = emit
            .lines()
            .filter(|l| l.contains("__arita_vec_set(&mut "))
            .collect();
        let propagated = set_lines
            .iter()
            .filter(|l| l.contains(")?;") && !l.contains(".ok()") && !l.contains("unwrap_or"))
            .count();
        let discard = emit.contains("let _ = __arita_vec_set");
        let rust_idx_assign = core09_count_index_assign(&emit);
        let index_trait = emit.contains("IndexMut") || emit.contains("std::ops::Index");
        let unwrap = emit.contains(".unwrap()");
        let expect = emit.contains(".expect(");
        let panic = emit.contains(concat!("panic!", "("));
        if set_lines.is_empty()
            || propagated != set_lines.len()
            || discard
            || rust_idx_assign > 0
            || index_trait
            || unwrap
            || expect
            || panic
        {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!(
                    "emit-ban fail set_calls={} propagated={propagated} discard={discard} rust_idx_assign={rust_idx_assign} index_trait={index_trait} unwrap={unwrap} expect={expect} panic={panic}",
                    set_lines.len()
                ),
            );
        }
        notes.push(format!("{rel} set?={propagated}"));
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/scenario-mut",
        Verdict::Accepted,
        format!(
            "emit: zero idx-assign/IndexMut/Index/unwrap/expect/panic; __arita_vec_set(&mut …)? ({})",
            notes.join(", ")
        ),
    )
}

/// ADR-284 L77 core09-scen-mut-emit-ban (gated).
fn run_core09_scen_mut_emit_ban_oracle(root: &Path) -> OracleResult {
    core09_scen_mut_gate(core09_scen_mut_emit_ban_probe(root))
}

/// ADR-284 L78 core09-scen-mut-build probe (ungated) — measure build verde (all 4 positives).
fn core09_scen_mut_build_probe(root: &Path) -> OracleResult {
    let id = "core09-scen-mut-build";
    let mut notes = Vec::new();
    for (_, rel, _) in CORE09_SCEN_MUT_POSITIVES {
        let full = root.join(rel);
        if !full.is_file() {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing: {}", full.display()),
            );
        }
        match build(full.to_str().unwrap_or(rel)) {
            Ok(bin) => notes.push(format!("{rel} → {}", bin.display())),
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!("build failed: {}", e.chars().take(200).collect::<String>()),
                )
            }
        }
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/scenario-mut",
        Verdict::Accepted,
        format!("build ok ×4: {}", notes.join("; ")),
    )
}

/// ADR-284 L78 core09-scen-mut-build (gated).
fn run_core09_scen_mut_build_oracle(root: &Path) -> OracleResult {
    core09_scen_mut_gate(core09_scen_mut_build_probe(root))
}

// ── ADR-285 REF-MUT (Core 0.9 slice 4) — CUT CORE-0.9-REF-MUT-20260926 ─────────────────────────
// Ref `arita-ref-mut` (ADR-285 sha256 856a11ca…, §2 L46–L69; pins en firme Ingeniero 07:42).
// Root workspace `ejemplos/core09/ref-mut/` (happy) + edge workspace `…/ref-mut/edge/` (OOB), both
// packages 0.4 lib+bin (§0 L22). Negs live OUTSIDE the workspace in `ejemplos/core09/ref-mut-neg/`
// (L22/L58: a `[workspace]` arita.toml above them would hijack `arita build`). Exact pins only:
// stdout L49/L50, codes L54–L56, emit counts L53, evidence gates §2.1 L64–L69. N = 834 (L62/L86).
// Skip ≠ PASS: a missing file is Inconclusive, never Accepted. No Result-discard oracle.

const CORE09_REF_MUT_BIN: &str = "ejemplos/core09/ref-mut/bin/main.arita";
const CORE09_REF_MUT_EDGE_BIN: &str = "ejemplos/core09/ref-mut/edge/bin/main.arita";
const CORE09_REF_MUT_LIB: &str = "ejemplos/core09/ref-mut/lib/lib.arita";
const CORE09_REF_MUT_EDGE_LIB: &str = "ejemplos/core09/ref-mut/edge/lib/lib.arita";
const CORE09_REF_MUT_FIXTURE: &str = "ejemplos/core09/ref-mut/fixture.json";
const CORE09_REF_MUT_EDGE_FIXTURE: &str = "ejemplos/core09/ref-mut/edge/fixture.json";
const CORE09_REF_MUT_EVIDENCE: &str = "ejemplos/core09/ref-mut/evidence.json";
/// ADR-285 L49 happy stdout (one line per token).
const CORE09_REF_MUT_HAPPY: &[&str] = &["2", "9", "3", "ref-mut-happy"];
/// ADR-285 L50 OOB stdout.
const CORE09_REF_MUT_OOB: &[&str] = &["ref-mut-err", "0"];
/// ADR-285 L52 / L68: `scenarios` == exactly the 9 ids of §2.
const CORE09_REF_MUT_IDS: &[&str] = &[
    "core09-ref-mut-build",
    "core09-ref-mut-cli-happy",
    "core09-ref-mut-cli-oob",
    "core09-ref-mut-scenario",
    "core09-ref-mut-evidence",
    "core09-ref-mut-emit-ban",
    "neg-core09-ref-vec-assign-outside",
    "neg-core09-ref-map-assign-non-mut",
    "neg-core09-ref-vec-assign-neg-lit",
];
/// ADR-285 L52 / L66: evidence key → source path (source↔evidence sha256).
const CORE09_REF_MUT_EVIDENCE_SOURCES: &[(&str, &str)] = &[
    ("lib_sha256", "ejemplos/core09/ref-mut/lib/lib.arita"),
    ("bin_sha256", "ejemplos/core09/ref-mut/bin/main.arita"),
    ("edge_sha256", "ejemplos/core09/ref-mut/edge/bin/main.arita"),
    (
        "edge_lib_sha256",
        "ejemplos/core09/ref-mut/edge/lib/lib.arita",
    ),
    ("arita_toml_sha256", "ejemplos/core09/ref-mut/arita.toml"),
    (
        "edge_arita_toml_sha256",
        "ejemplos/core09/ref-mut/edge/arita.toml",
    ),
    ("fixture_sha256", "ejemplos/core09/ref-mut/fixture.json"),
    (
        "edge_fixture_sha256",
        "ejemplos/core09/ref-mut/edge/fixture.json",
    ),
];

/// Source code for shape rules (ADR-285 L48): `//` comments stripped (string-literal aware) and
/// `scenario … { … }` blocks removed, so comments / acceptance text never count as code.
fn core09_ref_mut_code(src: &str) -> String {
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut in_scen = false;
    for line in src.lines() {
        let mut code = String::new();
        let mut in_str = false;
        let mut prev = '\0';
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if in_str {
                if c == '"' && prev != '\\' {
                    in_str = false;
                }
            } else if c == '"' {
                in_str = true;
            } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                break;
            }
            code.push(c);
            prev = c;
            i += 1;
        }
        if !in_scen && depth == 0 && code.trim_start().starts_with("scenario ") {
            in_scen = true;
        }
        if in_scen {
            depth += code.matches('{').count() as i32 - code.matches('}').count() as i32;
            if depth <= 0 && code.contains('}') {
                in_scen = false;
                depth = 0;
            }
            continue;
        }
        out.push(code);
    }
    out.join("\n")
}

/// Receivers (identifier before `[`) of every index-assign `ident[…] =` (not `==`) in `code`.
fn core09_ref_mut_assign_receivers(code: &str) -> Vec<String> {
    let b = code.as_bytes();
    let mut out = Vec::new();
    for i in 1..b.len() {
        if b[i] != b'[' {
            continue;
        }
        let mut s = i;
        while s > 0 && (b[s - 1].is_ascii_alphanumeric() || b[s - 1] == b'_') {
            s -= 1;
        }
        if s == i {
            continue;
        }
        let mut j = i + 1;
        while j < b.len() && b[j] != b']' && b[j] != b'\n' {
            j += 1;
        }
        if j >= b.len() || b[j] != b']' || j == i + 1 {
            continue;
        }
        let mut k = j + 1;
        while k < b.len() && (b[k] == b' ' || b[k] == b'\t') {
            k += 1;
        }
        if k + 1 < b.len() && b[k] == b'=' && b[k + 1] != b'=' {
            out.push(code[s..i].to_string());
        }
    }
    out
}

/// Bindings declared as `let [mut] <name>: <ty><…` in `code`.
fn core09_ref_mut_bindings_of(code: &str, ty: &str) -> Vec<String> {
    let mut out = Vec::new();
    for l in code.lines() {
        let t = l.trim_start();
        let Some(rest) = t.strip_prefix("let ") else {
            continue;
        };
        let rest = rest.strip_prefix("mut ").unwrap_or(rest);
        let Some((name, after)) = rest.split_once(':') else {
            continue;
        };
        if after.trim_start().starts_with(&format!("{ty}<")) {
            out.push(name.trim().to_string());
        }
    }
    out
}

/// (Map sites, non-Map sites) of index-assign in `code` (non-Map ⇒ `v[…] =` shape).
fn core09_ref_mut_assign_split(code: &str) -> (usize, usize) {
    let maps = core09_ref_mut_bindings_of(code, "Map");
    let recv = core09_ref_mut_assign_receivers(code);
    let m = recv.iter().filter(|r| maps.contains(r)).count();
    (m, recv.len() - m)
}

/// Exactly the `scenario` blocks of `src`: (name, acceptance lines in order).
fn core09_ref_mut_scenarios(src: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    let mut cur: Option<(String, Vec<String>)> = None;
    for line in src.lines() {
        let t = line.trim();
        if cur.is_none() {
            if let Some(rest) = t.strip_prefix("scenario ") {
                let name = rest.trim_end_matches('{').trim().to_string();
                cur = Some((name, Vec::new()));
            }
            continue;
        }
        if t == "}" {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("acceptance ") {
            let v = rest.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|x| x.strip_suffix('"'))
                .unwrap_or(v);
            if let Some(c) = cur.as_mut() {
                c.1.push(v.to_string());
            }
        }
    }
    out
}

/// Build one ref-mut workspace (packages 0.4). Returns (emit root, runnable bin path, lib.rs, main.rs).
fn core09_ref_mut_build_ws(
    root: &Path,
    bin_rel: &str,
) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), (Verdict, String)> {
    let full = root.join(bin_rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    let out_root =
        crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug)
            .map_err(|e| {
                (
                    Verdict::Rejected,
                    format!(
                        "{bin_rel}: workspace build failed: {}",
                        e.chars().take(220).collect::<String>()
                    ),
                )
            })?;
    let crates_dir = out_root.join("crates");
    let names: Vec<String> = fs::read_dir(&crates_dir)
        .map_err(|e| (Verdict::Rejected, format!("{bin_rel}: read crates: {e}")))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let Some(bin_pkg) = names.iter().find(|n| n.starts_with("bin_")) else {
        return Err((Verdict::Rejected, format!("{bin_rel}: no crates/bin_*")));
    };
    let Some(lib_pkg) = names.iter().find(|n| n.starts_with("lib_")) else {
        return Err((Verdict::Rejected, format!("{bin_rel}: no crates/lib_*")));
    };
    let lib_rs = crates_dir.join(lib_pkg).join("src").join("lib.rs");
    let main_rs = crates_dir.join(bin_pkg).join("src").join("main.rs");
    let bin = out_root.join("target").join("debug").join(bin_pkg);
    if !(lib_rs.is_file() && main_rs.is_file() && bin.is_file()) {
        return Err((
            Verdict::Rejected,
            format!(
                "{bin_rel}: emitted lib.rs={} main.rs={} bin={}",
                lib_rs.is_file(),
                main_rs.is_file(),
                bin.is_file()
            ),
        ));
    }
    Ok((out_root, bin, lib_rs, main_rs))
}

/// Run a built ref-mut bin (cwd = repo root). Non-zero exit / `panicked` → Rejected.
fn core09_ref_mut_run(
    root: &Path,
    bin: &Path,
    args: &[&str],
) -> Result<(Vec<String>, usize), (Verdict, String)> {
    let run = Command::new(bin)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            )
        })?;
    let stderr = String::from_utf8_lossy(&run.stderr).into_owned();
    if !run.status.success() {
        return Err((
            Verdict::Rejected,
            format!(
                "args={args:?}: non-zero exit {} stderr={:?}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(160).collect::<String>()
            ),
        ));
    }
    if stderr.contains("panicked") {
        return Err((
            Verdict::Rejected,
            format!(
                "args={args:?}: stderr contains `panicked`: {}",
                stderr.chars().take(160).collect::<String>()
            ),
        ));
    }
    Ok((
        normalize_stdout(&String::from_utf8_lossy(&run.stdout)),
        stderr.len(),
    ))
}

fn core09_ref_mut_want(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

/// ADR-285 L48 core09-ref-mut-build — both workspaces build (crates/lib_* + crates/bin_*) and the
/// source rules (1)–(5) hold on comment/scenario-stripped code; (6) edge lib ≠ lib is recorded only.
fn run_core09_ref_mut_build_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-build";
    let path = "ejemplos/core09/ref-mut";
    let mut srcs = Vec::new();
    for rel in [
        CORE09_REF_MUT_LIB,
        CORE09_REF_MUT_BIN,
        CORE09_REF_MUT_EDGE_BIN,
        CORE09_REF_MUT_EDGE_LIB,
    ] {
        match fs::read_to_string(root.join(rel)) {
            Ok(s) => srcs.push(s),
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        }
    }
    let lib = core09_ref_mut_code(&srcs[0]);
    let bins = [core09_ref_mut_code(&srcs[1]), core09_ref_mut_code(&srcs[2])];
    let pub_result = lib
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("pub fn ") && t.contains("-> Result<") && t.contains(", Int>")
        })
        .count();
    let (lib_map, lib_vec) = core09_ref_mut_assign_split(&lib);
    let lib_qmark = lib.matches(")?").count();
    let unwrap_any = std::iter::once(&lib)
        .chain(bins.iter())
        .any(|c| c.contains("unwrap") || c.contains("expect"));
    let mut fails = Vec::new();
    if pub_result < 2 {
        fails.push(format!(
            "(1) lib pub fn -> Result<…, Int> = {pub_result} < 2"
        ));
    }
    if lib_map < 1 || lib_vec < 1 {
        fails.push(format!(
            "(2) lib m[…]= {lib_map} v[…]= {lib_vec} (need ≥1 each)"
        ));
    }
    if lib_qmark < 1 {
        fails.push("(3) lib has no composing `?`".into());
    }
    if unwrap_any {
        fails.push("(4) unwrap/expect in lib/bin code".into());
    }
    let mut bin_notes = Vec::new();
    for (rel, code) in [CORE09_REF_MUT_BIN, CORE09_REF_MUT_EDGE_BIN]
        .iter()
        .zip(bins.iter())
    {
        let q = code.matches('?').count();
        let (m, v) = core09_ref_mut_assign_split(code);
        if q > 0 || v > 0 {
            fails.push(format!("(5) {rel}: `?`={q} v[…]={v} (both must be 0)"));
        }
        bin_notes.push(format!("{rel} ?=0 v[…]=0 m[…]={m}"));
    }
    if !fails.is_empty() {
        return core09_vec_fail(
            id,
            path,
            Verdict::Rejected,
            format!("source rules: {}", fails.join("; ")),
        );
    }
    let mut ws = Vec::new();
    for rel in [CORE09_REF_MUT_BIN, CORE09_REF_MUT_EDGE_BIN] {
        match core09_ref_mut_build_ws(root, rel) {
            Ok((out, ..)) => ws.push(format!("{rel} → {}", out.display())),
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        }
    }
    let edge_same = srcs[0] == srcs[3];
    core09_vec_fail(
        id,
        path,
        Verdict::Accepted,
        format!(
            "workspace build ok ×2 (lib_*+bin_*): {}; source rules (1) pub fn→Result<…,Int>={pub_result} (2) lib m[…]={lib_map} v[…]={lib_vec} (3) lib `?`={lib_qmark} (4) unwrap/expect=0 (5) {}; (6) edge/lib == lib: {edge_same} (recorded only)",
            ws.join("; "),
            bin_notes.join(", ")
        ),
    )
}

/// ADR-285 L49 core09-ref-mut-cli-happy — root bin with `fixture.json` AND without args: exit 0, no
/// `panicked`, exact stdout `["2","9","3","ref-mut-happy"]` in both runs.
fn run_core09_ref_mut_cli_happy_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-cli-happy";
    let rel = CORE09_REF_MUT_BIN;
    let (_, bin, _, _) = match core09_ref_mut_build_ws(root, rel) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
    };
    let want = core09_ref_mut_want(CORE09_REF_MUT_HAPPY);
    let mut notes = Vec::new();
    for args in [vec![CORE09_REF_MUT_FIXTURE], vec![]] {
        let (got, err_len) = match core09_ref_mut_run(root, &bin, &args) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        if got != want {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("args={args:?}: stdout mismatch got={got:?} expected={want:?}"),
            );
        }
        notes.push(format!("args={args:?} stderr={err_len}B"));
    }
    core09_vec_fail(
        id,
        rel,
        Verdict::Accepted,
        format!(
            "exit 0, no panic, exact stdout {want:?} ({})",
            notes.join("; ")
        ),
    )
}

/// ADR-285 L50 exact OOB check with explicit reasons (happy token / payload ≠ 0 / extra lines).
fn core09_ref_mut_oob_check(got: &[String]) -> Result<(), String> {
    if got.iter().any(|l| l == "ref-mut-happy") {
        return Err(format!(
            "happy token `ref-mut-happy` on the OOB path: {got:?}"
        ));
    }
    if got.len() != 2 {
        return Err(format!(
            "expected exactly 2 lines [\"ref-mut-err\",\"0\"], got {got:?}"
        ));
    }
    if got[0] != "ref-mut-err" {
        return Err(format!("first line must be `ref-mut-err`, got {got:?}"));
    }
    if got[1] != "0" {
        return Err(format!(
            "payload must be 0 (Err(0) Vec OOB), got {:?}",
            got[1]
        ));
    }
    Ok(())
}

/// ADR-285 L50 core09-ref-mut-cli-oob — edge bin with `edge/fixture.json` and without args, plus the
/// anti-theater run of the ROOT bin with `-- edge/fixture.json`: exit 0, no `panicked`, exactly
/// `["ref-mut-err","0"]` in all three runs.
fn run_core09_ref_mut_cli_oob_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-cli-oob";
    let mut notes = Vec::new();
    let runs: [(&str, Vec<&str>); 3] = [
        (CORE09_REF_MUT_EDGE_BIN, vec![CORE09_REF_MUT_EDGE_FIXTURE]),
        (CORE09_REF_MUT_EDGE_BIN, vec![]),
        (CORE09_REF_MUT_BIN, vec![CORE09_REF_MUT_EDGE_FIXTURE]),
    ];
    let mut built: Option<(String, PathBuf)> = None;
    for (rel, args) in runs {
        let bin = match &built {
            Some((r, b)) if r == rel => b.clone(),
            _ => match core09_ref_mut_build_ws(root, rel) {
                Ok((_, b, _, _)) => {
                    built = Some((rel.to_string(), b.clone()));
                    b
                }
                Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
            },
        };
        let (got, err_len) = match core09_ref_mut_run(root, &bin, &args) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, rel, verdict, detail),
        };
        if let Err(why) = core09_ref_mut_oob_check(&got) {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("{rel} args={args:?}: {why}"),
            );
        }
        notes.push(format!("{rel} args={args:?} stderr={err_len}B"));
    }
    core09_vec_fail(
        id,
        CORE09_REF_MUT_EDGE_BIN,
        Verdict::Accepted,
        format!(
            "exit 0, no panic, exact stdout [\"ref-mut-err\", \"0\"] ×3 ({})",
            notes.join("; ")
        ),
    )
}

/// ADR-285 L51 core09-ref-mut-scenario — exactly two `scenario` blocks over the ref-mut sources:
/// `ref_mut_happy` in bin/main.arita and `ref_mut_err` in edge/bin/main.arita, acceptance == the L49/L50
/// stdouts; `run_lang_contract_file` Accepted on both bins; both cli-* oracles Accepted.
fn run_core09_ref_mut_scenario_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-scenario";
    let path = "ejemplos/core09/ref-mut";
    let mut all = Vec::new();
    for rel in [
        CORE09_REF_MUT_LIB,
        CORE09_REF_MUT_BIN,
        CORE09_REF_MUT_EDGE_LIB,
        CORE09_REF_MUT_EDGE_BIN,
    ] {
        match fs::read_to_string(root.join(rel)) {
            Ok(s) => {
                for (name, acc) in core09_ref_mut_scenarios(&s) {
                    all.push((rel, name, acc));
                }
            }
            Err(e) => {
                return core09_vec_fail(
                    id,
                    rel,
                    Verdict::Inconclusive,
                    format!("missing: {rel}: {e}"),
                )
            }
        }
    }
    let expected: [(&str, &str, &[&str]); 2] = [
        (CORE09_REF_MUT_BIN, "ref_mut_happy", CORE09_REF_MUT_HAPPY),
        (CORE09_REF_MUT_EDGE_BIN, "ref_mut_err", CORE09_REF_MUT_OOB),
    ];
    if all.len() != 2 {
        return core09_vec_fail(
            id,
            path,
            Verdict::Rejected,
            format!(
                "expected exactly 2 scenario blocks, found {}: {:?}",
                all.len(),
                all.iter()
                    .map(|(r, n, _)| format!("{r}:{n}"))
                    .collect::<Vec<_>>()
            ),
        );
    }
    for (rel, name, acc) in expected {
        let Some((_, _, got)) = all.iter().find(|(r, n, _)| *r == rel && n == name) else {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("scenario `{name}` not found in {rel}"),
            );
        };
        let want = core09_ref_mut_want(acc);
        if *got != want {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("scenario `{name}` acceptance {got:?} != pinned stdout {want:?}"),
            );
        }
    }
    for rel in [CORE09_REF_MUT_BIN, CORE09_REF_MUT_EDGE_BIN] {
        let c = crate::contract::run_lang_contract_file(root, &root.join(rel));
        if c.verdict != Verdict::Accepted {
            return core09_vec_fail(
                id,
                rel,
                c.verdict,
                format!(
                    "arita contract {:?}: {}",
                    c.verdict,
                    c.detail.chars().take(240).collect::<String>()
                ),
            );
        }
    }
    let h = run_core09_ref_mut_cli_happy_oracle(root);
    let o = run_core09_ref_mut_cli_oob_oracle(root);
    if h.verdict != Verdict::Accepted || o.verdict != Verdict::Accepted {
        return core09_vec_fail(
            id,
            path,
            Verdict::Rejected,
            format!("cli-happy={:?} cli-oob={:?}", h.verdict, o.verdict),
        );
    }
    core09_vec_fail(
        id,
        path,
        Verdict::Accepted,
        "exactly 2 scenarios (ref_mut_happy bin · ref_mut_err edge), acceptance == pinned stdout; contract Accepted ×2; cli-happy + cli-oob Accepted".into(),
    )
}

/// sha256 of (lib.rs, main.rs) emitted by one fresh build of `bin_rel`.
fn core09_ref_mut_emit_shas(
    root: &Path,
    bin_rel: &str,
) -> Result<(String, String), (Verdict, String)> {
    let (_, _, lib_rs, main_rs) = core09_ref_mut_build_ws(root, bin_rel)?;
    let l = crate::attest::sha256_file(&lib_rs)
        .map_err(|e| (Verdict::Rejected, format!("hash lib.rs: {e}")))?;
    let m = crate::attest::sha256_file(&main_rs)
        .map_err(|e| (Verdict::Rejected, format!("hash main.rs: {e}")))?;
    Ok((l, m))
}

/// ADR-285 L52 + §2.1 L64–L69 core09-ref-mut-evidence — schema/cut_id/adr; source↔evidence sha256;
/// deterministic emit over 2 consecutive builds (and == declared emit sha256 when declared);
/// `scenarios` == exactly the 9 ids; `measure_pass` must be a bool (value informative, not gated).
fn run_core09_ref_mut_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-evidence";
    let rel = CORE09_REF_MUT_EVIDENCE;
    let raw = match fs::read_to_string(root.join(rel)) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing: {rel}: {e}"),
            )
        }
    };
    let ev: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("evidence not JSON: {e}"),
            )
        }
    };
    let s = |k: &str| ev.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    if s("schema_version") != "arita.evidence.v1"
        || s("cut_id") != "CORE-0.9-REF-MUT-20260926"
        || s("adr") != "285"
    {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "header schema_version={:?} cut_id={:?} adr={:?}",
                s("schema_version"),
                s("cut_id"),
                s("adr")
            ),
        );
    }
    for (key, src) in CORE09_REF_MUT_EVIDENCE_SOURCES {
        let declared = s(key);
        if declared.is_empty() {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("evidence missing `{key}`"),
            );
        }
        let got = match crate::attest::sha256_file(&root.join(src)) {
            Ok(h) => h,
            Err(e) => {
                return core09_vec_fail(id, src, Verdict::Inconclusive, format!("hash {src}: {e}"))
            }
        };
        if got != declared {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("{key} ({src}) got={got} declared={declared}"),
            );
        }
    }
    let mut emit_notes = Vec::new();
    for (bin_rel, lib_key, main_key) in [
        (CORE09_REF_MUT_BIN, "lib_rs_sha256", "main_rs_sha256"),
        (
            CORE09_REF_MUT_EDGE_BIN,
            "edge_lib_rs_sha256",
            "edge_main_rs_sha256",
        ),
    ] {
        let a = match core09_ref_mut_emit_shas(root, bin_rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, bin_rel, verdict, detail),
        };
        let b = match core09_ref_mut_emit_shas(root, bin_rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, bin_rel, verdict, detail),
        };
        if a != b {
            return core09_vec_fail(
                id,
                bin_rel,
                Verdict::Rejected,
                format!("emit not deterministic: build1={a:?} build2={b:?}"),
            );
        }
        let emit = ev.get("emit");
        for (key, got) in [(lib_key, &a.0), (main_key, &a.1)] {
            if let Some(decl) = emit.and_then(|e| e.get(key)).and_then(|v| v.as_str()) {
                if decl != got {
                    return core09_vec_fail(
                        id,
                        rel,
                        Verdict::Rejected,
                        format!("emit.{key} declared={decl} got={got}"),
                    );
                }
            }
        }
        emit_notes.push(format!(
            "{bin_rel} lib.rs={} main.rs={}",
            &a.0[..8],
            &a.1[..8]
        ));
    }
    let mut scen: Vec<String> = ev
        .get("scenarios")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let n_decl = scen.len();
    scen.sort();
    let mut want: Vec<String> = CORE09_REF_MUT_IDS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    want.sort();
    if scen != want || n_decl != CORE09_REF_MUT_IDS.len() {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!("scenarios {scen:?} != the 9 ADR-285 §2 ids"),
        );
    }
    let Some(mp) = ev.get("measure_pass").and_then(|v| v.as_bool()) else {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            "measure_pass is not a bool".into(),
        );
    };
    core09_vec_fail(
        id,
        rel,
        Verdict::Accepted,
        format!(
            "source↔evidence sha256 ×{} match; emit deterministic ×2 builds and == declared ({}); scenarios == 9 ids; measure_pass={mp} (bool, informative)",
            CORE09_REF_MUT_EVIDENCE_SOURCES.len(),
            emit_notes.join("; ")
        ),
    )
}

/// ADR-285 L53 core09-ref-mut-emit-ban — every emitted lib.rs/main.rs (root + edge).
fn run_core09_ref_mut_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core09-ref-mut-emit-ban";
    let mut notes = Vec::new();
    for bin_rel in [CORE09_REF_MUT_BIN, CORE09_REF_MUT_EDGE_BIN] {
        let (_, _, lib_rs, main_rs) = match core09_ref_mut_build_ws(root, bin_rel) {
            Ok(v) => v,
            Err((verdict, detail)) => return core09_vec_fail(id, bin_rel, verdict, detail),
        };
        let (lib, main) = match (fs::read_to_string(&lib_rs), fs::read_to_string(&main_rs)) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return core09_vec_fail(id, bin_rel, Verdict::Rejected, "read emit".into()),
        };
        let mut fails = Vec::new();
        for (name, t) in [("lib.rs", &lib), ("main.rs", &main)] {
            if t.contains(".unwrap()")
                || t.contains(".expect(")
                || t.contains(concat!("panic!", "("))
            {
                fails.push(format!("{name}: unwrap/expect/panic"));
            }
            if t.contains("IndexMut") || t.contains("std::ops::Index") {
                fails.push(format!("{name}: IndexMut/std::ops::Index"));
            }
            let ia = core09_count_index_assign(t);
            if ia > 0 {
                fails.push(format!("{name}: Rust index-assign ×{ia}"));
            }
            for helper in ["__arita_vec_set", "__arita_vec_insert"] {
                if let Some(b) = core09_vec_helper_body(t, helper) {
                    if b.contains("as usize") {
                        fails.push(format!("{name}: `as usize` in {helper} body"));
                    }
                }
            }
            if t.lines()
                .any(|l| l.contains("__arita_vec_set(&mut ") && l.contains("as usize"))
            {
                fails.push(format!("{name}: `as usize` at a __arita_vec_set call site"));
            }
        }
        let calls: Vec<&str> = lib
            .lines()
            .filter(|l| l.contains("__arita_vec_set(&mut "))
            .collect();
        let n_calls: usize = calls
            .iter()
            .map(|l| l.matches("__arita_vec_set(&mut ").count())
            .sum();
        let propagated = calls
            .iter()
            .filter(|l| {
                l.contains(")?;")
                    && !l.contains("let _ =")
                    && !l.contains(".ok()")
                    && !l.contains("unwrap_or")
            })
            .count();
        let lib_insert = lib.matches(".insert(").count();
        if n_calls != 4 || propagated != 4 || lib.contains("let _ = __arita_vec_set") {
            fails.push(format!("lib.rs: __arita_vec_set(&mut …) calls={n_calls} propagated `?`={propagated} (need exactly 4/4, no discard)"));
        }
        if core09_vec_helper_body(&lib, "__arita_vec_set").is_none() {
            fails.push("lib.rs: __arita_vec_set helper absent".into());
        }
        if lib_insert != 6 {
            fails.push(format!("lib.rs: .insert( = {lib_insert} (need exactly 6)"));
        }
        let m_q = main.matches('?').count();
        let m_set = main.matches("__arita_vec_set").count();
        let m_ins = main.matches(".insert(").count();
        if m_q != 0 || m_set != 0 || m_ins != 3 {
            fails.push(format!(
                "main.rs: `?`={m_q} __arita_vec_set={m_set} .insert(={m_ins} (need 0/0/3)"
            ));
        }
        if !fails.is_empty() {
            return core09_vec_fail(
                id,
                bin_rel,
                Verdict::Rejected,
                format!("emit-ban fail: {}", fails.join("; ")),
            );
        }
        notes.push(format!(
            "{bin_rel}: lib set?={propagated} insert={lib_insert}, main ?=0 set=0 insert={m_ins}"
        ));
    }
    core09_vec_fail(
        id,
        "ejemplos/core09/ref-mut",
        Verdict::Accepted,
        format!(
            "emit: zero unwrap/expect/panic/IndexMut/Index/idx-assign; zero `as usize` at set sites + helper bodies ({})",
            notes.join("; ")
        ),
    )
}

/// ADR-285 L54 neg-core09-ref-vec-assign-outside — EXACT `E0344: index assign outside result fn @812..820`
/// (span = bytes of `w[0] = 3`, recomputed from the fixture and cross-checked against the ADR value).
fn run_neg_core09_ref_vec_assign_outside_oracle(root: &Path) -> OracleResult {
    let id = "neg-core09-ref-vec-assign-outside";
    let rel = "ejemplos/core09/ref-mut-neg/01-vec-assign-outside.arita";
    if let Ok(src) = fs::read_to_string(root.join(rel)) {
        if core09_vec_stmt_span(&src, "w[0] = 3").as_deref() != Some("812..820") {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!(
                    "fixture drift: span of `w[0] = 3` = {:?}, ADR-285 L54 pins 812..820",
                    core09_vec_stmt_span(&src, "w[0] = 3")
                ),
            );
        }
    }
    run_core09_vec_neg_e0344(root, id, rel, "w[0] = 3")
}

/// Exact single code + exact diag head (message) for a ref-mut neg.
fn core09_ref_mut_neg_exact(
    root: &Path,
    id: &str,
    rel: &str,
    code: &str,
    head: &str,
) -> OracleResult {
    match core09_expect_exact(root, rel, code) {
        Ok(e) => {
            let h = core09_diag_head(&e);
            if h == head {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Accepted,
                    format!("rejected with exact {code}: {h}"),
                )
            } else {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!("expected `{head}`, got `{h}`"),
                )
            }
        }
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// ADR-285 L55 neg-core09-ref-map-assign-non-mut — EXACT `E0202: borrow conflict` (E0314/E0205/E0001
/// or any other code → rejected: every diag code must be E0202).
fn run_neg_core09_ref_map_assign_non_mut_oracle(root: &Path) -> OracleResult {
    core09_ref_mut_neg_exact(
        root,
        "neg-core09-ref-map-assign-non-mut",
        "ejemplos/core09/ref-mut-neg/02-map-assign-non-mut.arita",
        "E0202",
        "E0202: borrow conflict",
    )
}

/// ADR-285 L56 neg-core09-ref-vec-assign-neg-lit — EXACT `E0319: negative set index`.
fn run_neg_core09_ref_vec_assign_neg_lit_oracle(root: &Path) -> OracleResult {
    core09_ref_mut_neg_exact(
        root,
        "neg-core09-ref-vec-assign-neg-lit",
        "ejemplos/core09/ref-mut-neg/03-vec-assign-neg-lit.arita",
        "E0319",
        "E0319: negative set index",
    )
}

// ─── ADR-286 / CUT CORE-0.10-DEAD-SINK-20260927 — Core 0.10 errores fase 1, slice 1 (S1) ───
// Pins: DOC/ADR/286-core-0.10-errores-fase1.md (sha256 4288c47d…): §2 L65 (negs: build FAILS
// with exactly `E0272: result error swallowed`, one diagnostic; positives: build + exit 0 +
// stderr without «panicked» + exact stdout, one line per token) · L69–L74 (the 6 S1 ids) ·
// §4 L140 (N = 840). The Parser implements E0272 v2 (P2(d)/P3/P4, L38–L40) in arita-hir: until
// the Orquestador signals READY, the Parser-dependent probe of the 4 negs is forced to
// Inconclusive (probe verdict kept in the detail). Fixture shape + single-offender control are
// Measure-owned and reject regardless of the gate. Skip ≠ PASS. Nothing from S2 (L75–L81).

/// Parser E0272 v2 gate (ADR-286 slice 1). Flipped by Measure only on the Orquestador signal.
const CORE10_DEAD_SINK_PARSER_READY: bool = true;

/// ADR-286 §2 L65: exact diagnostic head for every S1 neg.
const CORE10_E0272_HEAD: &str = "E0272: result error swallowed";

/// Control edit on a code line (trimmed exact match): `Some(r)` replaces it, `None` deletes it.
type Core10CtlEdit = (&'static str, Option<&'static str>);

/// (id, fixture, offender lines — each exactly once as a code line, control edits → must build).
/// Offenders are the ADR-286 §2 L69–L72 arm bodies; the control swaps the dead sink for a real use
/// of the payload (P3 i / P4), proving the file has a single offender (P5 L41: uniqueness by
/// fixtures). Deleting the sink alone would leave `Err(e) => { print("ok") }` = E0272 v1.
const CORE10_DEAD_SINK_NEGS: &[(&str, &str, &[&str], &[Core10CtlEdit])] = &[
    (
        "neg-core10-dead-sink",
        "ejemplos/core10/errores-neg/01-dead-sink.arita",
        &["let _c: Int = e", "print(\"ok\")"],
        &[("let _c: Int = e", Some("print(e)"))],
    ),
    (
        "neg-core10-dead-sink-underscore",
        "ejemplos/core10/errores-neg/02-dead-sink-underscore.arita",
        &["let _: Int = e", "print(\"ok\")"],
        &[("let _: Int = e", Some("print(e)"))],
    ),
    (
        "neg-core10-dead-sink-transitive",
        "ejemplos/core10/errores-neg/03-dead-sink-transitive.arita",
        &["let _c: Int = e", "let _d: Int = _c", "print(\"ok\")"],
        &[
            ("let _c: Int = e", Some("print(e)")),
            ("let _d: Int = _c", None),
        ],
    ),
    (
        "neg-core10-dead-sink-tautology",
        "ejemplos/core10/errores-neg/04-dead-sink-tautology.arita",
        &["let code: Int = e", "if code == code { print(\"ok\") }"],
        &[("if code == code { print(\"ok\") }", Some("print(code)"))],
    ),
];

/// Shared T09-12 frame of every S1 neg (ADR-286 L69): fn→Result with `v[i] = 7` OOB → Err(0).
const CORE10_DEAD_SINK_FRAME: &[&str] = &[
    "fn tally(i: Int) -> Result<Int, Int> {",
    "v[i] = 7",
    "match tally(99) {",
    "Err(e) => {",
];

/// (id, program, exact stdout, required code lines, forbidden code-line prefixes).
type Core10Positive = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
);

const CORE10_DEAD_SINK_POSITIVES: &[Core10Positive] = &[
    (
        "core10-err-mapped",
        "ejemplos/core10/errores/01-err-mapped.arita",
        &["mapped", "3"],
        &[
            "Err(1)",
            "let _c: Int = e",
            "Err(3)",
            "print(\"mapped\")",
            "print(e)",
        ],
        &[],
    ),
    (
        "core10-err-used",
        "ejemplos/core10/errores/02-err-used.arita",
        &["err", "1"],
        &["Err(1)", "print(\"err\")", "print(e)"],
        &["let "],
    ),
];

/// Number of code lines (comments + scenario blocks stripped) whose trimmed text equals `line`.
fn core10_code_line_count(code: &str, line: &str) -> usize {
    code.lines().filter(|l| l.trim() == line).count()
}

/// Apply control edits to the RAW source line by line (comment lines are never edited).
fn core10_control_source(src: &str, edits: &[Core10CtlEdit]) -> Result<String, String> {
    let mut out = Vec::new();
    let mut hits = vec![0usize; edits.len()];
    for line in src.lines() {
        let t = line.trim();
        match edits.iter().position(|(from, _)| t == *from) {
            Some(k) => {
                hits[k] += 1;
                if let Some(to) = edits[k].1 {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out.push(format!("{indent}{to}"));
                }
            }
            None => out.push(line.to_string()),
        }
    }
    for (k, h) in hits.iter().enumerate() {
        if *h != 1 {
            return Err(format!(
                "control edit {:?} matched {h} lines (need 1)",
                edits[k].0
            ));
        }
    }
    Ok(out.join("\n") + "\n")
}

/// Ingeniero decision (ADR-286 S1, 12:18): the control (sink → `print(e)`; tautology: `if` removed,
/// `print(code)`) must build with NO diagnostic at all. In-process: diagnostics only travel in the
/// `Err` of `build` (the pipeline writes nothing to stderr on success), so `Ok` ⇒ zero; the `Err`
/// text is scanned for E-codes. When measure runs as the `arita` binary the control is also built
/// through the CLI and its stdout+stderr must contain no `E\d{4}` code and exit 0.
fn core10_control_zero_diag(ctl_path: &Path) -> Result<String, String> {
    if let Err(e) = build(ctl_path.to_str().unwrap_or_default()) {
        return Err(format!(
            "single-offender control did not build clean (codes={:?}): {}",
            core09_diag_codes(&e),
            e.chars().take(200).collect::<String>()
        ));
    }
    let exe = std::env::current_exe().ok();
    let is_cli = exe
        .as_ref()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n == "arita");
    let Some(exe) = exe.filter(|_| is_cli) else {
        return Ok("control 0 diagnostics (in-process build Ok; CLI stderr scan only under `arita measure`)".into());
    };
    let out = Command::new(&exe)
        .arg("build")
        .arg(ctl_path)
        .output()
        .map_err(|e| format!("control CLI spawn {}: {e}", exe.display()))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let codes: Vec<String> = core09_diag_codes(&stderr)
        .into_iter()
        .chain(core09_diag_codes(&stdout))
        .collect();
    if out.status.success() && codes.is_empty() {
        Ok(format!(
            "control 0 diagnostics (in-process Ok; CLI exit 0, stderr {}B, no E-code)",
            out.stderr.len()
        ))
    } else {
        Err(format!(
            "control CLI build not clean: exit {} codes={codes:?} stderr={}",
            out.status.code().unwrap_or(-1),
            stderr.chars().take(200).collect::<String>()
        ))
    }
}

fn core10_dead_sink_neg(
    root: &Path,
    id: &str,
    rel: &str,
    offenders: &[&str],
    edits: &[Core10CtlEdit],
) -> OracleResult {
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in CORE10_DEAD_SINK_FRAME.iter().chain(offenders.iter()) {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let ctl_src = match core10_control_source(&src, edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    let probe = match core09_expect_exact(root, rel, "E0272") {
        Ok(e) => {
            let head = core09_diag_head(&e);
            let codes = core09_diag_codes(&e);
            if head == CORE10_E0272_HEAD && codes.len() == 1 {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Accepted,
                    format!("rejected with exact E0272: {head} (1 diagnostic)"),
                )
            } else {
                core09_vec_fail(
                    id,
                    rel,
                    Verdict::Rejected,
                    format!(
                        "E0272 but not exact/unique: head={head:?} codes={codes:?} (want {CORE10_E0272_HEAD:?}, 1 diagnostic)"
                    ),
                )
            }
        }
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    };
    if CORE10_DEAD_SINK_PARSER_READY {
        if probe.verdict == Verdict::Accepted {
            return OracleResult {
                detail: format!("{}; {ctl_note}", probe.detail),
                ..probe
            };
        }
        return probe;
    }
    OracleResult {
        verdict: Verdict::Inconclusive,
        detail: format!(
            "not READY (Parser E0272 v2 gate, ADR-286 S1) — control build ok; {ctl_note}; probe verdict={:?}: {}",
            probe.verdict, probe.detail
        ),
        ..probe
    }
}

fn core10_dead_sink_neg_by_id(root: &Path, id: &str) -> OracleResult {
    match CORE10_DEAD_SINK_NEGS.iter().find(|n| n.0 == id) {
        Some((id, rel, offenders, edits)) => core10_dead_sink_neg(root, id, rel, offenders, edits),
        None => core09_vec_fail(
            id,
            "ejemplos/core10/errores-neg",
            Verdict::Rejected,
            "id not in CORE10_DEAD_SINK_NEGS".into(),
        ),
    }
}

/// ADR-286 §2 L69 — T09-12 / B-285-4: `let _c: Int = e` + `print("ok")` → exact E0272.
fn run_neg_core10_dead_sink_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_neg_by_id(root, "neg-core10-dead-sink")
}

/// ADR-286 §2 L70 — `let _: Int = e` + `print("ok")` → exact E0272.
fn run_neg_core10_dead_sink_underscore_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_neg_by_id(root, "neg-core10-dead-sink-underscore")
}

/// ADR-286 §2 L71 — transitive sink `_c` → `_d` (P4 recursive) → exact E0272.
fn run_neg_core10_dead_sink_transitive_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_neg_by_id(root, "neg-core10-dead-sink-transitive")
}

/// ADR-286 §2 L72 — `if code == code` self-comparison is not a use (P4 ⚑2) → exact E0272.
fn run_neg_core10_dead_sink_tautology_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_neg_by_id(root, "neg-core10-dead-sink-tautology")
}

fn core10_dead_sink_positive(root: &Path, id: &str) -> OracleResult {
    let Some((id, rel, want, required, forbidden)) =
        CORE10_DEAD_SINK_POSITIVES.iter().find(|p| p.0 == id)
    else {
        return core09_vec_fail(
            id,
            "ejemplos/core10/errores",
            Verdict::Rejected,
            "id not in CORE10_DEAD_SINK_POSITIVES".into(),
        );
    };
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable program {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in required.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    for p in forbidden.iter() {
        if let Some(l) = code.lines().find(|l| l.trim_start().starts_with(p)) {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: forbidden code line {:?}", l.trim()),
            );
        }
    }
    if *id == "core10-err-mapped" {
        // D-4 / P3(ii) shape: the dead binding is immediately followed by the explicit Err(3).
        let lines: Vec<&str> = code
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let ok = lines
            .windows(2)
            .any(|w| w[0] == "let _c: Int = e" && w[1] == "Err(3)");
        if !ok {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                "program shape: `let _c: Int = e` must be followed by `Err(3)` (P3 ii)".into(),
            );
        }
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(b) => b,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("build failed: {}", e.chars().take(200).collect::<String>()),
            );
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            );
        }
    };
    let stderr = String::from_utf8_lossy(&run.stderr);
    if !run.status.success() || stderr.contains("panicked") {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "exit {} / panicked={}: {}",
                run.status.code().unwrap_or(-1),
                stderr.contains("panicked"),
                stderr.chars().take(200).collect::<String>()
            ),
        );
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let want: Vec<String> = want.iter().map(|s| (*s).to_string()).collect();
    if got == want {
        core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "build ok, exit 0, no panic, exact stdout {want:?} (stderr={}B)",
                run.stderr.len()
            ),
        )
    } else {
        core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!("stdout mismatch got={got:?} expected={want:?}"),
        )
    }
}

/// ADR-286 §2 L73 — D-4 / P3(ii): dead binding + explicit `Err(3)` is handling → stdout mapped/3.
fn run_core10_err_mapped_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_positive(root, "core10-err-mapped")
}

/// ADR-286 §2 L74 — P3(i): `print(e)` is a real use → stdout err/1.
fn run_core10_err_used_oracle(root: &Path) -> OracleResult {
    core10_dead_sink_positive(root, "core10-err-used")
}

// ─── ADR-286 / CUT CORE-0.10-JOIN-SAFETY-20260927 — Core 0.10 errores fase 1, slice 1b (S1b) ───
// Pins: DOC/ADR/286-core-0.10-errores-fase1.md (sha256 2b1c26be…08d6) §0.1d «Oráculos measure
// (k = 4)»: (1) neg-core10-spawn-result `spawn(f())` suelto → E0203 · (2) neg-core10-join-spawn-
// inline `await join(spawn(f()))` → E0203 · (3) neg-core10-user-fn-join → E0007 · (4) pos
// core10-spawn-iounit → stdout ["w","done"] (forma `await join(spawn(<fn Io<()>>))`). N = 844.
// Measure checks the EXACT code only (every diag code == pin; Ingeniero 27-09: no span/anchor
// check — the `spawn` anchor is covered by Parser HIR tests). Until the Orquestador signals READY,
// the Parser-dependent probe of the 3 negs is forced to Inconclusive (probe verdict kept in the
// detail). Fixture shape + single-offender control are Measure-owned and reject regardless of the
// gate. P3/E0006 is NOT a measure neg (cargo test in arita-cli). Skip ≠ PASS.

/// Parser S1b gate (E0203 spawn contract + E0007 builtin fn names). Flipped by Measure only on
/// the Orquestador signal (Parser HIR 0623c77a READY, flipped to true 2026-10-01).
const CORE10_JOIN_SAFETY_PARSER_READY: bool = true;

/// (id, fixture, exact code, offender code lines — each exactly once, control edits → must build).
/// Controls turn the single offender into its legal twin: the spawned fn becomes `-> Io<()>`
/// (negs 1–2) / the builtin-named fn is renamed to `jn` and made `-> Io<()>` (neg 3).
type Core10JoinSafetyNeg = (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [Core10CtlEdit],
);

const CORE10_JOIN_SAFETY_NEGS: &[Core10JoinSafetyNeg] = &[
    (
        "neg-core10-spawn-result",
        "ejemplos/core10/join-safety/neg/01-spawn-result.arita",
        "E0203",
        &["async fn f() -> Result<Int, Int> {", "Err(7)", "spawn(f())"],
        &[
            (
                "async fn f() -> Result<Int, Int> {",
                Some("async fn f() -> Io<()> {"),
            ),
            ("Err(7)", Some("print(\"f\")")),
        ],
    ),
    (
        "neg-core10-join-spawn-inline",
        "ejemplos/core10/join-safety/neg/02-join-spawn-inline.arita",
        "E0203",
        &[
            "async fn f() -> Result<Int, Int> {",
            "Err(7)",
            "await join(spawn(f()))",
        ],
        &[
            (
                "async fn f() -> Result<Int, Int> {",
                Some("async fn f() -> Io<()> {"),
            ),
            ("Err(7)", Some("print(\"f\")")),
        ],
    ),
    (
        "neg-core10-user-fn-join",
        "ejemplos/core10/join-safety/neg/03-user-fn-join.arita",
        "E0007",
        &[
            "async fn join() -> Result<Int, Int> {",
            "Err(7)",
            "await join()",
        ],
        &[
            (
                "async fn join() -> Result<Int, Int> {",
                Some("async fn jn() -> Io<()> {"),
            ),
            ("Err(7)", Some("print(\"j\")")),
            ("await join()", Some("await jn()")),
        ],
    ),
];

/// (id, program, exact stdout, required code lines).
const CORE10_JOIN_SAFETY_POSITIVES: &[(&str, &str, &[&str], &[&str])] = &[(
    "core10-spawn-iounit",
    "ejemplos/core10/join-safety/pos/01-spawn-iounit.arita",
    &["w", "done"],
    &[
        "async fn worker() -> Io<()> {",
        "print(\"w\")",
        "await join(spawn(worker()))",
        "print(\"done\")",
    ],
)];

fn core10_join_safety_neg(
    root: &Path,
    id: &str,
    rel: &str,
    code_pin: &str,
    offenders: &[&str],
    edits: &[Core10CtlEdit],
) -> OracleResult {
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in offenders.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let ctl_src = match core10_control_source(&src, edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-s1b-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    let probe = match core09_expect_exact(root, rel, code_pin) {
        Ok(e) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "rejected with exact {code_pin}: {} (codes={:?})",
                core09_diag_head(&e),
                core09_diag_codes(&e)
            ),
        ),
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    };
    if CORE10_JOIN_SAFETY_PARSER_READY {
        if probe.verdict == Verdict::Accepted {
            return OracleResult {
                detail: format!("{}; {ctl_note}", probe.detail),
                ..probe
            };
        }
        return probe;
    }
    OracleResult {
        verdict: Verdict::Inconclusive,
        detail: format!(
            "not READY (Parser S1b gate, ADR-286 §0.1d) — control build ok; {ctl_note}; probe verdict={:?}: {}",
            probe.verdict, probe.detail
        ),
        ..probe
    }
}

fn core10_join_safety_neg_by_id(root: &Path, id: &str) -> OracleResult {
    match CORE10_JOIN_SAFETY_NEGS.iter().find(|n| n.0 == id) {
        Some((id, rel, code_pin, offenders, edits)) => {
            core10_join_safety_neg(root, id, rel, code_pin, offenders, edits)
        }
        None => core09_vec_fail(
            id,
            "ejemplos/core10/join-safety/neg",
            Verdict::Rejected,
            "id not in CORE10_JOIN_SAFETY_NEGS".into(),
        ),
    }
}

/// ADR-286 §0.1d oráculo 1 — bare `spawn(f())`, f → Result → exact E0203.
fn run_neg_core10_spawn_result_oracle(root: &Path) -> OracleResult {
    core10_join_safety_neg_by_id(root, "neg-core10-spawn-result")
}

/// ADR-286 §0.1d oráculo 2 — `await join(spawn(f()))`, f → Result → exact E0203.
fn run_neg_core10_join_spawn_inline_oracle(root: &Path) -> OracleResult {
    core10_join_safety_neg_by_id(root, "neg-core10-join-spawn-inline")
}

/// ADR-286 §0.1d oráculo 3 / (c) — user fn declared as `join` (closed list of 18) → exact E0007.
fn run_neg_core10_user_fn_join_oracle(root: &Path) -> OracleResult {
    core10_join_safety_neg_by_id(root, "neg-core10-user-fn-join")
}

/// ADR-286 §0.1d oráculo 4 — `await join(spawn(worker()))`, worker → Io<()> → stdout w/done.
fn run_core10_spawn_iounit_oracle(root: &Path) -> OracleResult {
    let (id, rel, want, required) = CORE10_JOIN_SAFETY_POSITIVES[0];
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable program {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in required.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(b) => b,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("build failed: {}", e.chars().take(200).collect::<String>()),
            );
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            );
        }
    };
    let stderr = String::from_utf8_lossy(&run.stderr);
    if !run.status.success() || stderr.contains("panicked") {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "exit {} / panicked={}: {}",
                run.status.code().unwrap_or(-1),
                stderr.contains("panicked"),
                stderr.chars().take(200).collect::<String>()
            ),
        );
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let want: Vec<String> = want.iter().map(|s| (*s).to_string()).collect();
    if got == want {
        core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "build ok, exit 0, no panic, exact stdout {want:?} (stderr={}B)",
                run.stderr.len()
            ),
        )
    } else {
        core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!("stdout mismatch got={got:?} expected={want:?}"),
        )
    }
}

// ─── ADR-288 / CUT CORE-EMIT-CLIPPY-B282-20261001 — B-282-1 / B-282-2 emit-clippy oracles ───
// Pins: DOC/ADR/288-core-emit-clippy-b282-v0.md (APROBADO Y CONGELADO 02-10, sha256 252d5dd4…; pins = v0.1 bbf27929…) D1–D5, §3, §4.
// k = 2 (one oracle per fix, no negs; N = N_CLOSED + 2 = 851 + 2 = 853, provisional: the Ingeniero
// fixes it at GO IMPL). Each oracle needs ALL of: (1) `arita build` of the fixture succeeds and the
// binary prints exactly the pinned stdout; (2) the emitted Rust has the good form and not the bad
// one; (3) the emitted Rust is clean under BOTH clippy configurations of D4: edition 2015 with
// `clippy-driver --edition 2015 -D warnings -D clippy::all` AND edition 2021 with
// `cargo clippy -- -D warnings` in a scratch crate. Missing `clippy-driver` or `cargo clippy` →
// Inconclusive (D5, never Accepted, skip ≠ PASS); clippy present with warnings → Rejected.
// No codegen change: the fixes already live in `arita-codegen` (ADR-283 FASE 2).

/// Scratch dir (under the repo `target/`, like `core09-map-assign-emit-clippy`).
const B282_SCRATCH: &str = "target/arita-out/b288_emit_clippy";

struct B282Spec {
    id: &'static str,
    rel: &'static str,
    stdout: &'static [&'static str],
    must_contain: &'static [&'static str],
    must_not_contain: &'static [&'static str],
}

const B282_ORACLES: [B282Spec; 2] = [
    B282Spec {
        id: "core09-b282-1-map-get-lit-emit-clippy",
        rel: "ejemplos/core10/emit-clippy/01-map-get-lit.arita",
        stdout: &["7"],
        must_contain: &["m.get(\"a\").cloned()"],
        must_not_contain: &[".get(&\"a\".to_string())"],
    },
    B282Spec {
        id: "core09-b282-2-let-len-emit-clippy",
        rel: "ejemplos/core10/emit-clippy/02-let-int-len.arita",
        stdout: &["1", "1"],
        must_contain: &[
            "let n: i64 = m.len() as i64;",
            "let k: i64 = v.len() as i64;",
        ],
        must_not_contain: &["(m.len() as i64)", "(v.len() as i64)"],
    },
];

/// ADR-288 D3.1: good form present, bad form absent in the emitted Rust.
fn b282_emit_shape(spec: &B282Spec, emit: &str) -> Result<(), String> {
    for want in spec.must_contain {
        if !emit.contains(want) {
            return Err(format!("emit lacks `{want}`"));
        }
    }
    for bad in spec.must_not_contain {
        if emit.contains(bad) {
            return Err(format!("emit still contains `{bad}`"));
        }
    }
    Ok(())
}

/// ADR-288 D5: clippy tooling missing (never Accepted). Same markers as
/// `core09-map-assign-emit-clippy`, plus rustup's «Unknown binary».
fn b282_clippy_tool_missing(combined: &str) -> bool {
    combined.contains("no such command: `clippy`")
        || combined.contains("no such command: \"clippy\"")
        || combined.contains("is not installed")
        || combined.contains("clippy-preview")
        || combined.contains("Unknown binary")
}

fn b282_error_lines(combined: &str) -> String {
    combined
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("error") || t.starts_with("warning") || t.starts_with("-->")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(600)
        .collect()
}

/// D4(a): edition 2015 (the real default `arita build` path) with `clippy-driver`.
fn b282_clippy_2015(dir: &Path, emit: &str) -> Result<(), (Verdict, String)> {
    let src = dir.join("prog2015.rs");
    let bin = dir.join("prog2015");
    fs::create_dir_all(dir)
        .and_then(|_| fs::write(&src, emit))
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("IO error preparing clippy-driver input: {e}"),
            )
        })?;
    let out = Command::new("clippy-driver")
        .args(["--edition", "2015", "-D", "warnings", "-D", "clippy::all"])
        .arg(&src)
        .arg("-o")
        .arg(&bin)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!(
                    "clippy-driver not available (cannot spawn: {e}); never treated as accepted"
                ),
            )
        })?;
    if out.status.success() {
        return Ok(());
    }
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if b282_clippy_tool_missing(&combined) {
        return Err((
            Verdict::Inconclusive,
            "clippy-driver not available (cannot run); never treated as accepted".into(),
        ));
    }
    Err((
        Verdict::Rejected,
        format!(
            "clippy-driver --edition 2015 -D warnings -D clippy::all failed (exit {}): {}",
            out.status.code().unwrap_or(-1),
            b282_error_lines(&combined)
        ),
    ))
}

/// D4(b): edition 2021 (Cargo path) with `cargo clippy -- -D warnings` in a scratch crate.
fn b282_clippy_2021(
    root: &Path,
    dir: &Path,
    krate: &str,
    emit: &str,
) -> Result<(), (Verdict, String)> {
    let manifest = dir.join("crate").join("Cargo.toml");
    let toml = format!(
        "[package]\nname = \"{krate}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[workspace]\n"
    );
    fs::create_dir_all(dir.join("crate").join("src"))
        .and_then(|_| fs::write(&manifest, toml))
        .and_then(|_| fs::write(dir.join("crate").join("src").join("main.rs"), emit))
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("IO error preparing clippy crate: {e}"),
            )
        })?;
    let out = Command::new("cargo")
        .args(["clippy", "--quiet", "--manifest-path"])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(dir.join("target"))
        .args(["--", "-D", "warnings"])
        .current_dir(root)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("cannot spawn cargo clippy: {e}"),
            )
        })?;
    if out.status.success() {
        return Ok(());
    }
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if b282_clippy_tool_missing(&combined) {
        return Err((
            Verdict::Inconclusive,
            "cargo clippy not available (cannot run); never treated as accepted".into(),
        ));
    }
    Err((
        Verdict::Rejected,
        format!(
            "cargo clippy -D warnings (edition 2021) failed (exit {}): {}",
            out.status.code().unwrap_or(-1),
            b282_error_lines(&combined)
        ),
    ))
}

/// ADR-288 §3 — one oracle of `B282_ORACLES`.
fn core09_b282_emit_clippy(root: &Path, spec: &B282Spec) -> OracleResult {
    let (lines, emit) = match core09_build_run_emit(root, spec.rel) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(spec.id, spec.rel, verdict, detail),
    };
    let want: Vec<String> = spec.stdout.iter().map(|s| s.to_string()).collect();
    if lines != want {
        return core09_vec_fail(
            spec.id,
            spec.rel,
            Verdict::Rejected,
            format!("stdout mismatch got={lines:?} expected={want:?}"),
        );
    }
    if let Err(e) = b282_emit_shape(spec, &emit) {
        return core09_vec_fail(spec.id, spec.rel, Verdict::Rejected, e);
    }
    let dir = root.join(B282_SCRATCH).join(spec.id.replace('-', "_"));
    let krate = spec.id.replace('-', "_");
    if let Err((verdict, detail)) = b282_clippy_2015(&dir, &emit) {
        return core09_vec_fail(spec.id, spec.rel, verdict, detail);
    }
    if let Err((verdict, detail)) = b282_clippy_2021(root, &dir, &krate, &emit) {
        return core09_vec_fail(spec.id, spec.rel, verdict, detail);
    }
    core09_vec_fail(
        spec.id,
        spec.rel,
        Verdict::Accepted,
        format!(
            "build ok, stdout {:?}; emit has good form and not the old one; clippy -D warnings clean on edition 2015 (clippy-driver, -D clippy::all) and edition 2021 (cargo clippy)",
            spec.stdout
        ),
    )
}

// ─── ADR-289 / CUT CORE-EMIT-UNUSED-PARENS-V0-20261002 — PU-1..PU-3 unused_parens oracles ───
// Pins: DOC/ADR/289-core-emit-unused-parens-v0.md (APROBADO Y CONGELADO 02-10, §3, §5, §11 Addendum 1).
// k = 3 (PU-1..PU-3; PU-4 is a regression criterion, not an oracle: `core09-ref-mut-evidence`
// stays green with `evidence.json` untouched; N = N_CLOSED + 3 = 853 + 3 = 856, provisional: the
// Ingeniero fixes it at GO IMPL). The ADR does not pin ids or fixtures: every id and fixture below
// is PROPOSED. Each oracle runs the REAL `arita build` pipeline, takes the emitted Rust and
// compiles it with `rustc --emit=metadata -W unused_parens --error-format=json` (edition 2015 =
// the real single-file `arita build` path, AND edition 2021); ZERO `unused_parens` diagnostics
// (other lints are out of scope, Addendum 1). Cargo-class positives (async / [deps] / host
// bridge; their generated crate is edition 2021) are checked with `cargo check --offline`
// on a scratch copy. Missing rustc / cargo, or a tool failure without any diagnostic →
// Inconclusive (skip ≠ PASS, never Accepted); a warning or a compile error → Rejected.

/// Scratch dir (under the repo `target/`, like `b288_emit_clippy`).
const PU_SCRATCH: &str = "target/arita-out/b289_unused_parens";
/// PU-1 (PROPOSED id): every `EJEMPLO_ORACLES` positive, emitted Rust has zero `unused_parens`.
const PU1_ID: &str = "core10-unused-parens-corpus";
/// PU-2 (PROPOSED id): new minimal fixtures (contexts + non-migrated sites).
const PU2_ID: &str = "core10-unused-parens-guards";
/// PU-3 (PROPOSED id): D5, `ejemplos/f2/184-shrink-to-fit-vec.arita`.
const PU3_ID: &str = "core10-unused-parens-capacity";
const PU2_DIR: &str = "ejemplos/core10/unused-parens";

struct PuFixture {
    rel: &'static str,
    stdout: &'static [&'static str],
    must_contain: &'static [&'static str],
    must_not_contain: &'static [&'static str],
}

const PU2_FIXTURES: [PuFixture; 2] = [
    PuFixture {
        rel: "ejemplos/core10/unused-parens/01-contexts.arita",
        stdout: &["10", "2", "12", "3", "6"],
        must_contain: &[
            "let mut s: i64 = a + b;",
            "s = s * 2;",
            "while i < a {",
            "i = i + 1;",
            "let ok: bool = s > a;",
            "10 => a + 10,",
            "let n: i64 = v.len() as i64;",
            "let d: i64 = double(v.len() as i64);",
            "    x + x\n}",
        ],
        must_not_contain: &[
            "(a + b)",
            "(s * 2)",
            "(i < a)",
            "(i + 1)",
            "(s > a)",
            "(a + 10)",
            "(x + x)",
            "= (v.len() as i64)",
            "double((v.len() as i64))",
        ],
    },
    PuFixture {
        rel: "ejemplos/core10/unused-parens/02-precedence-sites.arita",
        stdout: &["ok", "3"],
        must_contain: &[
            "v.resize((3).max(0) as usize, 9);",
            "v.reserve((8).max(0) as usize);",
            "let c: i64 = v.capacity() as i64;",
            "if c >= 8 {",
            "println!(\"{}\", (v.len() as i64));",
        ],
        must_not_contain: &["(v.capacity() as i64)", "((v.len() as i64))"],
    },
];

const PU3_FIXTURE: PuFixture = PuFixture {
    rel: "ejemplos/f2/184-shrink-to-fit-vec.arita",
    stdout: &["ok"],
    must_contain: &["let c: i64 = v.capacity() as i64;"],
    must_not_contain: &["(v.capacity() as i64)"],
};

/// (`unused_parens` count, error-level count, first `unused_parens` rendering) of rustc
/// `--error-format=json` lines (stderr) or cargo `--message-format=json` lines (stdout).
/// `only_main_rs`: for cargo output, count only messages of the generated crate (`src/main.rs`),
/// never those of its path / registry dependencies.
fn pu_parse_diagnostics(text: &str, only_main_rs: bool) -> (usize, usize, String) {
    let mut parens = 0usize;
    let mut errors = 0usize;
    let mut first = String::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with('{') {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(t) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let is_cargo = v.get("reason").and_then(|r| r.as_str()) == Some("compiler-message");
        let msg = if is_cargo {
            match v.get("message") {
                Some(m) => m,
                None => continue,
            }
        } else {
            &v
        };
        if is_cargo && only_main_rs {
            let sp = v
                .get("target")
                .and_then(|t| t.get("src_path"))
                .and_then(|s| s.as_str())
                .unwrap_or("");
            if !sp.ends_with("src/main.rs") {
                continue;
            }
        }
        let code = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|c| c.as_str());
        let level = msg.get("level").and_then(|l| l.as_str());
        if code == Some("unused_parens") {
            parens += 1;
            if first.is_empty() {
                let r = msg
                    .get("rendered")
                    .and_then(|r| r.as_str())
                    .or_else(|| msg.get("message").and_then(|m| m.as_str()))
                    .unwrap_or("");
                first = r
                    .lines()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" | ")
                    .chars()
                    .take(240)
                    .collect();
            }
        }
        if level == Some("error") {
            errors += 1;
        }
    }
    (parens, errors, first)
}

/// Verdict of one rustc / cargo invocation: any `unused_parens` → Rejected; a failing tool with
/// an error diagnostic → Rejected (emitted Rust does not compile); a failing tool without any
/// diagnostic (missing toolchain, IO) → Inconclusive; otherwise clean.
fn pu_classify(
    tool: &str,
    label: &str,
    success: bool,
    code: i32,
    text: &str,
    only_main_rs: bool,
) -> Result<(), (Verdict, String)> {
    let (parens, errors, first) = pu_parse_diagnostics(text, only_main_rs);
    if parens > 0 {
        return Err((
            Verdict::Rejected,
            format!("{tool} {label}: {parens} unused_parens warning(s): {first}"),
        ));
    }
    if !success {
        if errors > 0 {
            return Err((
                Verdict::Rejected,
                format!(
                    "{tool} {label}: emitted Rust does not compile (exit {code}, {errors} error diagnostic(s))"
                ),
            ));
        }
        let tail: String = text.chars().take(240).collect();
        return Err((
            Verdict::Inconclusive,
            format!("{tool} {label}: tool failed without a diagnostic (exit {code}): {tail}"),
        ));
    }
    Ok(())
}

/// `rustc --edition <ed> --emit=metadata -W unused_parens --error-format=json` on `emit`.
fn pu_run_rustc(
    program: &str,
    dir: &Path,
    emit: &str,
    edition: &str,
) -> Result<(), (Verdict, String)> {
    let src = dir.join(format!("prog{edition}.rs"));
    let outdir = dir.join(format!("meta{edition}"));
    fs::create_dir_all(&outdir)
        .and_then(|_| fs::write(&src, emit))
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("IO error preparing rustc input: {e}"),
            )
        })?;
    let out = Command::new(program)
        .args([
            "--edition",
            edition,
            "--crate-type",
            "bin",
            "--emit=metadata",
            "-W",
            "unused_parens",
            "--error-format=json",
            "--out-dir",
        ])
        .arg(&outdir)
        .arg(&src)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("{program} not available (cannot spawn: {e}); never treated as accepted"),
            )
        })?;
    let text = String::from_utf8_lossy(&out.stderr).into_owned();
    pu_classify(
        "rustc",
        &format!("edition {edition}"),
        out.status.success(),
        out.status.code().unwrap_or(-1),
        &text,
        false,
    )
}

/// Cargo-class positive (generated crate, edition 2021): `cargo check --offline` on a scratch
/// copy of `Cargo.toml` + `Cargo.lock` + `src/main.rs`, shared target dir across positives.
fn pu_run_cargo(
    root: &Path,
    dir: &Path,
    manifest_src: &Path,
    target: &Path,
) -> Result<(), (Verdict, String)> {
    let krate = manifest_src
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let main_rs = krate.join("src").join("main.rs");
    if !main_rs.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!(
                "unexpected cargo layout (no src/main.rs): {}",
                krate.display()
            ),
        ));
    }
    let work = dir.join("crate");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(work.join("src"))
        .and_then(|_| fs::copy(manifest_src, work.join("Cargo.toml")))
        .and_then(|_| fs::copy(&main_rs, work.join("src").join("main.rs")))
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("IO error preparing cargo check crate: {e}"),
            )
        })?;
    let lock = krate.join("Cargo.lock");
    if lock.is_file() {
        let _ = fs::copy(&lock, work.join("Cargo.lock"));
    }
    let out = Command::new("cargo")
        .args([
            "check",
            "--offline",
            "--quiet",
            "--message-format=json",
            "--manifest-path",
        ])
        .arg(work.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target)
        .env("RUSTFLAGS", "-W unused_parens")
        .current_dir(root)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("cannot spawn cargo check: {e}; never treated as accepted"),
            )
        })?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    pu_classify(
        "cargo check",
        "edition 2021",
        out.status.success(),
        out.status.code().unwrap_or(-1),
        &text,
        true,
    )
}

/// Canary (anti-theater): a deliberately redundant paren. The oracle may only trust a clean
/// result if the same `rustc -W unused_parens` invocation DOES report this one; a silent canary
/// (JSON format change, lint renamed) would make every oracle pass by construction.
const PU_CANARY: &str = "fn main() {\n    let a: i64 = (1 + 2);\n    println!(\"{}\", a);\n}\n";

fn pu_canary(program: &str, dir: &Path) -> Result<(), (Verdict, String)> {
    match pu_run_rustc(program, &dir.join("canary"), PU_CANARY, "2021") {
        Err((Verdict::Rejected, d)) if d.contains("unused_parens warning") => Ok(()),
        Err((Verdict::Inconclusive, d)) => Err((Verdict::Inconclusive, d)),
        Err((_, d)) => Err((
            Verdict::Inconclusive,
            format!("canary failed unexpectedly: {d}"),
        )),
        Ok(()) => Err((
            Verdict::Inconclusive,
            "canary silent: rustc -W unused_parens reported nothing for a redundant paren; oracle would be blind".into(),
        )),
    }
}

/// ADR-289 D3 exact forms: good form present, bad form absent in the emitted Rust.
fn pu_shape(fx: &PuFixture, emit: &str) -> Result<(), String> {
    for want in fx.must_contain {
        if !emit.contains(want) {
            return Err(format!("emit lacks `{want}`"));
        }
    }
    for bad in fx.must_not_contain {
        if emit.contains(bad) {
            return Err(format!("emit still contains `{bad}`"));
        }
    }
    Ok(())
}

/// One fixture: real build + run (exact stdout) + emit exact forms + rustc 2015 AND 2021.
fn pu_check_fixture(root: &Path, fx: &PuFixture, dir: &Path) -> Result<(), (Verdict, String)> {
    let full = root.join(fx.rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    pu_canary("rustc", dir)?;
    let (lines, emit) = core09_build_run_emit(root, fx.rel)?;
    let want: Vec<String> = fx.stdout.iter().map(|s| s.to_string()).collect();
    if lines != want {
        return Err((
            Verdict::Rejected,
            format!(
                "{}: stdout mismatch got={lines:?} expected={want:?}",
                fx.rel
            ),
        ));
    }
    pu_shape(fx, &emit).map_err(|e| (Verdict::Rejected, format!("{}: {e}", fx.rel)))?;
    for edition in ["2015", "2021"] {
        pu_run_rustc("rustc", dir, &emit, edition)
            .map_err(|(v, d)| (v, format!("{}: {d}", fx.rel)))?;
    }
    Ok(())
}

/// PU-1 invariant (ADR §5): positives reviewed == `EJEMPLO_ORACLES.len()`; the two classes
/// partition the table.
fn pu1_counts_consistent(total: usize, rustc_n: usize, cargo_n: usize) -> bool {
    total > 0 && rustc_n + cargo_n == total
}

/// Rustc-class positive: the emitted `.rs` sits next to the binary (`<bin>.rs`).
fn pu1_check_rustc_class(bin: &Path, dir: &Path) -> Result<(), (Verdict, String)> {
    let rs = PathBuf::from(format!("{}.rs", bin.display()));
    let emit = fs::read_to_string(&rs).map_err(|e| {
        (
            Verdict::Rejected,
            format!("read emit {}: {e}", rs.display()),
        )
    })?;
    for edition in ["2015", "2021"] {
        pu_run_rustc("rustc", dir, &emit, edition)?;
    }
    Ok(())
}

/// PU-1 — every `EJEMPLO_ORACLES` positive: real `arita build`, emitted Rust has zero
/// `unused_parens` (rustc-class: editions 2015 + 2021; cargo-class: edition 2021 of its crate).
fn core10_pu1_corpus(root: &Path) -> OracleResult {
    let total = EJEMPLO_ORACLES.len();
    let base = root.join(PU_SCRATCH).join("pu1");
    let shared_target = base.join("cargo-target");
    let mut rustc_n = 0usize;
    let mut cargo_n = 0usize;
    let mut rejected: Vec<String> = Vec::new();
    let mut inconclusive: Vec<String> = Vec::new();
    for o in EJEMPLO_ORACLES.iter() {
        let full = root.join(o.path);
        if !full.is_file() {
            return core09_vec_fail(
                PU1_ID,
                o.path,
                Verdict::Inconclusive,
                format!("missing: {}", full.display()),
            );
        }
    }
    if let Err((verdict, detail)) = pu_canary("rustc", &base) {
        return core09_vec_fail(PU1_ID, "ejemplos (EJEMPLO_ORACLES)", verdict, detail);
    }
    for o in EJEMPLO_ORACLES.iter() {
        let full = root.join(o.path);
        let built = match crate::build_with_profile(
            full.to_str().unwrap_or(o.path),
            arita_codegen::BuildProfile::Debug,
        ) {
            Ok(b) => b,
            Err(e) => {
                let why: String = e.chars().take(120).collect();
                rejected.push(format!("{}: build failed: {why}", o.id));
                continue;
            }
        };
        let dir = base.join(o.id.replace('-', "_"));
        let res = match &built.cargo_toml {
            None => {
                rustc_n += 1;
                pu1_check_rustc_class(&built.bin, &dir)
            }
            Some(manifest) => {
                cargo_n += 1;
                pu_run_cargo(root, &dir, manifest, &shared_target)
            }
        };
        if let Err((verdict, detail)) = res {
            let line = format!("{}: {detail}", o.id);
            if verdict == Verdict::Rejected {
                rejected.push(line);
            } else {
                // systematic tool problem: stop, never Accepted
                inconclusive.push(line);
                break;
            }
        }
    }
    if !rejected.is_empty() {
        let shown: Vec<String> = rejected.iter().take(5).cloned().collect();
        return core09_vec_fail(
            PU1_ID,
            "ejemplos (EJEMPLO_ORACLES)",
            Verdict::Rejected,
            format!(
                "{} of {} positives with problems: {}",
                rejected.len(),
                total,
                shown.join(" ; ")
            ),
        );
    }
    if let Some(first) = inconclusive.first() {
        return core09_vec_fail(
            PU1_ID,
            "ejemplos (EJEMPLO_ORACLES)",
            Verdict::Inconclusive,
            first.clone(),
        );
    }
    if !pu1_counts_consistent(total, rustc_n, cargo_n) {
        return core09_vec_fail(
            PU1_ID,
            "ejemplos (EJEMPLO_ORACLES)",
            Verdict::Rejected,
            format!(
                "invariant: reviewed {} (rustc {rustc_n} + cargo {cargo_n}) != EJEMPLO_ORACLES.len() {total}",
                rustc_n + cargo_n
            ),
        );
    }
    core09_vec_fail(
        PU1_ID,
        "ejemplos (EJEMPLO_ORACLES)",
        Verdict::Accepted,
        format!(
            "{total} positives reviewed (= EJEMPLO_ORACLES.len()): {rustc_n} rustc-class on editions 2015 and 2021, {cargo_n} cargo-class on edition 2021; 0 unused_parens"
        ),
    )
}

/// PU-2 — new minimal fixtures (contexts + non-migrated sites): all must pass.
fn core10_pu2_guards(root: &Path) -> OracleResult {
    for (i, fx) in PU2_FIXTURES.iter().enumerate() {
        let dir = root.join(PU_SCRATCH).join("pu2").join(format!("f{i}"));
        if let Err((verdict, detail)) = pu_check_fixture(root, fx, &dir) {
            return core09_vec_fail(PU2_ID, fx.rel, verdict, detail);
        }
    }
    core09_vec_fail(
        PU2_ID,
        PU2_DIR,
        Verdict::Accepted,
        format!(
            "{} fixtures: build ok, exact stdout, emit has the pinned forms, rustc -W unused_parens clean on edition 2015 and 2021",
            PU2_FIXTURES.len()
        ),
    )
}

/// PU-3 (ADR D5) — `let c: i64 = v.capacity() as i64;` without parentheses, same output.
fn core10_pu3_capacity(root: &Path) -> OracleResult {
    let dir = root.join(PU_SCRATCH).join("pu3");
    if let Err((verdict, detail)) = pu_check_fixture(root, &PU3_FIXTURE, &dir) {
        return core09_vec_fail(PU3_ID, PU3_FIXTURE.rel, verdict, detail);
    }
    core09_vec_fail(
        PU3_ID,
        PU3_FIXTURE.rel,
        Verdict::Accepted,
        "build ok, stdout [\"ok\"]; emit has `let c: i64 = v.capacity() as i64;` and not the parenthesised form; rustc -W unused_parens clean on edition 2015 and 2021".into(),
    )
}

// ─── ADR-290 / CUT CORE-0.10-INDEX-MUT-20261002 — slice A: IM-1..IM-6 compound index-assign ───
// Pins: DOC/ADR/290-core-index-mut-v0.md (v0.3 APROBADO Y CONGELADO 03-10; D1–D8, Pin 1–3, «Oráculos
// propuestos»). k = 6 (IM-1..IM-6), N = 856 + 6 = 862. The ADR does NOT pin ids or fixtures: every id
// and fixture below is PROPOSED (`core10-index-mut-*`, `ejemplos/core10/index-mut/`).
//   IM-1 `v[i] += x` / `-=` / `*=` on Vec<Int> in `fn -> Result`: real build, exact stdout (pos).
//   IM-2 out-of-range / negative / overflow ⇒ `Err(0)` propagated, no panic, EQUAL in debug AND
//        release (D5-ii: `checked_*`): three fixtures × two profiles, exact stdout.
//   IM-3 emit-ban: helper `__arita_vec_update` + `i64::checked_*` call sites present; 0 IndexMut /
//        Index / *Assign / unwrap / expect / panic!; rustc -W unused_parens clean (2015 and 2021)
//        and clippy -D warnings clean (2015 and 2021) on the emitted Rust of the four fixtures.
//   IM-4 neg, PROPOSED (conflicts with the ADR text, see below): `/=`, `%=` and `Vec<Text>` ⇒ EXACT
//        E0006. The ADR says E0333 for Vec<Text>, but the grammar only admits Vec<Int>/List<Int>, so
//        E0333 is NOT reachable from source (HIR tests only; Parser note §5.3). Pending decision.
//   IM-5 neg: non-mut E0202 · rhs Text E0203 · main E0344 · negative literal E0319 · Map/String
//        compound E0006 (emitted by HIR with the exact ADR text + statement span) · Int E0314 ·
//        precedence pairs (Pin 3) E0344>E0319, E0344>E0203, E0319>E0203 (the pairs that involve E0333
//        are unreachable from source). Exact code only (every code in the diagnostic must be the one).
//   IM-6 migration: `ejemplos/core09/vec-assign/neg/05-compound.arita` (byte-identical, frozen) is now
//        VALID; it is retired from `neg-core09-vec-assign-compound` (see `VEC_ASSIGN_COMPOUND_NEG`) and
//        this oracle pins the positive: the old file builds and prints `1`; `05-compound-ok.arita`
//        prints `2`. Missing rustc / clippy / fixture ⇒ Inconclusive (skip ≠ PASS).

/// Scratch dir (under the repo `target/`, like `b289_unused_parens`).
const IM_SCRATCH: &str = "target/arita-out/b290_index_mut";
const IM_DIR: &str = "ejemplos/core10/index-mut";
const IM1_ID: &str = "core10-index-mut-compound-ok";
const IM2_ID: &str = "core10-index-mut-err-propagation";
const IM3_ID: &str = "core10-index-mut-emit-ban";
const IM4_ID: &str = "core10-index-mut-neg-unsupported";
const IM5_ID: &str = "core10-index-mut-neg-rules";
const IM6_ID: &str = "core10-index-mut-migrated-05";

const IM1_FIXTURE: &str = "ejemplos/core10/index-mut/01-ok.arita";
const IM1_STDOUT: [&str; 5] = ["15", "13", "90", "12", "24"];

struct ImRun {
    rel: &'static str,
    stdout: &'static [&'static str],
}

/// IM-2: each fixture is run in debug AND release and must print exactly `stdout`.
const IM2_FIXTURES: [ImRun; 3] = [
    ImRun {
        rel: "ejemplos/core10/index-mut/02-oob-neg.arita",
        stdout: &["3", "0", "0", "0"],
    },
    ImRun {
        rel: "ejemplos/core10/index-mut/03-overflow.arita",
        stdout: &[
            "9223372036854775807",
            "0",
            "-9223372036854775808",
            "0",
            "9223372036854775807",
            "0",
            "-9223372036854775807",
        ],
    },
    ImRun {
        rel: "ejemplos/core10/index-mut/04-literal-overflow.arita",
        stdout: &["0", "0", "0"],
    },
];

struct ImEmit {
    rel: &'static str,
    stdout: &'static [&'static str],
    calls: &'static [&'static str],
}

const IM_CALL_ADD: &str = "__arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_add)?;";
const IM_CALL_SUB: &str = "__arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_sub)?;";
const IM_CALL_MUL: &str = "__arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_mul)?;";

/// IM-3: the four fixtures of IM-1/IM-2 (stdout re-checked, the emit is what is judged).
const IM3_FIXTURES: [ImEmit; 4] = [
    ImEmit {
        rel: "ejemplos/core10/index-mut/01-ok.arita",
        stdout: &IM1_STDOUT,
        calls: &[IM_CALL_ADD, IM_CALL_SUB, IM_CALL_MUL],
    },
    ImEmit {
        rel: "ejemplos/core10/index-mut/02-oob-neg.arita",
        stdout: &["3", "0", "0", "0"],
        calls: &[IM_CALL_ADD],
    },
    ImEmit {
        rel: "ejemplos/core10/index-mut/03-overflow.arita",
        stdout: &[
            "9223372036854775807",
            "0",
            "-9223372036854775808",
            "0",
            "9223372036854775807",
            "0",
            "-9223372036854775807",
        ],
        calls: &[IM_CALL_ADD, IM_CALL_SUB, IM_CALL_MUL],
    },
    ImEmit {
        rel: "ejemplos/core10/index-mut/04-literal-overflow.arita",
        stdout: &["0", "0", "0"],
        calls: &[IM_CALL_ADD, IM_CALL_SUB, IM_CALL_MUL],
    },
];

/// ADR D7 helper signature (exact).
const IM3_HELPER_SIG: &str = "fn __arita_vec_update(v: &mut [i64], i: i64, x: i64, op: fn(i64, i64) -> Option<i64>) -> Result<(), i64> {";
/// Inside the helper: `usize::try_from(i)`, one `get_mut` lookup, the op as a pointer to fn.
const IM3_HELPER_MUST: [&str; 3] = ["usize::try_from(i)", "v.get_mut(u)", "op(*slot, x)"];
/// Inside the helper: no panic path, no raw arithmetic between slot and x (D7 emit-bans).
const IM3_HELPER_BANS: [&str; 10] = [
    "unwrap", "expect(", "panic!", "as usize", "IndexMut", "*slot + ", "*slot - ", "*slot * ",
    " + x", " - x",
];
/// Anywhere in the emitted Rust.
const IM3_EMIT_BANS: [&str; 8] = [
    "IndexMut",
    "std::ops::Index",
    "AddAssign",
    "SubAssign",
    "MulAssign",
    ".unwrap(",
    ".expect(",
    "panic!",
];

/// Body of `fn __arita_vec_update(` in the emitted Rust (up to the first `\n}` at column 0).
fn im_helper_body(emit: &str) -> Option<&str> {
    let start = emit.find("fn __arita_vec_update(")?;
    let rest = &emit[start..];
    let end = rest.find("\n}")?;
    Some(&rest[..end + 2])
}

/// ADR-290 D7 emit-ban: helper, call sites and bans.
fn im_emit_shape(fx: &ImEmit, emit: &str) -> Result<(), String> {
    if !emit.contains(IM3_HELPER_SIG) {
        return Err("emit lacks the exact `__arita_vec_update` signature".into());
    }
    let Some(body) = im_helper_body(emit) else {
        return Err("cannot isolate the `__arita_vec_update` body".into());
    };
    for want in IM3_HELPER_MUST {
        if !body.contains(want) {
            return Err(format!("helper lacks `{want}`"));
        }
    }
    for bad in IM3_HELPER_BANS {
        if body.contains(bad) {
            return Err(format!("helper contains banned `{bad}`"));
        }
    }
    for call in fx.calls {
        if !emit.contains(call) {
            return Err(format!("emit lacks call `{call}`"));
        }
    }
    for bad in IM3_EMIT_BANS {
        if emit.contains(bad) {
            return Err(format!("emit contains banned `{bad}`"));
        }
    }
    Ok(())
}

/// Build + run one fixture in one profile (debug | release); exact stdout, exit 0, no panic.
fn im_run_profile(
    root: &Path,
    fx: &ImRun,
    profile: arita_codegen::BuildProfile,
    label: &str,
) -> Result<(), (Verdict, String)> {
    let full = root.join(fx.rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    let built =
        crate::build_with_profile(full.to_str().unwrap_or(fx.rel), profile).map_err(|e| {
            (
                Verdict::Rejected,
                format!(
                    "{}: {label} build failed: {}",
                    fx.rel,
                    e.chars().take(200).collect::<String>()
                ),
            )
        })?;
    let run = Command::new(&built.bin).output().map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("{}: cannot spawn {}: {e}", fx.rel, built.bin.display()),
        )
    })?;
    let stderr = String::from_utf8_lossy(&run.stderr).into_owned();
    if !run.status.success() || stderr.contains("panicked") {
        return Err((
            Verdict::Rejected,
            format!(
                "{}: {label} run failed (exit {}): {}",
                fx.rel,
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        ));
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let want: Vec<String> = fx.stdout.iter().map(|s| s.to_string()).collect();
    if got != want {
        return Err((
            Verdict::Rejected,
            format!(
                "{}: {label} stdout mismatch got={got:?} expected={want:?}",
                fx.rel
            ),
        ));
    }
    Ok(())
}

/// IM-1 — `v[i] += x` / `-=` / `*=` on Vec<Int> in `fn -> Result`: build, run, exact stdout.
fn core10_im1_compound_ok(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: IM1_ID,
            path: IM1_FIXTURE,
            expected: &IM1_STDOUT,
            require_test: false,
        },
    )
}

/// IM-2 — OOB / negative / overflow ⇒ `Err(0)`, v intact, no panic, same in debug and release.
fn core10_im2_err_propagation(root: &Path) -> OracleResult {
    for fx in IM2_FIXTURES.iter() {
        for (profile, label) in [
            (arita_codegen::BuildProfile::Debug, "debug"),
            (arita_codegen::BuildProfile::Release, "release"),
        ] {
            if let Err((verdict, detail)) = im_run_profile(root, fx, profile, label) {
                return core09_vec_fail(IM2_ID, fx.rel, verdict, detail);
            }
        }
    }
    core09_vec_fail(
        IM2_ID,
        IM_DIR,
        Verdict::Accepted,
        format!(
            "{} fixtures × (debug, release): exit 0, no panic, exact stdout (Err(0) for OOB, negative index and overflow of +=, -=, *=)",
            IM2_FIXTURES.len()
        ),
    )
}

/// One fixture of IM-3: build + run + emit-ban + rustc (2015, 2021) + clippy (2015, 2021).
fn im3_check(root: &Path, fx: &ImEmit, dir: &Path, krate: &str) -> Result<(), (Verdict, String)> {
    let (lines, emit) = core09_build_run_emit(root, fx.rel)?;
    let want: Vec<String> = fx.stdout.iter().map(|s| s.to_string()).collect();
    if lines != want {
        return Err((
            Verdict::Rejected,
            format!(
                "{}: stdout mismatch got={lines:?} expected={want:?}",
                fx.rel
            ),
        ));
    }
    im_emit_shape(fx, &emit).map_err(|e| (Verdict::Rejected, format!("{}: {e}", fx.rel)))?;
    for edition in ["2015", "2021"] {
        pu_run_rustc("rustc", &dir.join("rustc"), &emit, edition)
            .map_err(|(v, d)| (v, format!("{}: {d}", fx.rel)))?;
    }
    b282_clippy_2015(&dir.join("clippy"), &emit)
        .map_err(|(v, d)| (v, format!("{}: {d}", fx.rel)))?;
    b282_clippy_2021(root, &dir.join("clippy"), krate, &emit)
        .map_err(|(v, d)| (v, format!("{}: {d}", fx.rel)))?;
    Ok(())
}

/// IM-3 — emit-ban + rustc / clippy clean on the emitted Rust.
fn core10_im3_emit_ban(root: &Path) -> OracleResult {
    for fx in IM3_FIXTURES.iter() {
        let full = root.join(fx.rel);
        if !full.is_file() {
            return core09_vec_fail(
                IM3_ID,
                fx.rel,
                Verdict::Inconclusive,
                format!("missing: {}", full.display()),
            );
        }
    }
    let base = root.join(IM_SCRATCH).join("im3");
    if let Err((verdict, detail)) = pu_canary("rustc", &base) {
        return core09_vec_fail(IM3_ID, IM_DIR, verdict, detail);
    }
    for (i, fx) in IM3_FIXTURES.iter().enumerate() {
        let dir = base.join(format!("f{i}"));
        if let Err((verdict, detail)) = im3_check(root, fx, &dir, &format!("im3_f{i}")) {
            return core09_vec_fail(IM3_ID, fx.rel, verdict, detail);
        }
    }
    core09_vec_fail(
        IM3_ID,
        IM_DIR,
        Verdict::Accepted,
        format!(
            "{} fixtures: helper `__arita_vec_update` + `i64::checked_*` call sites present; 0 IndexMut/Index/*Assign/unwrap/expect/panic!; rustc -W unused_parens clean and clippy -D warnings clean on editions 2015 and 2021",
            IM3_FIXTURES.len()
        ),
    )
}

/// IM-4 (PROPOSED, E0006 version; the ADR text says E0333 for Vec<Text>, not reachable from source).
const IM4_CASES: [(&str, &str); 3] = [
    (
        "ejemplos/core10/index-mut/neg/01-div-assign.arita",
        "v[0] /= 2",
    ),
    (
        "ejemplos/core10/index-mut/neg/02-rem-assign.arita",
        "v[0] %= 2",
    ),
    (
        "ejemplos/core10/index-mut/neg/03-vec-text.arita",
        "let mut v: Vec<Text> = Vec::new()",
    ),
];

fn core10_im4_unsupported(root: &Path) -> OracleResult {
    run_core09_vec_neg_e0006(root, IM4_ID, &IM4_CASES)
}

/// IM-5 case: `code` is the EXACT ADR code; `head`, when set, is the exact diagnostic text up to
/// ` @<statement span>` (HIR-emitted diagnostics whose text the ADR/HIR pins).
struct ImNeg {
    rel: &'static str,
    code: &'static str,
    stmt: &'static str,
    head: Option<&'static str>,
}

const IM_E0344_HEAD: &str = "E0344: index assign outside result fn";
const IM_E0006_HEAD: &str = "E0006: construct outside F1.1 (parse failure)";

const IM5_CASES: [ImNeg; 10] = [
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/11-non-mut.arita",
        code: "E0202",
        stmt: "v[0] += 1",
        head: None,
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/12-rhs-text.arita",
        code: "E0203",
        stmt: "v[0] += \"x\"",
        head: None,
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/13-main.arita",
        code: "E0344",
        stmt: "v[0] += 2",
        head: Some(IM_E0344_HEAD),
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/14-neg-lit.arita",
        code: "E0319",
        stmt: "v[-1] += 2",
        head: None,
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/15-map.arita",
        code: "E0006",
        stmt: "m[\"a\"] += 1",
        head: Some(IM_E0006_HEAD),
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/16-string.arita",
        code: "E0006",
        stmt: "s[0] += 1",
        head: Some(IM_E0006_HEAD),
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/17-int.arita",
        code: "E0314",
        stmt: "n[0] += 1",
        head: None,
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/18-prec-e0344-e0319.arita",
        code: "E0344",
        stmt: "v[-1] += 2",
        head: Some(IM_E0344_HEAD),
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/19-prec-e0344-e0203.arita",
        code: "E0344",
        stmt: "v[0] += \"x\"",
        head: Some(IM_E0344_HEAD),
    },
    ImNeg {
        rel: "ejemplos/core10/index-mut/neg/20-prec-e0319-e0203.arita",
        code: "E0319",
        stmt: "v[-1] += \"x\"",
        head: None,
    },
];

/// First diagnostic line must start with `<code>:` and, when `exact` is set, equal it.
fn im_diag_check(diag: &str, code: &str, exact: Option<&str>) -> Result<(), String> {
    let first = diag.lines().next().unwrap_or("").trim();
    if !first.starts_with(&format!("{code}:")) {
        return Err(format!(
            "first line `{first}` does not start with `{code}:`"
        ));
    }
    if let Some(want) = exact {
        if first != want {
            return Err(format!("expected `{want}`, got `{first}`"));
        }
    }
    Ok(())
}

/// IM-5 — every case fails to build with the EXACT code (all codes in the diagnostic equal it).
fn core10_im5_neg_rules(root: &Path) -> OracleResult {
    let mut notes = Vec::new();
    for case in IM5_CASES.iter() {
        let src = match fs::read_to_string(root.join(case.rel)) {
            Ok(s) => s,
            Err(e) => {
                return core09_vec_fail(
                    IM5_ID,
                    case.rel,
                    Verdict::Inconclusive,
                    format!("missing: {}: {e}", case.rel),
                )
            }
        };
        let Some(span) = core09_vec_stmt_span(&src, case.stmt) else {
            return core09_vec_fail(
                IM5_ID,
                case.rel,
                Verdict::Rejected,
                format!("fixture shape: statement `{}` not found", case.stmt),
            );
        };
        let diag = match core09_expect_exact(root, case.rel, case.code) {
            Ok(d) => d,
            Err((verdict, detail)) => return core09_vec_fail(IM5_ID, case.rel, verdict, detail),
        };
        let exact = case.head.map(|h| format!("{h} @{span}"));
        if let Err(e) = im_diag_check(&diag, case.code, exact.as_deref()) {
            return core09_vec_fail(
                IM5_ID,
                case.rel,
                Verdict::Rejected,
                format!("{}: {e}", case.rel),
            );
        }
        notes.push(format!("{} → {}", case.stmt, case.code));
    }
    core09_vec_fail(
        IM5_ID,
        IM_DIR,
        Verdict::Accepted,
        format!(
            "{} negs rejected with their exact code: {}",
            IM5_CASES.len(),
            notes.join("; ")
        ),
    )
}

/// IM-6 — the frozen ADR-283 fixture (unchanged file) is now valid and the new positive prints 2.
const IM6_OLD: &str = "ejemplos/core09/vec-assign/neg/05-compound.arita";
const IM6_NEW: &str = "ejemplos/core10/index-mut/05-compound-ok.arita";

fn core10_im6_migration(root: &Path) -> OracleResult {
    let (old_lines, old_emit) = match core09_build_run_emit(root, IM6_OLD) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(IM6_ID, IM6_OLD, verdict, detail),
    };
    if old_lines != ["1"] {
        return core09_vec_fail(
            IM6_ID,
            IM6_OLD,
            Verdict::Rejected,
            format!("{IM6_OLD}: stdout mismatch got={old_lines:?} expected=[\"1\"]"),
        );
    }
    if !old_emit.contains(IM_CALL_ADD) {
        return core09_vec_fail(
            IM6_ID,
            IM6_OLD,
            Verdict::Rejected,
            format!("{IM6_OLD}: emit lacks `{IM_CALL_ADD}`"),
        );
    }
    let (new_lines, new_emit) = match core09_build_run_emit(root, IM6_NEW) {
        Ok(v) => v,
        Err((verdict, detail)) => return core09_vec_fail(IM6_ID, IM6_NEW, verdict, detail),
    };
    if new_lines != ["2"] {
        return core09_vec_fail(
            IM6_ID,
            IM6_NEW,
            Verdict::Rejected,
            format!("{IM6_NEW}: stdout mismatch got={new_lines:?} expected=[\"2\"]"),
        );
    }
    if !new_emit.contains(IM_CALL_ADD) {
        return core09_vec_fail(
            IM6_ID,
            IM6_NEW,
            Verdict::Rejected,
            format!("{IM6_NEW}: emit lacks `{IM_CALL_ADD}`"),
        );
    }
    core09_vec_fail(
        IM6_ID,
        IM6_NEW,
        Verdict::Accepted,
        "05-compound (ADR-283 fixture, unchanged) now builds and prints 1; 05-compound-ok prints 2; both emit the checked_add call".into(),
    )
}

// ─── ADR-292 INT-ARITH-RUNTIME (IA-1..IA-4) — CUT CORE-0.10-INT-ARITH-RUNTIME-20261003 ───────────
// Runtime oracles whose SUCCESS is exit 101: `a + b`, `a - b`, `a * b` on non-literal Int overflow →
// uniform rustc panic in debug (`arita build`), `--profile release`, `arita test` and a package
// workspace in release. Pins (ADR-292 v0.1 §4): ids, k = 4, N = 862 + 4 = 866. Harness: bounded
// runner (60 s, child killed by handle, timeout ⇒ Inconclusive) + pure checkers that can FAIL
// (exit exactly 101; stdout exactly `before`; stderr carries the phrase of THAT operation and none
// of the others). Nothing asserts `panicked`, a path, a line or a thread. Release/package binaries
// are executed directly (never `cargo run`: cargo itself exits 101 on its own errors).

const IA1_ID: &str = "core10-int-overflow-build";
const IA2_ID: &str = "core10-int-overflow-release";
const IA3_ID: &str = "core10-int-overflow-test";
const IA4_ID: &str = "core10-int-overflow-workspace-release";
const IA_DIR: &str = "ejemplos/core10/int-overflow";
const IA_SCRATCH: &str = "target/arita-out/b292_int_overflow";
/// Limit for one run of an emitted program (trivial programs: a hang is a defect, never a PASS).
const IA_RUN_SECS: u64 = 60;
/// Limit for `arita test` (spawns `rustc --test -O`).
const IA_CLI_TEST_SECS: u64 = 240;
const IA_BEFORE: [&str; 1] = ["before"];
const IA_ADD: &str = "attempt to add with overflow";
const IA_SUB: &str = "attempt to subtract with overflow";
const IA_MUL: &str = "attempt to multiply with overflow";
const IA_PHRASES: [&str; 3] = [IA_ADD, IA_SUB, IA_MUL];
/// Other arithmetic panics: a run that dies of one of these did not overflow `+ - *`.
const IA_OTHER_PANICS: [&str; 2] = ["attempt to divide", "attempt to calculate the remainder"];

struct IaOp {
    rel: &'static str,
    phrase: &'static str,
}

const IA_OPS: [IaOp; 3] = [
    IaOp {
        rel: "ejemplos/core10/int-overflow/01-add.arita",
        phrase: IA_ADD,
    },
    IaOp {
        rel: "ejemplos/core10/int-overflow/02-sub.arita",
        phrase: IA_SUB,
    },
    IaOp {
        rel: "ejemplos/core10/int-overflow/03-mul.arita",
        phrase: IA_MUL,
    },
];
/// Control: exact boundaries WITHOUT overflow ⇒ exit 0 and intact stdout in every profile.
const IA_CONTROL: &str = "ejemplos/core10/int-overflow/04-boundary-ok.arita";
const IA_CONTROL_STDOUT: [&str; 6] = [
    "before",
    "9223372036854775807",
    "-9223372036854775808",
    "9223372030926249001",
    "-9223372036854775808",
    "after",
];
const IA_TEST_FIXTURE: &str = "ejemplos/core10/int-overflow/05-test.arita";
const IA_TEST_CTL: &str = "ctl";
/// `arita test`: each of these tests asserts a value that HOLDS under wrap-around, so it can only
/// fail through the overflow panic of its own operation.
const IA_TEST_OVF: [(&str, &str); 3] = [
    ("ovf_add", IA_ADD),
    ("ovf_sub", IA_SUB),
    ("ovf_mul", IA_MUL),
];
const IA_TEST_SUMMARY: &str = "test result: FAILED. 1 passed; 3 failed";
const IA_TEST_EXIT_LINE: &str = "tests failed (exit 101)";
const IA_PKG_DIR: &str = "ejemplos/core10/int-overflow/pkg";
const IA_PKG_FILES: [&str; 3] = ["arita.toml", "lib/lib.arita", "bin/main.arita"];
const IA_PKG_ENTRY: &str = "bin/main.arita";

/// One finished run of a child process.
struct IaRun {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run `bin` with a hard time limit: spawn, drain both pipes on helper threads, poll `try_wait`;
/// on timeout kill the child through its handle (no pkill) ⇒ Inconclusive (timeout ≠ PASS).
fn ia_run_bounded(
    bin: &Path,
    args: &[&str],
    cwd: Option<&Path>,
    secs: u64,
) -> Result<IaRun, (Verdict, String)> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("RUST_BACKTRACE");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let mut child = cmd.spawn().map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("cannot spawn {}: {e}", bin.display()),
        )
    })?;
    let mut out_pipe = child.stdout.take();
    let mut err_pipe = child.stderr.take();
    let out_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = out_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = err_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let deadline = Instant::now() + Duration::from_secs(secs);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err((
                        Verdict::Inconclusive,
                        format!(
                            "timeout after {secs}s running {}; child killed",
                            bin.display()
                        ),
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err((
                    Verdict::Inconclusive,
                    format!("cannot wait for {}: {e}", bin.display()),
                ));
            }
        }
    };
    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    Ok(IaRun {
        code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

/// ADR-292 D1: the run must be the uniform overflow panic of `phrase`'s operation.
fn ia_check_panic(phrase: &str, run: &IaRun) -> Result<(), String> {
    if !IA_PHRASES.contains(&phrase) {
        return Err(format!("unknown operation phrase `{phrase}`"));
    }
    match run.code {
        Some(101) => {}
        Some(c) => return Err(format!("exit {c} (need exactly 101)")),
        None => return Err("no exit code: killed by a signal (need exactly 101)".into()),
    }
    let got = normalize_stdout(&run.stdout);
    if got != IA_BEFORE {
        return Err(format!(
            "stdout {got:?} (need exactly {IA_BEFORE:?}: prefix flushed, no wrapped result)"
        ));
    }
    if !run.stderr.contains(phrase) {
        return Err(format!(
            "stderr lacks `{phrase}`: {}",
            run.stderr.chars().take(200).collect::<String>()
        ));
    }
    for other in IA_PHRASES.iter().chain(IA_OTHER_PANICS.iter()) {
        if *other != phrase && run.stderr.contains(other) {
            return Err(format!("stderr also carries `{other}` (other operation)"));
        }
    }
    Ok(())
}

/// Control run: exit 0, exact stdout, silent stderr.
fn ia_check_control(run: &IaRun) -> Result<(), String> {
    if run.code != Some(0) {
        return Err(format!("control exit {:?} (need 0)", run.code));
    }
    let got = normalize_stdout(&run.stdout);
    if got != IA_CONTROL_STDOUT {
        return Err(format!(
            "control stdout {got:?} (need {IA_CONTROL_STDOUT:?})"
        ));
    }
    if !run.stderr.trim().is_empty() {
        return Err(format!(
            "control stderr not empty: {}",
            run.stderr.chars().take(200).collect::<String>()
        ));
    }
    Ok(())
}

/// libtest names the tests `tests::<name>` (module `tests` of the emit); accept both forms.
fn ia_name_matches(shown: &str, name: &str) -> bool {
    shown == name || shown.ends_with(&format!("::{name}"))
}

/// True iff libtest reported `test <name> ... <status>`.
fn ia_test_line(stdout: &str, name: &str, status: &str) -> bool {
    let tail = format!(" ... {status}");
    stdout.lines().any(|l| {
        l.trim()
            .strip_prefix("test ")
            .and_then(|r| r.strip_suffix(tail.as_str()))
            .is_some_and(|shown| ia_name_matches(shown, name))
    })
}

/// The `---- <name> stdout ----` section of libtest's `failures:` report (up to the next section).
fn ia_failure_block<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let mut pos = 0usize;
    let mut begin: Option<usize> = None;
    for line in text.split_inclusive('\n') {
        let t = line.trim();
        if let Some(b) = begin {
            if t.starts_with("---- ") || t == "failures:" {
                return Some(&text[b..pos]);
            }
        } else if let Some(shown) = t
            .strip_prefix("---- ")
            .and_then(|r| r.strip_suffix(" stdout ----"))
        {
            if ia_name_matches(shown, name) {
                begin = Some(pos + line.len());
            }
        }
        pos += line.len();
    }
    begin.map(|b| &text[b..])
}

/// `arita test` of `05-test.arita` through the CLI process: exit 1, `tests failed (exit 101)` on
/// stderr, `ctl` ok, the three overflow tests FAILED each with the phrase of its own operation.
fn ia_check_test(run: &IaRun) -> Result<(), String> {
    if run.code != Some(1) {
        return Err(format!("arita test exit {:?} (need 1)", run.code));
    }
    if !run.stderr.lines().any(|l| l.trim() == IA_TEST_EXIT_LINE) {
        return Err(format!(
            "stderr lacks the line `{IA_TEST_EXIT_LINE}`: {}",
            run.stderr.chars().take(200).collect::<String>()
        ));
    }
    if normalize_stdout(&run.stdout).iter().any(|l| l == "PASS") {
        return Err("stdout reports PASS (the overflow tests must fail)".into());
    }
    if !ia_test_line(&run.stdout, IA_TEST_CTL, "ok") {
        return Err(format!("control test `{IA_TEST_CTL}` is not reported ok"));
    }
    for (name, phrase) in IA_TEST_OVF.iter() {
        if !ia_test_line(&run.stdout, name, "FAILED") {
            return Err(format!("test `{name}` is not reported FAILED"));
        }
        let Some(block) = ia_failure_block(&run.stdout, name) else {
            return Err(format!("no failure section for `{name}`"));
        };
        if !block.contains(phrase) {
            return Err(format!("failure of `{name}` lacks `{phrase}`"));
        }
        for other in IA_PHRASES.iter() {
            if other != phrase && block.contains(other) {
                return Err(format!("failure of `{name}` carries `{other}`"));
            }
        }
    }
    if !run.stdout.contains(IA_TEST_SUMMARY) {
        return Err(format!("stdout lacks `{IA_TEST_SUMMARY}`"));
    }
    Ok(())
}

/// True iff `[profile.release]` carries `overflow-checks = true` (structural, ADR-292 D2).
fn ia_toml_release_overflow_checks(toml: &str) -> bool {
    let mut in_release = false;
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
        if compact == "overflow-checks=true" {
            return true;
        }
    }
    false
}

fn ia_first_missing(root: &Path, rels: &[&str]) -> Option<String> {
    rels.iter()
        .map(|r| root.join(r))
        .find(|p| !p.is_file())
        .map(|p| p.display().to_string())
}

/// Every fixture file of IA-1/IA-2 (overflow ops + control).
fn ia_op_rels() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = IA_OPS.iter().map(|o| o.rel).collect();
    v.push(IA_CONTROL);
    v
}

/// Real build (profile) + bounded run. Returns the generated Cargo.toml (if any) and the run.
fn ia_build_run(
    full: &Path,
    profile: arita_codegen::BuildProfile,
    label: &str,
) -> Result<(Option<PathBuf>, IaRun), (Verdict, String)> {
    let out =
        crate::build_with_profile(full.to_str().unwrap_or_default(), profile).map_err(|e| {
            (
                Verdict::Rejected,
                format!(
                    "{label} build failed: {}",
                    e.chars().take(200).collect::<String>()
                ),
            )
        })?;
    if !out.bin.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!(
                "{label}: built binary not found at {} (cargo target dir redirected?)",
                out.bin.display()
            ),
        ));
    }
    let run = ia_run_bounded(&out.bin, &[], None, IA_RUN_SECS)
        .map_err(|(v, d)| (v, format!("{label}: {d}")))?;
    Ok((out.cargo_toml.clone(), run))
}

/// One leg for one fixture: expected panic (op fixtures) or control (exit 0).
fn ia_leg(
    root: &Path,
    rel: &str,
    profile: arita_codegen::BuildProfile,
    label: &str,
    phrase: Option<&str>,
    need_toml: bool,
) -> Result<Option<PathBuf>, (Verdict, String)> {
    let (toml, run) = ia_build_run(&root.join(rel), profile, label)?;
    if need_toml {
        let Some(tp) = toml.as_ref() else {
            return Err((
                Verdict::Rejected,
                format!("{rel}: {label} build produced no Cargo.toml"),
            ));
        };
        let text = fs::read_to_string(tp).map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("cannot read {}: {e}", tp.display()),
            )
        })?;
        if !ia_toml_release_overflow_checks(&text) {
            return Err((
                Verdict::Rejected,
                format!(
                    "{rel}: {label} Cargo.toml lacks `overflow-checks = true` under [profile.release]: {}",
                    text.chars().take(300).collect::<String>()
                ),
            ));
        }
    }
    let verdict = match phrase {
        Some(p) => ia_check_panic(p, &run),
        None => ia_check_control(&run),
    };
    verdict.map_err(|d| (Verdict::Rejected, format!("{rel}: {label} run: {d}")))?;
    Ok(toml)
}

/// All overflow ops + the control under one profile.
fn ia_profile_legs(
    root: &Path,
    id: &str,
    profile: fn() -> arita_codegen::BuildProfile,
    label: &str,
    need_toml: bool,
) -> Result<Vec<Option<PathBuf>>, OracleResult> {
    let mut tomls = Vec::new();
    for op in IA_OPS.iter() {
        match ia_leg(root, op.rel, profile(), label, Some(op.phrase), need_toml) {
            Ok(t) => tomls.push(t),
            Err((v, d)) => return Err(core09_vec_fail(id, op.rel, v, d)),
        }
    }
    match ia_leg(root, IA_CONTROL, profile(), label, None, need_toml) {
        Ok(t) => tomls.push(t),
        Err((v, d)) => return Err(core09_vec_fail(id, IA_CONTROL, v, d)),
    }
    Ok(tomls)
}

fn ia_debug() -> arita_codegen::BuildProfile {
    arita_codegen::BuildProfile::Debug
}

fn ia_release() -> arita_codegen::BuildProfile {
    arita_codegen::BuildProfile::Release
}

/// IA-1 — `arita build` (debug, rustc without `-O`): baseline contract, no Cargo.toml required.
fn core10_ia1_build(root: &Path) -> OracleResult {
    if let Some(m) = ia_first_missing(root, &ia_op_rels()) {
        return core09_vec_fail(
            IA1_ID,
            IA_DIR,
            Verdict::Inconclusive,
            format!("missing: {m}"),
        );
    }
    if let Err(r) = ia_profile_legs(root, IA1_ID, ia_debug, "debug", false) {
        return r;
    }
    core09_vec_fail(
        IA1_ID,
        IA_DIR,
        Verdict::Accepted,
        "debug: add/sub/mul overflow ⇒ exit 101 + `before` + the phrase of its own operation; boundary control exit 0 with intact stdout".into(),
    )
}

/// Canary: the emitted Rust built WITHOUT overflow checks must wrap (exit 0) and the very same
/// checker must reject that run; otherwise the oracle would be blind.
fn ia_canary(emit_rs: &Path, dir: &Path) -> Result<(), (Verdict, String)> {
    let emit = fs::read_to_string(emit_rs).map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("canary: cannot read emitted {}: {e}", emit_rs.display()),
        )
    })?;
    fs::create_dir_all(dir).map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("canary: mkdir {}: {e}", dir.display()),
        )
    })?;
    let src = dir.join("canary.rs");
    let bin = dir.join("canary_bin");
    fs::write(&src, emit).map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("canary: write {}: {e}", src.display()),
        )
    })?;
    let built = Command::new("rustc")
        .args(["--edition", "2021", "-O", "-C", "overflow-checks=off", "-o"])
        .arg(&bin)
        .arg(&src)
        .output()
        .map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("canary: rustc not available ({e}); never treated as accepted"),
            )
        })?;
    if !built.status.success() {
        return Err((
            Verdict::Inconclusive,
            format!(
                "canary: rustc failed: {}",
                String::from_utf8_lossy(&built.stderr)
                    .chars()
                    .take(200)
                    .collect::<String>()
            ),
        ));
    }
    let run = ia_run_bounded(&bin, &[], None, IA_RUN_SECS)?;
    if run.code != Some(0) {
        return Err((
            Verdict::Inconclusive,
            format!(
                "canary did not wrap (exit {:?}); the oracle cannot prove it discriminates",
                run.code
            ),
        ));
    }
    match ia_check_panic(IA_ADD, &run) {
        Err(_) => Ok(()),
        Ok(()) => Err((
            Verdict::Rejected,
            "canary: the checker ACCEPTED a wrapping run (exit 0); it cannot fail".into(),
        )),
    }
}

/// IA-2 — `arita build --profile release` (cargo): `[profile.release]` carries the flag and the
/// binary panics like debug; boundary control stays exit 0; canary proves the checker rejects wrap.
fn core10_ia2_release(root: &Path) -> OracleResult {
    if let Some(m) = ia_first_missing(root, &ia_op_rels()) {
        return core09_vec_fail(
            IA2_ID,
            IA_DIR,
            Verdict::Inconclusive,
            format!("missing: {m}"),
        );
    }
    let tomls = match ia_profile_legs(root, IA2_ID, ia_release, "release", true) {
        Ok(t) => t,
        Err(r) => return r,
    };
    let Some(Some(toml0)) = tomls.first() else {
        return core09_vec_fail(
            IA2_ID,
            IA_DIR,
            Verdict::Rejected,
            "release build produced no Cargo.toml".into(),
        );
    };
    let emit_rs = toml0
        .parent()
        .map(|d| d.join("src").join("main.rs"))
        .unwrap_or_default();
    let canary_dir = root.join(IA_SCRATCH).join("ia2");
    if let Err((v, d)) = ia_canary(&emit_rs, &canary_dir) {
        return core09_vec_fail(IA2_ID, IA_OPS[0].rel, v, d);
    }
    core09_vec_fail(
        IA2_ID,
        IA_DIR,
        Verdict::Accepted,
        "release (cargo): Cargo.toml has `overflow-checks = true`; add/sub/mul overflow ⇒ exit 101 + `before` + own phrase; boundary control exit 0; `rustc -O` canary wraps and the checker rejects it".into(),
    )
}

/// IA-3 — `arita test` through the CLI process (needs `arita measure`; else Inconclusive).
fn core10_ia3_test(root: &Path) -> OracleResult {
    if let Some(m) = ia_first_missing(root, &[IA_TEST_FIXTURE]) {
        return core09_vec_fail(
            IA3_ID,
            IA_TEST_FIXTURE,
            Verdict::Inconclusive,
            format!("missing: {m}"),
        );
    }
    let Some(exe) = pm_cli_exe() else {
        return core09_vec_fail(
            IA3_ID,
            IA_TEST_FIXTURE,
            Verdict::Inconclusive,
            "needs the `arita` CLI process (run via `arita measure`); skip ≠ PASS".into(),
        );
    };
    let scratch = std::env::temp_dir().join(format!("arita_ia3_{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    let prepared = fs::create_dir_all(&scratch)
        .and_then(|_| fs::copy(root.join(IA_TEST_FIXTURE), scratch.join("t.arita")));
    if let Err(e) = prepared {
        let _ = fs::remove_dir_all(&scratch);
        return core09_vec_fail(
            IA3_ID,
            IA_TEST_FIXTURE,
            Verdict::Inconclusive,
            format!("cannot stage fixture in {}: {e}", scratch.display()),
        );
    }
    let ran = ia_run_bounded(&exe, &["test", "t.arita"], Some(&scratch), IA_CLI_TEST_SECS);
    let _ = fs::remove_dir_all(&scratch);
    let run = match ran {
        Ok(r) => r,
        Err((v, d)) => return core09_vec_fail(IA3_ID, IA_TEST_FIXTURE, v, d),
    };
    match ia_check_test(&run) {
        Ok(()) => core09_vec_fail(
            IA3_ID,
            IA_TEST_FIXTURE,
            Verdict::Accepted,
            "arita test (rustc --test -O -C overflow-checks=on): `ctl` ok; ovf_add/ovf_sub/ovf_mul FAILED each with its own overflow phrase; `tests failed (exit 101)`; 1 passed; 3 failed".into(),
        ),
        Err(d) => core09_vec_fail(
            IA3_ID,
            IA_TEST_FIXTURE,
            Verdict::Rejected,
            format!("arita test: {d}"),
        ),
    }
}

/// IA-4 — package workspace, `--profile release`: the ROOT Cargo.toml carries the flag and the
/// member binary (executed directly) panics. The fixture is copied to a scratch dir.
fn core10_ia4_workspace_release(root: &Path) -> OracleResult {
    let rels: Vec<String> = IA_PKG_FILES
        .iter()
        .map(|f| format!("{IA_PKG_DIR}/{f}"))
        .collect();
    let rel_refs: Vec<&str> = rels.iter().map(|s| s.as_str()).collect();
    if let Some(m) = ia_first_missing(root, &rel_refs) {
        return core09_vec_fail(
            IA4_ID,
            IA_PKG_DIR,
            Verdict::Inconclusive,
            format!("missing: {m}"),
        );
    }
    let scratch = std::env::temp_dir().join(format!("arita_ia4_{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    if let Err(e) = pm_copy_tree(&root.join(IA_PKG_DIR), &scratch) {
        let _ = fs::remove_dir_all(&scratch);
        return core09_vec_fail(IA4_ID, IA_PKG_DIR, Verdict::Inconclusive, e);
    }
    let entry = scratch.join(IA_PKG_ENTRY);
    let result = ia_build_run(
        &entry,
        arita_codegen::BuildProfile::Release,
        "workspace release",
    )
    .and_then(|(toml, run)| {
        let Some(tp) = toml else {
            return Err((
                Verdict::Rejected,
                "workspace release build produced no root Cargo.toml".to_string(),
            ));
        };
        let text = fs::read_to_string(&tp).map_err(|e| {
            (
                Verdict::Inconclusive,
                format!("cannot read {}: {e}", tp.display()),
            )
        })?;
        if !ia_toml_release_overflow_checks(&text) {
            return Err((
                Verdict::Rejected,
                format!(
                    "root Cargo.toml lacks `overflow-checks = true` under [profile.release]: {}",
                    text.chars().take(300).collect::<String>()
                ),
            ));
        }
        ia_check_panic(IA_ADD, &run)
            .map_err(|d| (Verdict::Rejected, format!("member binary run: {d}")))
    });
    let _ = fs::remove_dir_all(&scratch);
    match result {
        Ok(()) => core09_vec_fail(
            IA4_ID,
            IA_PKG_DIR,
            Verdict::Accepted,
            "workspace release: root Cargo.toml has `[profile.release]` `overflow-checks = true`; member binary run directly ⇒ exit 101 + `before` + add phrase".into(),
        ),
        Err((v, d)) => core09_vec_fail(IA4_ID, IA_PKG_DIR, v, d),
    }
}

// ─── ADR-293 UNDECLARED-CALL (UC-1..UC-5) — CUT CORE-0.10-UNDECLARED-CALL-20261003 ───────────────
// A free call to a fn that is neither declared in the module, imported by `use`, nor a builtin is
// rejected in HIR (`eval_expr`, `Call` arm, after E0320/E0313/E0206/E0321) with the pinned text
// `E0347: call to undeclared function `NAME`` (no span: `Call` carries none, B-286-11 stays open).
// Pins (ADR-293 v0.1 + Ingeniero seal Q1/A1/A2): ids, fixtures, k = 5, N = 866 + 5 = 871. Probe =
// parse → lower → check in process (the rule runs before emit, D5): E0347 must be the ONLY `E\d{4}`
// code and the message EXACTLY the pinned text. Each neg also needs its single-offender control
// (the call replaced by its legal twin) to build with zero diagnostics, so a fixture with a second
// problem can never be accepted. UC-5 accepts only exit 0 + stdout exactly `1` + empty stderr.
// Dedicated runners: NOT in `NEG_ORACLES` / `EJEMPLO_ORACLES` (PU-1 stays at 398). The migrated
// `neg-core03-compose-mutex-hold` (D6) uses the same probe and does not count toward N.

/// Name of the undeclared callee in every UC fixture.
const UC_CALL: &str = "ghost";
const UC_OK_ID: &str = "core10-declared-call-ok";
const UC_OK_REL: &str = "ejemplos/core10/undeclared-call/05-declared-ok.arita";
/// Code lines of the positive that must each appear exactly once (declared twin of neg 02).
const UC_OK_SHAPE: [&str; 2] = ["fn ghost(n: Int) -> Int {", "let n: Int = ghost(1)"];

struct UcNeg {
    id: &'static str,
    rel: &'static str,
    /// The single offending code line (exactly once, comments excluded).
    offender: &'static str,
    /// Control: offender → legal twin; the result must build with zero diagnostics.
    edits: &'static [Core10CtlEdit],
}

const CORE10_UC_NEGS: &[UcNeg] = &[
    UcNeg {
        id: "neg-core10-undeclared-call-stmt",
        rel: "ejemplos/core10/undeclared-call/neg/01-stmt.arita",
        offender: "ghost(1)",
        edits: &[("ghost(1)", Some("print(1)"))],
    },
    UcNeg {
        id: "neg-core10-undeclared-call-let",
        rel: "ejemplos/core10/undeclared-call/neg/02-let.arita",
        offender: "let n: Int = ghost(1)",
        edits: &[("let n: Int = ghost(1)", Some("let n: Int = 1"))],
    },
    UcNeg {
        id: "neg-core10-undeclared-call-print-arg",
        rel: "ejemplos/core10/undeclared-call/neg/03-print-arg.arita",
        offender: "print(ghost(1))",
        edits: &[("print(ghost(1))", Some("print(1)"))],
    },
    UcNeg {
        id: "neg-core10-undeclared-call-in-result-fn",
        rel: "ejemplos/core10/undeclared-call/neg/04-in-result-fn.arita",
        offender: "let r: Int = ghost(n)",
        edits: &[("let r: Int = ghost(n)", Some("let r: Int = n"))],
    },
];

/// ADR-293 D1 — pinned diagnostic text for an undeclared free call (no span).
fn uc_diag_text(name: &str) -> String {
    format!("E0347: call to undeclared function `{name}`")
}

/// Pure checker: accepted only if E0347 is the ONLY `E\d{4}` code in `msg` and the whole message
/// is EXACTLY the pinned text for `name` (no span, no extra lines, no other code, no prefix).
fn uc_check_diag(msg: &str, name: &str) -> Result<(), String> {
    let codes = core09_diag_codes(msg);
    if codes.len() != 1 || codes[0] != "E0347" {
        return Err(format!(
            "expected only E0347, got codes={codes:?}: {}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    let want = uc_diag_text(name);
    if msg.trim() != want {
        return Err(format!(
            "E0347 text differs from the ADR-293 pin: want {want:?}, got {:?}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    Ok(())
}

/// ADR-293 D5 probe: parse → lower → check only (no rustc/cargo even if the rule regressed).
/// Missing file → inconclusive; the program being ACCEPTED → rejected; wrong code/text → rejected.
fn uc_probe(root: &Path, rel: &str, name: &str) -> Result<String, (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing oracle file: {}", full.display()),
        ));
    }
    let src = fs::read_to_string(&full)
        .map_err(|e| (Verdict::Inconclusive, format!("cannot read {rel}: {e}")))?;
    match crate::parse_lower_check(&src) {
        Ok(_) => Err((
            Verdict::Rejected,
            format!("{rel}: expected exact E0347 for `{name}`, but parse+lower+check succeeded"),
        )),
        Err(msg) => match uc_check_diag(&msg, name) {
            Ok(()) => Ok(msg),
            Err(d) => Err((Verdict::Rejected, format!("{rel}: {d}"))),
        },
    }
}

/// UC-1..UC-4: shape (offender exactly once) → single-offender control builds clean → exact E0347.
fn core10_uc_neg(root: &Path, spec: &UcNeg) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    let n = core10_code_line_count(&code, spec.offender);
    if n != 1 {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "fixture shape: code line {:?} appears {n}× (need exactly 1)",
                spec.offender
            ),
        );
    }
    let ctl_src = match core10_control_source(&src, spec.edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-uc-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    match uc_probe(root, rel, UC_CALL) {
        Ok(msg) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!("rejected with exact E0347: {msg}; {ctl_note}"),
        ),
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// UC-5 pure checker: exit exactly `Some(0)`, stdout exactly one line `1`, stderr empty.
fn uc_check_run(code: Option<i32>, stdout: &str, stderr: &str) -> Result<(), String> {
    if code != Some(0) {
        return Err(format!("expected exit 0, got {code:?}"));
    }
    let got = normalize_stdout(stdout);
    if got.len() != 1 || got[0] != "1" {
        return Err(format!("stdout mismatch got={got:?} expected=[\"1\"]"));
    }
    if !stderr.is_empty() {
        return Err(format!(
            "stderr must be empty, got: {}",
            stderr.chars().take(200).collect::<String>()
        ));
    }
    Ok(())
}

/// UC-5 `core10-declared-call-ok`: the declared twin of neg 02 builds clean and prints exactly `1`.
fn core10_uc_positive(root: &Path) -> OracleResult {
    let (id, rel) = (UC_OK_ID, UC_OK_REL);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable program {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in UC_OK_SHAPE.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let note = match core10_control_zero_diag(&full) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(b) => b,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("build failed: {}", e.chars().take(200).collect::<String>()),
            );
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            );
        }
    };
    match uc_check_run(
        run.status.code(),
        &String::from_utf8_lossy(&run.stdout),
        &String::from_utf8_lossy(&run.stderr),
    ) {
        Ok(()) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "declared call builds and runs: exit 0, exact stdout [\"1\"], empty stderr; {note}"
            ),
        ),
        Err(d) => core09_vec_fail(id, rel, Verdict::Rejected, d),
    }
}

// ─── ADR-294 / CUT CORE-0.10-KNOWN-INT-SCOPE-20261004 — KI-1..KI-7 ───
// Pins: DOC/ADR/294-core-known-int-scope-v0.md (Sello c8b95315…) §4: ids, fixtures, k = 7,
// N = 871 + 7 = 878. Dedicated runners: NOT in `EJEMPLO_ORACLES` / `NEG_ORACLES` (PU-1 stays at
// 398). KI-1..KI-5: valid programs that the old `known_int` leak rejected with E0217/E0216; they
// must pass the checker, build, run with exit 0 and print the exact stdout, with an EMPTY stderr
// of the RUN. The rustc stderr of the build is never read (Ingeniero: `unused_assignments`
// warnings in fp1/fp2/fp7 do not invalidate the positive; no `-D warnings`, no `#[allow]`).
// KI-6/KI-7: straight-line overflow keeps EXACTLY `E0217: integer overflow` (no span); in-process
// probe (parse → lower → check, ADR-293 A1) plus a control that builds with zero diagnostics.

/// ADR-294 §4: the pinned diagnostic of both negs (code and exact text, no span).
const KI_NEG_CODE: &str = "E0217";
const KI_NEG_TEXT: &str = "E0217: integer overflow";

struct KiPos {
    id: &'static str,
    rel: &'static str,
    /// Code lines that must each appear exactly once (comments excluded).
    shape: &'static [&'static str],
    /// Exact stdout of the run (as measured with rustc by the Parser, ADR-294 v0.2).
    want: &'static [&'static str],
}

const CORE10_KI_POS: &[KiPos] = &[
    KiPos {
        id: "core10-known-int-fp1-else-bleed",
        rel: "ejemplos/core10/known-int/fp1-else-bleed.arita",
        shape: &[
            "let mut a: Int = 0",
            "a = 9223372036854775807",
            "let r: Int = a + 1",
            "if n < 5 {",
        ],
        want: &["ok"],
    },
    KiPos {
        id: "core10-known-int-fp2-match-arm",
        rel: "ejemplos/core10/known-int/fp2-match-arm-bleed.arita",
        shape: &[
            "let mut a: Int = 0",
            "a = 9223372036854775807",
            "let r: Int = a + 1",
            "match flag {",
        ],
        want: &["ok"],
    },
    KiPos {
        id: "core10-known-int-fp3-loop-carried",
        rel: "ejemplos/core10/known-int/fp3-loop-carried.arita",
        shape: &[
            "let mut a: Int = 9223372036854775807",
            "let r: Int = a + 1",
            "a = 0",
            "while i < 2 {",
            "if i == 1 {",
        ],
        want: &["1", "ok"],
    },
    KiPos {
        id: "core10-known-int-fp4-shadow-leak",
        rel: "ejemplos/core10/known-int/fp4-shadow-leak.arita",
        shape: &[
            "let a: Int = 0",
            "let a: Int = 9223372036854775807",
            "let r: Int = a + 1",
        ],
        want: &["9223372036854775807", "1", "ok"],
    },
    KiPos {
        id: "core10-known-int-fp7-e0216-else",
        rel: "ejemplos/core10/known-int/fp7-e0216-else-bleed.arita",
        shape: &[
            "let mut d: Int = 1",
            "d = 0",
            "let q: Int = x.div_euclid(d)",
        ],
        want: &["ok"],
    },
];

struct KiNeg {
    id: &'static str,
    rel: &'static str,
    /// The single offending code line (exactly once, comments excluded).
    offender: &'static str,
    /// Further code lines that must each appear exactly once.
    shape: &'static [&'static str],
    /// Control: offender → legal twin; the result must build with zero diagnostics.
    edits: &'static [Core10CtlEdit],
}

const CORE10_KI_NEGS: &[KiNeg] = &[
    KiNeg {
        // c5 REUSES the existing f2 fixture (ADR-294 Q1/A5): never edited, never duplicated.
        id: "neg-core10-known-int-c5-straight",
        rel: "ejemplos/f2/neg/e0217-runtime-max-plus.arita",
        offender: "let x: Int = a + 1",
        shape: &["let a: Int = 9223372036854775807"],
        edits: &[
            ("let x: Int = a + 1", Some("let x: Int = a - 1")),
            ("print(\"ok\")", Some("print(x)")),
        ],
    },
    KiNeg {
        id: "neg-core10-known-int-c6-assign-overflow",
        rel: "ejemplos/core10/known-int/neg/c6-assign-overflow-expr.arita",
        offender: "a = a + 1",
        shape: &["let mut a: Int = 9223372036854775807"],
        edits: &[("a = a + 1", Some("a = a - 1"))],
    },
];

/// Pure checker: accepted only if `code` is the ONLY `E\d{4}` code in `msg` and the whole message
/// is EXACTLY `text` (no span, no extra lines, no other code, no prefix).
fn ki_check_diag(msg: &str, code: &str, text: &str) -> Result<(), String> {
    let codes = core09_diag_codes(msg);
    if codes.len() != 1 || codes[0] != code {
        return Err(format!(
            "expected only {code}, got codes={codes:?}: {}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    if msg.trim() != text {
        return Err(format!(
            "{code} text differs from the ADR-294 pin: want {text:?}, got {:?}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    Ok(())
}

/// Pure checker of a positive RUN: exit exactly `Some(0)`, no panic, EMPTY run stderr, stdout
/// exactly `want` (line for line, in order). The stderr of rustc is not an input.
fn ki_check_run(
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
    want: &[&str],
) -> Result<(), String> {
    if code != Some(0) {
        return Err(format!("expected exit 0, got {code:?}"));
    }
    if stderr.contains("panicked") {
        return Err(format!(
            "run panicked: {}",
            stderr.chars().take(200).collect::<String>()
        ));
    }
    if !stderr.is_empty() {
        return Err(format!(
            "run stderr must be empty, got: {}",
            stderr.chars().take(200).collect::<String>()
        ));
    }
    let got = normalize_stdout(stdout);
    let want_v: Vec<String> = want.iter().map(|s| (*s).to_string()).collect();
    if got != want_v {
        return Err(format!("stdout mismatch got={got:?} expected={want_v:?}"));
    }
    Ok(())
}

/// In-process gate of a VALID program: parse → lower → check must succeed. A diagnostic here is
/// the ADR-294 false positive coming back (E0217/E0216 on a program that never overflows).
fn ki_hir_gate(src: &str) -> Result<(), String> {
    match crate::parse_lower_check(src) {
        Ok(_) => Ok(()),
        Err(msg) => Err(format!(
            "HIR rejects a valid program (known_int false positive): {}",
            msg.chars().take(160).collect::<String>()
        )),
    }
}

/// ADR-294 probe of a neg: parse → lower → check only (no rustc even if the rule regressed).
/// Missing file → inconclusive; the program being ACCEPTED → rejected; wrong code/text → rejected.
fn ki_probe(root: &Path, rel: &str, code: &str, text: &str) -> Result<String, (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing oracle file: {}", full.display()),
        ));
    }
    let src = fs::read_to_string(&full)
        .map_err(|e| (Verdict::Inconclusive, format!("cannot read {rel}: {e}")))?;
    match crate::parse_lower_check(&src) {
        Ok(_) => Err((
            Verdict::Rejected,
            format!("{rel}: expected exact {code}, but parse+lower+check succeeded"),
        )),
        Err(msg) => match ki_check_diag(&msg, code, text) {
            Ok(()) => Ok(msg),
            Err(d) => Err((Verdict::Rejected, format!("{rel}: {d}"))),
        },
    }
}

/// KI-1..KI-5: shape → HIR gate → build → run → exact stdout, exit 0, empty run stderr.
fn core10_ki_positive(root: &Path, spec: &KiPos) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable program {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in spec.shape.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    if let Err(d) = ki_hir_gate(&src) {
        return core09_vec_fail(id, rel, Verdict::Rejected, d);
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(b) => b,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("build failed: {}", e.chars().take(200).collect::<String>()),
            );
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            );
        }
    };
    match ki_check_run(
        run.status.code(),
        &String::from_utf8_lossy(&run.stdout),
        &String::from_utf8_lossy(&run.stderr),
        spec.want,
    ) {
        Ok(()) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "valid program builds and runs: exit 0, exact stdout {:?}, empty run stderr (rustc stderr not inspected)",
                spec.want
            ),
        ),
        Err(d) => core09_vec_fail(id, rel, Verdict::Rejected, d),
    }
}

/// KI-6/KI-7: shape → single-offender control builds clean → exact `E0217: integer overflow`.
fn core10_ki_neg(root: &Path, spec: &KiNeg) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in std::iter::once(&spec.offender).chain(spec.shape.iter()) {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let ctl_src = match core10_control_source(&src, spec.edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-ki-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    match ki_probe(root, rel, KI_NEG_CODE, KI_NEG_TEXT) {
        Ok(msg) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!("rejected with exact {KI_NEG_TEXT}: {msg}; {ctl_note}"),
        ),
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

// ─── ADR-291 / CUT CORE-0.10-MUTEX-REJECT-20261003 — MX-1..MX-4 ───
// Pins: DOC/ADR/291-core-mutex-reject-v0.md (Sello, sha256 53e9ef94…) §2, §3, §5: ids, fixtures,
// k = 4, N = 878 + 4 = 882. Dedicated runners: NOT in `EJEMPLO_ORACLES` / `NEG_ORACLES` (PU-1
// stays at 398). Four negs: the reserved concurrency surface (`Mutex<..>` / `Arc<..>` as a type,
// `Mutex.new` / `Arc.new` as a constructor, `.lock()` as a selector) is rejected at PARSE time
// with the exact prefix below plus the mandatory ` @a..b` suffix (non-empty span INSIDE the
// source, sitting on the anchor word of ADR D1). The exact offsets are not pinned (Sello §2).
// Probe in process (parse → lower → check, ADR-293 A1): exit 1 / empty stdout / no emitted Rust
// are properties of the CLI; E0346 is a parse-phase error, so nothing after it runs. A control
// with the offender replaced by a legal twin must build with zero diagnostics.

/// ADR-291 §5: the pinned code and the exact text that precedes the mandatory ` @a..b` suffix.
const MX_CODE: &str = "E0346";
const MX_PREFIX: &str = "E0346: mutex concurrency is not available in this surface";

struct MxNeg {
    id: &'static str,
    rel: &'static str,
    /// The single offending code line (exactly once, comments excluded).
    offender: &'static str,
    /// The word the span must cover (ADR D1): the identifier or the `lock` selector.
    anchor: &'static str,
    /// Control: offender → legal twin (and, where a value is bound, `print("x")` → that value; the
    /// `arc` twin is `fn f() -> Io<()> {` because a non-main `Io<()>` fn takes no params, E0006); zero diagnostics.
    edits: &'static [Core10CtlEdit],
}

const CORE10_MX_NEGS: &[MxNeg] = &[
    MxNeg {
        id: "neg-core10-mutex-type",
        rel: "ejemplos/core10/mutex-reject/01-type.arita",
        offender: "let m: Mutex<Int> = 0",
        anchor: "Mutex",
        edits: &[
            ("let m: Mutex<Int> = 0", Some("let m: Int = 0")),
            ("print(\"x\")", Some("print(m)")),
        ],
    },
    MxNeg {
        id: "neg-core10-mutex-arc",
        rel: "ejemplos/core10/mutex-reject/02-arc.arita",
        offender: "fn f(a: Arc<Mutex<Int>>) -> Io<()> {",
        anchor: "Arc",
        edits: &[(
            "fn f(a: Arc<Mutex<Int>>) -> Io<()> {",
            Some("fn f() -> Io<()> {"),
        )],
    },
    MxNeg {
        id: "neg-core10-mutex-constructor",
        rel: "ejemplos/core10/mutex-reject/03-constructor.arita",
        offender: "let m: Int = Mutex.new(0)",
        anchor: "Mutex",
        edits: &[
            ("let m: Int = Mutex.new(0)", Some("let m: Int = 0")),
            ("print(\"x\")", Some("print(m)")),
        ],
    },
    MxNeg {
        id: "neg-core10-mutex-lock",
        rel: "ejemplos/core10/mutex-reject/04-lock.arita",
        offender: "let g: Int = n.lock()",
        anchor: "lock",
        edits: &[
            ("let g: Int = n.lock()", Some("let g: Int = n")),
            ("print(\"x\")", Some("print(g)")),
        ],
    },
];

/// Pure checker: accepted only if `E0346` is the ONLY `E\d{4}` code in `msg`, the message is
/// EXACTLY `MX_PREFIX` + ` @a..b` (digits only, nothing else), the span is non-empty and INSIDE
/// `src`, and `src[a..b]` is the anchor word. Returns the span.
fn mx_check_diag(msg: &str, src: &str, anchor: &str) -> Result<(usize, usize), String> {
    let codes = core09_diag_codes(msg);
    if codes.len() != 1 || codes[0] != MX_CODE {
        return Err(format!(
            "expected only {MX_CODE}, got codes={codes:?}: {}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    let Some(rest) = msg.trim().strip_prefix(MX_PREFIX) else {
        return Err(format!(
            "{MX_CODE} prefix differs from the ADR-291 pin: want {MX_PREFIX:?}, got {:?}",
            msg.chars().take(160).collect::<String>()
        ));
    };
    let span = rest
        .strip_prefix(" @")
        .and_then(|t| t.split_once(".."))
        .filter(|(a, b)| {
            !a.is_empty()
                && !b.is_empty()
                && a.bytes().all(|c| c.is_ascii_digit())
                && b.bytes().all(|c| c.is_ascii_digit())
        })
        .and_then(|(a, b)| Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?)));
    let Some((a, b)) = span else {
        return Err(format!(
            "{MX_CODE} needs the exact suffix ` @a..b` after the prefix, got {rest:?}"
        ));
    };
    if a >= b || b > src.len() {
        return Err(format!(
            "span @{a}..{b} must be non-empty and inside the source ({} bytes)",
            src.len()
        ));
    }
    if &src.as_bytes()[a..b] != anchor.as_bytes() {
        return Err(format!(
            "span @{a}..{b} does not sit on the anchor `{anchor}` (ADR-291 D1)"
        ));
    }
    Ok((a, b))
}

/// ADR-291 probe of a neg: parse → lower → check only (no rustc even if the rule regressed).
/// Missing file → inconclusive; the program being ACCEPTED → rejected; wrong text/span → rejected.
fn mx_probe(root: &Path, rel: &str, anchor: &str) -> Result<(usize, usize), (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing oracle file: {}", full.display()),
        ));
    }
    let src = fs::read_to_string(&full)
        .map_err(|e| (Verdict::Inconclusive, format!("cannot read {rel}: {e}")))?;
    match crate::parse_lower_check(&src) {
        Ok(_) => Err((
            Verdict::Rejected,
            format!("{rel}: expected exact {MX_CODE}, but parse+lower+check succeeded"),
        )),
        Err(msg) => match mx_check_diag(&msg, &src, anchor) {
            Ok(span) => Ok(span),
            Err(d) => Err((Verdict::Rejected, format!("{rel}: {d}"))),
        },
    }
}

/// MX-1..MX-4: shape → single-offender control builds clean → exact E0346 prefix + span on the
/// anchor.
fn core10_mx_neg(root: &Path, spec: &MxNeg) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    let n = core10_code_line_count(&code, spec.offender);
    if n != 1 {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Rejected,
            format!(
                "fixture shape: code line {:?} appears {n}× (need exactly 1)",
                spec.offender
            ),
        );
    }
    let ctl_src = match core10_control_source(&src, spec.edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-mx-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    match mx_probe(root, rel, spec.anchor) {
        Ok((a, b)) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "rejected with exact {MX_PREFIX} @{a}..{b} on `{}`; {ctl_note}",
                spec.anchor
            ),
        ),
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

// ─── ADR-295 / CUT CORE-0.10-MUST-USE-20260927 — MS-1..MS-7 (S2 MUST-USE, E0272) ───
// Pins: DOC/ADR/295-core-must-use-v0.md (Sello, sha256 b12cb4be…) §4: ids, fixtures, k = 7,
// N = 882 + 7 = 889 (confirmed by the Ingeniero at GO IMPL, 2026-10-04). Dedicated runners: NOT in
// `EJEMPLO_ORACLES` / `NEG_ORACLES` (PU-1 stays at 398). MS-1..MS-3: the `Result` discard forms
// P2(a)/(b)/(c) keep EXACTLY `E0272: result error swallowed` (no span); in-process probe (parse →
// lower → check, ADR-293 A1) plus a control that builds with zero diagnostics (every control
// obeys E0006: no twin of a non-main fn changes its params). MS-4..MS-6: valid programs that use
// the `Result` (`?`, `match r`, `match _x`); they build, run with exit 0 and print the exact
// stdout, with an EMPTY stderr of the RUN (the rustc stderr of the build is never read). MS-7:
// emit-ban over a FIXED list of six sources (ADR-295 D7): no `let _ =` except E1 `put` and E2
// `join`, and no `.unwrap()` / `.expect(` / `panic!` + `(` in the emitted Rust.

/// ADR-295 §4: the pinned diagnostic of the three negs (code and exact text, no span).
const MS_NEG_CODE: &str = "E0272";
const MS_NEG_TEXT: &str = "E0272: result error swallowed";

/// Control twin of a discarded call: the `Result` is consumed by a `match`.
const MS_MATCH_F1: &str =
    "match f(1) {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }";

struct MsNeg {
    id: &'static str,
    rel: &'static str,
    /// The single offending code line (exactly once, comments excluded).
    offender: &'static str,
    /// Further code lines that must each appear exactly once.
    shape: &'static [&'static str],
    /// Control: offender → legal twin; the result must build with zero diagnostics.
    edits: &'static [Core10CtlEdit],
}

const MS_NEG_SHAPE: &[&str] = &[
    "fn f(x: Int) -> Result<Int, Int> {",
    "fn main() -> Io<()> {",
    "print(\"after\")",
];

const CORE10_MS_NEGS: &[MsNeg] = &[
    MsNeg {
        id: "neg-core10-let-underscore",
        rel: "ejemplos/core10/errores-neg/05-let-underscore.arita",
        offender: "let _: Result<Int, Int> = f(1)",
        shape: MS_NEG_SHAPE,
        edits: &[("let _: Result<Int, Int> = f(1)", Some(MS_MATCH_F1))],
    },
    MsNeg {
        id: "neg-core10-let-named-dead",
        rel: "ejemplos/core10/errores-neg/06-let-named-dead.arita",
        offender: "let _r: Result<Int, Int> = f(1)",
        shape: MS_NEG_SHAPE,
        edits: &[(
            "let _r: Result<Int, Int> = f(1)",
            Some(
                "let _r: Result<Int, Int> = f(1)\n  match _r {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }",
            ),
        )],
    },
    MsNeg {
        id: "neg-core10-bare-stmt",
        rel: "ejemplos/core10/errores-neg/07-bare-stmt.arita",
        offender: "f(1)",
        shape: MS_NEG_SHAPE,
        edits: &[("f(1)", Some(MS_MATCH_F1))],
    },
];

struct MsPos {
    id: &'static str,
    rel: &'static str,
    /// Code lines that must each appear exactly once (comments excluded).
    shape: &'static [&'static str],
    /// Exact stdout of the run (measured by the Parser with a real `arita build`: 1, 1, 2; this
    /// runner has not been executed by Measure yet).
    want: &'static [&'static str],
}

const CORE10_MS_POS: &[MsPos] = &[
    MsPos {
        id: "core10-qmark-use",
        rel: "ejemplos/core10/errores/03-qmark-use.arita",
        shape: &["let v: Int = f(1)?", "fn g() -> Result<Int, Int> {"],
        want: &["1"],
    },
    MsPos {
        id: "core10-let-used",
        rel: "ejemplos/core10/errores/04-let-used.arita",
        shape: &["let r: Result<Int, Int> = f(1)", "match r {"],
        want: &["1"],
    },
    MsPos {
        id: "core10-underscore-read",
        rel: "ejemplos/core10/errores/05-underscore-read.arita",
        shape: &["let _x: Result<Int, Int> = f(2)", "match _x {"],
        want: &["2"],
    },
];

/// MS-7: the FIXED list of emitted sources (ADR-295 §4 oracle 7; Sello 2: six, not extended).
const MS_EMIT_ID: &str = "core10-must-use-emit-ban";
const MS_EMIT_SOURCES: [&str; 6] = [
    "ejemplos/core10/errores/03-qmark-use.arita",
    "ejemplos/core10/errores/04-let-used.arita",
    "ejemplos/core10/errores/05-underscore-read.arita",
    "ejemplos/core10/errores/01-err-mapped.arita",
    "ejemplos/core10/errores/02-err-used.arita",
    "ejemplos/core10/join-safety/pos/01-spawn-iounit.arita",
];

/// Pure checker: accepted only if `E0272` is the ONLY `E\d{4}` code in `msg` and the whole message
/// is EXACTLY `MS_NEG_TEXT` (no span, no extra lines, no other code, no prefix).
fn ms_check_diag(msg: &str) -> Result<(), String> {
    let codes = core09_diag_codes(msg);
    if codes.len() != 1 || codes[0] != MS_NEG_CODE {
        return Err(format!(
            "expected only {MS_NEG_CODE}, got codes={codes:?}: {}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    if msg.trim() != MS_NEG_TEXT {
        return Err(format!(
            "{MS_NEG_CODE} text differs from the ADR-295 pin: want {MS_NEG_TEXT:?}, got {:?}",
            msg.chars().take(160).collect::<String>()
        ));
    }
    Ok(())
}

/// In-process gate of a VALID program: parse → lower → check must succeed. A diagnostic here is
/// the ADR-295 false positive (E0272 on a `Result` that IS used).
fn ms_hir_gate(src: &str) -> Result<(), String> {
    match crate::parse_lower_check(src) {
        Ok(_) => Ok(()),
        Err(msg) => Err(format!(
            "HIR rejects a valid program (must-use false positive): {}",
            msg.chars().take(160).collect::<String>()
        )),
    }
}

/// ADR-295 probe of a neg: parse → lower → check only (no rustc even if the rule regressed).
/// Missing file → inconclusive; the program being ACCEPTED → rejected; wrong code/text → rejected.
fn ms_probe(root: &Path, rel: &str) -> Result<String, (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing oracle file: {}", full.display()),
        ));
    }
    let src = fs::read_to_string(&full)
        .map_err(|e| (Verdict::Inconclusive, format!("cannot read {rel}: {e}")))?;
    match crate::parse_lower_check(&src) {
        Ok(_) => Err((
            Verdict::Rejected,
            format!("{rel}: expected exact {MS_NEG_CODE}, but parse+lower+check succeeded"),
        )),
        Err(msg) => match ms_check_diag(&msg) {
            Ok(()) => Ok(msg),
            Err(d) => Err((Verdict::Rejected, format!("{rel}: {d}"))),
        },
    }
}

/// MS-1..MS-3: shape → single-offender control builds clean → exact `E0272: result error swallowed`.
fn core10_ms_neg(root: &Path, spec: &MsNeg) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable fixture {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in std::iter::once(&spec.offender).chain(spec.shape.iter()) {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("fixture shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    let ctl_src = match core10_control_source(&src, spec.edits) {
        Ok(s) => s,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, format!("control: {e}")),
    };
    let ctl_dir = root.join("target").join("arita-core10-ctl");
    if let Err(e) = fs::create_dir_all(&ctl_dir) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("control dir {}: {e}", ctl_dir.display()),
        );
    }
    let fname = Path::new(rel)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ctl.arita".into());
    let ctl_path = ctl_dir.join(format!("ctl-ms-{fname}"));
    if let Err(e) = fs::write(&ctl_path, &ctl_src) {
        return core09_vec_fail(
            id,
            rel,
            Verdict::Inconclusive,
            format!("write control: {e}"),
        );
    }
    let ctl_note = match core10_control_zero_diag(&ctl_path) {
        Ok(n) => n,
        Err(e) => return core09_vec_fail(id, rel, Verdict::Rejected, e),
    };
    match ms_probe(root, rel) {
        Ok(msg) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!("rejected with exact {MS_NEG_TEXT}: {msg}; {ctl_note}"),
        ),
        Err((verdict, detail)) => core09_vec_fail(id, rel, verdict, detail),
    }
}

/// MS-4..MS-6: shape → HIR gate → build → run → exact stdout, exit 0, empty run stderr.
fn core10_ms_positive(root: &Path, spec: &MsPos) -> OracleResult {
    let (id, rel) = (spec.id, spec.rel);
    let full = root.join(rel);
    let src = match fs::read_to_string(&full) {
        Ok(s) => s,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("missing/unreadable program {}: {e}", full.display()),
            );
        }
    };
    let code = core09_ref_mut_code(&src);
    for l in spec.shape.iter() {
        let n = core10_code_line_count(&code, l);
        if n != 1 {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("program shape: code line {l:?} appears {n}× (need exactly 1)"),
            );
        }
    }
    if let Err(d) = ms_hir_gate(&src) {
        return core09_vec_fail(id, rel, Verdict::Rejected, d);
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(b) => b,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Rejected,
                format!("build failed: {}", e.chars().take(200).collect::<String>()),
            );
        }
    };
    let run = match Command::new(&bin).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            return core09_vec_fail(
                id,
                rel,
                Verdict::Inconclusive,
                format!("cannot spawn {}: {e}", bin.display()),
            );
        }
    };
    match ki_check_run(
        run.status.code(),
        &String::from_utf8_lossy(&run.stdout),
        &String::from_utf8_lossy(&run.stderr),
        spec.want,
    ) {
        Ok(()) => core09_vec_fail(
            id,
            rel,
            Verdict::Accepted,
            format!(
                "valid program builds and runs: exit 0, exact stdout {:?}, empty run stderr (rustc stderr not inspected)",
                spec.want
            ),
        ),
        Err(d) => core09_vec_fail(id, rel, Verdict::Rejected, d),
    }
}

/// Pure checker of the EMITTED Rust (ADR-295 D7): every `let _` must sit inside exactly one of
/// the two fixed exclusions — E1 `{ let _ = <recv>.insert(<k>, <v>); }` (`put`) or E2
/// `{ let _ = <expr>.await; }` (`join`, `<expr>` = identifier or `tokio::spawn(..)`) — and the
/// text must contain no `.unwrap()`, `.expect(` or `panic!` + `(` (`.unwrap_or(` does not count).
/// A `;` after the closing brace (`{ let _ = …; };`, the real emitted shape) is tolerated, and the
/// absence of E2/E1 is NOT an error (Ingeniero, A-M3: only what appears must be exactly E1/E2).
/// Returns the number of (E1, E2) blocks seen.
fn ms_emit_check(emit: &str) -> Result<(usize, usize), String> {
    let (mut e1, mut e2) = (0usize, 0usize);
    let mut from = 0;
    while let Some(off) = emit[from..].find("let _") {
        let p = from + off;
        from = p + "let _".len();
        let next = emit[from..].chars().next();
        let prev = emit[..p].chars().next_back();
        if next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            || prev.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            continue;
        }
        let snippet: String = emit[p..].chars().take(80).collect();
        if !emit[..p].trim_end().ends_with('{') {
            return Err(format!(
                "vetoed `let _` outside an E1/E2 block: {snippet:?}"
            ));
        }
        let Some(semi) = emit[p..].find(';') else {
            return Err(format!("vetoed `let _` without `;`: {snippet:?}"));
        };
        let stmt = &emit[p..p + semi];
        if !emit[p + semi + 1..].trim_start().starts_with('}') {
            return Err(format!("vetoed `let _` (block not closed): {snippet:?}"));
        }
        let Some(body) = stmt.strip_prefix("let _ = ").map(str::trim) else {
            return Err(format!("vetoed `let _` (not `let _ = `): {snippet:?}"));
        };
        if body.find(".insert(").is_some_and(|i| i > 0) && body.ends_with(')') {
            e1 += 1;
            continue;
        }
        if let Some(expr) = body.strip_suffix(".await") {
            let ident =
                !expr.is_empty() && expr.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_');
            let spawn = expr.starts_with("tokio::spawn(") && expr.ends_with(')');
            if ident || spawn {
                e2 += 1;
                continue;
            }
        }
        return Err(format!("vetoed `let _` (not E1/E2): {snippet:?}"));
    }
    for pat in [".unwrap()", ".expect(", concat!("panic!", "(")] {
        if let Some(i) = emit.find(pat) {
            let around: String = emit[i..].chars().take(60).collect();
            return Err(format!(
                "vetoed text {pat:?} in the emitted Rust: {around:?}"
            ));
        }
    }
    Ok((e1, e2))
}

/// Candidate paths of the emitted Rust of the binary `bin`: `<bin>.rs` (plain build) and, for a
/// Cargo project (async / tokio programs), `<project>/src/main.rs` with
/// `<project>` = `bin.parent()` ×3 (bin = `<project>/target/debug/<name>`). Pure: no I/O.
fn ms_emit_candidates(bin: &Path) -> Vec<PathBuf> {
    let mut out = vec![PathBuf::from(format!("{}.rs", bin.display()))];
    let project = bin.parent().and_then(Path::parent).and_then(Path::parent);
    if let Some(p) = project {
        out.push(p.join("src").join("main.rs"));
    }
    out
}

/// Reads the emitted Rust of `bin` from the first existing candidate of `ms_emit_candidates`.
/// No candidate exists → explicit error naming every path tried (never an empty string).
fn ms_read_emitted_rust(bin: &Path) -> Result<String, String> {
    let cands = ms_emit_candidates(bin);
    for c in &cands {
        if c.is_file() {
            return fs::read_to_string(c).map_err(|e| format!("read emit {}: {e}", c.display()));
        }
    }
    let tried: Vec<String> = cands.iter().map(|c| c.display().to_string()).collect();
    Err(format!(
        "read emit: no such file; tried {}",
        tried.join(" and ")
    ))
}

/// Build + run one program (same steps as `core09_build_run_emit`, which is shared and untouched,
/// but it only looks for `<bin>.rs` and fails for Cargo-emitted async programs before returning):
/// returns the emitted Rust resolved by `ms_read_emitted_rust`.
fn ms_build_run_emit(root: &Path, rel: &str) -> Result<String, (Verdict, String)> {
    let full = root.join(rel);
    if !full.is_file() {
        return Err((
            Verdict::Inconclusive,
            format!("missing: {}", full.display()),
        ));
    }
    let bin = build(full.to_str().unwrap_or(rel)).map_err(|e| {
        (
            Verdict::Rejected,
            format!(
                "{rel}: build failed: {}",
                e.chars().take(200).collect::<String>()
            ),
        )
    })?;
    let emit =
        ms_read_emitted_rust(&bin).map_err(|e| (Verdict::Rejected, format!("{rel}: {e}")))?;
    let run = Command::new(&bin).output().map_err(|e| {
        (
            Verdict::Inconclusive,
            format!("{rel}: cannot spawn {}: {e}", bin.display()),
        )
    })?;
    if !run.status.success() {
        return Err((
            Verdict::Rejected,
            format!("{rel}: non-zero exit {}", run.status.code().unwrap_or(-1)),
        ));
    }
    Ok(emit)
}

/// MS-7: build + run the six fixed sources and judge their EMITTED Rust with `ms_emit_check`.
fn core10_ms_emit_ban(root: &Path) -> OracleResult {
    for rel in MS_EMIT_SOURCES.iter() {
        let full = root.join(rel);
        if !full.is_file() {
            return core09_vec_fail(
                MS_EMIT_ID,
                rel,
                Verdict::Inconclusive,
                format!("missing: {}", full.display()),
            );
        }
    }
    let (mut e1, mut e2) = (0usize, 0usize);
    for rel in MS_EMIT_SOURCES.iter() {
        let emit = match ms_build_run_emit(root, rel) {
            Ok(emit) => emit,
            Err((verdict, detail)) => return core09_vec_fail(MS_EMIT_ID, rel, verdict, detail),
        };
        match ms_emit_check(&emit) {
            Ok((a, b)) => {
                e1 += a;
                e2 += b;
            }
            Err(d) => {
                return core09_vec_fail(MS_EMIT_ID, rel, Verdict::Rejected, format!("{rel}: {d}"))
            }
        }
    }
    core09_vec_fail(
        MS_EMIT_ID,
        "ejemplos/core10/errores",
        Verdict::Accepted,
        format!(
            "{} fixed sources: 0 vetoed `let _ =` (only E1 `put` x{e1} and E2 `join` x{e2}); 0 `.unwrap()` / `.expect(` / `panic!` + `(` in the emitted Rust",
            MS_EMIT_SOURCES.len()
        ),
    )
}

// ─── ADR-287 / CUT CORE-PKG-MEMBER-20261001 — `arita build <input>` only compiles workspace members ───
// Pins: DOC/ADR/287-core-package-member-v0.md (v0.3, sha256 b80cec57…) §5 «Oráculos measure»
// (k = 7) and D1–D5: PM-1 `bin/main.arita` → Ok, `ok:` names a `bin_*` file, exact member stdout ·
// PM-2 `bin/other.arita` → exit 1, stdout empty, stderr `E0332:` · PM-3 `./bin/../bin/main.arita`
// AND the plain relative `bin/main.arita` (cwd = scratch root) → same as PM-1 · PM-4
// `bin/other-record.arita` / PM-5 `bin/other-edge.arita` → as PM-2 · PM-6 missing input → exit 1,
// stdout empty, I/O stderr WITHOUT E0332 · PM-7 `lib/lib.arita` → exit 0, `ok: <…/target/<profile>>`
// with no `bin_*` component, stderr empty. N = 844 + 7 = 851 (provisional; the Ingeniero fixes it
// at GO IMPL). Exact code only (no spans). Every oracle runs the real `arita build` PROCESS
// (exit / stdout / stderr kept apart, ADR §5 last paragraph) on a scratch copy of the fixture in
// the system tmpdir (unique per id + pid, removed at the end, also on failure paths). The repo
// tree must be identical before and after: the fixture (sha256 over every file, README.md
// included) and `target/arita-pkg` (metadata digest). The 7 ids give a real verdict; the only
// Inconclusive paths are «not running as the `arita` process» (cargo test) and tool/IO failures:
// skip ≠ PASS. No gating flag (decision Ingeniero 02-10).

const PM_FIXTURE: &str = "ejemplos/core10/pkg-member";
const PM_FIXTURE_FILES: [&str; 7] = [
    "README.md",
    "arita.toml",
    "lib/lib.arita",
    "bin/main.arita",
    "bin/other.arita",
    "bin/other-record.arita",
    "bin/other-edge.arita",
];
/// Exact stdout of `bin/main.arita` (`print("member-main:")` then `print(add(20, 22))`).
const PM_MEMBER_STDOUT: [&str; 2] = ["member-main:", "42"];

#[derive(Clone, Copy)]
enum PmExpect {
    /// PM-1 / PM-3: Ok, `ok:` → `bin_*` file, run it → exact member stdout.
    MemberRun,
    /// PM-2 / PM-4 / PM-5: exit 1, stdout empty, stderr exactly E0332, nothing mutated.
    NonMemberE0332,
    /// PM-6: exit 1, stdout empty, I/O stderr without E0332, nothing mutated.
    MissingIo,
    /// PM-7: exit 0, `ok: <…/target/<profile>>`, no `bin_*` component, stderr empty.
    LibDir,
}

/// (id, inputs, expectation). Each input is `(path relative to the fixture root, run with that
/// RELATIVE input and cwd = scratch root)`; otherwise the absolute scratch path is passed. PM-3
/// carries two forms (`./bin/../bin/main.arita` and plain `bin/main.arita`), same id.
type PmOracle = (&'static str, &'static [(&'static str, bool)], PmExpect);

const PM_ORACLES: [PmOracle; 7] = [
    (
        "core10-pkg-member-bin-ok",
        &[("bin/main.arita", false)],
        PmExpect::MemberRun,
    ),
    (
        "neg-core10-pkg-member-other",
        &[("bin/other.arita", false)],
        PmExpect::NonMemberE0332,
    ),
    (
        "core10-pkg-member-dotdot",
        &[("./bin/../bin/main.arita", true), ("bin/main.arita", true)],
        PmExpect::MemberRun,
    ),
    (
        "neg-core10-pkg-member-other-record",
        &[("bin/other-record.arita", false)],
        PmExpect::NonMemberE0332,
    ),
    (
        "neg-core10-pkg-member-other-edge",
        &[("bin/other-edge.arita", false)],
        PmExpect::NonMemberE0332,
    ),
    (
        "neg-core10-pkg-member-missing-input",
        &[("bin/does-not-exist.arita", false)],
        PmExpect::MissingIo,
    ),
    (
        "core10-pkg-member-lib",
        &[("lib/lib.arita", false)],
        PmExpect::LibDir,
    ),
];

/// One `arita build <input>` process: exit code, stdout and stderr kept apart.
struct PmRun {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

struct PmCtx {
    label: String,
    fixture: PathBuf,
    scratch: PathBuf,
    scratch_before: String,
    repo_before: String,
    pkg_dir: PathBuf,
    pkg_before: String,
}

fn pm_head(s: &str) -> String {
    s.chars().take(160).collect()
}

/// The `arita` binary itself (measure running as the CLI); `None` under cargo test.
fn pm_cli_exe() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .filter(|p| p.file_name().is_some_and(|n| n == "arita"))
}

fn pm_walk(
    root: &Path,
    dir: &Path,
    skip_target: bool,
    out: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| format!("read_dir entry {}: {e}", dir.display()))?;
        let p = entry.path();
        let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
        if p.is_dir() {
            if skip_target && dir == root && entry.file_name() == "target" {
                continue;
            }
            out.push((format!("{rel}/"), Vec::new()));
            pm_walk(root, &p, skip_target, out)?;
        } else {
            let bytes = fs::read(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
            out.push((rel, bytes));
        }
    }
    Ok(())
}

/// sha256 over every file (path + bytes) and directory under `dir`; `skip_target` ignores a
/// top-level `target/`.
fn pm_digest(dir: &Path, skip_target: bool) -> Result<String, String> {
    let mut files = Vec::new();
    pm_walk(dir, dir, skip_target, &mut files)?;
    files.sort();
    let mut buf: Vec<u8> = Vec::new();
    for (rel, bytes) in &files {
        buf.extend_from_slice(rel.as_bytes());
        buf.push(0);
        buf.extend_from_slice(bytes.len().to_string().as_bytes());
        buf.push(0);
        buf.extend_from_slice(bytes);
    }
    Ok(crate::attest::sha256_hex(&buf))
}

/// Metadata digest (path, kind, length, mtime) of a tree that may be large and is only compared
/// for identity (`target/arita-pkg`); `absent` when it does not exist. Does not follow symlinks.
fn pm_meta_digest(dir: &Path) -> Result<String, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let rd = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for entry in rd {
            let entry = entry.map_err(|e| format!("read_dir entry {}: {e}", dir.display()))?;
            let p = entry.path();
            let md = fs::symlink_metadata(&p).map_err(|e| format!("stat {}: {e}", p.display()))?;
            let mtime = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
            out.push(format!("{rel}\t{}\t{}\t{mtime}", md.is_dir(), md.len()));
            if md.is_dir() {
                walk(root, &p, out)?;
            }
        }
        Ok(())
    }
    if !dir.exists() {
        return Ok("absent".into());
    }
    let mut rows = Vec::new();
    walk(dir, dir, &mut rows)?;
    rows.sort();
    Ok(crate::attest::sha256_hex(rows.join("\n").as_bytes()))
}

fn pm_copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
    let rd = fs::read_dir(from).map_err(|e| format!("read_dir {}: {e}", from.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| format!("read_dir entry {}: {e}", from.display()))?;
        let p = entry.path();
        let dst = to.join(entry.file_name());
        if p.is_dir() {
            if entry.file_name() == "target" {
                continue;
            }
            pm_copy_tree(&p, &dst)?;
        } else {
            fs::copy(&p, &dst).map_err(|e| format!("copy {}: {e}", p.display()))?;
        }
    }
    Ok(())
}

fn pm_run_cli(exe: &Path, cwd: &Path, input: &str) -> Result<PmRun, String> {
    // The workspace build must land under the scratch `target/arita-pkg`: drop any env that
    // redirects cargo's target dir (Codegen fragility b).
    let out = Command::new(exe)
        .current_dir(cwd)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .arg("build")
        .arg(input)
        .output()
        .map_err(|e| format!("cannot spawn {} build: {e}", exe.display()))?;
    Ok(PmRun {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// Exit 0 and stdout exactly one `ok: <path>` line → that path (relative paths resolved against
/// the scratch dir, the CLI's cwd).
fn pm_ok_path(run: &PmRun, scratch: &Path) -> Result<PathBuf, (Verdict, String)> {
    if run.code != Some(0) {
        return Err((
            Verdict::Rejected,
            format!(
                "exit {:?} (need 0) stderr={}",
                run.code,
                pm_head(&run.stderr)
            ),
        ));
    }
    let lines = normalize_stdout(&run.stdout);
    let [line] = lines.as_slice() else {
        return Err((
            Verdict::Rejected,
            format!("stdout {lines:?}: need exactly one `ok: <path>` line"),
        ));
    };
    let Some(p) = line.strip_prefix("ok: ") else {
        return Err((
            Verdict::Rejected,
            format!("stdout line {line:?} does not start with `ok: `"),
        ));
    };
    let p = PathBuf::from(p);
    Ok(if p.is_absolute() { p } else { scratch.join(p) })
}

/// Expectation check of one build process (ADR-287 §5). `Err((verdict, detail))`.
fn pm_check(expect: PmExpect, scratch: &Path, run: &PmRun) -> Result<String, (Verdict, String)> {
    match expect {
        PmExpect::MemberRun => {
            let bin = pm_ok_path(run, scratch)?;
            let name = bin
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !name.starts_with("bin_") || !bin.is_file() {
                return Err((
                    Verdict::Rejected,
                    format!("`ok:` artifact {} is not a `bin_*` file", bin.display()),
                ));
            }
            let out = Command::new(&bin)
                .current_dir(scratch)
                .output()
                .map_err(|e| {
                    (
                        Verdict::Inconclusive,
                        format!("cannot spawn {}: {e}", bin.display()),
                    )
                })?;
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !out.status.success() || stderr.contains("panicked") {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "member binary exit {} / panicked={}: {}",
                        out.status.code().unwrap_or(-1),
                        stderr.contains("panicked"),
                        pm_head(&stderr)
                    ),
                ));
            }
            let got = normalize_stdout(&String::from_utf8_lossy(&out.stdout));
            let want: Vec<String> = PM_MEMBER_STDOUT.iter().map(|s| (*s).to_string()).collect();
            if got != want {
                return Err((
                    Verdict::Rejected,
                    format!("stdout mismatch got={got:?} expected={want:?} (binary {name})"),
                ));
            }
            Ok(format!("exit 0, `ok:` names {name}, exact stdout {want:?}"))
        }
        PmExpect::NonMemberE0332 => {
            if run.code != Some(1) {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "exit {:?} (need 1) stdout={}",
                        run.code,
                        pm_head(&run.stdout)
                    ),
                ));
            }
            if !run.stdout.is_empty() {
                return Err((
                    Verdict::Rejected,
                    format!("stdout not empty: {}", pm_head(&run.stdout)),
                ));
            }
            let first = run.stderr.lines().next().unwrap_or("");
            let codes = core09_diag_codes(&run.stderr);
            if !first.starts_with("E0332:")
                || codes.is_empty()
                || codes.iter().any(|c| c != "E0332")
            {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "stderr is not exactly E0332 (codes={codes:?}): {}",
                        pm_head(&run.stderr)
                    ),
                ));
            }
            Ok(format!(
                "exit 1, stdout empty, exact E0332: {}",
                pm_head(&core09_diag_head(&run.stderr))
            ))
        }
        PmExpect::MissingIo => {
            if run.code != Some(1) {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "exit {:?} (need 1) stdout={}",
                        run.code,
                        pm_head(&run.stdout)
                    ),
                ));
            }
            if !run.stdout.is_empty() {
                return Err((
                    Verdict::Rejected,
                    format!("stdout not empty: {}", pm_head(&run.stdout)),
                ));
            }
            if run.stderr.trim().is_empty() || run.stderr.contains("E0332") {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "stderr must be a plain I/O error without E0332: {:?}",
                        pm_head(&run.stderr)
                    ),
                ));
            }
            Ok(format!(
                "exit 1, stdout empty, I/O stderr without E0332: {}",
                pm_head(run.stderr.lines().next().unwrap_or(""))
            ))
        }
        PmExpect::LibDir => {
            let dir = pm_ok_path(run, scratch)?;
            if !run.stderr.is_empty() {
                return Err((
                    Verdict::Rejected,
                    format!("stderr not empty: {}", pm_head(&run.stderr)),
                ));
            }
            let comps: Vec<String> = dir
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            if let Some(c) = comps.iter().find(|c| c.starts_with("bin_")) {
                return Err((
                    Verdict::Rejected,
                    format!("`ok:` path {} has a `bin_*` component {c:?}", dir.display()),
                ));
            }
            let n = comps.len();
            if n < 2 || comps[n - 2] != "target" || comps[n - 1] != "debug" || !dir.is_dir() {
                return Err((
                    Verdict::Rejected,
                    format!(
                        "`ok:` path {} is not an existing target/debug dir",
                        dir.display()
                    ),
                ));
            }
            Ok(format!(
                "exit 0, `ok:` = profile dir {} (no `bin_*` component), stderr empty",
                dir.display()
            ))
        }
    }
}

/// Build probe(s) on the scratch copy + no-mutation checks (neg oracles: scratch tree incl. the
/// planted `target/arita-pkg/marker`; every oracle: repo fixture tree and repo `target/arita-pkg`).
fn pm_probe(ctx: &PmCtx, spec: &PmOracle, exe: &Path) -> OracleResult {
    let (id, inputs, expect) = *spec;
    let fail = |verdict: Verdict, detail: String| core09_vec_fail(id, &ctx.label, verdict, detail);
    let mut notes: Vec<String> = Vec::new();
    for (input, rel_form) in inputs.iter().copied() {
        let arg = if rel_form {
            input.to_string()
        } else {
            ctx.scratch.join(input).display().to_string()
        };
        let run = match pm_run_cli(exe, &ctx.scratch, &arg) {
            Ok(r) => r,
            Err(e) => return fail(Verdict::Inconclusive, e),
        };
        match pm_check(expect, &ctx.scratch, &run) {
            Ok(d) => notes.push(format!("[{input}] {d}")),
            Err((verdict, d)) => return fail(verdict, format!("[{input}] {d}")),
        }
    }
    let guarded = matches!(expect, PmExpect::NonMemberE0332 | PmExpect::MissingIo);
    if guarded {
        let marker = ctx.scratch.join("target").join("arita-pkg").join("marker");
        if fs::read(&marker).ok().as_deref() != Some(b"keep-me".as_slice()) {
            return fail(
                Verdict::Rejected,
                format!("marker {} missing or changed", marker.display()),
            );
        }
        match pm_digest(&ctx.scratch, false) {
            Ok(after) if after == ctx.scratch_before => {}
            Ok(_) => {
                return fail(
                    Verdict::Rejected,
                    "scratch fixture tree mutated by a rejected build".into(),
                );
            }
            Err(e) => return fail(Verdict::Inconclusive, e),
        }
    }
    match pm_digest(&ctx.fixture, true) {
        Ok(after) if after == ctx.repo_before => {}
        Ok(_) => {
            return fail(
                Verdict::Rejected,
                format!(
                    "repo fixture {} changed during the oracle",
                    ctx.fixture.display()
                ),
            );
        }
        Err(e) => return fail(Verdict::Inconclusive, e),
    }
    match pm_meta_digest(&ctx.pkg_dir) {
        Ok(after) if after == ctx.pkg_before => {}
        Ok(_) => {
            return fail(
                Verdict::Rejected,
                format!("repo {} changed during the oracle", ctx.pkg_dir.display()),
            );
        }
        Err(e) => return fail(Verdict::Inconclusive, e),
    }
    let note = if guarded {
        "scratch tree + marker, repo fixture and repo target/arita-pkg unchanged"
    } else {
        "repo fixture and repo target/arita-pkg unchanged"
    };
    fail(Verdict::Accepted, format!("{}; {note}", notes.join(" | ")))
}

/// Everything that needs the scratch dir (the caller removes it afterwards, whatever happens).
fn pm_on_scratch(
    root: &Path,
    spec: &PmOracle,
    label: &str,
    fixture: &Path,
    scratch: &Path,
    exe: &Path,
) -> OracleResult {
    let (id, _inputs, expect) = *spec;
    let inconclusive = |detail: String| core09_vec_fail(id, label, Verdict::Inconclusive, detail);
    let repo_before = match pm_digest(fixture, true) {
        Ok(d) => d,
        Err(e) => return inconclusive(e),
    };
    let pkg_dir = root.join("target").join("arita-pkg");
    let pkg_before = match pm_meta_digest(&pkg_dir) {
        Ok(d) => d,
        Err(e) => return inconclusive(e),
    };
    if scratch.exists() {
        if let Err(e) = fs::remove_dir_all(scratch) {
            return inconclusive(format!("clean scratch {}: {e}", scratch.display()));
        }
    }
    if let Err(e) = pm_copy_tree(fixture, scratch) {
        return inconclusive(e);
    }
    if matches!(expect, PmExpect::NonMemberE0332 | PmExpect::MissingIo) {
        let marker = scratch.join("target").join("arita-pkg").join("marker");
        let planted = marker
            .parent()
            .map(fs::create_dir_all)
            .unwrap_or(Ok(()))
            .and_then(|_| fs::write(&marker, b"keep-me"));
        if let Err(e) = planted {
            return inconclusive(format!("plant marker {}: {e}", marker.display()));
        }
    }
    let scratch_before = match pm_digest(scratch, false) {
        Ok(d) => d,
        Err(e) => return inconclusive(e),
    };
    let ctx = PmCtx {
        label: label.to_string(),
        fixture: fixture.to_path_buf(),
        scratch: scratch.to_path_buf(),
        scratch_before,
        repo_before,
        pkg_dir,
        pkg_before,
    };
    pm_probe(&ctx, spec, exe)
}

/// ADR-287 §5 PM-1..PM-7 — one oracle of `PM_ORACLES`.
fn core10_pkg_member(root: &Path, spec: &PmOracle) -> OracleResult {
    let (id, inputs, expect) = *spec;
    let first = inputs[0].0;
    let label = format!("{PM_FIXTURE}/{first}");
    let fixture = root.join(PM_FIXTURE);
    for f in PM_FIXTURE_FILES {
        if !fixture.join(f).is_file() {
            return core09_vec_fail(
                id,
                &label,
                Verdict::Inconclusive,
                format!("missing fixture file {}", fixture.join(f).display()),
            );
        }
    }
    if matches!(expect, PmExpect::MissingIo) && fixture.join(first).exists() {
        return core09_vec_fail(
            id,
            &label,
            Verdict::Rejected,
            format!("fixture shape: {first} must NOT exist (PM-6 missing input)"),
        );
    }
    let Some(exe) = pm_cli_exe() else {
        return core09_vec_fail(
            id,
            &label,
            Verdict::Inconclusive,
            "needs the `arita` CLI process (run via `arita measure`); skip ≠ PASS".into(),
        );
    };
    let scratch = std::env::temp_dir().join(format!("arita_pm_{id}_{}", std::process::id()));
    let result = pm_on_scratch(root, spec, &label, &fixture, &scratch, &exe);
    let _ = fs::remove_dir_all(&scratch);
    result
}

/// ADR-261 — v[i] in-range → Some (stdout 42).
fn run_core05_index_sugar_some_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-index-sugar-some",
            path: "ejemplos/core05/index-sugar/01-some.arita",
            expected: &["42"],
            require_test: false,
        },
    )
}

/// ADR-261 — OOB/neg → None (stdout none).
fn run_core05_index_sugar_none_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-index-sugar-none",
            path: "ejemplos/core05/index-sugar/02-none.arita",
            expected: &["none"],
            require_test: false,
        },
    )
}

/// ADR-261 — v[i] ≡ v.get(i).
fn run_core05_index_sugar_eq_get_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-index-sugar-eq-get",
            path: "ejemplos/core05/index-sugar/03-eq-get.arita",
            expected: &["eq"],
            require_test: false,
        },
    )
}

/// ADR-261 — emit identical to get; no Index panic path.
fn run_core05_index_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core05-index-emit-ban";
    let rel = "ejemplos/core05/index-sugar/01-some.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("core05_index_sugar_some") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core05_index_sugar_some*.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    let has_get = txt.contains(".get(");
    let has_index_trait = txt.contains("IndexMut") || txt.contains("std::ops::Index");
    // panic-style recv[i] — allow only inside strings/comments heuristically: look for `}[` pattern
    let panic_index = "}[";
    let has_panic_index = txt.contains(panic_index);
    if has_get && !has_index_trait && !has_panic_index {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit uses get path; no Index/IndexMut panic".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail get={has_get} index_trait={has_index_trait} panic_index={has_panic_index}"
            ),
        }
    }
}

/// ADR-261 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @134..142` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core05_index_mut_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core05-index-mut",
        "ejemplos/core05/index-sugar/neg/01-index-mut.arita",
        "134..142",
    )
}

/// ADR-260 — insert in-range → Ok + order (stdout ok).
fn run_core05_insert_ok_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-insert-ok",
            path: "ejemplos/core05/insert/01-ok.arita",
            expected: &["ok"],
            require_test: false,
        },
    )
}

/// ADR-260 — i > len → Err(0); stdout err.
fn run_core05_insert_err_oob_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-insert-err-oob",
            path: "ejemplos/core05/insert/02-err-oob.arita",
            expected: &["err"],
            require_test: false,
        },
    )
}

/// ADR-260 — lit i < 0 → E0311.
fn run_neg_core05_insert_neg_lit_oracle(root: &Path) -> OracleResult {
    let id = "neg-core05-insert-neg-lit";
    let rel = "ejemplos/core05/insert/neg/01-neg-lit.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0311, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0311") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0311: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0311: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-260 — emit uses __arita_vec_insert; no bare Vec::insert outside helper.
fn run_core05_insert_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core05-insert-emit-ban";
    let rel = "ejemplos/core05/insert/01-ok.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    // Build to materialize .rs next to binary under target/arita-out
    let _ = build(full.to_str().unwrap_or(rel));
    let out_dir = root.join("target").join("arita-out");
    let mut rs_path = None;
    if let Ok(rd) = fs::read_dir(&out_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("core05_insert_ok") && name.ends_with(".rs") {
                rs_path = Some(e.path());
                break;
            }
        }
    }
    let Some(rs_path) = rs_path else {
        return OracleResult {
            id: id.into(),
            path: out_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted core05_insert_ok*.rs".into(),
        };
    };
    let txt = match fs::read_to_string(&rs_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rs_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit: {e}"),
            };
        }
    };
    let has_helper = txt.contains("fn __arita_vec_insert");
    let has_call = txt.contains("__arita_vec_insert(&mut");
    let has_bare_vec = txt.contains("Vec::insert");
    // Count `.insert(` occurrences outside helper body roughly: after helper, only helper call allowed
    let after_helper = txt.split("fn __arita_vec_insert").nth(1).unwrap_or("");
    let after_fn = after_helper.split("fn main").nth(1).unwrap_or("");
    let bad_main = after_fn.contains(".insert(") && !after_fn.contains("__arita_vec_insert");
    if has_helper && has_call && !has_bare_vec && !bad_main {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "emit has __arita_vec_insert; no bare Vec::insert / panic path in main".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rs_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail helper={has_helper} call={has_call} bare_vec={has_bare_vec} bad_main={bad_main}"
            ),
        }
    }
}

/// ADR-260 — Map.insert still park E0206 (historical was-e0206 suite).
fn run_neg_core05_insert_was_e0206_oracle(root: &Path) -> OracleResult {
    let id = "neg-core05-insert-was-e0206";
    let rel = "ejemplos/core01/neg/e0206-map-insert-banned.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0206 map insert park, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0206") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "map insert still E0206: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0206: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-262 — insert Ok + v[i] Some → stdout coll-happy.
fn run_core05_scen_coll_happy_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-scen-coll-happy",
            path: "ejemplos/core05/scenario-coll/01-happy.arita",
            expected: &["coll-happy"],
            require_test: false,
        },
    )
}

/// ADR-262 — insert OOB Err + v[999] None → stdout coll-oob (no panic).
fn run_core05_scen_coll_oob_oracle(root: &Path) -> OracleResult {
    run_ejemplo_oracle(
        root,
        &EjemploOracle {
            id: "core05-scen-coll-oob",
            path: "ejemplos/core05/scenario-coll/02-oob.arita",
            expected: &["coll-oob"],
            require_test: false,
        },
    )
}

/// ADR-262 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @144..152` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core05_scen_coll_index_mut_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core05-scen-coll-index-mut",
        "ejemplos/core05/scenario-coll/neg/01-index-mut.arita",
        "144..152",
    )
}

/// ADR-262 — lit i < 0 insert → E0311 (also covered by 260; wired per ADR-262 §2).
fn run_neg_core05_scen_coll_insert_neg_oracle(root: &Path) -> OracleResult {
    let id = "neg-core05-scen-coll-insert-neg";
    let rel = "ejemplos/core05/scenario-coll/neg/02-insert-neg.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0311, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0311") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0311: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0311: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-262 — happy scenario builds green (measure build).
fn run_core05_scen_coll_build_oracle(root: &Path) -> OracleResult {
    let id = "core05-scen-coll-build";
    let rel = "ejemplos/core05/scenario-coll/01-happy.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(bin) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("build ok → {}", bin.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("build failed: {}", e.chars().take(200).collect::<String>()),
        },
    }
}

/// ADR-263 — workspace lib+bin build green for arita-ref-collections.
fn run_core05_ref_coll_build_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-build";
    let rel = "ejemplos/core05/ref-collections/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace build ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-263 — CLI happy: lib build_mid/peek_mid (insert+get/[]) → coll-ref-happy.
fn run_core05_ref_coll_cli_happy_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-cli-happy";
    let rel = "ejemplos/core05/ref-collections/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["coll-ref-happy".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("CLI→lib build_mid/peek_mid → coll-ref-happy (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"coll-ref-happy\"]"),
        }
    }
}

/// ADR-263 — CLI oob: insert Err + index None → coll-ref-oob (no panic).
fn run_core05_ref_coll_cli_oob_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-cli-oob";
    let rel = "ejemplos/core05/ref-collections/edge/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["coll-ref-oob".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("oob path stable → coll-ref-oob (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"coll-ref-oob\"]"),
        }
    }
}

/// ADR-263 — ≥2 scenarios PASS (happy + oob CLIs; aligned ADR-262).
fn run_core05_ref_coll_scenario_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-scenario";
    let happy = root.join("ejemplos/core05/ref-collections/bin/main.arita");
    let oob = root.join("ejemplos/core05/ref-collections/edge/bin/main.arita");
    let scen_coll = root.join("ejemplos/core05/scenario-coll/01-happy.arita");
    if !(happy.is_file() && oob.is_file()) {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core05/ref-collections/bin".into(),
            verdict: Verdict::Rejected,
            detail: "need ≥2 acceptance bins (main+edge)".into(),
        };
    }
    let scen_ok = scen_coll.is_file();
    let h = run_core05_ref_coll_cli_happy_oracle(root);
    let e = run_core05_ref_coll_cli_oob_oracle(root);
    if h.verdict == Verdict::Accepted && e.verdict == Verdict::Accepted {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core05/ref-collections".into(),
            verdict: Verdict::Accepted,
            detail: format!("≥2 scenarios PASS (happy+oob); scenario-coll present={scen_ok}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core05/ref-collections".into(),
            verdict: Verdict::Rejected,
            detail: format!("scenario fail happy={:?} oob={:?}", h.verdict, e.verdict),
        }
    }
}

/// ADR-263 — evidence JSON + stable sha256 for lib/bin/edge.
fn run_core05_ref_coll_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-evidence";
    let ev_rel = "ejemplos/core05/ref-collections/evidence.json";
    let pairs = [
        (
            "\"lib_sha256\"",
            "ejemplos/core05/ref-collections/lib/lib.arita",
        ),
        (
            "\"bin_sha256\"",
            "ejemplos/core05/ref-collections/bin/main.arita",
        ),
        (
            "\"edge_sha256\"",
            "ejemplos/core05/ref-collections/edge/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    if !ev_raw.contains("\"measure_pass\": true") {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence must keep measure_pass:true after Core 0.5 CLOSED".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        Ok(rest[..q2].to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence lib+bin+edge sha256 match; measure_pass:true".into(),
    }
}

/// ADR-263 — emit grep: __arita_vec_insert + .get(; no Index/IndexMut / panic [].
fn run_core05_ref_coll_emit_ban_oracle(root: &Path) -> OracleResult {
    let id = "core05-ref-coll-emit-ban";
    let rel = "ejemplos/core05/ref-collections/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let mut rs_files: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&crates_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("bin_") {
                let p = e.path().join("src").join("main.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
            if name.starts_with("lib_") {
                let p = e.path().join("src").join("lib.rs");
                if p.is_file() {
                    rs_files.push(p);
                }
            }
        }
    }
    if rs_files.is_empty() {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no emitted crates/*.rs".into(),
        };
    }
    let mut combined = String::new();
    for p in &rs_files {
        match fs::read_to_string(p) {
            Ok(s) => {
                combined.push_str(&s);
                combined.push('\n');
            }
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: p.display().to_string(),
                    verdict: Verdict::Rejected,
                    detail: format!("read emit: {e}"),
                };
            }
        }
    }
    let has_helper = combined.contains("fn __arita_vec_insert");
    let has_call = combined.contains("__arita_vec_insert(&mut");
    let has_get = combined.contains(".get(");
    let has_index_trait = combined.contains("IndexMut") || combined.contains("std::ops::Index");
    let has_panic_index = combined.contains("}[");
    let has_bare_vec_insert = combined.contains("Vec::insert");
    // after helper, main must not call .insert( directly
    let after_helper = combined.split("fn __arita_vec_insert").nth(1).unwrap_or("");
    let after_fn = after_helper.split("fn main").nth(1).unwrap_or("");
    let bad_main = after_fn.contains(".insert(") && !after_fn.contains("__arita_vec_insert");
    if has_helper
        && has_call
        && has_get
        && !has_index_trait
        && !has_panic_index
        && !has_bare_vec_insert
        && !bad_main
    {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "emit has __arita_vec_insert + get; no Index/IndexMut / bare Vec::insert"
                .into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "emit-ban fail helper={has_helper} call={has_call} get={has_get} index_trait={has_index_trait} panic_index={has_panic_index} bare_vec={has_bare_vec_insert} bad_main={bad_main}"
            ),
        }
    }
}

/// ADR-263 — Vec `v[i] = x` in main. ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): EXACT
/// `E0344: index assign outside result fn @179..187` (was E0314 IndexMut HOLD). Same id, still neg.
fn run_neg_core05_ref_coll_index_mut_oracle(root: &Path) -> OracleResult {
    adr283_m1_vec_assign_e0344_oracle(
        root,
        "neg-core05-ref-coll-index-mut",
        "ejemplos/core05/ref-coll-neg/01-index-mut.arita",
        "179..187",
    )
}

/// ADR-258 — workspace lib+bin build green for arita-ref-pkg-lib.
fn run_core04_ref_lib_build_oracle(root: &Path) -> OracleResult {
    let id = "core04-ref-lib-build";
    let rel = "ejemplos/core04/ref-pkg-lib/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace build ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-258 — CLI happy: apply(Config{2},21) → 42.
fn run_core04_ref_cli_happy_oracle(root: &Path) -> OracleResult {
    let id = "core04-ref-cli-happy";
    let rel = "ejemplos/core04/ref-pkg-lib/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["42".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("CLI→lib apply → 42 (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"42\"]"),
        }
    }
}

/// ADR-258 — CLI edge: factor 0 → stdout "edge" (no panic).
fn run_core04_ref_cli_edge_oracle(root: &Path) -> OracleResult {
    let id = "core04-ref-cli-edge";
    let rel = "ejemplos/core04/ref-pkg-lib/edge/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    // Edge lives beside main under same package root — build via edge entry
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["edge".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("edge path stable → edge (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"edge\"]"),
        }
    }
}

/// ADR-258 — ≥2 scenario-pkg acceptance still present (happy+edge bins as scenarios).
fn run_core04_ref_scenario_pkg_oracle(root: &Path) -> OracleResult {
    let id = "core04-ref-scenario-pkg";
    let happy = root.join("ejemplos/core04/ref-pkg-lib/bin/main.arita");
    let edge = root.join("ejemplos/core04/ref-pkg-lib/edge/bin/main.arita");
    let scen_pkg = root.join("ejemplos/core04/scenario-pkg/bin/main.arita");
    if !(happy.is_file() && edge.is_file()) {
        return OracleResult {
            id: id.into(),
            path: "ejemplos/core04/ref-pkg-lib/bin".into(),
            verdict: Verdict::Rejected,
            detail: "need ≥2 acceptance bins (main+edge)".into(),
        };
    }
    // Prefer also keeping ADR-257 scenario-pkg tree as cross-slice continuity
    let scen_ok = scen_pkg.is_file();
    let h = run_core04_ref_cli_happy_oracle(root);
    let e = run_core04_ref_cli_edge_oracle(root);
    if h.verdict == Verdict::Accepted && e.verdict == Verdict::Accepted {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core04/ref-pkg-lib".into(),
            verdict: Verdict::Accepted,
            detail: format!("≥2 scenarios PASS (happy+edge); scenario-pkg present={scen_ok}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: "ejemplos/core04/ref-pkg-lib".into(),
            verdict: Verdict::Rejected,
            detail: format!("scenario fail happy={:?} edge={:?}", h.verdict, e.verdict),
        }
    }
}

/// ADR-258 — evidence JSON + stable sha256 for lib/bin.
fn run_core04_ref_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core04-ref-evidence";
    let ev_rel = "ejemplos/core04/ref-pkg-lib/evidence.json";
    let pairs = [
        (
            "\"lib_sha256\"",
            "ejemplos/core04/ref-pkg-lib/lib/lib.arita",
        ),
        (
            "\"bin_sha256\"",
            "ejemplos/core04/ref-pkg-lib/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        Ok(rest[..q2].to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence lib+bin sha256 match".into(),
    }
}

/// ADR-258 — private cross-crate → E0331.
fn run_neg_core04_ref_private_oracle(root: &Path) -> OracleResult {
    let id = "neg-core04-ref-private";
    let rel = "ejemplos/core04/ref-pkg-lib/neg/private/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0331 private, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0331") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0331: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0331: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-257 — bin+lib scenario happy: print(double(21)) → 42.
fn run_core04_scen_pkg_happy_oracle(root: &Path) -> OracleResult {
    let id = "core04-scen-pkg-happy";
    let rel = "ejemplos/core04/scenario-pkg/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["42".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("pub fn double via bin → 42 (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"42\"]"),
        }
    }
}

/// ADR-257 — emitted lib contains `pub fn double` and bin uses it.
fn run_core04_scen_pkg_lib_fn_oracle(root: &Path) -> OracleResult {
    let id = "core04-scen-pkg-lib-fn";
    let rel = "ejemplos/core04/scenario-pkg/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let lib_rs = match fs::read_dir(out_root.join("crates")) {
        Ok(rd) => {
            let mut found = None;
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("lib_") {
                    let p = e.path().join("src").join("lib.rs");
                    if p.is_file() {
                        found = Some(p);
                        break;
                    }
                }
            }
            found
        }
        Err(_) => None,
    };
    let Some(lib_rs) = lib_rs else {
        return OracleResult {
            id: id.into(),
            path: out_root.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted lib.rs".into(),
        };
    };
    let lib_txt = match fs::read_to_string(&lib_rs) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: lib_rs.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read lib.rs: {e}"),
            };
        }
    };
    let bin_rs = match fs::read_dir(out_root.join("crates")) {
        Ok(rd) => {
            let mut found = None;
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("bin_") {
                    let p = e.path().join("src").join("main.rs");
                    if p.is_file() {
                        found = Some(p);
                        break;
                    }
                }
            }
            found
        }
        Err(_) => None,
    };
    let Some(bin_rs) = bin_rs else {
        return OracleResult {
            id: id.into(),
            path: out_root.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted main.rs".into(),
        };
    };
    let bin_txt = match fs::read_to_string(&bin_rs) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: bin_rs.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read main.rs: {e}"),
            };
        }
    };
    let lib_ok = lib_txt.contains("pub fn double");
    let bin_ok = bin_txt.contains("double") && bin_txt.contains("lib_");
    if lib_ok && bin_ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "pub fn double emitted in lib + used in bin".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("pub double emit check lib_ok={lib_ok} bin_ok={bin_ok}"),
        }
    }
}

/// ADR-257 — workspace cargo check green for scenario-pkg.
fn run_core04_scen_pkg_workspace_build_oracle(root: &Path) -> OracleResult {
    let id = "core04-scen-pkg-workspace-build";
    let rel = "ejemplos/core04/scenario-pkg/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace cargo check ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-257 — non-pub item across crate → E0331.
fn run_neg_core04_scen_pkg_private_oracle(root: &Path) -> OracleResult {
    let id = "neg-core04-scen-pkg-private";
    let rel = "ejemplos/core04/scenario-pkg/neg/private/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0331 private, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0331") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0331: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0331: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-256 — bin calls pub fn add from lib → stdout 3.
fn run_core04_lib_call_from_bin_oracle(root: &Path) -> OracleResult {
    let id = "core04-lib-call-from-bin";
    let rel = "ejemplos/core04/lib-api/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_*".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cargo run spawn: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(220)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["3".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("pub fn add via bin → 3 (cargo run -p {bin_pkg})"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch got={got:?} expected=[\"3\"]"),
        }
    }
}

/// ADR-256 — pub record Point crosses crate (emit + bin use).
fn run_core04_lib_pub_record_oracle(root: &Path) -> OracleResult {
    let id = "core04-lib-pub-record";
    let rel = "ejemplos/core04/lib-api/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let lib_rs = match fs::read_dir(out_root.join("crates")) {
        Ok(rd) => {
            let mut found = None;
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("lib_") {
                    let p = e.path().join("src").join("lib.rs");
                    if p.is_file() {
                        found = Some(p);
                        break;
                    }
                }
            }
            found
        }
        Err(_) => None,
    };
    let Some(lib_rs) = lib_rs else {
        return OracleResult {
            id: id.into(),
            path: out_root.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted lib.rs".into(),
        };
    };
    let lib_txt = match fs::read_to_string(&lib_rs) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: lib_rs.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read lib.rs: {e}"),
            };
        }
    };
    let bin_rs = match fs::read_dir(out_root.join("crates")) {
        Ok(rd) => {
            let mut found = None;
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with("bin_") {
                    let p = e.path().join("src").join("main.rs");
                    if p.is_file() {
                        found = Some(p);
                        break;
                    }
                }
            }
            found
        }
        Err(_) => None,
    };
    let Some(bin_rs) = bin_rs else {
        return OracleResult {
            id: id.into(),
            path: out_root.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "missing emitted main.rs".into(),
        };
    };
    let bin_txt = match fs::read_to_string(&bin_rs) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: bin_rs.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read main.rs: {e}"),
            };
        }
    };
    let lib_ok = lib_txt.contains("pub struct Point");
    let bin_ok = bin_txt.contains("Point") && bin_txt.contains("lib_");
    if lib_ok && bin_ok {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "pub record Point emitted in lib + used in bin".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("pub Point emit check lib_ok={lib_ok} bin_ok={bin_ok}"),
        }
    }
}

/// ADR-256 — workspace cargo check green (build_package_workspace).
fn run_core04_lib_build_workspace_oracle(root: &Path) -> OracleResult {
    let id = "core04-lib-build-workspace";
    let rel = "ejemplos/core04/lib-api/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
        Ok(p) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("workspace cargo check ok at {}", p.display()),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("workspace build failed: {e}"),
        },
    }
}

/// ADR-256 — non-pub item across crate → E0331.
fn run_neg_core04_lib_private_oracle(root: &Path) -> OracleResult {
    let id = "neg-core04-lib-private";
    let rel = "ejemplos/core04/lib-api/neg/private/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0331 private, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0331") && e.to_lowercase().contains("private") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0331 private: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else if e.contains("E0331") {
                // code match alone (message may vary)
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0331: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0331: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-256 — evidence lib+bin sha256.
fn run_core04_lib_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core04-lib-evidence";
    let ev_rel = "ejemplos/core04/lib-api/evidence.json";
    let pairs = [
        ("\"lib_sha256\"", "ejemplos/core04/lib-api/lib/lib.arita"),
        ("\"bin_sha256\"", "ejemplos/core04/lib-api/bin/main.arita"),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        let v = &rest[..q2];
        if v.is_empty() {
            return Err(format!("{key} empty"));
        }
        Ok(v.to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let got = match crate::attest::sha256_file(&root.join(rel)) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence ok lib+bin sha256".into(),
    }
}

/// ADR-255 — valid arita.toml [workspace] loads beside package entry.
fn run_core04_pkg_manifest_parse_oracle(root: &Path) -> OracleResult {
    let id = "core04-pkg-manifest-parse";
    let rel = "ejemplos/core04/package-manifest/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing entry: {}", full.display()),
        };
    }
    match crate::package::load_package_manifest(&full) {
        Ok(Some(m)) => {
            if m.members.is_empty() {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: "workspace members empty".into(),
                };
            }
            OracleResult {
                id: id.into(),
                path: "ejemplos/core04/package-manifest/arita.toml".into(),
                verdict: Verdict::Accepted,
                detail: format!("manifest ok name={} members={:?}", m.name, m.members),
            }
        }
        Ok(None) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected [workspace] package manifest, got None".into(),
        },
        Err(e) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("load_package_manifest: {e}"),
        },
    }
}

/// ADR-255 — emit workspace Cargo.toml with crates/lib_* + crates/bin_*.
fn run_core04_pkg_lib_bin_emit_oracle(root: &Path) -> OracleResult {
    let id = "core04-pkg-lib-bin-emit";
    let rel = "ejemplos/core04/package-manifest/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing entry: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let cargo = out_root.join("Cargo.toml");
    let text = match fs::read_to_string(&cargo) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: cargo.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read emit Cargo.toml: {e}"),
            };
        }
    };
    let has_lib = text.contains("crates/lib_");
    let has_bin = text.contains("crates/bin_");
    if has_lib && has_bin {
        OracleResult {
            id: id.into(),
            path: cargo.display().to_string(),
            verdict: Verdict::Accepted,
            detail: format!("emit members ok path={}", out_root.display()),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: cargo.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!("missing lib_/bin_ members in Cargo.toml: {text}"),
        }
    }
}

/// ADR-255 — package build + cargo run bin → stdout 42.
fn run_core04_pkg_build_oracle(root: &Path) -> OracleResult {
    let id = "core04-pkg-build";
    let rel = "ejemplos/core04/package-manifest/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing entry: {}", full.display()),
        };
    }
    let out_root =
        match crate::package::build_package_workspace(&full, arita_codegen::BuildProfile::Debug) {
            Ok(p) => p,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("build_package_workspace: {e}"),
                };
            }
        };
    let crates_dir = out_root.join("crates");
    let bin_pkg = match fs::read_dir(&crates_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .find(|n| n.starts_with("bin_")),
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: crates_dir.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("read crates dir: {e}"),
            };
        }
    };
    let Some(bin_pkg) = bin_pkg else {
        return OracleResult {
            id: id.into(),
            path: crates_dir.display().to_string(),
            verdict: Verdict::Rejected,
            detail: "no crates/bin_* member".into(),
        };
    };
    let cargo_toml = out_root.join("Cargo.toml");
    let run = match Command::new("cargo")
        .args(["run", "-q", "-p", &bin_pkg, "--manifest-path"])
        .arg(&cargo_toml)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn cargo run: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "cargo run -p {bin_pkg} failed: {}",
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(240)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == ["42".to_string()] {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("package cargo run -p {bin_pkg} → 42"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected=[\"42\"]"),
        }
    }
}

/// ADR-255 — crates.io deps → E0330 on build.
fn run_neg_core04_pkg_cratesio_oracle(root: &Path) -> OracleResult {
    let id = "neg-core04-pkg-cratesio";
    let rel = "ejemplos/core04/package-manifest/neg/cratesio/bin/main.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0330, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0330") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0330: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0330: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-255 — bad arita.toml → E0331 on build.
fn run_neg_core04_pkg_bad_toml_oracle(root: &Path) -> OracleResult {
    let id = "neg-core04-pkg-bad-toml";
    let rel = "ejemplos/core04/package-manifest/neg/bad-toml/lib.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0331, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0331") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0331: {}",
                        e.chars().take(160).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "failed without E0331: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-255 — evidence manifest/lib/bin sha256 match.
fn run_core04_pkg_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core04-pkg-evidence";
    let ev_rel = "ejemplos/core04/package-manifest/evidence.json";
    let pairs = [
        (
            "\"manifest_sha256\"",
            "ejemplos/core04/package-manifest/arita.toml",
        ),
        (
            "\"lib_sha256\"",
            "ejemplos/core04/package-manifest/lib/lib.arita",
        ),
        (
            "\"bin_sha256\"",
            "ejemplos/core04/package-manifest/bin/main.arita",
        ),
    ];
    let ev_path = root.join(ev_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let extract = |raw: &str, key: &str| -> Result<String, String> {
        let Some(pos) = raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        let v = &rest[..q2];
        if v.is_empty() {
            return Err(format!("{key} empty"));
        }
        Ok(v.to_string())
    };
    for (key, rel) in pairs {
        let expected = match extract(&ev_raw, key) {
            Ok(v) => v,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: ev_rel.into(),
                    verdict: Verdict::Rejected,
                    detail: e,
                };
            }
        };
        let path = root.join(rel);
        let got = match crate::attest::sha256_file(&path) {
            Ok(h) => h,
            Err(e) => {
                return OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("hash {rel}: {e}"),
                };
            }
        };
        if got != expected {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("{key} mismatch got={got} expected={expected}"),
            };
        }
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: "evidence ok manifest+lib+bin sha256".into(),
    }
}

/// ADR-254 — missing sibling module must fail build with E0404 (parse alone may succeed).
fn run_core04_mm_neg_missing_oracle(root: &Path) -> OracleResult {
    let id = "core04-mm-neg-missing";
    let rel = "ejemplos/core04/multi-module/neg/missing-mod.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    match build(full.to_str().unwrap_or(rel)) {
        Ok(_) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: "expected E0404 missing module, but build succeeded".into(),
        },
        Err(e) => {
            if e.contains("E0404") {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Accepted,
                    detail: format!(
                        "rejected with E0404: {}",
                        e.chars().take(180).collect::<String>()
                    ),
                }
            } else {
                OracleResult {
                    id: id.into(),
                    path: rel.into(),
                    verdict: Verdict::Rejected,
                    detail: format!(
                        "build failed without E0404: {}",
                        e.chars().take(200).collect::<String>()
                    ),
                }
            }
        }
    }
}

/// ADR-254 — evidence.json entry_sha256 + lib_sha256 match.
fn run_core04_mm_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core04-mm-evidence";
    let ev_rel = "ejemplos/core04/multi-module/evidence.json";
    let entry_rel = "ejemplos/core04/multi-module/main.arita";
    let lib_rel = "ejemplos/core04/multi-module/util.arita";
    let ev_path = root.join(ev_rel);
    let entry_path = root.join(entry_rel);
    let lib_path = root.join(lib_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    if !entry_path.is_file() || !lib_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: "missing main.arita or util.arita".into(),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let extract = |key: &str| -> Result<String, String> {
        let Some(pos) = ev_raw.find(key) else {
            return Err(format!("missing {key}"));
        };
        let after = &ev_raw[pos + key.len()..];
        let Some(q1) = after.find('"') else {
            return Err(format!("{key} value missing"));
        };
        let rest = &after[q1 + 1..];
        let Some(q2) = rest.find('"') else {
            return Err(format!("{key} unclosed"));
        };
        let v = &rest[..q2];
        if v.is_empty() {
            return Err(format!("{key} empty"));
        }
        Ok(v.to_string())
    };
    let expected_entry = match extract("\"entry_sha256\"") {
        Ok(v) => v,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: e,
            };
        }
    };
    let expected_lib = match extract("\"lib_sha256\"") {
        Ok(v) => v,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: e,
            };
        }
    };
    let got_entry = match crate::attest::sha256_file(&entry_path) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: entry_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("hash entry: {e}"),
            };
        }
    };
    let got_lib = match crate::attest::sha256_file(&lib_path) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: lib_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("hash lib: {e}"),
            };
        }
    };
    if got_entry != expected_entry || got_lib != expected_lib {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "sha mismatch entry got={got_entry} expected={expected_entry}; lib got={got_lib} expected={expected_lib}"
            ),
        };
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: format!("evidence ok entry_sha256={got_entry} lib_sha256={got_lib}"),
    }
}

/// ADR-253 — dual port0 bind + shutdown markers (ephemeral ports).
fn run_core03_ref_shutdown_oracle(root: &Path) -> OracleResult {
    let id = "core03-ref-shutdown";
    let rel = "ejemplos/core03/ref-http-compose/scen-shutdown.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn shutdown: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(200)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got.len() != 4 {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected 4 lines got={got:?}"),
        };
    }
    let p0_ok = got[0].parse::<i64>().map(|n| n > 0).unwrap_or(false);
    let p1_ok = got[1].parse::<i64>().map(|n| n > 0).unwrap_or(false);
    if !p0_ok || !p1_ok || got[0] == got[1] {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("dual ephemeral ports invalid: {:?}", &got[..2]),
        };
    }
    if got[2] != "dual-bound" || got[3] != "ref-compose-shutdown-ok" {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("markers mismatch: got={got:?}"),
        };
    }
    OracleResult {
        id: id.into(),
        path: rel.into(),
        verdict: Verdict::Accepted,
        detail: format!(
            "ref-compose dual port0 shutdown ok ports={},{}",
            got[0], got[1]
        ),
    }
}

/// ADR-253 — evidence.json + stable source_sha256 of arita-ref-http-compose.arita.
fn run_core03_ref_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core03-ref-evidence";
    let ev_rel = "ejemplos/core03/ref-http-compose/evidence.json";
    let src_rel = "ejemplos/core03/ref-http-compose/arita-ref-http-compose.arita";
    let ev_path = root.join(ev_rel);
    let src_path = root.join(src_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    if !src_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: src_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing source: {}", src_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let key = "\"source_sha256\"";
    let Some(pos) = ev_raw.find(key) else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing source_sha256".into(),
        };
    };
    let after = &ev_raw[pos + key.len()..];
    let Some(q1) = after.find('"') else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 value missing".into(),
        };
    };
    let rest = &after[q1 + 1..];
    let Some(q2) = rest.find('"') else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 unclosed".into(),
        };
    };
    let expected = &rest[..q2];
    if expected.is_empty() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 empty".into(),
        };
    }
    let got = match crate::attest::sha256_file(&src_path) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: src_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("hash source: {e}"),
            };
        }
    };
    if got != expected {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("source_sha256 mismatch: got={got} expected={expected}"),
        };
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: format!("evidence ok arita.evidence.v1 source_sha256={got}"),
    }
}

/// ADR-252 — dual port0 bind + shutdown markers (ephemeral ports).
fn run_core03_scen_compose_shutdown_oracle(root: &Path) -> OracleResult {
    let id = "core03-scen-compose-shutdown";
    let rel = "ejemplos/core03/scenario-compose/04-scen-shutdown.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn shutdown: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(200)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got.len() != 4 {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected 4 lines [port,port,dual-bound,ok] got={got:?}"),
        };
    }
    let p0_ok = got[0].parse::<i64>().map(|n| n > 0).unwrap_or(false);
    let p1_ok = got[1].parse::<i64>().map(|n| n > 0).unwrap_or(false);
    if !p0_ok || !p1_ok || got[0] == got[1] {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("dual ephemeral ports invalid: {:?}", &got[..2]),
        };
    }
    if got[2] != "dual-bound" || got[3] != "scen-compose-shutdown-ok" {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("markers mismatch: got={got:?}"),
        };
    }
    OracleResult {
        id: id.into(),
        path: rel.into(),
        verdict: Verdict::Accepted,
        detail: format!("dual port0 shutdown ok ports={},{}", got[0], got[1]),
    }
}

/// ADR-249 — upstream echo server :19921 + curl POST body roundtrip.
fn run_core03_compose_upstream_echo_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core03-compose-upstream-echo";
    let rel = "ejemplos/core03/client-compose/01-upstream-echo.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let mut child = match Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn upstream: {e}"),
            };
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: "missing child stdout pipe".into(),
            };
        }
    };
    let mut lines = BufReader::new(stdout).lines();
    let port_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: "EOF/err before port line".into(),
            };
        }
    };
    let ready = match lines.next() {
        Some(Ok(l)) => l,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("missing ready after port={port_line}"),
            };
        }
    };
    if ready != "upstream-ready" || port_line != "19921" {
        let _ = child.kill();
        let _ = child.wait();
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected 19921/upstream-ready got {port_line:?}/{ready:?}"),
        };
    }
    let url = "http://127.0.0.1:19921/";
    let payload = "compose-ping";
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut body = String::new();
    let mut code = String::new();
    let mut curl_err = String::new();
    while Instant::now() < deadline {
        let out_path = "/tmp/arita-core03-upstream-echo.out";
        match Command::new("curl")
            .args([
                "-sS",
                "-m",
                "2",
                "-o",
                out_path,
                "-w",
                "%{http_code}",
                "-X",
                "POST",
                "-d",
                payload,
                url,
            ])
            .output()
        {
            Ok(o) if o.status.success() => {
                code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                body = fs::read_to_string(out_path).unwrap_or_default();
                if code == "200" && body.trim() == payload {
                    break;
                }
            }
            Ok(o) => {
                curl_err = format!(
                    "curl exit {} stderr={}",
                    o.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&o.stderr)
                        .chars()
                        .take(120)
                        .collect::<String>()
                );
            }
            Err(e) => curl_err = format!("curl spawn: {e}"),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
    if code == "200" && body.trim() == payload {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("upstream echo ok port=19921 body={payload}"),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("echo fail code={code:?} body={body:?} curl={curl_err}"),
        }
    }
}

/// ADR-249 — gateway :19920 proxies via HttpClient to upstream :19921.
fn run_core03_compose_gateway_ok_oracle(root: &Path) -> OracleResult {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let id = "core03-compose-gateway-ok";
    let up_rel = "ejemplos/core03/client-compose/01-upstream-echo.arita";
    let gw_rel = "ejemplos/core03/client-compose/02-gateway-compose.arita";
    let up_full = root.join(up_rel);
    let gw_full = root.join(gw_rel);
    if !up_full.is_file() || !gw_full.is_file() {
        return OracleResult {
            id: id.into(),
            path: gw_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: "missing upstream or gateway ejemplo".into(),
        };
    }
    let up_bin = match build(up_full.to_str().unwrap_or(up_rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: up_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("upstream build failed: {e}"),
            };
        }
    };
    let gw_bin = match build(gw_full.to_str().unwrap_or(gw_rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: gw_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("gateway build failed: {e}"),
            };
        }
    };

    let spawn_ready = |bin: &Path,
                       expect_port: &str,
                       expect_ready: &str|
     -> Result<std::process::Child, String> {
        let mut child = Command::new(bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn: {e}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "missing stdout".to_string())?;
        let mut lines = BufReader::new(stdout).lines();
        let port_line = lines
            .next()
            .ok_or_else(|| "EOF port".to_string())?
            .map_err(|e| format!("port read: {e}"))?;
        let ready = lines
            .next()
            .ok_or_else(|| "EOF ready".to_string())?
            .map_err(|e| format!("ready read: {e}"))?;
        if port_line != expect_port || ready != expect_ready {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "expected {expect_port}/{expect_ready} got {port_line:?}/{ready:?}"
            ));
        }
        // Keep reading in background? Drop lines → may fill pipe. Detach by leaking reader thread.
        std::thread::spawn(move || for _ in lines {});
        Ok(child)
    };

    let mut up = match spawn_ready(&up_bin, "19921", "upstream-ready") {
        Ok(c) => c,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: up_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("upstream ready fail: {e}"),
            };
        }
    };
    let mut gw = match spawn_ready(&gw_bin, "19920", "gateway-ready") {
        Ok(c) => c,
        Err(e) => {
            let _ = up.kill();
            let _ = up.wait();
            return OracleResult {
                id: id.into(),
                path: gw_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("gateway ready fail: {e}"),
            };
        }
    };

    let url = "http://127.0.0.1:19920/";
    let payload = "compose-ping";
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut body = String::new();
    let mut code = String::new();
    let mut curl_err = String::new();
    while Instant::now() < deadline {
        let out_path = "/tmp/arita-core03-gateway.out";
        match Command::new("curl")
            .args([
                "-sS",
                "-m",
                "2",
                "-o",
                out_path,
                "-w",
                "%{http_code}",
                "-X",
                "POST",
                "-d",
                payload,
                url,
            ])
            .output()
        {
            Ok(o) if o.status.success() => {
                code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                body = fs::read_to_string(out_path).unwrap_or_default();
                if code == "200" && body.trim() == payload {
                    break;
                }
            }
            Ok(o) => {
                curl_err = format!(
                    "curl exit {} stderr={}",
                    o.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&o.stderr)
                        .chars()
                        .take(120)
                        .collect::<String>()
                );
            }
            Err(e) => curl_err = format!("curl spawn: {e}"),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = gw.kill();
    let _ = gw.wait();
    let _ = up.kill();
    let _ = up.wait();
    if code == "200" && body.trim() == payload {
        OracleResult {
            id: id.into(),
            path: gw_rel.into(),
            verdict: Verdict::Accepted,
            detail: "gateway HttpClient.post_text proxy ok 19920→19921".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: gw_rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("gateway fail code={code:?} body={body:?} curl={curl_err}"),
        }
    }
}

/// ADR-249 — timeout path inside compose smoke (ADR-244).
fn run_core03_compose_client_timeout_oracle(root: &Path) -> OracleResult {
    let id = "core03-compose-client-timeout";
    let rel = "ejemplos/core03/client-compose/03-compose-smoke.arita";
    let full = root.join(rel);
    if !full.is_file() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing oracle file: {}", full.display()),
        };
    }
    let bin = match build(full.to_str().unwrap_or(rel)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };
    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: rel.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn smoke: {e}"),
            };
        }
    };
    if !run.status.success() {
        return OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&run.stderr)
                    .chars()
                    .take(200)
                    .collect::<String>()
            ),
        };
    }
    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let has_timeout = got.iter().any(|l| l == "timeout-ok");
    let has_compose = got.iter().any(|l| l == "compose-timeout-ok");
    if has_timeout && has_compose {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: "compose smoke includes timeout-ok + compose-timeout-ok".into(),
        }
    } else {
        OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("missing timeout markers in stdout={got:?}"),
        }
    }
}

/// ADR-249 fixture / ADR-293 D6 — `mutex_new(0)` is a free call to an undeclared fn: exact E0347 in HIR
/// (was the accidental rustc/cargo E0100). Id and fixture frozen; no build; does not count toward N.
fn run_core03_compose_mutex_hold_oracle(root: &Path) -> OracleResult {
    let id = "neg-core03-compose-mutex-hold";
    let rel = "ejemplos/core03/client-compose/neg/02-mutex-hold.arita";
    match uc_probe(root, rel, "mutex_new") {
        Ok(msg) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict: Verdict::Accepted,
            detail: format!("Mutex HOLD rejected with exact E0347: {msg}"),
        },
        Err((verdict, detail)) => OracleResult {
            id: id.into(),
            path: rel.into(),
            verdict,
            detail,
        },
    }
}

/// ADR-248 — evidence.json present + source_sha256 matches re-hash of ref .arita.
fn run_core02_ref_evidence_oracle(root: &Path) -> OracleResult {
    let id = "core02-ref-evidence";
    let ev_rel = "ejemplos/core02/ref-http-json/evidence.json";
    let src_rel = "ejemplos/core02/ref-http-json/arita-ref-http-json.arita";
    let ev_path = root.join(ev_rel);
    let src_path = root.join(src_rel);
    if !ev_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing evidence: {}", ev_path.display()),
        };
    }
    if !src_path.is_file() {
        return OracleResult {
            id: id.into(),
            path: src_rel.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing source: {}", src_path.display()),
        };
    }
    let ev_raw = match fs::read_to_string(&ev_path) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: ev_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("read evidence: {e}"),
            };
        }
    };
    if !(ev_raw.contains("\"schema_version\"") && ev_raw.contains("arita.evidence.v1")) {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing schema_version arita.evidence.v1".into(),
        };
    }
    let key = "\"source_sha256\"";
    let Some(pos) = ev_raw.find(key) else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "evidence missing source_sha256".into(),
        };
    };
    let after = &ev_raw[pos + key.len()..];
    let Some(q1) = after.find('"') else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 value missing".into(),
        };
    };
    let rest = &after[q1 + 1..];
    let Some(q2) = rest.find('"') else {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 unclosed".into(),
        };
    };
    let expected = &rest[..q2];
    if expected.is_empty() {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: "source_sha256 empty".into(),
        };
    }
    let got = match crate::attest::sha256_file(&src_path) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: id.into(),
                path: src_rel.into(),
                verdict: Verdict::Rejected,
                detail: format!("hash source: {e}"),
            };
        }
    };
    if got != expected {
        return OracleResult {
            id: id.into(),
            path: ev_rel.into(),
            verdict: Verdict::Rejected,
            detail: format!("source_sha256 mismatch: got={got} expected={expected}"),
        };
    }
    OracleResult {
        id: id.into(),
        path: ev_rel.into(),
        verdict: Verdict::Accepted,
        detail: format!("evidence ok arita.evidence.v1 source_sha256={got}"),
    }
}

/// Run measure with a custom ejemplo oracle list (tests) plus required clippy+miri first.
pub fn run_measure_with(
    root: &Path,
    ejemplos: &[EjemploOracle],
    neg: &[NegOracle],
    logic: &[LogicOracle],
    contracts: &[crate::contract::ContractOracle],
    include_clippy: bool,
) -> (Verdict, Vec<OracleResult>) {
    let mut results = Vec::new();
    if include_clippy {
        results.push(run_clippy_oracle(root));
        // Same gate as clippy: required toolchain oracle; missing → inconclusive (never PASS).
        results.push(run_miri_oracle(root));
    }
    for o in ejemplos {
        results.push(run_ejemplo_oracle(root, o));
    }
    // ADR-248 REF-HTTP-DAEMON evidence (file + stable sha)
    results.push(run_core02_ref_evidence_oracle(root));
    // ADR-249 CLIENT-COMPOSE
    results.push(run_core03_compose_upstream_echo_oracle(root));
    results.push(run_core03_compose_gateway_ok_oracle(root));
    results.push(run_core03_compose_client_timeout_oracle(root));
    results.push(run_core03_compose_mutex_hold_oracle(root));
    // ADR-252 SCENARIO-COMPOSE shutdown (ephemeral dual port0)
    results.push(run_core03_scen_compose_shutdown_oracle(root));
    // ADR-253 REF-COMPOSE
    results.push(run_core03_ref_shutdown_oracle(root));
    results.push(run_core03_ref_evidence_oracle(root));
    // ADR-254 MULTI-MODULE
    results.push(run_core04_mm_neg_missing_oracle(root));
    results.push(run_core04_mm_evidence_oracle(root));
    // ADR-255 PACKAGE-MANIFEST
    results.push(run_core04_pkg_manifest_parse_oracle(root));
    results.push(run_core04_pkg_lib_bin_emit_oracle(root));
    results.push(run_core04_pkg_build_oracle(root));
    results.push(run_neg_core04_pkg_cratesio_oracle(root));
    results.push(run_neg_core04_pkg_bad_toml_oracle(root));
    results.push(run_core04_pkg_evidence_oracle(root));
    // ADR-256 LIB-API
    results.push(run_core04_lib_call_from_bin_oracle(root));
    results.push(run_core04_lib_pub_record_oracle(root));
    results.push(run_core04_lib_build_workspace_oracle(root));
    results.push(run_neg_core04_lib_private_oracle(root));
    results.push(run_core04_lib_evidence_oracle(root));
    // ADR-257 SCENARIO-PKG
    results.push(run_core04_scen_pkg_happy_oracle(root));
    results.push(run_core04_scen_pkg_lib_fn_oracle(root));
    results.push(run_core04_scen_pkg_workspace_build_oracle(root));
    results.push(run_neg_core04_scen_pkg_private_oracle(root));
    // ADR-258 REF-PKG (closes Core 0.4)
    results.push(run_core04_ref_lib_build_oracle(root));
    results.push(run_core04_ref_cli_happy_oracle(root));
    results.push(run_core04_ref_cli_edge_oracle(root));
    results.push(run_core04_ref_scenario_pkg_oracle(root));
    results.push(run_core04_ref_evidence_oracle(root));
    results.push(run_neg_core04_ref_private_oracle(root));
    // ADR-260 INSERT-FALLIBLE (Core 0.5 slice 1)
    results.push(run_core05_insert_ok_oracle(root));
    results.push(run_core05_insert_err_oob_oracle(root));
    results.push(run_neg_core05_insert_neg_lit_oracle(root));
    results.push(run_core05_insert_emit_ban_oracle(root));
    results.push(run_neg_core05_insert_was_e0206_oracle(root));
    // ADR-261 INDEX-SUGAR (Core 0.5 slice 2)
    results.push(run_core05_index_sugar_some_oracle(root));
    results.push(run_core05_index_sugar_none_oracle(root));
    results.push(run_core05_index_sugar_eq_get_oracle(root));
    results.push(run_core05_index_emit_ban_oracle(root));
    results.push(run_neg_core05_index_mut_oracle(root));
    // ADR-262 SCENARIO-COLL (Core 0.5 slice 3)
    results.push(run_core05_scen_coll_happy_oracle(root));
    results.push(run_core05_scen_coll_oob_oracle(root));
    results.push(run_neg_core05_scen_coll_index_mut_oracle(root));
    results.push(run_neg_core05_scen_coll_insert_neg_oracle(root));
    results.push(run_core05_scen_coll_build_oracle(root));
    // ADR-263 REF-COLL (Core 0.5 slice 4 — closes vertical after Measure; NOT CLOSED here)
    results.push(run_core05_ref_coll_build_oracle(root));
    results.push(run_core05_ref_coll_cli_happy_oracle(root));
    results.push(run_core05_ref_coll_cli_oob_oracle(root));
    results.push(run_core05_ref_coll_scenario_oracle(root));
    results.push(run_core05_ref_coll_evidence_oracle(root));
    results.push(run_core05_ref_coll_emit_ban_oracle(root));
    results.push(run_neg_core05_ref_coll_index_mut_oracle(root));
    // ADR-265 SET-FALLIBLE (Core 0.6 slice 1) — CUT CORE-0.6-SET-FALLIBLE-20260926
    results.push(run_core06_set_ok_oracle(root));
    results.push(run_core06_set_oob_oracle(root));
    results.push(run_neg_core06_set_neg_lit_oracle(root));
    results.push(run_core06_set_emit_ban_oracle(root));
    results.push(run_neg_core06_index_mut_assign_oracle(root));
    // ADR-266 MAP-INDEX (Core 0.6 slice 2) — CUT CORE-0.6-MAP-INDEX-20260926
    results.push(run_core06_map_index_some_oracle(root));
    results.push(run_core06_map_index_none_oracle(root));
    results.push(run_core06_map_index_eq_get_oracle(root));
    results.push(run_core06_map_emit_ban_oracle(root));
    results.push(run_neg_core06_map_index_mut_oracle(root));
    // ADR-267 SCENARIO-GP (Core 0.6 slice 3) — CUT CORE-0.6-SCENARIO-GP-20260926
    results.push(run_core06_scen_gp_happy_oracle(root));
    results.push(run_core06_scen_gp_oob_oracle(root));
    results.push(run_neg_core06_scen_gp_index_mut_oracle(root));
    results.push(run_neg_core06_scen_gp_set_neg_oracle(root));
    results.push(run_core06_scen_gp_build_oracle(root));
    // ADR-268 REF-GP (Core 0.6 slice 4 — closes vertical after Measure; NOT CLOSED here)
    results.push(run_core06_ref_gp_build_oracle(root));
    results.push(run_core06_ref_gp_cli_happy_oracle(root));
    results.push(run_core06_ref_gp_cli_oob_oracle(root));
    results.push(run_core06_ref_gp_scenario_oracle(root));
    results.push(run_core06_ref_gp_evidence_oracle(root));
    results.push(run_core06_ref_gp_emit_ban_oracle(root));
    results.push(run_neg_core06_ref_gp_index_mut_oracle(root));
    results.push(run_neg_core06_ref_gp_set_neg_oracle(root));
    // ADR-272 IO-PARSE-H1 (Core 0.7 slice 1) — CUT CORE-0.7-IO-PARSE-H1-20260926
    results.push(run_core07_io_h1_read_ok_oracle(root));
    results.push(run_core07_io_h1_read_err_oracle(root));
    results.push(run_neg_core07_io_h1_discard_oracle(root));
    results.push(run_neg_core07_io_h1_default_oracle(root));
    results.push(run_core07_io_h1_emit_ban_oracle(root));
    // ADR-273 CLI-ARGV-H2 (Core 0.7 slice 2) — CUT CORE-0.7-CLI-ARGV-H2-20260926
    results.push(run_core07_io_h2_arg_ok_oracle(root));
    results.push(run_core07_io_h2_arg_miss_oracle(root));
    results.push(run_core07_io_h2_json_miss_oracle(root));
    results.push(run_neg_core07_io_h2_default_oracle(root));
    results.push(run_neg_core07_io_h2_index_oracle(root));
    results.push(run_core07_io_h2_emit_ban_oracle(root));
    // ADR-274 SCENARIO-IO (Core 0.7 slice 3) — CUT CORE-0.7-SCENARIO-IO-20260926
    results.push(run_core07_scen_io_happy_oracle(root));
    results.push(run_core07_scen_io_fail_oracle(root));
    results.push(run_neg_core07_scen_io_h1_oracle(root));
    results.push(run_neg_core07_scen_io_h2_oracle(root));
    results.push(run_core07_scen_io_build_oracle(root));
    // ADR-275 REF-IO (Core 0.7 slice 4 — closes vertical after Measure; NOT CLOSED here)
    results.push(run_core07_ref_io_build_oracle(root));
    results.push(run_core07_ref_io_cli_happy_oracle(root));
    results.push(run_core07_ref_io_cli_fail_oracle(root));
    results.push(run_core07_ref_io_scenario_oracle(root));
    results.push(run_core07_ref_io_evidence_oracle(root));
    results.push(run_core07_ref_io_emit_ban_oracle(root));
    results.push(run_neg_core07_ref_io_h1_oracle(root));
    results.push(run_neg_core07_ref_io_h2_oracle(root));
    // ADR-277 FN-RESULT (Core 0.8 slice 1) — CUT CORE-0.8-FN-RESULT-20260926
    results.push(run_core08_fn_result_ok_oracle(root));
    results.push(run_core08_fn_result_err_oracle(root));
    results.push(run_core08_fn_result_main_io_oracle(root));
    results.push(run_neg_core08_fn_result_theater_oracle(root));
    results.push(run_neg_core08_fn_result_unwrap_oracle(root));
    results.push(run_core08_fn_result_emit_ban_oracle(root));
    // ADR-278 QMARK (Core 0.8 slice 2) — CUT CORE-0.8-QMARK-20260926
    results.push(run_core08_qmark_chain_oracle(root));
    results.push(run_core08_qmark_err_oracle(root));
    results.push(run_core08_qmark_main_io_match_oracle(root));
    results.push(run_neg_core08_qmark_outside_oracle(root));
    results.push(run_neg_core08_qmark_option_oracle(root));
    results.push(run_core08_qmark_emit_ban_oracle(root));
    // ADR-279 SCENARIO-ERRPROP (Core 0.8 slice 3) — CUT CORE-0.8-SCENARIO-ERRPROP-20260926
    results.push(run_core08_scen_errprop_happy_oracle(root));
    results.push(run_core08_scen_errprop_err_oracle(root));
    results.push(run_core08_scen_errprop_host238_oracle(root));
    results.push(run_neg_core08_scen_qmark_outside_oracle(root));
    results.push(run_neg_core08_scen_unwrap_theater_oracle(root));
    results.push(run_core08_scen_errprop_build_oracle(root));
    // ADR-280 REF-ERRPROP (Core 0.8 slice 4) — CUT CORE-0.8-REF-ERRPROP-20260926
    results.push(run_core08_ref_errprop_build_oracle(root));
    results.push(run_core08_ref_errprop_cli_happy_oracle(root));
    results.push(run_core08_ref_errprop_cli_fail_oracle(root));
    results.push(run_core08_ref_errprop_scenario_oracle(root));
    results.push(run_core08_ref_errprop_evidence_oracle(root));
    results.push(run_core08_ref_errprop_emit_ban_oracle(root));
    results.push(run_neg_core08_ref_qmark_outside_oracle(root));
    results.push(run_neg_core08_ref_fn_theater_oracle(root));
    // ADR-282 MAP-ASSIGN (Core 0.9 slice 1) — CUT CORE-0.9-MAP-ASSIGN-20260926
    results.push(run_core09_map_assign_insert_oracle(root));
    results.push(run_core09_map_assign_overwrite_oracle(root));
    results.push(run_core09_map_assign_eq_put_oracle(root));
    results.push(run_core09_map_assign_fn_result_oracle(root));
    results.push(run_core09_map_assign_self_ref_oracle(root));
    results.push(run_neg_core09_map_assign_non_mut_oracle(root));
    results.push(run_neg_core09_map_assign_shared_loan_oracle(root));
    results.push(run_neg_core09_map_assign_live_loan_oracle(root));
    results.push(run_neg_core09_map_assign_compound_oracle(root));
    results.push(run_neg_core09_map_assign_unwrap_oracle(root));
    results.push(run_core09_map_assign_emit_ban_oracle(root));
    results.push(run_core09_map_assign_emit_clippy_oracle(root));
    results.push(run_neg_core09_map_assign_type_oracle(root));
    results.push(run_core09_map_assign_put_emit_unchanged_oracle(root));
    // ADR-283 VEC-ASSIGN (CORE-0.9-VEC-ASSIGN-20260926) — B1–B4 wired by Measure.
    results.push(run_core09_vec_assign_ok_oracle(root));
    results.push(run_core09_vec_assign_overwrite_oracle(root));
    results.push(run_core09_vec_assign_eq_set_oracle(root));
    results.push(run_core09_vec_assign_fn_result_oracle(root));
    results.push(run_core09_vec_assign_self_ref_oracle(root));
    results.push(run_core09_vec_assign_list_oracle(root));
    results.push(run_core09_vec_assign_map_in_if_oracle(root));
    results.push(run_core09_vec_assign_qmark_chain_oracle(root));
    results.push(run_neg_core09_vec_assign_outside_main_oracle(root));
    results.push(run_neg_core09_vec_assign_outside_io_helper_oracle(root));
    results.push(run_neg_core09_vec_assign_outside_fn_oracle(root));
    results.push(run_neg_core09_vec_assign_neg_lit_oracle(root));
    results.push(run_neg_core09_vec_assign_neg_lit_vs_set_oracle(root));
    results.push(run_neg_core09_vec_assign_compound_oracle(root));
    results.push(run_neg_core09_vec_assign_err_type_oracle(root));
    results.push(run_neg_core09_vec_assign_non_mut_oracle(root));
    results.push(run_neg_core09_vec_assign_unwrap_oracle(root));
    results.push(run_neg_core09_vec_assign_bad_return_oracle(root));
    results.push(run_neg_core09_vec_assign_err_swallow_oracle(root));
    results.push(run_neg_core09_vec_assign_qmark_chain_oracle(root));
    results.push(run_neg_core09_vec_assign_qmark_chain_rev_oracle(root));
    results.push(run_core09_vec_assign_emit_ban_oracle(root));
    results.push(run_core09_vec_assign_emit_no_as_usize_oracle(root));
    results.push(run_core09_vec_assign_emit_no_deferred_shape_oracle(root));
    // ADR-284 SCENARIO-MUT (Core 0.9 slice 3) — CUT CORE-0.9-SCENARIO-MUT-20260926
    results.push(run_core09_scen_mut_happy_oracle(root));
    results.push(run_core09_scen_mut_err_oracle(root));
    results.push(run_core09_scen_mut_chain_oracle(root));
    results.push(run_core09_scen_mut_map_main_oracle(root));
    results.push(run_neg_core09_scen_vec_assign_outside_oracle(root));
    results.push(run_neg_core09_scen_map_assign_non_mut_oracle(root));
    results.push(run_neg_core09_scen_unwrap_theater_oracle(root));
    results.push(run_core09_scen_mut_emit_ban_oracle(root));
    results.push(run_core09_scen_mut_build_oracle(root));
    // ADR-285 REF-MUT (Core 0.9 slice 4) — CUT CORE-0.9-REF-MUT-20260926
    results.push(run_core09_ref_mut_build_oracle(root));
    results.push(run_core09_ref_mut_cli_happy_oracle(root));
    results.push(run_core09_ref_mut_cli_oob_oracle(root));
    results.push(run_core09_ref_mut_scenario_oracle(root));
    results.push(run_core09_ref_mut_evidence_oracle(root));
    results.push(run_core09_ref_mut_emit_ban_oracle(root));
    results.push(run_neg_core09_ref_vec_assign_outside_oracle(root));
    results.push(run_neg_core09_ref_map_assign_non_mut_oracle(root));
    results.push(run_neg_core09_ref_vec_assign_neg_lit_oracle(root));
    // ADR-286 DEAD-SINK (Core 0.10 errores fase 1, slice 1) — CUT CORE-0.10-DEAD-SINK-20260927
    results.push(run_neg_core10_dead_sink_oracle(root));
    results.push(run_neg_core10_dead_sink_underscore_oracle(root));
    results.push(run_neg_core10_dead_sink_transitive_oracle(root));
    results.push(run_neg_core10_dead_sink_tautology_oracle(root));
    results.push(run_core10_err_mapped_oracle(root));
    results.push(run_core10_err_used_oracle(root));
    // ADR-286 JOIN-SAFETY (Core 0.10 errores fase 1, slice 1b) — CUT CORE-0.10-JOIN-SAFETY-20260927
    results.push(run_neg_core10_spawn_result_oracle(root));
    results.push(run_neg_core10_join_spawn_inline_oracle(root));
    results.push(run_neg_core10_user_fn_join_oracle(root));
    results.push(run_core10_spawn_iounit_oracle(root));
    // ADR-287 PKG-MEMBER (PM-1..PM-7) — CUT CORE-PKG-MEMBER-20261001
    results.extend(PM_ORACLES.iter().map(|spec| core10_pkg_member(root, spec)));
    // ADR-288 EMIT-CLIPPY-B282 (B-282-1 / B-282-2) — CUT CORE-EMIT-CLIPPY-B282-20261001
    results.extend(
        B282_ORACLES
            .iter()
            .map(|spec| core09_b282_emit_clippy(root, spec)),
    );
    // ADR-289 UNUSED-PARENS (PU-1..PU-3) — CUT CORE-EMIT-UNUSED-PARENS-V0-20261002
    results.push(core10_pu1_corpus(root));
    results.push(core10_pu2_guards(root));
    results.push(core10_pu3_capacity(root));
    // ADR-290 INDEX-MUT slice A (IM-1..IM-6) — CUT CORE-0.10-INDEX-MUT-20261002
    results.push(core10_im1_compound_ok(root));
    results.push(core10_im2_err_propagation(root));
    results.push(core10_im3_emit_ban(root));
    results.push(core10_im4_unsupported(root));
    results.push(core10_im5_neg_rules(root));
    results.push(core10_im6_migration(root));
    // ADR-292 INT-ARITH-RUNTIME (IA-1..IA-4) — CUT CORE-0.10-INT-ARITH-RUNTIME-20261003
    results.push(core10_ia1_build(root));
    results.push(core10_ia2_release(root));
    results.push(core10_ia3_test(root));
    results.push(core10_ia4_workspace_release(root));
    // ADR-293 UNDECLARED-CALL (UC-1..UC-5) — CUT CORE-0.10-UNDECLARED-CALL-20261003 (N = 866 + 5 = 871)
    results.extend(CORE10_UC_NEGS.iter().map(|n| core10_uc_neg(root, n)));
    results.push(core10_uc_positive(root));
    // ADR-294 KNOWN-INT-SCOPE (KI-1..KI-7) — CUT CORE-0.10-KNOWN-INT-SCOPE-20261004 (N = 871 + 7 = 878)
    results.extend(CORE10_KI_POS.iter().map(|p| core10_ki_positive(root, p)));
    results.extend(CORE10_KI_NEGS.iter().map(|n| core10_ki_neg(root, n)));
    // ADR-291 MUTEX-REJECT (MX-1..MX-4) — CUT CORE-0.10-MUTEX-REJECT-20261003 (N = 878 + 4 = 882)
    results.extend(CORE10_MX_NEGS.iter().map(|n| core10_mx_neg(root, n)));
    // ADR-295 MUST-USE (MS-1..MS-7) — CUT CORE-0.10-MUST-USE-20260927 (N = 882 + 7 = 889)
    results.extend(CORE10_MS_NEGS.iter().map(|n| core10_ms_neg(root, n)));
    results.extend(CORE10_MS_POS.iter().map(|p| core10_ms_positive(root, p)));
    results.push(core10_ms_emit_ban(root));

    // ADR-028 PERF-V0 (dedicated runners; not in EJEMPLO_ORACLES default debug path)
    results.push(run_perf_release_run_oracle(root));
    results.push(run_perf_optlevel_oracle(root));
    results.push(run_perf_e0250_oracle(root));
    // ADR-029 DEPS-V0
    results.push(run_deps_tokio_bridge_oracle(root));
    results.push(run_deps_e0260_oracle(root));
    // ADR-035 / 033b HOST-BORDER-HELLO
    results.push(run_emit_ban_r0r2_oracle(root));
    results.push(run_host_bridge_hello_oracle(root));
    // ADR-245 typed HttpServer.bind + serve+client
    results.push(run_core02_http_health_oracle(root));
    results.push(run_core02_http_serve_client_oracle(root));
    results.push(run_core02_http_echo_text_oracle(root));
    results.push(run_core02_http_body_cap_oracle(root));
    results.push(run_core02_policy_body_cap_oracle(root));
    results.push(run_core02_policy_header_cap_oracle(root));
    // ADR-034 TARGETS-V0 (gated; missing target → inconclusive, does not tumble overall)
    results.push(run_target_win_gnu_oracle(root));
    results.push(run_target_linux_arm64_oracle(root));
    for o in neg {
        results.push(run_neg_oracle(root, o));
    }
    for o in logic {
        results.push(run_logic_oracle(root, o));
    }
    for o in contracts {
        results.push(crate::contract::run_contract_oracle(root, o));
    }
    let overall = combine_verdicts(&results);
    (overall, results)
}

pub fn run_measure(root: &Path) -> (Verdict, Vec<OracleResult>) {
    run_measure_with(
        root,
        EJEMPLO_ORACLES,
        NEG_ORACLES,
        LOGIC_ORACLES,
        crate::contract::CONTRACT_ORACLES,
        true,
    )
}

pub fn cmd_measure() -> ExitCode {
    let Some(root) = find_workspace_root() else {
        eprintln!("arita measure: cannot locate workspace root (Cargo.toml [workspace])");
        return ExitCode::from(2);
    };

    // Ensure build artifacts land under workspace target/
    if let Err(e) = std::env::set_current_dir(&root) {
        eprintln!("arita measure: cannot cd to {}: {e}", root.display());
        return ExitCode::from(2);
    }

    let (overall, results) = run_measure(&root);
    let json = format_measure_json(overall, &results);
    println!("{json}");
    eprintln!(
        "arita measure {}: {} oracle(s)",
        overall.as_str(),
        results.len()
    );

    match overall {
        Verdict::Accepted => ExitCode::SUCCESS,
        Verdict::Rejected => ExitCode::FAILURE,
        Verdict::Inconclusive => ExitCode::from(2),
    }
}

/// Measure a single temp .arita against expected lines (for reject tests). No clippy.
#[cfg(test)]
pub fn measure_one_source(
    name: &str,
    arita_src: &str,
    expected: &[&str],
    out_dir: &Path,
) -> OracleResult {
    let arita_path = out_dir.join(format!("{name}.arita"));
    if let Err(e) = fs::create_dir_all(out_dir) {
        return OracleResult {
            id: name.into(),
            path: arita_path.display().to_string(),
            verdict: Verdict::Inconclusive,
            detail: format!("IO error creating out_dir: {e}"),
        };
    }
    if let Err(e) = fs::write(&arita_path, arita_src) {
        return OracleResult {
            id: name.into(),
            path: arita_path.display().to_string(),
            verdict: Verdict::Inconclusive,
            detail: format!("IO error writing source: {e}"),
        };
    }

    // Parse+emit+rustc via same compile path as build, but keep artifacts in out_dir
    // by temporarily using crate::build which writes to target/arita-out — fine for tests.
    let bin = match build(arita_path.to_str().unwrap_or(name)) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: name.into(),
                path: arita_path.display().to_string(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };

    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: name.into(),
                path: arita_path.display().to_string(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary: {e}"),
            };
        }
    };

    if !run.status.success() {
        return OracleResult {
            id: name.into(),
            path: arita_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!("non-zero exit {}", run.status.code().unwrap_or(-1)),
        };
    }

    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    let exp: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
    if got == exp {
        OracleResult {
            id: name.into(),
            path: arita_path.display().to_string(),
            verdict: Verdict::Accepted,
            detail: "stdout match".into(),
        }
    } else {
        OracleResult {
            id: name.into(),
            path: arita_path.display().to_string(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={exp:?}"),
        }
    }
}

#[cfg(test)]
mod measure_tests {
    use super::*;
    use std::path::PathBuf;

    fn workspace() -> PathBuf {
        find_workspace_root().expect("workspace root for tests")
    }

    #[test]
    fn normalize_stdout_trims_final_empty() {
        assert_eq!(normalize_stdout("hello\n"), vec!["hello".to_string()]);
        assert_eq!(
            normalize_stdout("a\nb\n"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(normalize_stdout("solo"), vec!["solo".to_string()]);
    }

    #[test]
    fn combine_rejected_wins() {
        let r = vec![
            OracleResult {
                id: "a".into(),
                path: "a".into(),
                verdict: Verdict::Accepted,
                detail: String::new(),
            },
            OracleResult {
                id: "b".into(),
                path: "b".into(),
                verdict: Verdict::Rejected,
                detail: String::new(),
            },
            OracleResult {
                id: "c".into(),
                path: "c".into(),
                verdict: Verdict::Inconclusive,
                detail: String::new(),
            },
        ];
        assert_eq!(combine_verdicts(&r), Verdict::Rejected);
    }

    #[test]
    fn combine_inconclusive_not_accepted() {
        let r = vec![
            OracleResult {
                id: "a".into(),
                path: "a".into(),
                verdict: Verdict::Accepted,
                detail: String::new(),
            },
            OracleResult {
                id: "b".into(),
                path: "b".into(),
                verdict: Verdict::Inconclusive,
                detail: String::new(),
            },
        ];
        assert_eq!(combine_verdicts(&r), Verdict::Inconclusive);
    }

    #[test]
    fn combine_gated_target_inconclusive_does_not_tumble() {
        // ADR-034: gated target-* inconclusive must not tumble overall (≠ fake PASS).
        let r = vec![
            OracleResult {
                id: "01-hello".into(),
                path: "ejemplos/01-hello.arita".into(),
                verdict: Verdict::Accepted,
                detail: String::new(),
            },
            OracleResult {
                id: "target-win-gnu".into(),
                path: "--target x86_64-pc-windows-gnu".into(),
                verdict: Verdict::Inconclusive,
                detail: "rustup target missing".into(),
            },
            OracleResult {
                id: "target-linux-arm64".into(),
                path: "--target aarch64-unknown-linux-gnu".into(),
                verdict: Verdict::Inconclusive,
                detail: "rustup target missing".into(),
            },
        ];
        assert_eq!(combine_verdicts(&r), Verdict::Accepted);
    }

    #[test]
    fn combine_gated_target_rejected_still_rejects() {
        let r = vec![
            OracleResult {
                id: "01-hello".into(),
                path: "a".into(),
                verdict: Verdict::Accepted,
                detail: String::new(),
            },
            OracleResult {
                id: "target-win-gnu".into(),
                path: "--target x86_64-pc-windows-gnu".into(),
                verdict: Verdict::Rejected,
                detail: "real build failure".into(),
            },
        ];
        assert_eq!(combine_verdicts(&r), Verdict::Rejected);
    }

    #[test]
    fn target_oracles_never_silently_accepted_when_missing() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        for (id, run) in [
            ("target-win-gnu", run_target_win_gnu_oracle(&root)),
            ("target-linux-arm64", run_target_linux_arm64_oracle(&root)),
        ] {
            assert_eq!(run.id, id);
            match run.verdict {
                Verdict::Inconclusive => {
                    assert!(
                        run.detail.contains("not installed")
                            || run.detail.contains("unavailable")
                            || run.detail.contains("gated"),
                        "inconclusive detail must explain gate: {}",
                        run.detail
                    );
                }
                Verdict::Accepted | Verdict::Rejected => {
                    // Target present on this host — must be a real verdict, never silent PASS-from-skip.
                    assert!(
                        !run.detail.is_empty(),
                        "target present must produce real detail"
                    );
                }
            }
        }
    }

    #[test]
    fn real_hello_oracle_accepted() {
        let root = workspace();
        // Run from workspace so target/arita-out is correct
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = &EJEMPLO_ORACLES[0];
        assert_eq!(o.id, "01-hello");
        let r = run_ejemplo_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "hello must be accepted with real rustc; detail={}",
            r.detail
        );
    }

    #[test]
    fn deliberate_stdout_mismatch_rejected() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let out = root.join("target/arita-measure-test");
        // Program prints "hello" but we expect wrong string → rejected
        let src = r#"module measure_wrong
fn main() -> Io<()> {
  print("hello")
}
"#;
        let r = measure_one_source("measure_wrong", src, &["NOT_HELLO"], &out);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "mismatch must be rejected, never accepted; detail={}",
            r.detail
        );
    }

    #[test]
    fn missing_oracle_file_inconclusive() {
        let root = workspace();
        let fake = EjemploOracle {
            id: "missing-x",
            path: "ejemplos/__does_not_exist_measure_v0__.arita",
            expected: &["x"],
            require_test: false,
        };
        let r = run_ejemplo_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Inconclusive,
            "missing file must be inconclusive, never accepted; detail={}",
            r.detail
        );
    }

    #[test]
    fn clippy_oracle_never_silently_accepted_when_missing() {
        let root = workspace();
        let r = run_clippy_oracle(&root);
        // On this box clippy may be missing → inconclusive; if present must be accepted|rejected.
        assert_ne!(r.verdict.as_str(), "pass", "no fake pass vocabulary");
        if r.detail.contains("not available") || r.detail.contains("cannot spawn") {
            assert_eq!(r.verdict, Verdict::Inconclusive);
        } else {
            assert!(
                matches!(r.verdict, Verdict::Accepted | Verdict::Rejected),
                "clippy ran: {:?}",
                r
            );
        }
    }

    #[test]
    fn miri_oracle_never_silently_accepted_when_missing() {
        let root = workspace();
        let r = run_miri_oracle(&root);
        // On this box miri/nightly may be missing → inconclusive; if present must be accepted|rejected.
        assert_ne!(r.verdict.as_str(), "pass", "no fake pass vocabulary");
        if r.detail.contains("not available")
            || r.detail.contains("cannot spawn")
            || r.detail.contains("cannot run")
        {
            assert_eq!(r.verdict, Verdict::Inconclusive);
        } else {
            assert!(
                matches!(r.verdict, Verdict::Accepted | Verdict::Rejected),
                "miri ran: {:?}",
                r
            );
        }
    }

    #[test]
    fn neg_e0210_oracle_accepted() {
        let root = workspace();
        let o = &NEG_ORACLES[0];
        assert_eq!(o.id, "neg-e0210-todo");
        assert_eq!(o.code, "E0210");
        let r = run_neg_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "neg e0210 must be accepted when parse_lower_check fails with E0210; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0210"), "detail={}", r.detail);
    }

    #[test]
    fn neg_e0216_oracle_accepted() {
        let root = workspace();
        let o = NEG_ORACLES
            .iter()
            .find(|o| o.id == "neg-e0216-div0")
            .expect("neg-e0216-div0 in NEG_ORACLES");
        assert_eq!(o.code, "E0216");
        let r = run_neg_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "neg e0216 must be accepted when parse_lower_check fails with E0216; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0216"), "detail={}", r.detail);
        assert!(
            !r.detail.contains("E0100"),
            "must not rely on rustc E0100; detail={}",
            r.detail
        );
    }

    #[test]
    fn neg_e0217_oracle_accepted() {
        let root = workspace();
        for id in [
            "neg-e0217-add-overflow",
            "neg-e0217-sub-overflow",
            "neg-e0217-mul-overflow",
        ] {
            let o = NEG_ORACLES
                .iter()
                .find(|o| o.id == id)
                .unwrap_or_else(|| unreachable!("{id} in NEG_ORACLES"));
            assert_eq!(o.code, "E0217");
            let r = run_neg_oracle(&root, o);
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{id} must be accepted when parse_lower_check fails with E0217; detail={}",
                r.detail
            );
            assert!(r.detail.contains("E0217"), "{id} detail={}", r.detail);
            assert!(
                !r.detail.contains("E0100"),
                "{id} must not rely on rustc E0100; detail={}",
                r.detail
            );
            assert!(
                !r.detail.contains("E0216"),
                "{id} must not dilute E0216; detail={}",
                r.detail
            );
        }
    }

    #[test]
    fn neg_accidental_parse_ok_rejected() {
        let root = workspace();
        // A valid F1 program should pass parse_lower_check — expecting E0210 must reject the measure oracle.
        let out = root.join("target/arita-measure-test");
        let _ = fs::create_dir_all(&out);
        let path = out.join("neg_should_fail_but_ok.arita");
        fs::write(
            &path,
            r#"module neg_ok
fn main() -> Io<()> {
  print("hello")
}
"#,
        )
        .expect("write temp");
        // Use a relative path from workspace via an absolute NegOracle path string
        // by pointing at the temp file under root.
        let rel = path
            .strip_prefix(&root)
            .expect("temp under root")
            .to_string_lossy()
            .into_owned();
        // Leak for 'static NegOracle fields used only in this test.
        let rel_static: &'static str = Box::leak(rel.into_boxed_str());
        let fake = NegOracle {
            id: "neg-accidental-ok",
            path: rel_static,
            code: "E0210",
        };
        let r = run_neg_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "accidental success must reject measure oracle; detail={}",
            r.detail
        );
        assert!(r.detail.contains("succeeded"), "detail={}", r.detail);
    }

    #[test]
    fn neg_wrong_code_rejected() {
        let root = workspace();
        // Real E0210 file, but oracle expects E0211 → rejected (wrong code).
        let fake = NegOracle {
            id: "neg-wrong-code",
            path: "ejemplos/f2/neg/e0210-todo.arita",
            code: "E0211",
        };
        let r = run_neg_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "wrong E0xxx must reject; detail={}",
            r.detail
        );
        assert!(
            r.detail.contains("without expected E0211"),
            "detail={}",
            r.detail
        );
    }

    #[test]
    fn missing_neg_oracle_inconclusive() {
        let root = workspace();
        let fake = NegOracle {
            id: "neg-missing",
            path: "ejemplos/f2/neg/__does_not_exist__.arita",
            code: "E0210",
        };
        let r = run_neg_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Inconclusive,
            "missing neg file must be inconclusive; detail={}",
            r.detail
        );
    }

    #[test]
    fn neg_e0201_oracle_accepted() {
        let root = workspace();
        let o = NEG_ORACLES
            .iter()
            .find(|o| o.id == "neg-e0201-use-after-move")
            .expect("neg-e0201 in NEG_ORACLES");
        assert_eq!(o.code, "E0201");
        let r = run_neg_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "neg e0201 must be accepted when parse_lower_check fails with E0201; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0201"), "detail={}", r.detail);
    }

    #[test]
    fn neg_e0202_oracle_accepted() {
        let root = workspace();
        let o = NEG_ORACLES
            .iter()
            .find(|o| o.id == "neg-e0202-double-mut")
            .expect("neg-e0202 in NEG_ORACLES");
        assert_eq!(o.code, "E0202");
        let r = run_neg_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "neg e0202 must be accepted when parse_lower_check fails with E0202; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0202"), "detail={}", r.detail);
    }

    #[test]
    fn f3_01_path_ok_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-01-path-ok")
            .expect("f3-01 in LOGIC_ORACLES");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-01 must be accepted when engine sat; detail={}",
            r.detail
        );
    }

    #[test]
    fn f3_02_path_fail_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-02-path-fail")
            .expect("f3-02 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0301");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-02 must be accepted when engine rejects with E0301; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0301"), "detail={}", r.detail);
    }

    #[test]
    fn f3_03_ancestor_ok_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-03-ancestor-ok")
            .expect("f3-03 in LOGIC_ORACLES");
        assert!(o.expect_sat);
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-03 must be accepted when engine sat; detail={}",
            r.detail
        );
    }

    #[test]
    fn f3_04_sibling_fail_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-04-sibling-fail")
            .expect("f3-04 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0301");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-04 must be accepted when engine rejects with E0301; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0301"), "detail={}", r.detail);
    }

    #[test]
    fn f3_09_twohop_ok_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-09-twohop-ok")
            .expect("f3-09 in LOGIC_ORACLES");
        assert!(o.expect_sat);
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-09 must be accepted when engine sat; detail={}",
            r.detail
        );
    }

    #[test]
    fn f3_10_cycle_reach_ok_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-10-cycle-reach-ok")
            .expect("f3-10 in LOGIC_ORACLES");
        assert!(o.expect_sat);
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-10 must be accepted when engine sat; detail={}",
            r.detail
        );
    }

    #[test]
    fn f3_05_edge_ok_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-05-edge-ok")
            .expect("f3-05 in LOGIC_ORACLES");
        assert!(o.expect_sat);
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-05 must be accepted when engine sat; detail={}",
            r.detail
        );
    }

    #[test]
    fn f3_06_unknown_pred_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-06-unknown-pred")
            .expect("f3-06 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0303");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-06 must be accepted when engine rejects with E0303; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0303"), "detail={}", r.detail);
    }

    #[test]
    fn f3_07_forbidden_fn_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-07-forbidden-fn")
            .expect("f3-07 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0304");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-07 must be accepted when engine rejects with E0304; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0304"), "detail={}", r.detail);
    }

    #[test]
    fn f3_08_arity_mismatch_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-08-arity-mismatch")
            .expect("f3-08 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0303");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-08 must be accepted when engine rejects with E0303; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0303"), "detail={}", r.detail);
    }

    #[test]
    fn f3_11_multi_query_fail_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-11-multi-query-fail")
            .expect("f3-11 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0301");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-11 must be accepted when any query fails with E0301 (anti partial-PASS); detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0301"), "detail={}", r.detail);
    }

    #[test]
    fn f3_12_join_miss_fail_oracle_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        let o = LOGIC_ORACLES
            .iter()
            .find(|o| o.id == "f3-12-join-miss-fail")
            .expect("f3-12 in LOGIC_ORACLES");
        assert!(!o.expect_sat);
        assert_eq!(o.code, "E0301");
        let r = run_logic_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "f3-12 must be accepted when join miss rejects with E0301; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0301"), "detail={}", r.detail);
    }

    #[test]
    fn missing_logic_oracle_inconclusive() {
        let root = workspace();
        let fake = LogicOracle {
            id: "f3-missing",
            path: "ejemplos/f3/__does_not_exist__.arita",
            expect_sat: true,
            code: "E0301",
        };
        let r = run_logic_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Inconclusive,
            "missing logic file must be inconclusive; detail={}",
            r.detail
        );
    }

    #[test]
    fn logic_expect_fail_but_sat_rejected() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd workspace");
        // Point expect_sat=false at the OK file → accidental success → rejected
        let fake = LogicOracle {
            id: "f3-accidental-sat",
            path: "ejemplos/f3/01-path-ok.arita",
            expect_sat: false,
            code: "E0301",
        };
        let r = run_logic_oracle(&root, &fake);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "accidental sat must reject neg-style logic oracle; detail={}",
            r.detail
        );
    }
}

#[cfg(test)]
mod adr263_ref_coll_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn core05_ref_coll_oracles_accept() {
        let root = workspace_root();
        let ids = [
            run_core05_ref_coll_build_oracle(&root),
            run_core05_ref_coll_cli_happy_oracle(&root),
            run_core05_ref_coll_cli_oob_oracle(&root),
            run_core05_ref_coll_scenario_oracle(&root),
            run_core05_ref_coll_evidence_oracle(&root),
            run_core05_ref_coll_emit_ban_oracle(&root),
            run_neg_core05_ref_coll_index_mut_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        // ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): Vec `v[i] = x` in main → exact E0344.
        let m1 = ids
            .iter()
            .find(|r| r.detail.contains("M1 ADR-283"))
            .unwrap_or_else(|| unreachable!("adr263_ref_coll_smoke: M1 E0344 oracle missing"));
        assert!(
            m1.detail
                .contains("E0344: index assign outside result fn @")
                && !m1.detail.contains("E0314"),
            "M1 expects exact E0344, got: {}",
            m1.detail
        );
    }
}

#[cfg(test)]
mod adr265_set_fallible_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.6-SET-FALLIBLE-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core06_set_fallible_oracles_accept() {
        let root = workspace_root();
        let ids = [
            run_core06_set_ok_oracle(&root),
            run_core06_set_oob_oracle(&root),
            run_neg_core06_set_neg_lit_oracle(&root),
            run_core06_set_emit_ban_oracle(&root),
            run_neg_core06_index_mut_assign_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        // ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): Vec `v[i] = x` in main → exact E0344.
        let m1 = ids
            .iter()
            .find(|r| r.detail.contains("M1 ADR-283"))
            .unwrap_or_else(|| unreachable!("adr265_set_fallible_smoke: M1 E0344 oracle missing"));
        assert!(
            m1.detail
                .contains("E0344: index assign outside result fn @")
                && !m1.detail.contains("E0314"),
            "M1 expects exact E0344, got: {}",
            m1.detail
        );
    }
}

#[cfg(test)]
mod adr266_map_index_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.6-MAP-INDEX-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core06_map_index_oracles_accept() {
        let root = workspace_root();
        let ids = [
            run_core06_map_index_some_oracle(&root),
            run_core06_map_index_none_oracle(&root),
            run_core06_map_index_eq_get_oracle(&root),
            run_core06_map_emit_ban_oracle(&root),
            run_neg_core06_map_index_mut_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        // ADR-282 M1 retarget (CORE-0.9-MAP-ASSIGN-20260926): neg-core06-map-index-mut now
        // pins non-mut `m[k] = v` → E0202 (was ADR-266 E0314).
        let neg = &ids[4];
        assert_eq!(neg.id, "neg-core06-map-index-mut");
        assert!(
            neg.detail.contains("E0202") && !neg.detail.contains("E0314"),
            "M1 retarget expects E0202, got: {}",
            neg.detail
        );
    }
}

#[cfg(test)]
mod adr267_scenario_gp_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.6-SCENARIO-GP-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core06_scen_gp_oracles_accept() {
        let root = workspace_root();
        let ids = [
            run_core06_scen_gp_happy_oracle(&root),
            run_core06_scen_gp_oob_oracle(&root),
            run_neg_core06_scen_gp_index_mut_oracle(&root),
            run_neg_core06_scen_gp_set_neg_oracle(&root),
            run_core06_scen_gp_build_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        // ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): Vec `v[i] = x` in main → exact E0344.
        let m1 = ids
            .iter()
            .find(|r| r.detail.contains("M1 ADR-283"))
            .unwrap_or_else(|| unreachable!("adr267_scenario_gp_smoke: M1 E0344 oracle missing"));
        assert!(
            m1.detail
                .contains("E0344: index assign outside result fn @")
                && !m1.detail.contains("E0314"),
            "M1 expects exact E0344, got: {}",
            m1.detail
        );
    }
}

#[cfg(test)]
mod adr268_ref_gp_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.6-REF-GP-20260926 — local accept; measure_pass:true after Core 0.6 CLOSED.
    #[test]
    fn core06_ref_gp_oracles_accept() {
        let root = workspace_root();
        let ids = [
            run_core06_ref_gp_build_oracle(&root),
            run_core06_ref_gp_cli_happy_oracle(&root),
            run_core06_ref_gp_cli_oob_oracle(&root),
            run_core06_ref_gp_scenario_oracle(&root),
            run_core06_ref_gp_evidence_oracle(&root),
            run_core06_ref_gp_emit_ban_oracle(&root),
            run_neg_core06_ref_gp_index_mut_oracle(&root),
            run_neg_core06_ref_gp_set_neg_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        // ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): Vec `v[i] = x` in main → exact E0344.
        let m1 = ids
            .iter()
            .find(|r| r.detail.contains("M1 ADR-283"))
            .unwrap_or_else(|| unreachable!("adr268_ref_gp_smoke: M1 E0344 oracle missing"));
        assert!(
            m1.detail
                .contains("E0344: index assign outside result fn @")
                && !m1.detail.contains("E0314"),
            "M1 expects exact E0344, got: {}",
            m1.detail
        );
    }
}

#[cfg(test)]
mod adr272_io_parse_h1_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.7-IO-PARSE-H1-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core07_io_parse_h1_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core07_io_h1_read_ok_oracle(&root),
            run_core07_io_h1_read_err_oracle(&root),
            run_neg_core07_io_h1_discard_oracle(&root),
            run_neg_core07_io_h1_default_oracle(&root),
            run_core07_io_h1_emit_ban_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr273_cli_argv_h2_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.7-CLI-ARGV-H2-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core07_cli_argv_h2_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core07_io_h2_arg_ok_oracle(&root),
            run_core07_io_h2_arg_miss_oracle(&root),
            run_core07_io_h2_json_miss_oracle(&root),
            run_neg_core07_io_h2_default_oracle(&root),
            run_neg_core07_io_h2_index_oracle(&root),
            run_core07_io_h2_emit_ban_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr274_scenario_io_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.7-SCENARIO-IO-20260926 — local accept; measure_pass stays false until Measure.
    #[test]
    fn core07_scenario_io_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core07_scen_io_happy_oracle(&root),
            run_core07_scen_io_fail_oracle(&root),
            run_neg_core07_scen_io_h1_oracle(&root),
            run_neg_core07_scen_io_h2_oracle(&root),
            run_core07_scen_io_build_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr275_ref_io_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.7-REF-IO-20260926 — local accept; measure_pass:true after Core 0.7 CLOSED.
    #[test]
    fn core07_ref_io_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core07_ref_io_build_oracle(&root),
            run_core07_ref_io_cli_happy_oracle(&root),
            run_core07_ref_io_cli_fail_oracle(&root),
            run_core07_ref_io_scenario_oracle(&root),
            run_core07_ref_io_evidence_oracle(&root),
            run_core07_ref_io_emit_ban_oracle(&root),
            run_neg_core07_ref_io_h1_oracle(&root),
            run_neg_core07_ref_io_h2_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr277_fn_result_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.8-FN-RESULT-20260926 — §2 oracles local accept (measure_pass:false until Measure).
    /// Parser E0342 CLOSED — theater asserts Accepted.
    #[test]
    fn core08_fn_result_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core08_fn_result_ok_oracle(&root),
            run_core08_fn_result_err_oracle(&root),
            run_core08_fn_result_main_io_oracle(&root),
            run_neg_core08_fn_result_theater_oracle(&root),
            run_neg_core08_fn_result_unwrap_oracle(&root),
            run_core08_fn_result_emit_ban_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr278_qmark_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.8-QMARK-20260926 — §2 oracles local accept (measure_pass:false until Measure).
    #[test]
    fn core08_qmark_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core08_qmark_chain_oracle(&root),
            run_core08_qmark_err_oracle(&root),
            run_core08_qmark_main_io_match_oracle(&root),
            run_neg_core08_qmark_outside_oracle(&root),
            run_neg_core08_qmark_option_oracle(&root),
            run_core08_qmark_emit_ban_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr279_scen_errprop_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.8-SCENARIO-ERRPROP-20260926 — §2 oracles local accept (measure_pass:false until Ingeniero).
    #[test]
    fn core08_scen_errprop_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core08_scen_errprop_happy_oracle(&root),
            run_core08_scen_errprop_err_oracle(&root),
            run_core08_scen_errprop_host238_oracle(&root),
            run_neg_core08_scen_qmark_outside_oracle(&root),
            run_neg_core08_scen_unwrap_theater_oracle(&root),
            run_core08_scen_errprop_build_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr280_ref_errprop_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// CUT CORE-0.8-REF-ERRPROP-20260926 — §2 oracles local accept (measure_pass:true after Core 0.8 CLOSED).
    #[test]
    fn core08_ref_errprop_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core08_ref_errprop_build_oracle(&root),
            run_core08_ref_errprop_cli_happy_oracle(&root),
            run_core08_ref_errprop_cli_fail_oracle(&root),
            run_core08_ref_errprop_scenario_oracle(&root),
            run_core08_ref_errprop_evidence_oracle(&root),
            run_core08_ref_errprop_emit_ban_oracle(&root),
            run_neg_core08_ref_qmark_outside_oracle(&root),
            run_neg_core08_ref_fn_theater_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr282_map_assign_smoke {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// Exact-code discipline: OR-codes / neighbour codes never count as the pinned code.
    #[test]
    fn core09_exact_code_helpers() {
        assert_eq!(core09_diag_codes("E0202: borrow conflict"), vec!["E0202"]);
        let or_codes = core09_diag_codes("E0206: method not in F2 std whitelist; E0342: x");
        assert!(!or_codes.iter().all(|c| c == "E0206"));
        assert_eq!(
            core09_diag_head("E0006: construct outside F1.1 (parse failure):  --> 8:7\n  |"),
            "E0006: construct outside F1.1 (parse failure)"
        );
        assert_eq!(core09_count_index_assign("  m[\"a\"] = 1\n"), 1);
        assert_eq!(core09_count_index_assign("  if m[q] == 1 {"), 0);
        assert_eq!(
            core09_count_index_assign("let o: Option<i64> = m.get(&q).cloned();"),
            0
        );
        assert!(core09_has_let_discard_insert(
            "    { let _ = m.insert(\"a\".to_string(), 7); };"
        ));
        assert!(!core09_has_let_discard_insert(
            "{ let __arita_mk = k; let __arita_mv = v; m.insert(__arita_mk, __arita_mv); }"
        ));
    }

    /// CUT CORE-0.9-MAP-ASSIGN-20260926 — ADR-282 oracles local accept (measure_pass stays false
    /// until Measure GATE + Ingeniero; this is not PASS/CLOSED).
    #[test]
    fn core09_map_assign_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let ids = [
            run_core09_map_assign_insert_oracle(&root),
            run_core09_map_assign_overwrite_oracle(&root),
            run_core09_map_assign_eq_put_oracle(&root),
            run_core09_map_assign_fn_result_oracle(&root),
            run_core09_map_assign_self_ref_oracle(&root),
            run_neg_core09_map_assign_non_mut_oracle(&root),
            run_neg_core09_map_assign_shared_loan_oracle(&root),
            run_neg_core09_map_assign_live_loan_oracle(&root),
            run_neg_core09_map_assign_compound_oracle(&root),
            run_neg_core09_map_assign_unwrap_oracle(&root),
            run_core09_map_assign_emit_ban_oracle(&root),
            run_core09_map_assign_emit_clippy_oracle(&root),
            run_neg_core09_map_assign_type_oracle(&root),
            run_core09_map_assign_put_emit_unchanged_oracle(&root),
        ];
        for r in &ids {
            assert_eq!(
                r.verdict,
                Verdict::Accepted,
                "{} => {:?} detail={}",
                r.id,
                r.verdict,
                r.detail
            );
        }
        for (i, code) in [
            (5, "E0202"),
            (6, "E0202"),
            (7, "E0202"),
            (8, "E0006"),
            (9, "E0206"),
            (12, "E0203"),
        ] {
            assert!(
                ids[i].detail.contains(&format!("exact {code}")),
                "{} must pin exact {code}: {}",
                ids[i].id,
                ids[i].detail
            );
        }
    }
}

#[cfg(test)]
mod adr283_m1_vec_negs_smoke {
    //! ADR-283 M1 (CORE-0.9-VEC-ASSIGN-20260926): the 7 declared Vec-in-main negatives now pin
    //! EXACT E0344 + span (was E0314); the two neighbours stay: string E0314, map E0202.
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn neg(id: &str) -> &'static NegOracle {
        NEG_ORACLES
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| unreachable!("{id} in NEG_ORACLES"))
    }

    #[test]
    fn adr283_m1_seven_vec_negs_exact_e0344() {
        let root = workspace_root();
        let dedicated = [
            (run_neg_core05_index_mut_oracle(&root), "134..142"),
            (run_neg_core05_scen_coll_index_mut_oracle(&root), "144..152"),
            (run_neg_core05_ref_coll_index_mut_oracle(&root), "179..187"),
            (run_neg_core06_index_mut_assign_oracle(&root), "175..183"),
            (run_neg_core06_scen_gp_index_mut_oracle(&root), "149..157"),
            (run_neg_core06_ref_gp_index_mut_oracle(&root), "148..156"),
        ];
        for (r, span) in &dedicated {
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
            assert!(
                r.detail
                    .ends_with(&format!("E0344: index assign outside result fn @{span}")),
                "{} detail={}",
                r.id,
                r.detail
            );
        }
        let o = neg("neg-e0310-index-vec");
        assert_eq!(o.code, "E0344");
        let r = run_neg_oracle(&root, o);
        assert_eq!(r.verdict, Verdict::Accepted, "{}", r.detail);
        assert!(
            r.detail
                .contains("E0344: index assign outside result fn @225..233")
                && !r.detail.contains("E0314"),
            "{}",
            r.detail
        );
    }

    #[test]
    fn adr283_m1_neighbours_unchanged() {
        let root = workspace_root();
        let o = neg("neg-e0310-index-string");
        assert_eq!(o.code, "E0314");
        let r = run_neg_oracle(&root, o);
        assert_eq!(r.verdict, Verdict::Accepted, "{}", r.detail);
        assert!(!r.detail.contains("E0344"), "{}", r.detail);
        let m = run_neg_core06_map_index_mut_oracle(&root);
        assert_eq!(m.verdict, Verdict::Accepted, "{}", m.detail);
        assert!(m.detail.contains("E0202"), "{}", m.detail);
    }
}

#[cfg(test)]
mod adr283_vec_assign_smoke {
    //! CUT CORE-0.9-VEC-ASSIGN-20260926 — ADR-283 B1–B4 oracles wired by Measure. Asserts every
    //! pinned id Accepted with its EXACT code; a red pin fails here (never ignored, never re-pinned).
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn core09_vec_span_helpers() {
        let src = "// c — é\nfn main() -> Io<()> {\n  v[0] = 2\n}\n";
        let a = src.find("v[0] = 2").unwrap();
        assert_eq!(
            core09_vec_stmt_span(src, "v[0] = 2"),
            Some(format!("{}..{}", a, a + 8))
        );
        assert_eq!(core09_vec_stmt_line(src, "v[0] = 2"), Some(3));
        assert_eq!(core09_vec_stmt_span(src, "v[1] = 2"), None);
        let emit = "fn __arita_vec_set<T>(v: &mut [T], i: i64, x: T) -> Result<(), i64> {\n    let Ok(u) = usize::try_from(i) else {\n        return Err(0);\n    };\n}\nfn f() {}\n";
        let b = core09_vec_helper_body(emit, "__arita_vec_set").unwrap();
        assert!(b.contains("usize::try_from(i)") && !b.contains("fn f()"));
        assert!(core09_vec_helper_body(emit, "__arita_vec_insert").is_none());
    }

    #[test]
    fn core09_vec_assign_oracles_accept() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs = [
            run_core09_vec_assign_ok_oracle(&root),
            run_core09_vec_assign_overwrite_oracle(&root),
            run_core09_vec_assign_eq_set_oracle(&root),
            run_core09_vec_assign_fn_result_oracle(&root),
            run_core09_vec_assign_self_ref_oracle(&root),
            run_core09_vec_assign_list_oracle(&root),
            run_core09_vec_assign_map_in_if_oracle(&root),
            run_core09_vec_assign_qmark_chain_oracle(&root),
            run_neg_core09_vec_assign_outside_main_oracle(&root),
            run_neg_core09_vec_assign_outside_io_helper_oracle(&root),
            run_neg_core09_vec_assign_outside_fn_oracle(&root),
            run_neg_core09_vec_assign_neg_lit_oracle(&root),
            run_neg_core09_vec_assign_neg_lit_vs_set_oracle(&root),
            run_neg_core09_vec_assign_compound_oracle(&root),
            run_neg_core09_vec_assign_err_type_oracle(&root),
            run_neg_core09_vec_assign_non_mut_oracle(&root),
            run_neg_core09_vec_assign_unwrap_oracle(&root),
            run_neg_core09_vec_assign_bad_return_oracle(&root),
            run_core09_vec_assign_emit_ban_oracle(&root),
            run_core09_vec_assign_emit_no_as_usize_oracle(&root),
            run_core09_vec_assign_emit_no_deferred_shape_oracle(&root),
            run_neg_core09_vec_assign_err_swallow_oracle(&root),
            run_neg_core09_vec_assign_qmark_chain_oracle(&root),
            run_neg_core09_vec_assign_qmark_chain_rev_oracle(&root),
        ];
        let red: Vec<String> = rs
            .iter()
            .filter(|r| r.verdict != Verdict::Accepted)
            .map(|r| format!("{} => {:?} detail={}", r.id, r.verdict, r.detail))
            .collect();
        assert!(
            red.is_empty(),
            "{} red ADR-283 oracle(s):\n{}",
            red.len(),
            red.join("\n")
        );
        for (i, code) in [
            (8, "exact E0344"),
            (9, "exact E0344"),
            (10, "exact E0344"),
            (11, "exact E0319"),
            (12, "exact E0319"),
            (12, "exact E0203"),
            (13, "exact E0006"),
            (14, "exact E0203"),
            (15, "exact E0202"),
            (16, "exact E0206"),
            (17, "exact E0342"),
        ] {
            assert!(
                rs[i].detail.contains(code),
                "{} must pin {code}: {}",
                rs[i].id,
                rs[i].detail
            );
        }
        assert!(rs[8]
            .detail
            .contains("E0344: index assign outside result fn @"));
        for (i, code) in [
            (21, "exact E0272"),
            (22, "exact E0344"),
            (23, "exact E0343"),
        ] {
            assert!(
                rs[i].detail.contains(code),
                "{} must pin {code}: {}",
                rs[i].id,
                rs[i].detail
            );
        }
        assert!(!rs[16].detail.contains("E0342"), "{}", rs[16].detail);
    }
}

#[cfg(test)]
mod adr284_scenario_mut_smoke {
    //! CUT CORE-0.9-SCENARIO-MUT-20260926 — ADR-284 §2 oracles wired by Measure. Negs assert their
    //! EXACT code (E0344 / E0202 / E0206). The Codegen positives (+ build + emit-ban) must be
    //! Inconclusive while the READY gate is closed and Accepted once it opens. Nothing ignored,
    //! nothing re-pinned.
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn core09_scen_mut_chain_order_helper() {
        assert!(core09_scen_mut_chain_order(&lines(&["40", "err", "77", "err", "0"])).is_ok());
        // post stage ran after the OOB → tripwire 77 after the Vec failure
        let e = core09_scen_mut_chain_order(&lines(&["40", "err", "77", "err", "77"])).unwrap_err();
        assert!(e.contains("tripwire"), "{e}");
        let e = core09_scen_mut_chain_order(&lines(&["40", "err", "77", "77", "0"])).unwrap_err();
        assert!(e.contains("tripwire"), "{e}");
        // OOB did not early-return (fell through to v.get → Err(1))
        assert!(core09_scen_mut_chain_order(&lines(&["40", "err", "77", "err", "1"])).is_err());
        // control 77 missing on the Vec-ok path
        assert!(core09_scen_mut_chain_order(&lines(&["40", "40", "err", "0", "x"])).is_err());
        assert!(core09_scen_mut_chain_order(&lines(&["40", "err", "77"])).is_err());
        let chain = CORE09_SCEN_MUT_POSITIVES[2];
        assert_eq!(chain.0, "core09-scen-mut-chain");
        assert!(core09_scen_mut_chain_order(&lines(chain.2)).is_ok());
    }

    #[test]
    fn core09_scen_mut_negs_exact() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let v = run_neg_core09_scen_vec_assign_outside_oracle(&root);
        assert_eq!(v.verdict, Verdict::Accepted, "{} detail={}", v.id, v.detail);
        assert!(
            v.detail
                .contains("exact E0344: E0344: index assign outside result fn @"),
            "{}",
            v.detail
        );
        let m = run_neg_core09_scen_map_assign_non_mut_oracle(&root);
        assert_eq!(m.verdict, Verdict::Accepted, "{} detail={}", m.id, m.detail);
        assert!(m.detail.contains("exact E0202"), "{}", m.detail);
        let u = run_neg_core09_scen_unwrap_theater_oracle(&root);
        assert_eq!(CORE09_SCEN_MUT_UNWRAP_THEATER_CODE, "E0206");
        assert_eq!(u.verdict, Verdict::Accepted, "{} detail={}", u.id, u.detail);
        assert!(
            u.detail
                .contains("exact E0206: E0206: method not in F2 std whitelist"),
            "{}",
            u.detail
        );
        assert!(!u.detail.contains("E0291"), "{}", u.detail);
    }

    #[test]
    fn core09_scen_mut_positives_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs = [
            run_core09_scen_mut_happy_oracle(&root),
            run_core09_scen_mut_err_oracle(&root),
            run_core09_scen_mut_chain_oracle(&root),
            run_core09_scen_mut_map_main_oracle(&root),
            run_core09_scen_mut_emit_ban_oracle(&root),
            run_core09_scen_mut_build_oracle(&root),
        ];
        for r in &rs {
            if CORE09_SCEN_MUT_POSITIVES_READY {
                assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
            } else {
                assert_eq!(
                    r.verdict,
                    Verdict::Inconclusive,
                    "{} detail={}",
                    r.id,
                    r.detail
                );
                assert!(
                    r.detail.starts_with("not READY_FOR_MEASURE"),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
            }
        }
    }
}

#[cfg(test)]
mod adr285_ref_mut_smoke {
    //! CUT CORE-0.9-REF-MUT-20260926 — ADR-285 §2 oracles wired by Measure (sha256 856a11ca…).
    //! Negs assert their EXACT code + message (E0344 @812..820 / E0202 / E0319); the six ref
    //! positives must be Accepted on the READY tree. Nothing ignored, nothing re-pinned.
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn core09_ref_mut_source_helpers() {
        let src = "// header: v[0] = 1 ? unwrap\nfn main() -> Io<()> {\n  let mut m: Map<Text, Int> = Map::new()\n  let mut v: Vec<Int> = Vec::new()\n  m[\"a\"] = 1 // v[1] = 2 ?\n  v[0] = 3\n  if v[0] == 3 { print(\"x // not a comment\") }\n}\n\nscenario s1 {\n  acceptance \"v[9] = 9 ?\"\n}\n";
        let code = core09_ref_mut_code(src);
        assert!(!code.contains("header"), "{code}");
        assert!(!code.contains('?'), "{code}");
        assert!(!code.contains("acceptance"), "{code}");
        assert!(code.contains("x // not a comment"), "{code}");
        assert_eq!(core09_ref_mut_assign_split(&code), (1, 1));
        let sc = core09_ref_mut_scenarios(src);
        assert_eq!(sc, vec![("s1".to_string(), lines(&["v[9] = 9 ?"]))]);
        assert!(core09_ref_mut_oob_check(&lines(CORE09_REF_MUT_OOB)).is_ok());
        assert!(core09_ref_mut_oob_check(&lines(&["ref-mut-err", "1"])).is_err());
        assert!(core09_ref_mut_oob_check(&lines(&["ref-mut-err", "0", "x"])).is_err());
        let e =
            core09_ref_mut_oob_check(&lines(&["ref-mut-happy", "ref-mut-err", "0"])).unwrap_err();
        assert!(e.contains("happy token"), "{e}");
        assert_eq!(CORE09_REF_MUT_IDS.len(), 9);
    }

    #[test]
    fn core09_ref_mut_negs_exact() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let v = run_neg_core09_ref_vec_assign_outside_oracle(&root);
        assert_eq!(v.verdict, Verdict::Accepted, "{} detail={}", v.id, v.detail);
        assert!(
            v.detail
                .contains("exact E0344: E0344: index assign outside result fn @812..820"),
            "{}",
            v.detail
        );
        let m = run_neg_core09_ref_map_assign_non_mut_oracle(&root);
        assert_eq!(m.verdict, Verdict::Accepted, "{} detail={}", m.id, m.detail);
        assert!(
            m.detail.contains("exact E0202: E0202: borrow conflict"),
            "{}",
            m.detail
        );
        let n = run_neg_core09_ref_vec_assign_neg_lit_oracle(&root);
        assert_eq!(n.verdict, Verdict::Accepted, "{} detail={}", n.id, n.detail);
        assert!(
            n.detail.contains("exact E0319: E0319: negative set index"),
            "{}",
            n.detail
        );
    }

    #[test]
    fn core09_ref_mut_positives_accepted() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs = [
            run_core09_ref_mut_build_oracle(&root),
            run_core09_ref_mut_cli_happy_oracle(&root),
            run_core09_ref_mut_cli_oob_oracle(&root),
            run_core09_ref_mut_scenario_oracle(&root),
            run_core09_ref_mut_evidence_oracle(&root),
            run_core09_ref_mut_emit_ban_oracle(&root),
        ];
        for r in &rs {
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
    }
}

#[cfg(test)]
mod adr286_dead_sink_smoke {
    //! CUT CORE-0.10-DEAD-SINK-20260927 — ADR-286 S1 oracles wired by Measure (sha256 4288c47d…).
    //! Pre-READY the 4 negs must be Inconclusive (never Accepted) with the single-offender control
    //! built; once the Parser gate opens they must be Accepted with the exact E0272 head. The 2
    //! positives and the §3 migrated pins must be Accepted. Nothing ignored, nothing re-pinned.
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn core10_dead_sink_helpers() {
        let src = "// let _c: Int = e in a comment\nfn main() -> Io<()> {\n  match f() {\n    Err(e) => {\n      let _c: Int = e\n      let _d: Int = _c\n      print(\"ok\")\n    }\n  }\n}\n\nscenario s {\n  acceptance \"let _c: Int = e\"\n}\n";
        let code = core09_ref_mut_code(src);
        assert_eq!(core10_code_line_count(&code, "let _c: Int = e"), 1);
        assert_eq!(core10_code_line_count(&code, "print(\"ok\")"), 1);
        let ctl = core10_control_source(
            src,
            &[
                ("let _c: Int = e", Some("print(e)")),
                ("let _d: Int = _c", None),
            ],
        )
        .expect("control");
        assert!(ctl.contains("      print(e)\n      print(\"ok\")"), "{ctl}");
        assert!(
            ctl.starts_with("// let _c: Int = e in a comment\n"),
            "{ctl}"
        );
        assert!(!ctl.contains("let _d"), "{ctl}");
        assert!(core10_control_source(src, &[("let _x: Int = e", None)]).is_err());
        assert_eq!(
            CORE10_DEAD_SINK_NEGS.len() + CORE10_DEAD_SINK_POSITIVES.len(),
            6
        );
    }

    #[test]
    fn core10_dead_sink_negs_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs = [
            run_neg_core10_dead_sink_oracle(&root),
            run_neg_core10_dead_sink_underscore_oracle(&root),
            run_neg_core10_dead_sink_transitive_oracle(&root),
            run_neg_core10_dead_sink_tautology_oracle(&root),
        ];
        for r in &rs {
            println!("adr286-neg {} {:?} :: {}", r.id, r.verdict, r.detail);
            if CORE10_DEAD_SINK_PARSER_READY {
                assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
                assert!(
                    r.detail
                        .contains("rejected with exact E0272: E0272: result error swallowed"),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
                assert!(
                    r.detail.contains("; control 0 diagnostics"),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
            } else {
                assert_eq!(
                    r.verdict,
                    Verdict::Inconclusive,
                    "{} detail={}",
                    r.id,
                    r.detail
                );
                assert!(
                    r.detail.starts_with(
                        "not READY (Parser E0272 v2 gate, ADR-286 S1) — control build ok; control 0 diagnostics"
                    ),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
            }
        }
    }

    #[test]
    fn core10_dead_sink_positives_accepted() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        for r in [
            run_core10_err_mapped_oracle(&root),
            run_core10_err_used_oracle(&root),
        ] {
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
    }

    /// Workspace packages the ADR-286 §3 migration touches that other smokes (adr275 ref-io,
    /// adr280 ref-errprop) also build. `build_package_workspace` wipes and rebuilds
    /// `<pkg>/target/arita-pkg`, so two tests building the same package concurrently race
    /// (file lock / missing rlib). This smoke builds them from its own copy instead.
    const ADR286_ISOLATED_TREES: &[&str] =
        &["ejemplos/core07/ref-io", "ejemplos/core08/ref-errprop"];
    /// Single files the isolated ref-io / ref-errprop scenario oracles probe for presence.
    const ADR286_ISOLATED_FILES: &[&str] = &[
        "ejemplos/core07/scenario-io/01-happy.arita",
        "ejemplos/core08/scenario-errprop/01-happy.arita",
    ];

    /// Recursive copy without `target/` dirs and `.DS_Store`; returns the copied files (relative).
    fn adr286_copy_tree(src: &Path, dst: &Path, rel: &Path, out: &mut Vec<PathBuf>) {
        fs::create_dir_all(dst.join(rel)).expect("mkdir isolated tree");
        for e in fs::read_dir(src.join(rel))
            .expect("read_dir source tree")
            .flatten()
        {
            let name = e.file_name();
            if name == "target" || name == ".DS_Store" {
                continue;
            }
            let child = rel.join(&name);
            if e.file_type().expect("file type").is_dir() {
                adr286_copy_tree(src, dst, &child, out);
            } else {
                fs::copy(src.join(&child), dst.join(&child)).expect("copy isolated file");
                out.push(child);
            }
        }
    }

    /// Own root under the OS temp dir (per process): copies of the shared workspace packages,
    /// verified byte-identical (sha256) to the repo files, so this smoke measures the same
    /// sources without sharing `<pkg>/target/arita-pkg` with any other test.
    fn adr286_isolated_root(root: &Path) -> PathBuf {
        let iso = std::env::temp_dir().join(format!("arita-adr286-smoke-{}", std::process::id()));
        let _ = fs::remove_dir_all(&iso);
        let mut copied: Vec<PathBuf> = Vec::new();
        for tree in ADR286_ISOLATED_TREES {
            adr286_copy_tree(root, &iso, Path::new(tree), &mut copied);
        }
        for f in ADR286_ISOLATED_FILES {
            let dst = iso.join(f);
            fs::create_dir_all(dst.parent().expect("parent")).expect("mkdir isolated file");
            fs::copy(root.join(f), &dst).expect("copy isolated file");
            copied.push(PathBuf::from(f));
        }
        assert!(copied.len() >= 18, "isolated copy too small: {copied:?}");
        for rel in &copied {
            let a = crate::attest::sha256_file(&root.join(rel)).expect("sha repo");
            let b = crate::attest::sha256_file(&iso.join(rel)).expect("sha copy");
            assert_eq!(a, b, "isolated copy differs: {}", rel.display());
        }
        iso
    }

    #[test]
    fn adr286_migrated_pins_accepted() {
        // Every oracle over the 13 ADR-286 §3 files + the 3 re-frozen evidence (one by one).
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        // Workspace-package oracles (ref-io, ref-errprop) run on this smoke's own copy.
        let iso = adr286_isolated_root(&root);
        let isolated: &[fn(&Path) -> OracleResult] = &[
            run_core07_ref_io_build_oracle,
            run_core07_ref_io_cli_happy_oracle,
            run_core07_ref_io_cli_fail_oracle,
            run_core07_ref_io_scenario_oracle,
            run_core07_ref_io_evidence_oracle,
            run_core07_ref_io_emit_ban_oracle,
            run_core08_ref_errprop_build_oracle,
            run_core08_ref_errprop_cli_happy_oracle,
            run_core08_ref_errprop_cli_fail_oracle,
            run_core08_ref_errprop_scenario_oracle,
            run_core08_ref_errprop_evidence_oracle,
            run_core08_ref_errprop_emit_ban_oracle,
        ];
        for run in isolated {
            let r = run(&iso);
            println!("adr286-migrated {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
        let _ = fs::remove_dir_all(&iso);
        // Single-file oracles: `build()` stamps outputs per source+thread (race-free) and the
        // emit-ban oracles read `<root>/target/arita-out`, so they stay on the repo root.
        let oracles: &[fn(&Path) -> OracleResult] = &[
            run_core08_fn_result_ok_oracle,
            run_core08_fn_result_err_oracle,
            run_core08_fn_result_main_io_oracle,
            run_core08_fn_result_emit_ban_oracle,
            run_core08_qmark_chain_oracle,
            run_core08_qmark_err_oracle,
            run_core08_qmark_main_io_match_oracle,
            run_core08_qmark_emit_ban_oracle,
            run_neg_core08_qmark_option_oracle,
            run_core08_scen_errprop_happy_oracle,
            run_core08_scen_errprop_err_oracle,
            run_core08_scen_errprop_host238_oracle,
            run_core08_scen_errprop_build_oracle,
        ];
        for run in oracles {
            let r = run(&root);
            println!("adr286-migrated {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
        let q = run_neg_core08_qmark_option_oracle(&root);
        assert_eq!(
            q.detail,
            "rejected with exact E0203: E0203: type mismatch (1 diagnostic)"
        );
    }
}

#[cfg(test)]
mod adr286_join_safety_smoke {
    //! CUT CORE-0.10-JOIN-SAFETY-20260927 — ADR-286 S1b oracles wired by Measure (§0.1d, sha256
    //! 2b1c26be…). Pre-READY the 3 negs must be Inconclusive (never Accepted) with the
    //! single-offender control built; once the Parser gate opens they must be Accepted with the
    //! exact code (E0203 / E0007). The positive must be Accepted. Nothing ignored or re-pinned.
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn core10_join_safety_inventory() {
        assert_eq!(
            CORE10_JOIN_SAFETY_NEGS.len() + CORE10_JOIN_SAFETY_POSITIVES.len(),
            4
        );
        let codes: Vec<&str> = CORE10_JOIN_SAFETY_NEGS.iter().map(|n| n.2).collect();
        assert_eq!(codes, ["E0203", "E0203", "E0007"]);
    }

    #[test]
    fn core10_join_safety_negs_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs = [
            run_neg_core10_spawn_result_oracle(&root),
            run_neg_core10_join_spawn_inline_oracle(&root),
            run_neg_core10_user_fn_join_oracle(&root),
        ];
        for r in &rs {
            println!("adr286-s1b-neg {} {:?} :: {}", r.id, r.verdict, r.detail);
            if CORE10_JOIN_SAFETY_PARSER_READY {
                assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
                assert!(
                    r.detail.contains("; control 0 diagnostics"),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
            } else {
                assert_eq!(
                    r.verdict,
                    Verdict::Inconclusive,
                    "{} detail={}",
                    r.id,
                    r.detail
                );
                assert!(
                    r.detail.starts_with(
                        "not READY (Parser S1b gate, ADR-286 §0.1d) — control build ok; control 0 diagnostics"
                    ),
                    "{} detail={}",
                    r.id,
                    r.detail
                );
            }
        }
    }

    #[test]
    fn core10_join_safety_positive_accepted() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let r = run_core10_spawn_iounit_oracle(&root);
        assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
    }
}

#[cfg(test)]
mod adr287_pkg_member_smoke {
    //! CUT CORE-PKG-MEMBER-20261001 — ADR-287 PM-1..PM-7 wired by Measure (§5, v0.3 sha256
    //! b80cec57…). No build here: cargo test is not the `arita` binary, so the 7 oracles must be
    //! Inconclusive (never Accepted, skip ≠ PASS); the expectation checks and the digests are
    //! exercised on synthetic data. The real PM verdicts (accepted / rejected) come from
    //! `arita measure` (dry / exclusive run). There is no gating flag.
    use super::*;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn run(code: Option<i32>, out: &str, err: &str) -> PmRun {
        PmRun {
            code,
            stdout: out.into(),
            stderr: err.into(),
        }
    }

    #[test]
    fn pkg_member_inventory() {
        assert_eq!(PM_ORACLES.len(), 7);
        assert_eq!(PM_ORACLES[2].1.len(), 2, "PM-3 carries both relative forms");
        let mut ids: Vec<&str> = PM_ORACLES.iter().map(|s| s.0).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 7);
        let root = workspace_root();
        for f in PM_FIXTURE_FILES {
            assert!(root.join(PM_FIXTURE).join(f).is_file(), "missing {f}");
        }
    }

    #[test]
    fn pkg_member_without_cli_is_inconclusive() {
        let root = workspace_root();
        for spec in PM_ORACLES.iter() {
            let r = core10_pkg_member(&root, spec);
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(
                r.detail.contains("needs the `arita` CLI"),
                "{} {}",
                r.id,
                r.detail
            );
        }
    }

    #[test]
    fn pkg_member_digests_detect_mutation() {
        let tmp = std::env::temp_dir().join(format!("arita_pm_dig_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("sub")).unwrap();
        fs::write(tmp.join("README.md"), b"a").unwrap();
        fs::write(tmp.join("sub").join("x.arita"), b"b").unwrap();
        let d0 = pm_digest(&tmp, true).unwrap();
        assert_eq!(d0, pm_digest(&tmp, true).unwrap());
        fs::write(tmp.join("README.md"), b"a2").unwrap();
        assert_ne!(d0, pm_digest(&tmp, true).unwrap(), "README.md is covered");
        // a top-level target/ is ignored by the fixture digest, not by the scratch digest
        let d1 = pm_digest(&tmp, true).unwrap();
        fs::create_dir_all(tmp.join("target")).unwrap();
        fs::write(tmp.join("target").join("t"), b"t").unwrap();
        assert_eq!(d1, pm_digest(&tmp, true).unwrap());
        assert_ne!(d1, pm_digest(&tmp, false).unwrap());
        // metadata digest: absent dir is stable, a new file changes it
        assert_eq!(pm_meta_digest(&tmp.join("nope")).unwrap(), "absent");
        let m0 = pm_meta_digest(&tmp).unwrap();
        fs::write(tmp.join("sub").join("y"), b"y").unwrap();
        assert_ne!(m0, pm_meta_digest(&tmp).unwrap());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn pkg_member_checks_on_synthetic_runs() {
        let scratch = Path::new("/nonexistent-scratch");
        let e332 = "E0332: input /x/bin/other.arita is not a member of workspace /x/arita.toml (members: lib, bin)\n";
        assert!(pm_check(PmExpect::NonMemberE0332, scratch, &run(Some(1), "", e332)).is_ok());
        for bad in [
            run(Some(0), "", e332),
            run(Some(1), "ok: /x\n", e332),
            run(Some(1), "", "E0331: manifest\n"),
            run(Some(1), "", "no such file\n"),
            run(Some(1), "", "E0332: a\nE0331: b\n"),
        ] {
            assert!(pm_check(PmExpect::NonMemberE0332, scratch, &bad).is_err());
        }
        let io = "No such file or directory (os error 2)\n";
        assert!(pm_check(PmExpect::MissingIo, scratch, &run(Some(1), "", io)).is_ok());
        assert!(pm_check(PmExpect::MissingIo, scratch, &run(Some(1), "", e332)).is_err());
        assert!(pm_check(PmExpect::MissingIo, scratch, &run(Some(1), "", "")).is_err());
        assert!(pm_check(PmExpect::MissingIo, scratch, &run(Some(0), "", io)).is_err());

        let tmp = std::env::temp_dir().join(format!("arita_pm_smoke_{}", std::process::id()));
        let prof = tmp.join("w").join("target").join("debug");
        let other = tmp.join("bin_x").join("target").join("debug");
        fs::create_dir_all(&prof).unwrap();
        fs::create_dir_all(&other).unwrap();
        let ok = format!("ok: {}\n", prof.display());
        assert!(pm_check(PmExpect::LibDir, &tmp, &run(Some(0), &ok, "")).is_ok());
        assert!(pm_check(PmExpect::LibDir, &tmp, &run(Some(0), &ok, "warn\n")).is_err());
        let bad = format!("ok: {}\n", other.display());
        assert!(pm_check(PmExpect::LibDir, &tmp, &run(Some(0), &bad, "")).is_err());
        assert!(pm_check(PmExpect::LibDir, &tmp, &run(Some(1), &ok, "")).is_err());
        let _ = fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod adr288_emit_clippy_smoke {
    //! CUT CORE-EMIT-CLIPPY-B282-20261001 — ADR-288 B-282-1/2 wired by Measure (PINS v0.1, sha256
    //! bbf27929…). No build, no clippy here: the real verdicts (accepted / rejected /
    //! inconclusive) come from `arita measure`. These tests check the inventory, the fixtures'
    //! source text, the emit-shape and tool-missing classifiers on synthetic data, and that a
    //! missing fixture is Inconclusive (never Accepted).
    use super::*;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn b282_inventory_and_fixtures() {
        assert_eq!(B282_ORACLES.len(), 2);
        let mut ids: Vec<&str> = B282_ORACLES.iter().map(|s| s.id).collect();
        assert_eq!(ids[0], "core09-b282-1-map-get-lit-emit-clippy");
        assert_eq!(ids[1], "core09-b282-2-let-len-emit-clippy");
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 2);
        let root = workspace_root();
        let src1 = fs::read_to_string(root.join(B282_ORACLES[0].rel)).expect("fixture 01");
        assert!(src1.contains("m.get(\"a\")") && src1.contains("m[\"a\"] = 7"));
        let src2 = fs::read_to_string(root.join(B282_ORACLES[1].rel)).expect("fixture 02");
        assert!(src2.contains("let n: Int = m.len()") && src2.contains("let k: Int = v.len()"));
        assert_eq!(B282_ORACLES[0].stdout, ["7"]);
        assert_eq!(B282_ORACLES[1].stdout, ["1", "1"]);
    }

    #[test]
    fn b282_emit_shape_checks() {
        let good1 = "    let o: Option<i64> = m.get(\"a\").cloned();\n";
        let bad1 = "    let o: Option<i64> = m.get(&\"a\".to_string()).cloned();\n";
        assert!(b282_emit_shape(&B282_ORACLES[0], good1).is_ok());
        assert!(b282_emit_shape(&B282_ORACLES[0], bad1).is_err());
        assert!(b282_emit_shape(&B282_ORACLES[0], "nothing").is_err());
        let good2 = "    let n: i64 = m.len() as i64;\n    let k: i64 = v.len() as i64;\n";
        let bad2 = "    let n: i64 = (m.len() as i64);\n    let k: i64 = v.len() as i64;\n";
        assert!(b282_emit_shape(&B282_ORACLES[1], good2).is_ok());
        assert!(b282_emit_shape(&B282_ORACLES[1], bad2).is_err());
        assert!(b282_emit_shape(&B282_ORACLES[1], "    let n: i64 = m.len() as i64;\n").is_err());
    }

    #[test]
    fn b282_tool_missing_classifier() {
        assert!(b282_clippy_tool_missing("error: no such command: `clippy`"));
        assert!(b282_clippy_tool_missing(
            "error: 'clippy-driver' is not installed for the toolchain"
        ));
        assert!(b282_clippy_tool_missing(
            "error: Unknown binary 'clippy-driver'"
        ));
        assert!(!b282_clippy_tool_missing(
            "error: this argument is passed by value, but not consumed"
        ));
    }

    #[test]
    fn b282_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b288_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        for spec in B282_ORACLES.iter() {
            let r = core09_b282_emit_clippy(&tmp, spec);
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod adr289_unused_parens_smoke {
    //! CUT CORE-EMIT-UNUSED-PARENS-V0-20261002 — ADR-289 PU-1..PU-3 wired by Measure. No build, no
    //! cargo, no rustc compile here: the real verdicts (accepted / rejected / inconclusive) come
    //! from `arita measure`. These tests check the inventory, the fixtures' source text, the
    //! diagnostic parser and the verdict classifier on synthetic rustc / cargo JSON, the exact-form
    //! checks (derived from the specs, one mutation per entry) and that a missing fixture or a
    //! missing rustc is Inconclusive (never Accepted).
    use super::*;

    const PU_JSON_PARENS: &str = "{\"$message_type\":\"diagnostic\",\"message\":\"unnecessary parentheses around assigned value\",\"code\":{\"code\":\"unused_parens\",\"explanation\":null},\"level\":\"warning\",\"spans\":[],\"children\":[],\"rendered\":\"warning: unnecessary parentheses around assigned value\\n\"}\n";
    const PU_JSON_OTHER: &str = "{\"$message_type\":\"diagnostic\",\"message\":\"unused variable: `x`\",\"code\":{\"code\":\"unused_variables\",\"explanation\":null},\"level\":\"warning\",\"spans\":[],\"children\":[],\"rendered\":\"warning: unused variable\\n\"}\n";
    const PU_JSON_ERROR: &str = "{\"$message_type\":\"diagnostic\",\"message\":\"cannot find value `y`\",\"code\":{\"code\":\"E0425\",\"explanation\":null},\"level\":\"error\",\"spans\":[],\"children\":[],\"rendered\":\"error[E0425]\\n\"}\n";
    const PU_JSON_CARGO_MAIN: &str = "{\"reason\":\"compiler-message\",\"package_id\":\"pkg 0.0.0\",\"target\":{\"src_path\":\"/w/crate/src/main.rs\"},\"message\":{\"message\":\"unnecessary parentheses around assigned value\",\"code\":{\"code\":\"unused_parens\",\"explanation\":null},\"level\":\"warning\",\"rendered\":\"warning: unnecessary parentheses\\n\"}}\n";
    const PU_JSON_CARGO_DEP: &str = "{\"reason\":\"compiler-message\",\"package_id\":\"dep 1.0.0\",\"target\":{\"src_path\":\"/reg/dep/src/lib.rs\"},\"message\":{\"message\":\"unnecessary parentheses around assigned value\",\"code\":{\"code\":\"unused_parens\",\"explanation\":null},\"level\":\"warning\",\"rendered\":\"warning: unnecessary parentheses\\n\"}}\n";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn pu_inventory_and_fixtures() {
        let mut ids = vec![PU1_ID, PU2_ID, PU3_ID];
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
        assert_eq!(PU2_FIXTURES.len(), 2);
        assert_eq!(PU2_FIXTURES[0].stdout, ["10", "2", "12", "3", "6"]);
        assert_eq!(PU2_FIXTURES[1].stdout, ["ok", "3"]);
        assert_eq!(PU3_FIXTURE.stdout, ["ok"]);
        let root = workspace_root();
        let src0 = fs::read_to_string(root.join(PU2_FIXTURES[0].rel)).expect("fixture 01");
        assert!(src0.contains("fn double(x: Int) -> Int") && src0.contains("while i < a"));
        assert!(src0.contains("let d: Int = double(v.len())"));
        let src1 = fs::read_to_string(root.join(PU2_FIXTURES[1].rel)).expect("fixture 02");
        assert!(src1.contains("v.resize(3, 9)") && src1.contains("v.reserve(8)"));
        let src3 = fs::read_to_string(root.join(PU3_FIXTURE.rel)).expect("f2/184");
        assert!(src3.contains("v.shrink_to_fit()") && src3.contains("let c: Int = v.capacity()"));
        let row = EJEMPLO_ORACLES
            .iter()
            .find(|o| o.id == "std-shrink-to-fit-vec")
            .expect("std-shrink-to-fit-vec stays in the table");
        assert_eq!(row.path, PU3_FIXTURE.rel);
        assert_eq!(row.expected, PU3_FIXTURE.stdout);
        let clash = EJEMPLO_ORACLES
            .iter()
            .filter(|o| o.id == PU1_ID || o.id == PU2_ID || o.id == PU3_ID)
            .count();
        assert_eq!(clash, 0);
    }

    #[test]
    fn pu_diagnostic_parser() {
        assert_eq!(pu_parse_diagnostics(PU_JSON_PARENS, false).0, 1);
        assert_eq!(pu_parse_diagnostics(PU_JSON_OTHER, false).0, 0);
        assert_eq!(
            pu_parse_diagnostics("not json at all\n", false),
            (0, 0, String::new())
        );
        let mixed = [PU_JSON_OTHER, PU_JSON_PARENS, PU_JSON_PARENS, PU_JSON_ERROR].concat();
        let (parens, errors, first) = pu_parse_diagnostics(&mixed, false);
        assert_eq!((parens, errors), (2, 1));
        assert!(first.contains("unnecessary parentheses"), "{first}");
        assert_eq!(pu_parse_diagnostics(PU_JSON_CARGO_MAIN, true).0, 1);
        assert_eq!(pu_parse_diagnostics(PU_JSON_CARGO_DEP, true).0, 0);
        assert_eq!(pu_parse_diagnostics(PU_JSON_CARGO_DEP, false).0, 1);
    }

    #[test]
    fn pu_classifier_verdicts() {
        let clean = pu_classify("rustc", "edition 2015", true, 0, PU_JSON_OTHER, false);
        assert!(clean.is_ok());
        let warned = pu_classify("rustc", "edition 2021", true, 0, PU_JSON_PARENS, false);
        assert_eq!(warned.unwrap_err().0, Verdict::Rejected);
        let broke = pu_classify("rustc", "edition 2021", false, 1, PU_JSON_ERROR, false);
        assert_eq!(broke.unwrap_err().0, Verdict::Rejected);
        let silent = pu_classify("rustc", "edition 2021", false, 1, "", false);
        assert_eq!(silent.unwrap_err().0, Verdict::Inconclusive);
        let rustup = "error: toolchain 'x' is not installed\n";
        let missing = pu_classify("rustc", "edition 2015", false, 1, rustup, false);
        assert_eq!(missing.unwrap_err().0, Verdict::Inconclusive);
        let cargo_main = pu_classify(
            "cargo check",
            "edition 2021",
            true,
            0,
            PU_JSON_CARGO_MAIN,
            true,
        );
        assert_eq!(cargo_main.unwrap_err().0, Verdict::Rejected);
        let cargo_dep = pu_classify(
            "cargo check",
            "edition 2021",
            true,
            0,
            PU_JSON_CARGO_DEP,
            true,
        );
        assert!(cargo_dep.is_ok());
    }

    #[test]
    fn pu_shape_checks_flip_on_each_entry() {
        let mut specs: Vec<&PuFixture> = PU2_FIXTURES.iter().collect();
        specs.push(&PU3_FIXTURE);
        for fx in specs {
            let good = fx.must_contain.join("\n");
            assert!(pu_shape(fx, &good).is_ok(), "{}", fx.rel);
            for (i, _) in fx.must_contain.iter().enumerate() {
                let mut v: Vec<&str> = fx.must_contain.to_vec();
                v.remove(i);
                assert!(pu_shape(fx, &v.join("\n")).is_err(), "{} drop {i}", fx.rel);
            }
            for bad in fx.must_not_contain {
                let tainted = format!("{good}\n{bad}\n");
                assert!(pu_shape(fx, &tainted).is_err(), "{} {bad}", fx.rel);
            }
            assert!(pu_shape(fx, "nothing").is_err(), "{}", fx.rel);
        }
    }

    #[test]
    fn pu1_counts_invariant() {
        assert!(pu1_counts_consistent(398, 363, 35));
        assert!(!pu1_counts_consistent(398, 363, 34));
        assert!(!pu1_counts_consistent(0, 0, 0));
        assert!(pu1_counts_consistent(
            EJEMPLO_ORACLES.len(),
            EJEMPLO_ORACLES.len(),
            0
        ));
    }

    #[test]
    fn pu_missing_rustc_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b289_rustc_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let r = pu_run_rustc("arita-no-such-rustc-289", &tmp, "fn main() {}\n", "2021");
        let (verdict, detail) = r.unwrap_err();
        assert_eq!(verdict, Verdict::Inconclusive);
        assert!(detail.contains("not available"), "{detail}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn pu_canary_missing_rustc_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b289_canary_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let (verdict, detail) = pu_canary("arita-no-such-rustc-289", &tmp).unwrap_err();
        assert_eq!(verdict, Verdict::Inconclusive);
        assert!(detail.contains("not available"), "{detail}");
        assert!(PU_CANARY.contains("(1 + 2)"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn pu_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b289_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let results = [
            core10_pu1_corpus(&tmp),
            core10_pu2_guards(&tmp),
            core10_pu3_capacity(&tmp),
        ];
        for r in results.iter() {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod adr290_index_mut_smoke {
    //! CUT CORE-0.10-INDEX-MUT-20261002 — ADR-290 slice A (IM-1..IM-6) wired by Measure. No build, no
    //! cargo, no rustc compile here: the real verdicts (accepted / rejected / inconclusive) come from
    //! `arita measure`. These tests check the inventory, the fixtures' source text, the emit-shape
    //! and diagnostic checkers on synthetic data (one mutation per entry), the IM-6 migration table
    //! and that a missing fixture is Inconclusive (never Accepted).
    use super::*;

    const IM_GOOD_EMIT: &str = "fn __arita_vec_update(v: &mut [i64], i: i64, x: i64, op: fn(i64, i64) -> Option<i64>) -> Result<(), i64> {\n    use std::convert::TryFrom as _;\n    let Ok(u) = usize::try_from(i) else {\n        return Err(0);\n    };\n    let Some(slot) = v.get_mut(u) else {\n        return Err(0);\n    };\n    let Some(new) = op(*slot, x) else {\n        return Err(0);\n    };\n    *slot = new;\n    Ok(())\n}\nfn run() -> Result<i64, i64> {\n    __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_add)?;\n    __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_sub)?;\n    __arita_vec_update(&mut v, __arita_vi, __arita_vx, i64::checked_mul)?;\n    Ok(1)\n}\n";
    const IM_E0344_LINE: &str = "E0344: index assign outside result fn @81..90";
    const IM_E0006_PEST: &str = "E0006: construct outside F1.1 (parse failure):  --> 7:8\n  |\n7 |   v[0] /= 2\n  |        ^---\n";
    const IM_TWO_CODES: &str =
        "E0344: index assign outside result fn @81..90\nE0319: negative set index";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn im_inventory_and_fixtures() {
        let mut ids = vec![IM1_ID, IM2_ID, IM3_ID, IM4_ID, IM5_ID, IM6_ID];
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 6);
        assert!(ids.iter().all(|i| i.starts_with("core10-index-mut-")));
        let clash = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash, 0);
        assert_eq!(IM1_STDOUT, ["15", "13", "90", "12", "24"]);
        assert_eq!(IM2_FIXTURES.len(), 3);
        assert_eq!(IM2_FIXTURES[0].stdout, ["3", "0", "0", "0"]);
        assert_eq!(IM2_FIXTURES[1].stdout.len(), 7);
        assert_eq!(IM2_FIXTURES[2].stdout, ["0", "0", "0"]);
        assert_eq!(IM3_FIXTURES.len(), 4);
        assert_eq!(IM4_CASES.len(), 3);
        assert_eq!(IM5_CASES.len(), 10);
        let root = workspace_root();
        let src1 = fs::read_to_string(root.join(IM1_FIXTURE)).expect("fixture 01");
        assert!(
            src1.contains("v[0] += 5") && src1.contains("v[1] -= 7") && src1.contains("v[2] *= 3")
        );
        for fx in IM2_FIXTURES.iter() {
            let src = fs::read_to_string(root.join(fx.rel)).expect("im2 fixture");
            assert!(
                src.contains("Result<Int, Int>") && src.contains(" += "),
                "{}",
                fx.rel
            );
        }
        let ok = fs::read_to_string(root.join(IM6_NEW)).expect("05-compound-ok");
        assert!(ok.contains("v[0] += 1") && ok.contains("v.get(0)"));
        for (rel, stmt) in IM4_CASES.iter() {
            let src = fs::read_to_string(root.join(rel)).expect("im4 fixture");
            assert!(src.lines().any(|l| l.trim() == *stmt), "{rel} {stmt}");
        }
        for case in IM5_CASES.iter() {
            let src = fs::read_to_string(root.join(case.rel)).expect("im5 fixture");
            assert!(src.lines().any(|l| l.trim() == case.stmt), "{}", case.rel);
        }
    }

    #[test]
    fn im_codes_follow_the_adr_text() {
        // E0333 is not reachable from source (grammar admits Vec<Int>/List<Int> only): no neg here
        // may pin it; operators/typed Vec<Text> are E0006 (PROPOSED, pending the Ingeniero).
        assert!(IM5_CASES.iter().all(|c| c.code != "E0333"));
        let codes: Vec<&str> = IM5_CASES.iter().map(|c| c.code).collect();
        for want in ["E0202", "E0203", "E0344", "E0319", "E0006", "E0314"] {
            assert!(codes.contains(&want), "{want}");
        }
        // Pin 3 pairs reachable from source: E0344>E0319, E0344>E0203, E0319>E0203.
        let prec: Vec<&str> = IM5_CASES
            .iter()
            .filter(|c| c.rel.contains("prec"))
            .map(|c| c.code)
            .collect();
        assert_eq!(prec, ["E0344", "E0344", "E0319"]);
        // HIR-pinned heads carry the exact ADR text.
        let head6 = IM5_CASES
            .iter()
            .find(|c| c.rel.ends_with("15-map.arita"))
            .unwrap();
        assert_eq!(
            head6.head,
            Some("E0006: construct outside F1.1 (parse failure)")
        );
    }

    #[test]
    fn im_emit_shape_flips_on_each_entry() {
        for fx in IM3_FIXTURES.iter() {
            assert!(im_emit_shape(fx, IM_GOOD_EMIT).is_ok(), "{}", fx.rel);
            for call in fx.calls {
                let dropped = IM_GOOD_EMIT.replace(call, "");
                assert!(
                    im_emit_shape(fx, &dropped).is_err(),
                    "{} drop {call}",
                    fx.rel
                );
            }
            for bad in IM3_EMIT_BANS {
                let tainted = format!("{IM_GOOD_EMIT}\n{bad}\n");
                assert!(im_emit_shape(fx, &tainted).is_err(), "{} {bad}", fx.rel);
            }
            for bad in IM3_HELPER_BANS {
                let line = format!("    {bad};\n    *slot = new;");
                let tainted = IM_GOOD_EMIT.replace("    *slot = new;", &line);
                assert!(
                    im_emit_shape(fx, &tainted).is_err(),
                    "{} helper {bad}",
                    fx.rel
                );
            }
            for want in IM3_HELPER_MUST {
                let dropped = IM_GOOD_EMIT.replace(want, "gone");
                assert!(
                    im_emit_shape(fx, &dropped).is_err(),
                    "{} helper must {want}",
                    fx.rel
                );
            }
            assert!(im_emit_shape(fx, "nothing").is_err(), "{}", fx.rel);
        }
    }

    #[test]
    fn im_helper_body_extraction() {
        let body = im_helper_body(IM_GOOD_EMIT).expect("helper body");
        assert!(body.starts_with("fn __arita_vec_update("));
        assert!(body.ends_with("\n}"));
        assert!(!body.contains("fn run()"));
        assert!(im_helper_body("fn other() {\n}\n").is_none());
    }

    #[test]
    fn im_diag_check_logic() {
        assert!(im_diag_check(IM_E0344_LINE, "E0344", Some(IM_E0344_LINE)).is_ok());
        assert!(im_diag_check(IM_E0344_LINE, "E0344", None).is_ok());
        assert!(im_diag_check(IM_E0344_LINE, "E0319", None).is_err());
        assert!(im_diag_check(
            IM_E0344_LINE,
            "E0344",
            Some("E0344: index assign outside result fn @1..2")
        )
        .is_err());
        assert!(im_diag_check("", "E0344", None).is_err());
        let hir = "E0006: construct outside F1.1 (parse failure) @235..246";
        assert!(im_diag_check(hir, "E0006", Some(hir)).is_ok());
        // the parser's pest dump has no ` @span` on the first line: an exact-head check fails on it,
        // which is why the Map/String cases are pinned to the HIR text and the parser cases are not.
        assert!(im_diag_check(IM_E0006_PEST, "E0006", Some(hir)).is_err());
        assert!(im_diag_check(IM_E0006_PEST, "E0006", None).is_ok());
        // a diagnostic mentioning two codes is not an exact single-code match
        assert_eq!(core09_diag_codes(IM_TWO_CODES), ["E0344", "E0319"]);
        assert_eq!(core09_diag_codes(IM_E0344_LINE), ["E0344"]);
    }

    #[test]
    fn im_migration_retires_05_from_the_neg_oracle() {
        assert_eq!(VEC_ASSIGN_COMPOUND_NEG.len(), 2);
        assert!(VEC_ASSIGN_COMPOUND_NEG
            .iter()
            .all(|c| !c.0.contains("05-compound")));
        assert!(VEC_ASSIGN_COMPOUND_NEG[0].0.ends_with("06-nested.arita"));
        assert!(VEC_ASSIGN_COMPOUND_NEG[1].0.ends_with("07-field.arita"));
        let root = workspace_root();
        // the frozen ADR-283 fixture is NOT rewritten: still the same compound statement
        let old = fs::read_to_string(root.join(IM6_OLD)).expect("05-compound");
        assert!(old.contains("v[0] += 1") && old.contains("-> Result<Int, Int>"));
        assert!(root.join(IM6_NEW).is_file());
        assert_ne!(IM6_OLD, IM6_NEW);
    }

    #[test]
    fn im_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b290_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let results = [
            core10_im1_compound_ok(&tmp),
            core10_im2_err_propagation(&tmp),
            core10_im3_emit_ban(&tmp),
            core10_im4_unsupported(&tmp),
            core10_im5_neg_rules(&tmp),
            core10_im6_migration(&tmp),
        ];
        for r in results.iter() {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod adr292_int_overflow_smoke {
    //! CUT CORE-0.10-INT-ARITH-RUNTIME-20261003 — ADR-292 IA-1..IA-4 wired by Measure. No cargo, no
    //! rustc, no arita here: the real verdicts (accepted / rejected / inconclusive) come from
    //! `arita measure`. These tests check the inventory, the fixtures' source text, the pure
    //! checkers on synthetic runs (one mutation per assertion, so each can FAIL), the bounded runner
    //! on tiny `sh` children, and that a missing fixture is Inconclusive (never Accepted).
    use super::*;

    const IA_GOOD_ADD_ERR: &str = "thread 'main' panicked at src/main.rs:3:5:\nattempt to add with overflow\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n";
    const IA_GOOD_SUB_ERR: &str =
        "thread 'main' panicked at src/main.rs:3:5:\nattempt to subtract with overflow\n";
    const IA_GOOD_MUL_ERR: &str =
        "thread 'main' panicked at src/main.rs:3:5:\nattempt to multiply with overflow\n";
    const IA_BOTH_ERR: &str = "attempt to add with overflow\nattempt to multiply with overflow\n";
    const IA_DIV_ERR: &str = "attempt to add with overflow\nattempt to divide by zero\n";
    const IA_BEFORE_OUT: &str = "before\n";
    const IA_WRAPPED_OUT: &str = "before\n-9223372036854775808\n";
    const IA_CONTROL_OUT: &str = "before\n9223372036854775807\n-9223372036854775808\n9223372030926249001\n-9223372036854775808\nafter\n";
    const IA_TEST_OUT_GOOD: &str = "\nrunning 4 tests\ntest tests::ctl ... ok\ntest tests::ovf_add ... FAILED\ntest tests::ovf_mul ... FAILED\ntest tests::ovf_sub ... FAILED\n\nfailures:\n\n---- tests::ovf_add stdout ----\n\nthread 'tests::ovf_add' panicked at t.rs:9:5:\nattempt to add with overflow\n\n---- tests::ovf_mul stdout ----\n\nthread 'tests::ovf_mul' panicked at t.rs:17:5:\nattempt to multiply with overflow\n\n---- tests::ovf_sub stdout ----\n\nthread 'tests::ovf_sub' panicked at t.rs:13:5:\nattempt to subtract with overflow\n\n\nfailures:\n    tests::ovf_add\n    tests::ovf_mul\n    tests::ovf_sub\n\ntest result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n";
    const IA_TEST_OUT_SWAPPED: &str = "test tests::ctl ... ok\ntest tests::ovf_add ... FAILED\ntest tests::ovf_mul ... FAILED\ntest tests::ovf_sub ... FAILED\n\nfailures:\n\n---- tests::ovf_add stdout ----\nattempt to multiply with overflow\n\n---- tests::ovf_mul stdout ----\nattempt to add with overflow\n\n---- tests::ovf_sub stdout ----\nattempt to subtract with overflow\n\ntest result: FAILED. 1 passed; 3 failed; 0 ignored\n";
    const IA_TEST_OUT_CTL_FAILED: &str = "test tests::ctl ... FAILED\ntest tests::ovf_add ... FAILED\ntest tests::ovf_mul ... FAILED\ntest tests::ovf_sub ... FAILED\n\nfailures:\n\n---- tests::ovf_add stdout ----\nattempt to add with overflow\n\n---- tests::ovf_mul stdout ----\nattempt to multiply with overflow\n\n---- tests::ovf_sub stdout ----\nattempt to subtract with overflow\n\ntest result: FAILED. 0 passed; 4 failed; 0 ignored\n";
    const IA_TEST_OUT_PASS: &str = "test tests::ctl ... ok\ntest tests::ovf_add ... ok\ntest tests::ovf_mul ... ok\ntest tests::ovf_sub ... ok\n\ntest result: ok. 4 passed; 0 failed; 0 ignored\nPASS\n";
    const IA_TEST_ERR_GOOD: &str = "tests failed (exit 101)\n";
    const IA_TOML_GOOD: &str =
        "[package]\nname = \"x\"\n\n[profile.release]\nopt-level = 3\noverflow-checks = true\n";
    const IA_TOML_WORKSPACE: &str = "[workspace]\nresolver = \"2\"\nmembers = [\n  \"crates/lib_p\",\n]\n\n[profile.release]\noverflow-checks = true\n";
    const IA_TOML_ABSENT: &str = "[package]\nname = \"x\"\n\n[profile.release]\nopt-level = 3\n";
    const IA_TOML_FALSE: &str = "[profile.release]\nopt-level = 3\noverflow-checks = false\n";
    const IA_TOML_OTHER_SECTION: &str =
        "[profile.dev]\noverflow-checks = true\n\n[profile.release]\nopt-level = 3\n";
    const IA_TOML_COMMENT: &str = "[profile.release]\n# overflow-checks = true\nopt-level = 3\n";

    fn run(code: Option<i32>, stdout: &str, stderr: &str) -> IaRun {
        IaRun {
            code,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn ia_inventory_ids_and_fixtures() {
        let mut ids = vec![IA1_ID, IA2_ID, IA3_ID, IA4_ID];
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 4);
        assert!(ids.iter().all(|i| i.starts_with("core10-int-overflow-")));
        let clash = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash, 0);
        assert_eq!(IA_OPS.len(), 3);
        assert_eq!(IA_OPS[0].phrase, "attempt to add with overflow");
        assert_eq!(IA_OPS[1].phrase, "attempt to subtract with overflow");
        assert_eq!(IA_OPS[2].phrase, "attempt to multiply with overflow");
        assert_eq!(IA_CONTROL_STDOUT.len(), 6);
        assert_eq!(IA_TEST_OVF.len(), 3);
        let root = workspace_root();
        for (op, sym) in IA_OPS.iter().zip(["a + b", "a - b", "a * b"]) {
            let src = fs::read_to_string(root.join(op.rel)).expect("op fixture");
            assert!(src.contains(sym), "{}", op.rel);
            assert!(src.contains("print(\"before\")"), "{}", op.rel);
            assert!(src.contains("9223372036854775807"), "{}", op.rel);
        }
        let ctl = fs::read_to_string(root.join(IA_CONTROL)).expect("control");
        assert!(ctl.contains("print(\"after\")") && ctl.contains("3037000499"));
        let tests = fs::read_to_string(root.join(IA_TEST_FIXTURE)).expect("test fixture");
        assert_eq!(tests.matches("\ntest ").count(), 4);
        assert!(tests.contains("test ctl ") && tests.contains("test ovf_mul "));
        for f in IA_PKG_FILES {
            assert!(root.join(IA_PKG_DIR).join(f).is_file(), "{f}");
        }
        let lib = fs::read_to_string(root.join(IA_PKG_DIR).join("lib/lib.arita")).expect("lib");
        assert!(lib.contains("pub fn add") && lib.contains("a + b"));
    }

    #[test]
    fn ia_check_panic_accepts_exactly_the_uniform_panic() {
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_ADD_ERR)).is_ok());
        assert!(ia_check_panic(IA_SUB, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_SUB_ERR)).is_ok());
        assert!(ia_check_panic(IA_MUL, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_MUL_ERR)).is_ok());
        // the panic line itself (path, line, thread) is NOT asserted: a bare phrase is enough
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, IA_ADD)).is_ok());
    }

    #[test]
    fn ia_check_panic_rejects_wrong_exit_codes() {
        for code in [
            Some(0),
            Some(1),
            Some(2),
            Some(102),
            Some(134),
            Some(139),
            None,
        ] {
            let r = run(code, IA_BEFORE_OUT, IA_GOOD_ADD_ERR);
            assert!(ia_check_panic(IA_ADD, &r).is_err(), "{code:?}");
        }
        // the wrap-around of a release build without the flag: exit 0 and the wrapped value
        let wrapped = run(Some(0), IA_WRAPPED_OUT, "");
        assert!(ia_check_panic(IA_ADD, &wrapped).is_err());
        let err = ia_check_panic(IA_ADD, &run(None, IA_BEFORE_OUT, IA_GOOD_ADD_ERR)).unwrap_err();
        assert!(err.contains("signal"), "{err}");
    }

    #[test]
    fn ia_check_panic_rejects_wrong_stdout_and_stderr() {
        // stdout must be exactly `before`
        assert!(ia_check_panic(IA_ADD, &run(Some(101), "", IA_GOOD_ADD_ERR)).is_err());
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_WRAPPED_OUT, IA_GOOD_ADD_ERR)).is_err());
        assert!(ia_check_panic(IA_ADD, &run(Some(101), "after\n", IA_GOOD_ADD_ERR)).is_err());
        // the phrase must be the one of THIS operation
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, "")).is_err());
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, "boom\n")).is_err());
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_SUB_ERR)).is_err());
        assert!(ia_check_panic(IA_SUB, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_MUL_ERR)).is_err());
        assert!(ia_check_panic(IA_MUL, &run(Some(101), IA_BEFORE_OUT, IA_GOOD_ADD_ERR)).is_err());
        // two operations in one stderr, or a division panic next to it, is not the expected run
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, IA_BOTH_ERR)).is_err());
        assert!(ia_check_panic(IA_ADD, &run(Some(101), IA_BEFORE_OUT, IA_DIV_ERR)).is_err());
        // an unknown phrase can never pass vacuously
        assert!(
            ia_check_panic("overflow", &run(Some(101), IA_BEFORE_OUT, IA_GOOD_ADD_ERR)).is_err()
        );
    }

    #[test]
    fn ia_check_control_logic() {
        assert!(ia_check_control(&run(Some(0), IA_CONTROL_OUT, "")).is_ok());
        assert!(ia_check_control(&run(Some(101), IA_CONTROL_OUT, "")).is_err());
        assert!(ia_check_control(&run(None, IA_CONTROL_OUT, "")).is_err());
        assert!(ia_check_control(&run(Some(0), IA_WRAPPED_OUT, "")).is_err());
        assert!(ia_check_control(&run(Some(0), "before\nafter\n", "")).is_err());
        assert!(ia_check_control(&run(Some(0), IA_CONTROL_OUT, IA_GOOD_ADD_ERR)).is_err());
    }

    #[test]
    fn ia_check_test_logic() {
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_GOOD, IA_TEST_ERR_GOOD)).is_ok());
        // exit 0 / PASS: the flag is missing and the overflow tests passed through wrap-around
        assert!(ia_check_test(&run(Some(0), IA_TEST_OUT_PASS, "")).is_err());
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_PASS, IA_TEST_ERR_GOOD)).is_err());
        assert!(ia_check_test(&run(Some(2), IA_TEST_OUT_GOOD, IA_TEST_ERR_GOOD)).is_err());
        assert!(ia_check_test(&run(None, IA_TEST_OUT_GOOD, IA_TEST_ERR_GOOD)).is_err());
        // the libtest exit must be the one printed by the CLI
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_GOOD, "tests failed (exit 1)\n")).is_err());
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_GOOD, "")).is_err());
        // a failing control, or the phrase of another operation in a failure section
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_CTL_FAILED, IA_TEST_ERR_GOOD)).is_err());
        assert!(ia_check_test(&run(Some(1), IA_TEST_OUT_SWAPPED, IA_TEST_ERR_GOOD)).is_err());
        assert!(ia_check_test(&run(Some(1), "", IA_TEST_ERR_GOOD)).is_err());
        let no_summary = IA_TEST_OUT_GOOD.replace("1 passed; 3 failed", "2 passed; 2 failed");
        assert!(ia_check_test(&run(Some(1), &no_summary, IA_TEST_ERR_GOOD)).is_err());
    }

    #[test]
    fn ia_libtest_parsers() {
        assert!(ia_test_line(IA_TEST_OUT_GOOD, "ctl", "ok"));
        assert!(ia_test_line(IA_TEST_OUT_GOOD, "ovf_sub", "FAILED"));
        assert!(!ia_test_line(IA_TEST_OUT_GOOD, "ctl", "FAILED"));
        assert!(!ia_test_line(IA_TEST_OUT_GOOD, "ovf", "FAILED"));
        assert!(ia_name_matches("tests::ctl", "ctl") && ia_name_matches("ctl", "ctl"));
        assert!(!ia_name_matches("tests::xctl", "ctl"));
        let block = ia_failure_block(IA_TEST_OUT_GOOD, "ovf_add").expect("block");
        assert!(block.contains(IA_ADD) && !block.contains(IA_MUL) && !block.contains(IA_SUB));
        let last = ia_failure_block(IA_TEST_OUT_GOOD, "ovf_sub").expect("last block");
        assert!(last.contains(IA_SUB) && !last.contains("tests::ovf_add"));
        assert!(ia_failure_block(IA_TEST_OUT_GOOD, "ctl").is_none());
    }

    #[test]
    fn ia_toml_release_flag_checker() {
        assert!(ia_toml_release_overflow_checks(IA_TOML_GOOD));
        assert!(ia_toml_release_overflow_checks(IA_TOML_WORKSPACE));
        assert!(!ia_toml_release_overflow_checks(IA_TOML_ABSENT));
        assert!(!ia_toml_release_overflow_checks(IA_TOML_FALSE));
        assert!(!ia_toml_release_overflow_checks(IA_TOML_OTHER_SECTION));
        assert!(!ia_toml_release_overflow_checks(IA_TOML_COMMENT));
        assert!(!ia_toml_release_overflow_checks(""));
    }

    #[cfg(unix)]
    #[test]
    fn ia_bounded_runner_captures_exit_and_streams() {
        let sh = Path::new("/bin/sh");
        let script = "printf 'before\\n'; echo 'attempt to add with overflow' >&2; exit 101";
        let r = ia_run_bounded(sh, &["-c", script], None, 20).expect("run");
        assert_eq!(r.code, Some(101));
        assert_eq!(r.stdout, "before\n");
        assert!(r.stderr.contains(IA_ADD));
        // the same real process passes the checker; with another operation it does not
        assert!(ia_check_panic(IA_ADD, &r).is_ok());
        assert!(ia_check_panic(IA_MUL, &r).is_err());
        let ok = ia_run_bounded(sh, &["-c", "exit 0"], None, 20).expect("run ok");
        assert_eq!(ok.code, Some(0));
        assert!(ia_check_panic(IA_ADD, &ok).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn ia_bounded_runner_signal_and_timeout_and_missing() {
        let sh = Path::new("/bin/sh");
        let sig = ia_run_bounded(sh, &["-c", "kill -9 $$"], None, 20).expect("signal run");
        assert_eq!(sig.code, None);
        assert!(ia_check_panic(IA_ADD, &sig).is_err());
        let started = std::time::Instant::now();
        let slow = ia_run_bounded(sh, &["-c", "exec sleep 30"], None, 1);
        let (verdict, detail) = slow.err().expect("timeout is an error");
        assert_eq!(verdict, Verdict::Inconclusive);
        assert!(detail.contains("timeout"), "{detail}");
        assert!(started.elapsed().as_secs() < 20);
        let gone = ia_run_bounded(Path::new("/nonexistent/arita_ia_bin"), &[], None, 5);
        let (v2, d2) = gone.err().expect("spawn error");
        assert_eq!(v2, Verdict::Inconclusive);
        assert!(d2.contains("cannot spawn"), "{d2}");
    }

    #[test]
    fn ia_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b292_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let results = [
            core10_ia1_build(&tmp),
            core10_ia2_release(&tmp),
            core10_ia3_test(&tmp),
            core10_ia4_workspace_release(&tmp),
        ];
        for r in results.iter() {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        assert_eq!(results[0].id, IA1_ID);
        assert_eq!(results[3].id, IA4_ID);
        let _ = fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod adr293_undeclared_call_smoke {
    //! CUT CORE-0.10-UNDECLARED-CALL-20261003 — ADR-293 UC-1..UC-5 wired by Measure. No cargo, no
    //! rustc here except the `gated` test (same pattern as dead-sink / join-safety): the real
    //! verdicts come from `arita measure`. These tests check the inventory, the fixtures' source
    //! text, the pure checkers on synthetic messages/runs (one mutation per assertion, so each can
    //! FAIL), the in-process probe on mutated temp programs, and that a missing fixture is
    //! Inconclusive (never Accepted). Each test asserts in its own body (VT006).
    use super::*;

    const UC_GOOD: &str = "E0347: call to undeclared function `ghost`";
    const UC_GOOD_NL: &str = "E0347: call to undeclared function `ghost`\n";
    const UC_MUTEX: &str = "E0347: call to undeclared function `mutex_new`";
    const UC_BAD_NAME: &str = "E0347: call to undeclared function `other`";
    const UC_BAD_SPAN: &str = "E0347: call to undeclared function `ghost` @3..9";
    const UC_BAD_EXTRA: &str = "E0347: call to undeclared function `ghost`\nnote: more";
    const UC_BAD_PREFIX: &str = "error: E0347: call to undeclared function `ghost`";
    const UC_BAD_LOWER: &str = "e0347: call to undeclared function `ghost`";
    const UC_BAD_TWO: &str = "E0347: call to undeclared function `ghost`\nE0100: more";
    const UC_BAD_EMPTY: &str = "";
    const UC_E0321: &str = "E0321: reqwest not in surface (use HttpClient.get / post_text)";
    const UC_E0206: &str = "E0206: http free-fn not in surface (use HttpServer.bind / .serve)";
    const UC_E0320: &str = "E0320: hang theater / busy-spin not allowed (use timeout)";
    const UC_E0100: &str = "E0100: emitted Rust failed rustc\nerror[E0425]: cannot find function";
    const UC_E0006: &str = "E0006: illegal ident / keyword misuse";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn uc_check_diag_accepts_only_the_pinned_text() {
        assert!(uc_check_diag(UC_GOOD, "ghost").is_ok());
        assert!(uc_check_diag(UC_GOOD_NL, "ghost").is_ok());
        assert!(uc_check_diag(UC_MUTEX, "mutex_new").is_ok());
        for bad in [
            UC_BAD_NAME,
            UC_BAD_SPAN,
            UC_BAD_EXTRA,
            UC_BAD_PREFIX,
            UC_BAD_LOWER,
            UC_BAD_TWO,
            UC_BAD_EMPTY,
            UC_E0321,
            UC_E0206,
            UC_E0320,
            UC_E0100,
            UC_E0006,
        ] {
            assert!(uc_check_diag(bad, "ghost").is_err(), "{bad:?}");
        }
        assert!(uc_check_diag(UC_MUTEX, "ghost").is_err());
        assert!(uc_check_diag(UC_GOOD, "mutex_new").is_err());
    }

    #[test]
    fn uc_check_diag_error_kinds_are_distinct() {
        let wrong_code = uc_check_diag(UC_E0321, "ghost").unwrap_err();
        assert!(wrong_code.contains("expected only E0347"), "{wrong_code}");
        assert!(wrong_code.contains("E0321"), "{wrong_code}");
        let two_codes = uc_check_diag(UC_BAD_TWO, "ghost").unwrap_err();
        assert!(two_codes.contains("expected only E0347"), "{two_codes}");
        let wrong_text = uc_check_diag(UC_BAD_NAME, "ghost").unwrap_err();
        assert!(wrong_text.contains("text differs"), "{wrong_text}");
        let with_span = uc_check_diag(UC_BAD_SPAN, "ghost").unwrap_err();
        assert!(with_span.contains("text differs"), "{with_span}");
        assert_eq!(uc_diag_text("ghost"), UC_GOOD);
    }

    #[test]
    fn uc_check_run_accepts_only_exit0_stdout1_empty_stderr() {
        assert!(uc_check_run(Some(0), "1\n", "").is_ok());
        assert!(uc_check_run(Some(0), "1", "").is_ok());
        assert!(uc_check_run(Some(1), "1\n", "").is_err());
        assert!(uc_check_run(Some(101), "1\n", "").is_err());
        assert!(uc_check_run(None, "1\n", "").is_err());
        assert!(uc_check_run(Some(0), "", "").is_err());
        assert!(uc_check_run(Some(0), "2\n", "").is_err());
        assert!(uc_check_run(Some(0), "1\n1\n", "").is_err());
        assert!(uc_check_run(Some(0), "1\nafter\n", "").is_err());
        assert!(uc_check_run(Some(0), "1\n", "warning: x\n").is_err());
        let e = uc_check_run(Some(0), "2\n", "").unwrap_err();
        assert!(e.contains("stdout mismatch"), "{e}");
    }

    #[test]
    fn uc_inventory_ids_fixtures_and_shapes() {
        let mut ids: Vec<&str> = CORE10_UC_NEGS.iter().map(|n| n.id).collect();
        ids.push(UC_OK_ID);
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5);
        assert_eq!(UC_OK_ID, "core10-declared-call-ok");
        let clash_pos = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash_pos, 0);
        let clash_neg = NEG_ORACLES.iter().filter(|o| ids.contains(&o.id)).count();
        assert_eq!(clash_neg, 0);
        let root = workspace_root();
        for n in CORE10_UC_NEGS {
            assert!(n.id.starts_with("neg-core10-undeclared-call-"), "{}", n.id);
            assert!(
                n.rel.starts_with("ejemplos/core10/undeclared-call/neg/"),
                "{}",
                n.rel
            );
            assert_eq!(n.edits.len(), 1, "{}", n.id);
            assert_eq!(n.edits[0].0, n.offender, "{}", n.id);
            assert!(n.offender.contains("ghost("), "{}", n.id);
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let code = core09_ref_mut_code(&src);
            assert_eq!(core10_code_line_count(&code, n.offender), 1, "{}", n.rel);
            assert_eq!(code.matches("ghost").count(), 1, "{}", n.rel);
            assert!(!code.contains("fn ghost"), "{}", n.rel);
        }
        let ok = fs::read_to_string(root.join(UC_OK_REL)).expect("positive");
        let ok_code = core09_ref_mut_code(&ok);
        for l in UC_OK_SHAPE.iter() {
            assert_eq!(core10_code_line_count(&ok_code, l), 1, "{l}");
        }
        assert_eq!(
            UC_OK_REL,
            "ejemplos/core10/undeclared-call/05-declared-ok.arita"
        );
    }

    #[test]
    fn uc_controls_are_legal_twins_and_check_clean() {
        let root = workspace_root();
        for n in CORE10_UC_NEGS {
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let ctl = core10_control_source(&src, n.edits).expect("control edits");
            assert_eq!(ctl.lines().count(), src.lines().count(), "{}", n.id);
            let ctl_code = core09_ref_mut_code(&ctl);
            assert!(!ctl_code.contains("ghost"), "{} {ctl_code}", n.id);
            let checked = crate::parse_lower_check(&ctl);
            assert!(checked.is_ok(), "{} control: {:?}", n.id, checked.err());
        }
        let ok = fs::read_to_string(root.join(UC_OK_REL)).expect("positive");
        let ok_checked = crate::parse_lower_check(&ok);
        assert!(ok_checked.is_ok(), "positive: {:?}", ok_checked.err());
        let bad_edit: &[Core10CtlEdit] = &[("ghost(2)", Some("print(2)"))];
        let first = fs::read_to_string(root.join(CORE10_UC_NEGS[0].rel)).expect("neg 01");
        assert!(core10_control_source(&first, bad_edit).is_err());
    }

    #[test]
    fn uc_probe_can_fail_on_mutated_programs() {
        let root = workspace_root();
        let tmp = std::env::temp_dir().join(format!("arita_b293_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // (a) the declared twin is ACCEPTED by the checker ⇒ the probe must reject (never accept).
        let declared = fs::read_to_string(root.join(UC_OK_REL)).expect("positive");
        fs::write(tmp.join("declared.arita"), &declared).unwrap();
        let (v1, d1) = uc_probe(&tmp, "declared.arita", UC_CALL).unwrap_err();
        assert_eq!(v1, Verdict::Rejected);
        assert!(d1.contains("succeeded"), "{d1}");
        // (b) the same undeclared shape under another builtin-guard code (E0321) ⇒ rejected.
        let neg = fs::read_to_string(root.join(CORE10_UC_NEGS[1].rel)).expect("neg 02");
        let reqwest = neg.replace("ghost(1)", "reqwest_get(1)");
        assert_ne!(reqwest, neg);
        fs::write(tmp.join("reqwest.arita"), &reqwest).unwrap();
        let (v2, d2) = uc_probe(&tmp, "reqwest.arita", UC_CALL).unwrap_err();
        assert_eq!(v2, Verdict::Rejected);
        assert!(d2.contains("E0321"), "{d2}");
        // (c) the real neg probed with the WRONG callee name ⇒ rejected on the text.
        let (v3, d3) = uc_probe(&root, CORE10_UC_NEGS[1].rel, "other").unwrap_err();
        assert_eq!(v3, Verdict::Rejected);
        assert!(
            d3.contains("text differs") || d3.contains("succeeded"),
            "{d3}"
        );
        // (d) missing file ⇒ inconclusive.
        let (v4, d4) = uc_probe(&tmp, "nope.arita", UC_CALL).unwrap_err();
        assert_eq!(v4, Verdict::Inconclusive);
        assert!(d4.contains("missing"), "{d4}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn uc_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b293_missing_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let negs: Vec<OracleResult> = CORE10_UC_NEGS
            .iter()
            .map(|n| core10_uc_neg(&tmp, n))
            .collect();
        assert_eq!(negs.len(), 4);
        for r in negs.iter() {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        assert_eq!(negs[0].id, "neg-core10-undeclared-call-stmt");
        assert_eq!(negs[3].id, "neg-core10-undeclared-call-in-result-fn");
        let pos = core10_uc_positive(&tmp);
        assert_eq!(pos.verdict, Verdict::Inconclusive, "{}", pos.detail);
        assert!(pos.detail.contains("missing"), "{}", pos.detail);
        assert_eq!(pos.id, UC_OK_ID);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn mutex_hold_oracle_is_migrated_to_exact_e0347() {
        let root = workspace_root();
        let r = run_core03_compose_mutex_hold_oracle(&root);
        assert_eq!(r.id, "neg-core03-compose-mutex-hold");
        assert_eq!(r.verdict, Verdict::Accepted, "{}", r.detail);
        assert!(r.detail.contains(UC_MUTEX), "{}", r.detail);
        assert!(!r.detail.contains("E0100"), "{}", r.detail);
        let tmp = std::env::temp_dir().join(format!("arita_b293_mutex_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let missing = run_core03_compose_mutex_hold_oracle(&tmp);
        assert_eq!(missing.verdict, Verdict::Inconclusive, "{}", missing.detail);
        assert!(missing.detail.contains("missing"), "{}", missing.detail);
    }

    #[test]
    fn core10_uc_oracles_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let mut rs: Vec<OracleResult> = CORE10_UC_NEGS
            .iter()
            .map(|n| core10_uc_neg(&root, n))
            .collect();
        rs.push(core10_uc_positive(&root));
        assert_eq!(rs.len(), 5);
        for r in &rs {
            println!("adr293 {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
            assert!(
                r.detail.contains("control 0 diagnostics"),
                "{} detail={}",
                r.id,
                r.detail
            );
        }
        for r in rs.iter().take(4) {
            assert!(r.detail.contains(UC_GOOD), "{} detail={}", r.id, r.detail);
        }
        assert!(rs[4].detail.contains("exact stdout"), "{}", rs[4].detail);
    }
}

#[cfg(test)]
mod adr294_known_int_smoke {
    //! CUT CORE-0.10-KNOWN-INT-SCOPE-20261004 — ADR-294 KI-1..KI-7 wired by Measure. No cargo, no
    //! rustc here except the `gated` test (same pattern as dead-sink / join-safety / UC): the real
    //! verdicts come from `arita measure`. These tests check the inventory, the fixtures' source
    //! text, the pure checkers on synthetic messages/runs (one mutation per assertion, so each can
    //! FAIL), the in-process probe/gate on mutated programs, and that a missing fixture is
    //! Inconclusive (never Accepted). Each test asserts in its own body (VT006).
    use super::*;

    const KI_GOOD: &str = "E0217: integer overflow";
    const KI_GOOD_NL: &str = "E0217: integer overflow\n";
    const KI_E0216: &str = "E0216: integer division by zero";
    const KI_BAD_SPAN: &str = "E0217: integer overflow @3..9";
    const KI_BAD_EXTRA: &str = "E0217: integer overflow\nnote: more";
    const KI_BAD_PREFIX: &str = "error: E0217: integer overflow";
    const KI_BAD_LOWER: &str = "e0217: integer overflow";
    const KI_BAD_TWO: &str = "E0217: integer overflow\nE0216: integer division by zero";
    const KI_BAD_TEXT: &str = "E0217: integer overflow in expression";
    const KI_BAD_EMPTY: &str = "";
    const KI_E0100: &str = "E0100: emitted Rust failed rustc\nerror[E0080]: evaluation panicked";
    const KI_E0347: &str = "E0347: call to undeclared function `ghost`";
    const KI_WANT_OK: [&str; 1] = ["ok"];
    const KI_WANT_FP3: [&str; 2] = ["1", "ok"];
    const KI_WANT_FP4: [&str; 3] = ["9223372036854775807", "1", "ok"];
    /// Straight-line overflow: the literal is assigned and used in the SAME block ⇒ E0217 stays.
    const KI_SRC_STRAIGHT_E0217: &str = "module ki_smoke_straight_ovf\nfn main() -> Io<()> {\n  let mut a: Int = 0\n  a = 9223372036854775807\n  let r: Int = a + 1\n  print(r)\n  print(\"ok\")\n}\n";
    /// Straight-line division by a known zero ⇒ E0216 stays (a different code than the neg pins).
    const KI_SRC_STRAIGHT_E0216: &str = "module ki_smoke_straight_div\nfn main() -> Io<()> {\n  let d: Int = 0\n  let x: Int = 7\n  let q: Int = x.div_euclid(d)\n  print(q)\n  print(\"ok\")\n}\n";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn ki_check_diag_accepts_only_the_pinned_text() {
        assert!(ki_check_diag(KI_GOOD, KI_NEG_CODE, KI_NEG_TEXT).is_ok());
        assert!(ki_check_diag(KI_GOOD_NL, KI_NEG_CODE, KI_NEG_TEXT).is_ok());
        assert!(ki_check_diag(KI_E0216, "E0216", KI_E0216).is_ok());
        for bad in [
            KI_BAD_SPAN,
            KI_BAD_EXTRA,
            KI_BAD_PREFIX,
            KI_BAD_LOWER,
            KI_BAD_TWO,
            KI_BAD_TEXT,
            KI_BAD_EMPTY,
            KI_E0216,
            KI_E0100,
            KI_E0347,
        ] {
            assert!(
                ki_check_diag(bad, KI_NEG_CODE, KI_NEG_TEXT).is_err(),
                "{bad:?}"
            );
        }
        assert!(ki_check_diag(KI_GOOD, "E0216", KI_E0216).is_err());
    }

    #[test]
    fn ki_check_diag_error_kinds_are_distinct() {
        let wrong_code = ki_check_diag(KI_E0216, KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert!(wrong_code.contains("expected only E0217"), "{wrong_code}");
        assert!(wrong_code.contains("E0216"), "{wrong_code}");
        let two_codes = ki_check_diag(KI_BAD_TWO, KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert!(two_codes.contains("expected only E0217"), "{two_codes}");
        let wrong_text = ki_check_diag(KI_BAD_TEXT, KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert!(wrong_text.contains("text differs"), "{wrong_text}");
        let with_span = ki_check_diag(KI_BAD_SPAN, KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert!(with_span.contains("text differs"), "{with_span}");
        assert_eq!(KI_NEG_TEXT, KI_GOOD);
        assert!(KI_NEG_TEXT.starts_with(KI_NEG_CODE));
    }

    #[test]
    fn ki_check_run_accepts_only_exact_runs() {
        assert!(ki_check_run(Some(0), "ok\n", "", &KI_WANT_OK).is_ok());
        assert!(ki_check_run(Some(0), "ok", "", &KI_WANT_OK).is_ok());
        assert!(ki_check_run(Some(0), "1\nok\n", "", &KI_WANT_FP3).is_ok());
        assert!(ki_check_run(Some(0), "9223372036854775807\n1\nok\n", "", &KI_WANT_FP4).is_ok());
        assert!(ki_check_run(Some(1), "ok\n", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(Some(101), "ok\n", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(None, "ok\n", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(Some(0), "", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(Some(0), "okay\n", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(Some(0), "ok\nok\n", "", &KI_WANT_OK).is_err());
        assert!(ki_check_run(Some(0), "1\n", "", &KI_WANT_FP3).is_err());
        assert!(ki_check_run(Some(0), "ok\n1\n", "", &KI_WANT_FP3).is_err());
        assert!(ki_check_run(Some(0), "ok\n", "", &KI_WANT_FP3).is_err());
        assert!(ki_check_run(Some(0), "1\nok\n", "", &KI_WANT_FP4).is_err());
        assert!(ki_check_run(Some(0), "ok\n", "warning: x\n", &KI_WANT_OK).is_err());
        let e = ki_check_run(Some(0), "no\n", "", &KI_WANT_OK).unwrap_err();
        assert!(e.contains("stdout mismatch"), "{e}");
        let p =
            ki_check_run(Some(0), "ok\n", "thread main panicked at x", &KI_WANT_OK).unwrap_err();
        assert!(p.contains("panicked"), "{p}");
    }

    #[test]
    fn ki_inventory_ids_fixtures_and_shapes() {
        let mut ids: Vec<&str> = CORE10_KI_POS.iter().map(|p| p.id).collect();
        ids.extend(CORE10_KI_NEGS.iter().map(|n| n.id));
        assert_eq!(ids.len(), 7);
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 7);
        assert_eq!(
            ids,
            vec![
                "core10-known-int-fp1-else-bleed",
                "core10-known-int-fp2-match-arm",
                "core10-known-int-fp3-loop-carried",
                "core10-known-int-fp4-shadow-leak",
                "core10-known-int-fp7-e0216-else",
                "neg-core10-known-int-c5-straight",
                "neg-core10-known-int-c6-assign-overflow",
            ]
        );
        let clash_pos = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash_pos, 0);
        let clash_neg = NEG_ORACLES.iter().filter(|o| ids.contains(&o.id)).count();
        assert_eq!(clash_neg, 0);
        let root = workspace_root();
        for p in CORE10_KI_POS {
            assert!(p.rel.starts_with("ejemplos/core10/known-int/"), "{}", p.rel);
            assert!(!p.rel.contains("/neg/"), "{}", p.rel);
            assert!(!p.want.is_empty(), "{}", p.id);
            let src = fs::read_to_string(root.join(p.rel)).expect("positive fixture");
            let code = core09_ref_mut_code(&src);
            for l in p.shape.iter() {
                assert_eq!(core10_code_line_count(&code, l), 1, "{} {l}", p.rel);
            }
            assert!(code.contains("print(\"ok\")"), "{}", p.rel);
        }
        assert_eq!(CORE10_KI_POS[2].want, ["1", "ok"]);
        assert_eq!(CORE10_KI_POS[3].want, ["9223372036854775807", "1", "ok"]);
        assert_eq!(CORE10_KI_POS[0].want, ["ok"]);
        assert_eq!(CORE10_KI_POS[1].want, ["ok"]);
        assert_eq!(CORE10_KI_POS[4].want, ["ok"]);
        for n in CORE10_KI_NEGS {
            assert!(n.id.starts_with("neg-core10-known-int-"), "{}", n.id);
            assert_eq!(n.edits[0].0, n.offender, "{}", n.id);
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let code = core09_ref_mut_code(&src);
            assert_eq!(core10_code_line_count(&code, n.offender), 1, "{}", n.rel);
            for l in n.shape.iter() {
                assert_eq!(core10_code_line_count(&code, l), 1, "{} {l}", n.rel);
            }
        }
        assert_eq!(
            CORE10_KI_NEGS[0].rel,
            "ejemplos/f2/neg/e0217-runtime-max-plus.arita"
        );
        assert_eq!(
            CORE10_KI_NEGS[1].rel,
            "ejemplos/core10/known-int/neg/c6-assign-overflow-expr.arita"
        );
    }

    #[test]
    fn ki_controls_are_legal_twins_and_check_clean() {
        let root = workspace_root();
        for n in CORE10_KI_NEGS {
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let ctl = core10_control_source(&src, n.edits).expect("control edits");
            assert_eq!(ctl.lines().count(), src.lines().count(), "{}", n.id);
            let ctl_code = core09_ref_mut_code(&ctl);
            assert_eq!(core10_code_line_count(&ctl_code, n.offender), 0, "{}", n.id);
            assert!(ctl_code.contains("a - 1"), "{} {ctl_code}", n.id);
            let checked = crate::parse_lower_check(&ctl);
            assert!(checked.is_ok(), "{} control: {:?}", n.id, checked.err());
        }
        let c5 = fs::read_to_string(root.join(CORE10_KI_NEGS[0].rel)).expect("c5");
        let c5_ctl = core10_control_source(&c5, CORE10_KI_NEGS[0].edits).expect("c5 control");
        assert!(c5_ctl.contains("print(x)"), "{c5_ctl}");
        assert!(!c5_ctl.contains("print(\"ok\")"), "{c5_ctl}");
        let bad_edit: &[Core10CtlEdit] = &[("a = a + 2", Some("a = a - 2"))];
        let c6 = fs::read_to_string(root.join(CORE10_KI_NEGS[1].rel)).expect("c6");
        assert!(core10_control_source(&c6, bad_edit).is_err());
    }

    #[test]
    fn ki_gate_and_probe_can_fail_on_mutated_programs() {
        let root = workspace_root();
        let tmp = std::env::temp_dir().join(format!("arita_b294_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // (a) straight-line overflow keeps EXACT E0217 ⇒ the probe accepts it (D4).
        fs::write(tmp.join("straight.arita"), KI_SRC_STRAIGHT_E0217).unwrap();
        let ok = ki_probe(&tmp, "straight.arita", KI_NEG_CODE, KI_NEG_TEXT);
        assert!(ok.is_ok(), "{:?}", ok.as_ref().err());
        assert_eq!(ok.unwrap().trim(), KI_NEG_TEXT);
        // (b) the same straight shape seen as a VALID program ⇒ the HIR gate must fail with the text.
        let gate = ki_hir_gate(KI_SRC_STRAIGHT_E0217).unwrap_err();
        assert!(gate.contains("false positive"), "{gate}");
        assert!(gate.contains(KI_GOOD), "{gate}");
        // (c) a different straight diagnostic (E0216) ⇒ the probe rejects on the CODE.
        fs::write(tmp.join("div.arita"), KI_SRC_STRAIGHT_E0216).unwrap();
        let (v1, d1) = ki_probe(&tmp, "div.arita", KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert_eq!(v1, Verdict::Rejected);
        assert!(d1.contains("E0216"), "{d1}");
        // (d) the real c6 control (a - 1) is ACCEPTED by the checker ⇒ the probe must reject it.
        let c6 = fs::read_to_string(root.join(CORE10_KI_NEGS[1].rel)).expect("c6");
        let c6_ctl = core10_control_source(&c6, CORE10_KI_NEGS[1].edits).expect("c6 control");
        fs::write(tmp.join("c6ctl.arita"), &c6_ctl).unwrap();
        let (v2, d2) = ki_probe(&tmp, "c6ctl.arita", KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert_eq!(v2, Verdict::Rejected);
        assert!(d2.contains("succeeded"), "{d2}");
        // (e) the real neg probed with the WRONG pin ⇒ rejected.
        let (v3, d3) = ki_probe(&root, CORE10_KI_NEGS[1].rel, "E0216", KI_E0216).unwrap_err();
        assert_eq!(v3, Verdict::Rejected);
        assert!(d3.contains("E0217"), "{d3}");
        // (f) missing file ⇒ inconclusive.
        let (v4, d4) = ki_probe(&tmp, "nope.arita", KI_NEG_CODE, KI_NEG_TEXT).unwrap_err();
        assert_eq!(v4, Verdict::Inconclusive);
        assert!(d4.contains("missing"), "{d4}");
        // (g) the five valid fixtures pass the gate once ADR-294 is in the HIR.
        for p in CORE10_KI_POS {
            let src = fs::read_to_string(root.join(p.rel)).expect("positive fixture");
            let gated = ki_hir_gate(&src);
            assert!(gated.is_ok(), "{}: {:?}", p.id, gated.err());
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ki_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b294_missing_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let pos: Vec<OracleResult> = CORE10_KI_POS
            .iter()
            .map(|p| core10_ki_positive(&tmp, p))
            .collect();
        let negs: Vec<OracleResult> = CORE10_KI_NEGS
            .iter()
            .map(|n| core10_ki_neg(&tmp, n))
            .collect();
        assert_eq!(pos.len(), 5);
        assert_eq!(negs.len(), 2);
        for r in pos.iter().chain(negs.iter()) {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        assert_eq!(pos[0].id, "core10-known-int-fp1-else-bleed");
        assert_eq!(pos[4].id, "core10-known-int-fp7-e0216-else");
        assert_eq!(negs[0].id, "neg-core10-known-int-c5-straight");
        assert_eq!(negs[1].id, "neg-core10-known-int-c6-assign-overflow");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn core10_ki_oracles_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let mut rs: Vec<OracleResult> = CORE10_KI_POS
            .iter()
            .map(|p| core10_ki_positive(&root, p))
            .collect();
        rs.extend(CORE10_KI_NEGS.iter().map(|n| core10_ki_neg(&root, n)));
        assert_eq!(rs.len(), 7);
        for r in &rs {
            println!("adr294 {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
        for r in rs.iter().take(5) {
            assert!(r.detail.contains("exact stdout"), "{} {}", r.id, r.detail);
        }
        for r in rs.iter().skip(5) {
            assert!(r.detail.contains(KI_GOOD), "{} {}", r.id, r.detail);
            assert!(
                r.detail.contains("control 0 diagnostics"),
                "{} {}",
                r.id,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr291_mutex_reject_smoke {
    //! CUT CORE-0.10-MUTEX-REJECT-20261003 — ADR-291 MX-1..MX-4 wired by Measure. No cargo, no
    //! rustc here except the `gated` test (same pattern as UC / KI): the real verdicts come from
    //! `arita measure`. These tests check the inventory, the fixtures' source text, the pure
    //! checker on synthetic messages (one mutation per assertion, so each can FAIL), the
    //! in-process probe on mutated programs (including the E0346 > E0347 precedence in both
    //! orders, Sello §4) and that a missing fixture is Inconclusive (never Accepted). Each test
    //! asserts in its own body (VT006).
    use super::*;

    /// Synthetic source: `Mutex` sits at bytes 11..16, a later `Arc` at 24..27.
    const MX_SRC: &str = "let m: T = Mutex.new(0) Arc";
    const MX_ANCHOR: &str = "Mutex";
    const MX_GOOD: &str = "E0346: mutex concurrency is not available in this surface @11..16";
    const MX_GOOD_NL: &str = "E0346: mutex concurrency is not available in this surface @11..16\n";
    const MX_BAD_NO_SPAN: &str = "E0346: mutex concurrency is not available in this surface";
    const MX_BAD_EMPTY: &str = "E0346: mutex concurrency is not available in this surface @11..11";
    const MX_BAD_REVERSED: &str =
        "E0346: mutex concurrency is not available in this surface @16..11";
    const MX_BAD_OUTSIDE: &str =
        "E0346: mutex concurrency is not available in this surface @11..99";
    const MX_BAD_WRONG_WORD: &str =
        "E0346: mutex concurrency is not available in this surface @24..27";
    const MX_BAD_SPACES: &str =
        "E0346: mutex concurrency is not available in this surface @11 .. 16";
    const MX_BAD_SIGN: &str = "E0346: mutex concurrency is not available in this surface @-1..16";
    const MX_BAD_NO_AT: &str = "E0346: mutex concurrency is not available in this surface 11..16";
    const MX_BAD_EXTRA: &str =
        "E0346: mutex concurrency is not available in this surface @11..16\nnote: more";
    const MX_BAD_TAIL: &str =
        "E0346: mutex concurrency is not available in this surface @11..16 (Mutex)";
    const MX_BAD_PREFIX: &str = "E0346: mutex concurrency is not available @11..16";
    const MX_BAD_PREFIX_LONG: &str =
        "E0346: mutex concurrency is not available in this surface yet @11..16";
    const MX_BAD_ERROR_WORD: &str =
        "error: E0346: mutex concurrency is not available in this surface @11..16";
    const MX_BAD_LOWER: &str = "e0346: mutex concurrency is not available in this surface @11..16";
    const MX_BAD_TWO: &str = "E0346: mutex concurrency is not available in this surface @11..16\nE0347: call to undeclared function `ghost`";
    const MX_E0347: &str = "E0347: call to undeclared function `mutex_new` @11..16";
    const MX_E0206: &str = "E0206: unknown method @11..16";
    const MX_E0006: &str = "E0006: syntax error @11..16";
    const MX_E0100: &str = "E0100: emitted Rust failed rustc";
    /// Free call `mutex_new(0)`: NOT an E0346 form (ADR-291 D2), E0347 stays (ADR-293).
    const MX_SRC_FREE_CALL: &str =
        "module mx_smoke_free\nfn main() -> Io<()> {\n  let m: Int = mutex_new(0)\n  print(m)\n}\n";
    /// Precedence (Sello §4): undeclared call BEFORE the constructor, then AFTER it.
    const MX_SRC_GHOST_FIRST: &str = "module mx_smoke_order_a\nfn main() -> Io<()> {\n  let g: Int = ghost(1)\n  let m: Int = Mutex.new(0)\n  print(m)\n}\n";
    const MX_SRC_GHOST_LAST: &str = "module mx_smoke_order_b\nfn main() -> Io<()> {\n  let m: Int = Mutex.new(0)\n  let g: Int = ghost(1)\n  print(m)\n}\n";
    /// The three forms only inside a comment: a valid program (the word list never applies there).
    const MX_SRC_COMMENT_ONLY: &str = "module mx_smoke_comment\n// Mutex<Int> Arc<Int> n.lock() Mutex.new(0)\nfn main() -> Io<()> {\n  print(\"x\")\n}\n";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn mx_check_diag_accepts_only_the_pinned_text() {
        assert_eq!(MX_CODE, "E0346");
        assert_eq!(
            MX_PREFIX,
            "E0346: mutex concurrency is not available in this surface"
        );
        assert_eq!(&MX_SRC[11..16], MX_ANCHOR);
        assert_eq!(&MX_SRC[24..27], "Arc");
        assert_eq!(mx_check_diag(MX_GOOD, MX_SRC, MX_ANCHOR), Ok((11, 16)));
        assert_eq!(mx_check_diag(MX_GOOD_NL, MX_SRC, MX_ANCHOR), Ok((11, 16)));
        for bad in [
            MX_BAD_NO_SPAN,
            MX_BAD_EMPTY,
            MX_BAD_REVERSED,
            MX_BAD_OUTSIDE,
            MX_BAD_WRONG_WORD,
            MX_BAD_SPACES,
            MX_BAD_SIGN,
            MX_BAD_NO_AT,
            MX_BAD_EXTRA,
            MX_BAD_TAIL,
            MX_BAD_PREFIX,
            MX_BAD_PREFIX_LONG,
            MX_BAD_ERROR_WORD,
            MX_BAD_LOWER,
            MX_BAD_TWO,
            MX_E0347,
            MX_E0206,
            MX_E0006,
            MX_E0100,
            "",
        ] {
            assert!(
                mx_check_diag(bad, MX_SRC, MX_ANCHOR).is_err(),
                "must reject {bad:?}"
            );
        }
        // The same good message is rejected when the anchor is another word (span not on it).
        assert!(mx_check_diag(MX_GOOD, MX_SRC, "Arc").is_err());
        // A span that exactly covers the later `Arc` is fine for `Arc`, not for `Mutex`.
        let arc = "E0346: mutex concurrency is not available in this surface @24..27";
        assert_eq!(mx_check_diag(arc, MX_SRC, "Arc"), Ok((24, 27)));
        assert!(mx_check_diag(arc, MX_SRC, MX_ANCHOR).is_err());
    }

    #[test]
    fn mx_check_diag_error_kinds_are_distinct() {
        let wrong_code = mx_check_diag(MX_E0347, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(wrong_code.contains("expected only E0346"), "{wrong_code}");
        assert!(wrong_code.contains("E0347"), "{wrong_code}");
        let two_codes = mx_check_diag(MX_BAD_TWO, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(two_codes.contains("expected only E0346"), "{two_codes}");
        let prefix = mx_check_diag(MX_BAD_PREFIX, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(prefix.contains("prefix differs"), "{prefix}");
        let long = mx_check_diag(MX_BAD_PREFIX_LONG, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(long.contains("exact suffix"), "{long}");
        let no_span = mx_check_diag(MX_BAD_NO_SPAN, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(no_span.contains("exact suffix"), "{no_span}");
        let empty = mx_check_diag(MX_BAD_EMPTY, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(empty.contains("non-empty and inside"), "{empty}");
        let outside = mx_check_diag(MX_BAD_OUTSIDE, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(outside.contains("non-empty and inside"), "{outside}");
        let word = mx_check_diag(MX_BAD_WRONG_WORD, MX_SRC, MX_ANCHOR).unwrap_err();
        assert!(word.contains("does not sit on the anchor"), "{word}");
        assert!(MX_PREFIX.starts_with(MX_CODE));
    }

    #[test]
    fn mx_inventory_ids_fixtures_and_shapes() {
        let ids: Vec<&str> = CORE10_MX_NEGS.iter().map(|n| n.id).collect();
        assert_eq!(
            ids,
            vec![
                "neg-core10-mutex-type",
                "neg-core10-mutex-arc",
                "neg-core10-mutex-constructor",
                "neg-core10-mutex-lock",
            ]
        );
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 4);
        let clash_pos = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash_pos, 0);
        let clash_neg = NEG_ORACLES.iter().filter(|o| ids.contains(&o.id)).count();
        assert_eq!(clash_neg, 0);
        let rels: Vec<&str> = CORE10_MX_NEGS.iter().map(|n| n.rel).collect();
        assert_eq!(
            rels,
            vec![
                "ejemplos/core10/mutex-reject/01-type.arita",
                "ejemplos/core10/mutex-reject/02-arc.arita",
                "ejemplos/core10/mutex-reject/03-constructor.arita",
                "ejemplos/core10/mutex-reject/04-lock.arita",
            ]
        );
        let anchors: Vec<&str> = CORE10_MX_NEGS.iter().map(|n| n.anchor).collect();
        assert_eq!(anchors, vec!["Mutex", "Arc", "Mutex", "lock"]);
        let root = workspace_root();
        for n in CORE10_MX_NEGS {
            assert_eq!(n.edits[0].0, n.offender, "{}", n.id);
            assert!(n.offender.contains(n.anchor), "{}", n.id);
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let code = core09_ref_mut_code(&src);
            assert_eq!(core10_code_line_count(&code, n.offender), 1, "{}", n.rel);
            assert_eq!(
                core10_code_line_count(&code, "print(\"x\")"),
                1,
                "{}",
                n.rel
            );
            assert!(src.starts_with("module core10_neg_mutex_"), "{}", n.rel);
            assert!(src.contains(n.anchor), "{}", n.rel);
        }
    }

    #[test]
    fn mx_controls_are_legal_twins_and_check_clean() {
        let root = workspace_root();
        for n in CORE10_MX_NEGS {
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let ctl = core10_control_source(&src, n.edits).expect("control");
            let code = core09_ref_mut_code(&ctl);
            assert!(!code.contains("Mutex"), "{} {code}", n.id);
            assert!(!code.contains("Arc"), "{} {code}", n.id);
            assert!(!code.contains(".lock("), "{} {code}", n.id);
            assert_eq!(ctl.lines().count(), src.lines().count(), "{}", n.id);
            let checked = crate::parse_lower_check(&ctl);
            assert!(checked.is_ok(), "{}: {:?}", n.id, checked.as_ref().err());
        }
        // A control edit that matches nothing is an error, never a silent pass.
        let bad: &[Core10CtlEdit] = &[("let nothing: Int = 0", Some("let nothing: Int = 1"))];
        let src = fs::read_to_string(root.join(CORE10_MX_NEGS[0].rel)).expect("01");
        let err = core10_control_source(&src, bad).unwrap_err();
        assert!(err.contains("matched 0 lines"), "{err}");
    }

    #[test]
    fn mx_probe_can_fail_on_mutated_programs() {
        let root = workspace_root();
        let tmp = std::env::temp_dir().join(format!("arita_b291_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // (a) the real neg is ACCEPTED by the probe, and its span sits on the anchor.
        for n in CORE10_MX_NEGS {
            let ok = mx_probe(&root, n.rel, n.anchor);
            assert!(ok.is_ok(), "{}: {:?}", n.id, ok.as_ref().err());
            let (a, b) = ok.unwrap();
            assert!(a < b, "{}", n.id);
        }
        // (b) the legal twin (control of 04) parses clean ⇒ the probe must reject it.
        let lock = fs::read_to_string(root.join(CORE10_MX_NEGS[3].rel)).expect("04");
        let lock_ctl = core10_control_source(&lock, CORE10_MX_NEGS[3].edits).expect("control");
        fs::write(tmp.join("ctl04.arita"), &lock_ctl).unwrap();
        let (v1, d1) = mx_probe(&tmp, "ctl04.arita", "lock").unwrap_err();
        assert_eq!(v1, Verdict::Rejected);
        assert!(d1.contains("succeeded"), "{d1}");
        // (c) the real neg 04 probed with the WRONG anchor ⇒ rejected on the span.
        let (v2, d2) = mx_probe(&root, CORE10_MX_NEGS[3].rel, "Mutex").unwrap_err();
        assert_eq!(v2, Verdict::Rejected);
        assert!(d2.contains("does not sit on the anchor"), "{d2}");
        // (d) a free `mutex_new(0)` is E0347, not E0346 ⇒ rejected on the CODE.
        fs::write(tmp.join("free.arita"), MX_SRC_FREE_CALL).unwrap();
        let (v3, d3) = mx_probe(&tmp, "free.arita", "mutex_new").unwrap_err();
        assert_eq!(v3, Verdict::Rejected);
        assert!(d3.contains("expected only E0346"), "{d3}");
        assert!(d3.contains("E0347"), "{d3}");
        // (e) precedence (Sello §4): E0346 beats E0347 with the undeclared call before AND after.
        fs::write(tmp.join("order_a.arita"), MX_SRC_GHOST_FIRST).unwrap();
        let a_ok = mx_probe(&tmp, "order_a.arita", "Mutex");
        assert!(a_ok.is_ok(), "{:?}", a_ok.as_ref().err());
        fs::write(tmp.join("order_b.arita"), MX_SRC_GHOST_LAST).unwrap();
        let b_ok = mx_probe(&tmp, "order_b.arita", "Mutex");
        assert!(b_ok.is_ok(), "{:?}", b_ok.as_ref().err());
        // (f) the forms only inside a comment are a valid program ⇒ the probe must reject it.
        fs::write(tmp.join("comment.arita"), MX_SRC_COMMENT_ONLY).unwrap();
        let (v4, d4) = mx_probe(&tmp, "comment.arita", "Mutex").unwrap_err();
        assert_eq!(v4, Verdict::Rejected);
        assert!(d4.contains("succeeded"), "{d4}");
        // (g) missing file ⇒ inconclusive.
        let (v5, d5) = mx_probe(&tmp, "nope.arita", "Mutex").unwrap_err();
        assert_eq!(v5, Verdict::Inconclusive);
        assert!(d5.contains("missing"), "{d5}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn mx_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b291_missing_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let negs: Vec<OracleResult> = CORE10_MX_NEGS
            .iter()
            .map(|n| core10_mx_neg(&tmp, n))
            .collect();
        assert_eq!(negs.len(), 4);
        for r in &negs {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        assert_eq!(negs[0].id, "neg-core10-mutex-type");
        assert_eq!(negs[1].id, "neg-core10-mutex-arc");
        assert_eq!(negs[2].id, "neg-core10-mutex-constructor");
        assert_eq!(negs[3].id, "neg-core10-mutex-lock");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn core10_mx_oracles_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let rs: Vec<OracleResult> = CORE10_MX_NEGS
            .iter()
            .map(|n| core10_mx_neg(&root, n))
            .collect();
        assert_eq!(rs.len(), 4);
        for r in &rs {
            println!("adr291 {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
            assert!(r.detail.contains(MX_PREFIX), "{} {}", r.id, r.detail);
            assert!(
                r.detail.contains("control 0 diagnostics"),
                "{} {}",
                r.id,
                r.detail
            );
        }
    }
}

#[cfg(test)]
mod adr295_must_use_smoke {
    //! CUT CORE-0.10-MUST-USE-20260927 — ADR-295 MS-1..MS-7 wired by Measure. No cargo, no rustc
    //! here except the `gated` test (same pattern as UC / KI / MX): the real verdicts come from
    //! `arita measure`. These tests check the inventory, the fixtures' source text, the pure
    //! checkers on synthetic messages / runs / emitted Rust (one mutation per assertion, so each
    //! can FAIL), the in-process probe on mutated programs (including S2-D1: the RHS error wins
    //! inside the same `let`) and that a missing fixture is Inconclusive (never Accepted). The
    //! probe test needs the S2 detector of `arita-hir` (ADR-295 D3). Each test asserts in its
    //! own body (VT006).
    use super::*;

    const MS_GOOD: &str = "E0272: result error swallowed";
    const MS_GOOD_NL: &str = "E0272: result error swallowed\n";
    const MS_BAD_SPAN: &str = "E0272: result error swallowed @3..9";
    const MS_BAD_EXTRA: &str = "E0272: result error swallowed\nnote: more";
    const MS_BAD_PREFIX: &str = "error: E0272: result error swallowed";
    const MS_BAD_LOWER: &str = "e0272: result error swallowed";
    const MS_BAD_TEXT: &str = "E0272: result error swallowed here";
    const MS_BAD_TWO: &str = "E0272: result error swallowed\nE0340: io result discarded";
    const MS_E0340: &str = "E0340: io result discarded";
    const MS_E0343: &str = "E0343: question mark outside result fn";
    const MS_E0347: &str = "E0347: call to undeclared function `ghost`";
    const MS_E0006: &str = "E0006: construct outside F1.1 (parse failure)";
    const MS_E0100: &str = "E0100: emitted Rust failed rustc";
    const MS_WANT_1: [&str; 1] = ["1"];
    const MS_WANT_2: [&str; 1] = ["2"];
    const MS_EMIT_CLEAN: &str = "fn main() { let x = 1; println!(\"{}\", x); }";
    const MS_EMIT_E1: &str = "fn main() { { let _ = self.m.insert(k, v); } }";
    const MS_EMIT_E2_IDENT: &str = "async fn main() { { let _ = t.await; } }";
    const MS_EMIT_E2_SPAWN: &str = "async fn main() { { let _ = tokio::spawn(worker()).await; } }";
    /// Real shape measured by the Parser: the E2 block ends in `};` (a `;` after the closing brace).
    const MS_EMIT_E2_SEMI: &str = "async fn main() { { let _ = tokio::spawn(worker()).await; }; }";
    /// What the Cargo project of an async program holds in `src/main.rs` (real E2 shape).
    const MS_EMIT_CARGO_MAIN: &str =
        "async fn main() { { let _ = tokio::spawn(worker()).await; }; }";
    const MS_EMIT_E2_IDENT_SEMI: &str = "async fn main() { { let _ = t.await; }; }";
    const MS_EMIT_UNDERSCORE_NAME: &str = "fn main() { let _x = f(); let r_ = 1; }";
    const MS_EMIT_UNWRAP_OR: &str = "fn main() { let n = o.unwrap_or(0); }";
    const MS_EMIT_BARE: &str = "fn main() { let _ = f(); }";
    const MS_EMIT_OUTSIDE: &str = "fn main() { f(); let _ = g(); h(); }";
    const MS_SIG_F: &str = "fn f(x: Int) -> Result<Int, Int> {";
    const MS_SIG_MAIN: &str = "fn main() -> Io<()> {";
    const MS_EMIT_BLOCK_OTHER: &str = "fn main() { { let _ = f(); } }";
    const MS_EMIT_TYPED: &str = "fn main() { { let _: Result<(), ()> = g(); } }";
    const MS_EMIT_E2_OTHER: &str = "async fn main() { { let _ = foo(x).await; } }";
    const MS_EMIT_E1_NO_RECV: &str = "fn main() { { let _ = .insert(k, v); } }";
    const MS_EMIT_NO_SEMI: &str = "fn main() { { let _ = t.await } }";
    const MS_EMIT_UNWRAP: &str = "fn main() { let n = o.unwrap(); }";
    const MS_EMIT_EXPECT: &str = "fn main() { let n = o.expect(\"x\"); }";
    const MS_EMIT_PANIC: &str = concat!("fn main() { ", "panic!", "(", "\"x\"); }");
    /// RHS error wins inside the same `let` (S2-D1): E0347, never a cascade E0272.
    const MS_SRC_GHOST_RHS: &str = "module ms_smoke_rhs\nfn main() -> Io<()> {\n  let _: Result<Int, Int> = ghost(1)\n  print(\"after\")\n}\n";
    /// A used binding: the `match` reads it, so the program is valid.
    const MS_SRC_USED: &str = "module ms_smoke_used\nfn f(x: Int) -> Result<Int, Int> {\n  Ok(x)\n}\nfn main() -> Io<()> {\n  let r: Result<Int, Int> = f(1)\n  match r {\n    Ok(v) => { print(v) }\n    Err(e) => { print(e) }\n  }\n}\n";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn ms_check_diag_accepts_only_the_pinned_text() {
        assert_eq!(MS_GOOD, MS_NEG_TEXT);
        assert!(MS_NEG_TEXT.starts_with(MS_NEG_CODE));
        assert!(ms_check_diag(MS_GOOD).is_ok());
        assert!(ms_check_diag(MS_GOOD_NL).is_ok());
        for bad in [
            MS_BAD_SPAN,
            MS_BAD_EXTRA,
            MS_BAD_PREFIX,
            MS_BAD_LOWER,
            MS_BAD_TEXT,
            MS_BAD_TWO,
            MS_E0340,
            MS_E0343,
            MS_E0347,
            MS_E0006,
            MS_E0100,
            "",
        ] {
            assert!(ms_check_diag(bad).is_err(), "must reject {bad:?}");
        }
    }

    #[test]
    fn ms_check_diag_error_kinds_are_distinct() {
        let wrong_code = ms_check_diag(MS_E0347).unwrap_err();
        assert!(wrong_code.contains("expected only E0272"), "{wrong_code}");
        assert!(wrong_code.contains("E0347"), "{wrong_code}");
        let two_codes = ms_check_diag(MS_BAD_TWO).unwrap_err();
        assert!(two_codes.contains("expected only E0272"), "{two_codes}");
        let with_span = ms_check_diag(MS_BAD_SPAN).unwrap_err();
        assert!(with_span.contains("text differs"), "{with_span}");
        let wrong_text = ms_check_diag(MS_BAD_TEXT).unwrap_err();
        assert!(wrong_text.contains("text differs"), "{wrong_text}");
        let extra = ms_check_diag(MS_BAD_EXTRA).unwrap_err();
        assert!(extra.contains("text differs"), "{extra}");
    }

    #[test]
    fn ms_check_run_and_emit_check_can_fail() {
        // Positives: the generic run checker, with the S2 stdout pins.
        assert!(ki_check_run(Some(0), "1\n", "", &MS_WANT_1).is_ok());
        assert!(ki_check_run(Some(0), "2\n", "", &MS_WANT_2).is_ok());
        assert!(ki_check_run(Some(0), "2\n", "", &MS_WANT_1).is_err());
        assert!(ki_check_run(Some(0), "1\nextra\n", "", &MS_WANT_1).is_err());
        assert!(ki_check_run(Some(1), "1\n", "", &MS_WANT_1).is_err());
        assert!(ki_check_run(Some(0), "1\n", "warning", &MS_WANT_1).is_err());
        let p = ki_check_run(Some(0), "1\n", "thread main panicked at x", &MS_WANT_1).unwrap_err();
        assert!(p.contains("panicked"), "{p}");
        // Emit-ban: clean emits and the two fixed exclusions pass.
        assert_eq!(ms_emit_check(MS_EMIT_CLEAN), Ok((0, 0)));
        assert_eq!(ms_emit_check(MS_EMIT_E1), Ok((1, 0)));
        assert_eq!(ms_emit_check(MS_EMIT_E2_IDENT), Ok((0, 1)));
        assert_eq!(ms_emit_check(MS_EMIT_E2_SPAWN), Ok((0, 1)));
        assert_eq!(ms_emit_check(MS_EMIT_E2_SEMI), Ok((0, 1)));
        assert_eq!(ms_emit_check(MS_EMIT_E2_IDENT_SEMI), Ok((0, 1)));
        assert_eq!(ms_emit_check(MS_EMIT_UNDERSCORE_NAME), Ok((0, 0)));
        assert_eq!(ms_emit_check(MS_EMIT_UNWRAP_OR), Ok((0, 0)));
        // Everything else is vetoed.
        for bad in [
            MS_EMIT_BARE,
            MS_EMIT_OUTSIDE,
            MS_EMIT_BLOCK_OTHER,
            MS_EMIT_TYPED,
            MS_EMIT_E2_OTHER,
            MS_EMIT_E1_NO_RECV,
            MS_EMIT_NO_SEMI,
            MS_EMIT_UNWRAP,
            MS_EMIT_EXPECT,
            MS_EMIT_PANIC,
        ] {
            assert!(ms_emit_check(bad).is_err(), "must veto {bad:?}");
        }
        let unwrap = ms_emit_check(MS_EMIT_UNWRAP).unwrap_err();
        assert!(unwrap.contains(".unwrap()"), "{unwrap}");
        let bare = ms_emit_check(MS_EMIT_BARE).unwrap_err();
        assert!(bare.contains("not E1/E2"), "{bare}");
        let outside = ms_emit_check(MS_EMIT_OUTSIDE).unwrap_err();
        assert!(outside.contains("outside an E1/E2 block"), "{outside}");
    }

    #[test]
    fn ms_emit_rust_resolution_covers_cargo_layout() {
        // Pure candidates: sibling `<bin>.rs` first, then `<bin>/../../../src/main.rs`.
        let cands = ms_emit_candidates(Path::new("/p/target/debug/bin"));
        assert_eq!(
            cands,
            vec![
                PathBuf::from("/p/target/debug/bin.rs"),
                PathBuf::from("/p/src/main.rs")
            ]
        );
        // A path without three parents keeps only the sibling candidate.
        assert_eq!(ms_emit_candidates(Path::new("bin")).len(), 1);
        // Simulated layouts under a private temp dir (plain file I/O only; nothing is executed).
        let base = std::env::temp_dir().join(format!("arita-ms-resolve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let proj = base.join("cargo_proj");
        let debug = proj.join("target").join("debug");
        fs::create_dir_all(&debug).expect("debug dir");
        fs::create_dir_all(proj.join("src")).expect("src dir");
        let bin = debug.join("worker_bin");
        // 1. Cargo layout: no `<bin>.rs`, Rust in `src/main.rs` → resolved and judged E2.
        fs::write(proj.join("src").join("main.rs"), MS_EMIT_CARGO_MAIN).expect("main.rs");
        let got = ms_read_emitted_rust(&bin);
        assert_eq!(
            got.as_ref().ok().map(String::as_str),
            Some(MS_EMIT_CARGO_MAIN)
        );
        assert_eq!(ms_emit_check(&got.unwrap()), Ok((0, 1)));
        // 2. A sibling `<bin>.rs` wins over `src/main.rs`.
        fs::write(debug.join("worker_bin.rs"), MS_EMIT_CLEAN).expect("sibling");
        assert_eq!(
            ms_read_emitted_rust(&bin).ok().as_deref(),
            Some(MS_EMIT_CLEAN)
        );
        // 3. Neither exists → explicit error naming both paths, never an empty Ok.
        let bare = base
            .join("bare")
            .join("target")
            .join("debug")
            .join("none_bin");
        let err = ms_read_emitted_rust(&bare).unwrap_err();
        assert!(err.contains("none_bin.rs"), "{err}");
        assert!(err.contains("src/main.rs"), "{err}");
        assert!(err.contains(" and "), "{err}");
        // 4. The vetoed text in a Cargo `src/main.rs` is still judged (not skipped).
        let bad = base.join("bad");
        fs::create_dir_all(bad.join("target").join("debug")).expect("bad debug");
        fs::create_dir_all(bad.join("src")).expect("bad src");
        fs::write(bad.join("src").join("main.rs"), MS_EMIT_UNWRAP).expect("bad main.rs");
        let bad_rs = ms_read_emitted_rust(&bad.join("target").join("debug").join("b"));
        assert!(ms_emit_check(&bad_rs.unwrap()).is_err());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn ms_inventory_ids_fixtures_and_shapes() {
        let mut ids: Vec<&str> = CORE10_MS_NEGS.iter().map(|n| n.id).collect();
        ids.extend(CORE10_MS_POS.iter().map(|p| p.id));
        ids.push(MS_EMIT_ID);
        assert_eq!(
            ids,
            vec![
                "neg-core10-let-underscore",
                "neg-core10-let-named-dead",
                "neg-core10-bare-stmt",
                "core10-qmark-use",
                "core10-let-used",
                "core10-underscore-read",
                "core10-must-use-emit-ban",
            ]
        );
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 7);
        let clash_pos = EJEMPLO_ORACLES
            .iter()
            .filter(|o| ids.contains(&o.id))
            .count();
        assert_eq!(clash_pos, 0);
        let clash_neg = NEG_ORACLES.iter().filter(|o| ids.contains(&o.id)).count();
        assert_eq!(clash_neg, 0);
        let root = workspace_root();
        for n in CORE10_MS_NEGS {
            assert!(
                n.rel.starts_with("ejemplos/core10/errores-neg/"),
                "{}",
                n.rel
            );
            assert_eq!(n.edits[0].0, n.offender, "{}", n.id);
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let code = core09_ref_mut_code(&src);
            assert_eq!(core10_code_line_count(&code, n.offender), 1, "{}", n.rel);
            for l in n.shape.iter() {
                assert_eq!(core10_code_line_count(&code, l), 1, "{} {l}", n.rel);
            }
            assert!(src.contains(&format!("// oracle: {}", n.id)), "{}", n.rel);
        }
        for p in CORE10_MS_POS {
            assert!(p.rel.starts_with("ejemplos/core10/errores/"), "{}", p.rel);
            assert!(!p.want.is_empty(), "{}", p.id);
            let src = fs::read_to_string(root.join(p.rel)).expect("positive fixture");
            let code = core09_ref_mut_code(&src);
            for l in p.shape.iter() {
                assert_eq!(core10_code_line_count(&code, l), 1, "{} {l}", p.rel);
            }
            assert!(src.contains(&format!("// oracle: {}", p.id)), "{}", p.rel);
        }
        assert_eq!(CORE10_MS_POS[0].want, ["1"]);
        assert_eq!(CORE10_MS_POS[1].want, ["1"]);
        assert_eq!(CORE10_MS_POS[2].want, ["2"]);
        assert_eq!(MS_EMIT_SOURCES.len(), 6);
        assert_eq!(MS_EMIT_SOURCES[0], CORE10_MS_POS[0].rel);
        assert_eq!(MS_EMIT_SOURCES[1], CORE10_MS_POS[1].rel);
        assert_eq!(MS_EMIT_SOURCES[2], CORE10_MS_POS[2].rel);
        assert_eq!(
            MS_EMIT_SOURCES[5],
            "ejemplos/core10/join-safety/pos/01-spawn-iounit.arita"
        );
        for rel in MS_EMIT_SOURCES.iter() {
            assert!(root.join(rel).is_file(), "missing emit source {rel}");
        }
    }

    #[test]
    fn ms_controls_are_legal_twins_and_check_clean() {
        let root = workspace_root();
        for n in CORE10_MS_NEGS {
            let src = fs::read_to_string(root.join(n.rel)).expect("neg fixture");
            let ctl = core10_control_source(&src, n.edits).expect("control");
            let code = core09_ref_mut_code(&ctl);
            assert!(code.contains("match "), "{} {code}", n.id);
            assert!(
                code.contains(MS_SIG_F),
                "{} keeps the params of f (E0006)",
                n.id
            );
            assert!(code.contains(MS_SIG_MAIN), "{}", n.id);
            if n.offender != "f(1)" {
                assert_eq!(core10_code_line_count(&code, "f(1)"), 0, "{}", n.id);
            }
            let checked = crate::parse_lower_check(&ctl);
            assert!(checked.is_ok(), "{}: {:?}", n.id, checked.as_ref().err());
        }
        for p in CORE10_MS_POS {
            let src = fs::read_to_string(root.join(p.rel)).expect("positive fixture");
            let gated = ms_hir_gate(&src);
            assert!(gated.is_ok(), "{}: {:?}", p.id, gated.as_ref().err());
        }
        // A control edit that matches nothing is an error, never a silent pass.
        let bad: &[Core10CtlEdit] = &[("let nothing: Int = 0", Some("let nothing: Int = 1"))];
        let src = fs::read_to_string(root.join(CORE10_MS_NEGS[0].rel)).expect("05");
        let err = core10_control_source(&src, bad).unwrap_err();
        assert!(err.contains("matched 0 lines"), "{err}");
    }

    #[test]
    fn ms_probe_can_fail_on_mutated_programs() {
        let root = workspace_root();
        let tmp = std::env::temp_dir().join(format!("arita_b295_smoke_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // (a) the three real negs are ACCEPTED by the probe (needs the S2 detector in the HIR).
        for n in CORE10_MS_NEGS {
            let ok = ms_probe(&root, n.rel);
            assert!(ok.is_ok(), "{}: {:?}", n.id, ok.as_ref().err());
            assert_eq!(ok.unwrap().trim(), MS_NEG_TEXT, "{}", n.id);
        }
        // (b) the legal twin (control of the bare statement) checks clean ⇒ the probe must reject.
        let bare = fs::read_to_string(root.join(CORE10_MS_NEGS[2].rel)).expect("07");
        let bare_ctl = core10_control_source(&bare, CORE10_MS_NEGS[2].edits).expect("control");
        fs::write(tmp.join("ctl07.arita"), &bare_ctl).unwrap();
        let (v1, d1) = ms_probe(&tmp, "ctl07.arita").unwrap_err();
        assert_eq!(v1, Verdict::Rejected);
        assert!(d1.contains("succeeded"), "{d1}");
        // (c) a USED binding is a valid program ⇒ the probe must reject it (no false positive).
        fs::write(tmp.join("used.arita"), MS_SRC_USED).unwrap();
        let (v2, d2) = ms_probe(&tmp, "used.arita").unwrap_err();
        assert_eq!(v2, Verdict::Rejected);
        assert!(d2.contains("succeeded"), "{d2}");
        // (d) S2-D1: the RHS error wins inside the same `let` ⇒ E0347, rejected on the CODE.
        fs::write(tmp.join("rhs.arita"), MS_SRC_GHOST_RHS).unwrap();
        let (v3, d3) = ms_probe(&tmp, "rhs.arita").unwrap_err();
        assert_eq!(v3, Verdict::Rejected);
        assert!(d3.contains("expected only E0272"), "{d3}");
        assert!(d3.contains("E0347"), "{d3}");
        // (e) missing file ⇒ inconclusive.
        let (v4, d4) = ms_probe(&tmp, "nope.arita").unwrap_err();
        assert_eq!(v4, Verdict::Inconclusive);
        assert!(d4.contains("missing"), "{d4}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn ms_missing_fixture_is_inconclusive() {
        let tmp = std::env::temp_dir().join(format!("arita_b295_missing_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let negs: Vec<OracleResult> = CORE10_MS_NEGS
            .iter()
            .map(|n| core10_ms_neg(&tmp, n))
            .collect();
        let pos: Vec<OracleResult> = CORE10_MS_POS
            .iter()
            .map(|p| core10_ms_positive(&tmp, p))
            .collect();
        let emit = core10_ms_emit_ban(&tmp);
        assert_eq!(negs.len(), 3);
        assert_eq!(pos.len(), 3);
        for r in negs.iter().chain(pos.iter()).chain(std::iter::once(&emit)) {
            assert_eq!(r.verdict, Verdict::Inconclusive, "{} {}", r.id, r.detail);
            assert!(r.detail.contains("missing"), "{} {}", r.id, r.detail);
        }
        assert_eq!(negs[0].id, "neg-core10-let-underscore");
        assert_eq!(negs[2].id, "neg-core10-bare-stmt");
        assert_eq!(pos[0].id, "core10-qmark-use");
        assert_eq!(pos[2].id, "core10-underscore-read");
        assert_eq!(emit.id, "core10-must-use-emit-ban");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn core10_ms_oracles_gated() {
        let root = workspace_root();
        std::env::set_current_dir(&root).expect("cd workspace");
        let mut rs: Vec<OracleResult> = CORE10_MS_NEGS
            .iter()
            .map(|n| core10_ms_neg(&root, n))
            .collect();
        rs.extend(CORE10_MS_POS.iter().map(|p| core10_ms_positive(&root, p)));
        rs.push(core10_ms_emit_ban(&root));
        assert_eq!(rs.len(), 7);
        for r in &rs {
            println!("adr295 {} {:?} :: {}", r.id, r.verdict, r.detail);
            assert_eq!(r.verdict, Verdict::Accepted, "{} detail={}", r.id, r.detail);
        }
        for r in rs.iter().take(3) {
            assert!(r.detail.contains(MS_GOOD), "{} {}", r.id, r.detail);
            assert!(
                r.detail.contains("control 0 diagnostics"),
                "{} {}",
                r.id,
                r.detail
            );
        }
        for r in rs.iter().skip(3).take(3) {
            assert!(r.detail.contains("exact stdout"), "{} {}", r.id, r.detail);
        }
        assert!(rs[6].detail.contains("6 fixed sources"), "{}", rs[6].detail);
    }
}
