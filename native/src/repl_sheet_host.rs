//! REPL-only bounded typed Sheet routing; other capabilities retain LocalHost behavior.
use vibe::{DiagCode, Diagnostic, Host, LocalHost, Span, Value};

#[cfg(feature = "native-graphical")]
fn bounded_numeric_list(value: &Value) -> bool {
    fn scalar(value: &Value) -> bool {
        matches!(value, Value::I64(_) | Value::U64(_) | Value::F64(_))
    }
    let Value::List(items) = value else { return false; };
    if items.len() > 32 { return false; }
    if items.iter().all(scalar) { return true; }
    items.iter().all(|row| matches!(row, Value::List(cells) if cells.len() <= 32 && cells.iter().all(scalar)))
}

#[cfg(feature = "native-graphical")]
fn same_numeric_list(a: &Value, b: &Value) -> bool {
    fn same(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::I64(x), Value::I64(y)) => x == y,
            (Value::U64(x), Value::U64(y)) => x == y,
            (Value::F64(x), Value::F64(y)) => x.to_bits() == y.to_bits(),
            (Value::List(x), Value::List(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y)),
            _ => false,
        }
    }
    bounded_numeric_list(a) && bounded_numeric_list(b) && same(a, b)
}

pub(crate) struct ReplHost {
    local: LocalHost,
    grants: Vec<String>,
    #[cfg(feature = "native-graphical")]
    core: qualia_core_db::poet_host::PoetSnapshot,
    remaining: usize,
    dispatches: usize,
}
impl ReplHost {
    pub(crate) fn new(grants: Vec<String>) -> Self {
        Self { local: LocalHost::default(), grants, #[cfg(feature = "native-graphical")] core: Default::default(), remaining: 400_000, dispatches: 256 }
    }
    pub(super) fn begin_submission(&mut self) { self.remaining = 400_000; self.dispatches = 256; }
    fn refuse(span: Span, message: &str) -> Diagnostic { Diagnostic::new(DiagCode::E300, span, message) }
    #[cfg(feature = "native-graphical")]
    fn sheet(&mut self, id: &str, args: &Value, span: Span) -> Result<Value, Diagnostic> {
        if !self.grants.iter().any(|grant| grant == id) { return Err(Self::refuse(span, "Sheet target capability not granted")); }
        if !matches!(id, "Sheet.stats" | "Sheet.sum_range" | "Sheet.set_cell" | "Sheet.eval_formula") {
            return Err(Self::refuse(span, "Sheet operation is not implemented by this REPL profile"));
        }
        // Vibe wraps a single non-record dotted argument in equal `t` and
        // `value` fields. This form was not accepted by the Sheet owner as a
        // record; unwrap only that exact list wrapper for positional stats.
        let args = match args {
            Value::Record(fields) if id == "Sheet.stats" && fields.len() == 2 => {
                match (fields.get("t"), fields.get("value")) {
                    (Some(a), Some(b @ Value::List(_))) if same_numeric_list(a, b) => b,
                    _ => args,
                }
            }
            _ => args,
        };
        let fields = match args { Value::Record(fields) => Some(fields), _ => None };
        if id == "Sheet.eval_formula" {
            let formula = fields.and_then(|f| f.get("formula"));
            if !matches!(formula, Some(Value::String(s)) if s.starts_with('=') && s.len() <= 256) { return Err(Self::refuse(span, "Invalid bounded Sheet formula")); }
        }
        let grid = fields.and_then(|f| f.get("grid")).or_else(|| if id == "Sheet.stats" { Some(args) } else { None });
        if let Some(grid) = grid {
            let Value::List(rows) = grid else { return Err(Self::refuse(span, "Sheet grid must be a list")); };
            if rows.len() > 32 { return Err(Self::refuse(span, "Sheet grid exceeds 32 rows")); }
            for row in rows {
                if id == "Sheet.stats" && matches!(row, Value::I64(_) | Value::U64(_) | Value::F64(_)) { continue; }
                let Value::List(cells) = row else { return Err(Self::refuse(span, "Sheet grid row must be a list")); };
                if cells.len() > 32 { return Err(Self::refuse(span, "Sheet grid exceeds 32 columns")); }
            }
        } else if id != "Sheet.eval_formula" { return Err(Self::refuse(span, "Sheet grid is required")); }
        if id == "Sheet.sum_range" {
            let range = fields.and_then(|f| f.get("range").or_else(|| f.get("cells")));
            if !matches!(range, Some(Value::String(s)) if s.len() <= 128) { return Err(Self::refuse(span, "Invalid bounded Sheet range")); }
        }
        let work = if id == "Sheet.eval_formula" { 131_072 } else { 2_048 };
        if self.dispatches == 0 || self.remaining < work { return Err(Self::refuse(span, "REPL Sheet work budget exhausted")); }
        self.dispatches -= 1; self.remaining -= work;
        self.core.capability_invoke(id, args, span)
    }
}
impl Host for ReplHost {
    fn graph_query(&mut self, a: &[Value], n: u64, s: Span) -> Result<Value, Diagnostic> { self.local.graph_query(a,n,s) }
    fn graph_stage(&mut self, v: &Value, s: Span) -> Result<Value, Diagnostic> { self.local.graph_stage(v,s) }
    fn graph_commit(&mut self, s: Span) -> Result<Value, Diagnostic> { self.local.graph_commit(s) }
    fn aura_validate(&mut self, n: &Value, sh: &Value, s: Span) -> Result<Value, Diagnostic> { self.local.aura_validate(n,sh,s) }
    fn pulse_publish(&mut self, t: &str, v: &Value, s: Span) -> Result<Value, Diagnostic> { self.local.pulse_publish(t,v,s) }
    fn time_unix(&mut self, s: Span) -> Result<Value, Diagnostic> { self.local.time_unix(s) }
    fn time_now(&mut self, s: Span) -> Result<Value, Diagnostic> { self.local.time_now(s) }
    fn time_unix_nanos(&mut self, s: Span) -> Result<Value, Diagnostic> { self.local.time_unix_nanos(s) }
    fn time_monotonic_nanos(&mut self, s: Span) -> Result<Value, Diagnostic> { self.local.time_monotonic_nanos(s) }
    fn available_acceleration(&self) -> vibe::AccelerationTier { self.local.available_acceleration() }
    fn capability_invoke(&mut self, id: &str, args: &Value, span: Span) -> Result<Value, Diagnostic> {
        if id.starts_with("Sheet.") {
            #[cfg(feature = "native-graphical")]
            { return self.sheet(id, args, span); }
            #[cfg(not(feature = "native-graphical"))]
            { return Err(Self::refuse(span, "Sheet operations need Poet's native QualiaDB host profile")); }
        }
        self.local.capability_invoke(id, args, span)
    }
}
