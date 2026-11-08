use super::*;
use vibe::{session::SourceSession, DiagCode, Diagnostic, Host, Span, Value};

pub struct ProductionSession {
    pub(super) producer: Arc<dyn AssetProducer>,
    pub(super) inputs: Vec<Resource>,
    pub(super) limits: ProductionLimits,
    source: SourceSession,
    candidate: Option<Candidate>,
    assets: assets::AssetWorkspace,
}
impl ProductionSession {
    pub(super) fn new(
        producer: Arc<dyn AssetProducer>,
        inputs: Vec<Resource>,
        limits: ProductionLimits,
    ) -> Self {
        Self {
            producer,
            inputs,
            source: SourceSession::new(vec!["capability.invoke".into()], limits.source.clone()),
            limits,
            candidate: None,
            assets: assets::AssetWorkspace::default(),
        }
    }
    pub fn sources(&self) -> &[String] {
        self.source.sources()
    }
    pub fn revision(&self) -> u64 {
        self.source.revision()
    }
    pub fn candidate(&self) -> Option<&Candidate> {
        self.candidate.as_ref()
    }
    pub fn profile(&self) -> (&str, &str) {
        (self.producer.id(), self.producer.version())
    }
    pub fn limits(&self) -> &ProductionLimits {
        &self.limits
    }

    /// Native document binding includes producer identity, input hashes and source history.
    pub fn document_digest(&self) -> Result<[u8; 32], String> {
        use qualia_core_db::bundle::BundleReader;
        use sha2::{Digest, Sha256};
        let candidate = self.candidate().ok_or("No candidate to bind")?;
        let reader = BundleReader::parse(candidate.bundle()).map_err(|e| e.to_string())?;
        Ok(Sha256::digest(
            reader
                .get("synth/production.json")
                .ok_or("Missing manifest")?,
        )
        .into())
    }

    /// Prepare an isolated candidate for an authenticated host to promote atomically.
    pub fn prepare_edit(&self, source: &str, cancel: &AtomicBool) -> Result<Self, String> {
        if cancel.load(Ordering::Acquire) {
            return Err("Production cancelled".into());
        }
        if source.len() > self.limits.source.submission_bytes {
            return Err("Source bytes exceed limit".into());
        }
        let mut next = Self::new(
            self.producer.clone(),
            self.inputs.clone(),
            self.limits.clone(),
        );
        for cell in self.sources() {
            next.submit(cell, cancel)?;
        }
        if next.candidate().map(Candidate::bundle) != self.candidate().map(Candidate::bundle) {
            return Err("Producer replay changed the current candidate before editing".into());
        }
        next.submit(source, cancel)?;
        Ok(next)
    }

    /// Reopen using this session's producer implementation and limits.
    pub fn reopen(&self, bytes: &[u8], cancel: &AtomicBool) -> Result<Self, String> {
        let mut registry = ProducerRegistry::default();
        registry
            .producers
            .insert(self.producer.id().to_owned(), self.producer.clone());
        registry.reopen(bytes, self.limits.clone(), cancel)
    }

    /// Setup cells may define imports/helpers before `asset` exists and return
    /// `None`. Once `asset` exists, validation, generation and packaging succeed
    /// together. Rejection preserves the preceding bindings and candidate.
    pub fn submit(
        &mut self,
        source: &str,
        cancel: &AtomicBool,
    ) -> Result<Option<&Candidate>, String> {
        if cancel.load(Ordering::Acquire) {
            return Err("Production cancelled".into());
        }
        // Reject before cloning retained history; SourceSession enforces its other bounds.
        let total = self
            .sources()
            .iter()
            .try_fold(source.len(), |n, s| n.checked_add(s.len()))
            .ok_or("Source size overflow")?;
        if source.len() > self.limits.source.submission_bytes
            || total > self.limits.source.retained_source_bytes
        {
            return Err("Source bytes exceed limit".into());
        }
        let revision = self.revision().checked_add(1).ok_or("Revision overflow")?;
        let mut sources = self.sources().to_vec();
        sources.push(source.to_owned());
        let mut candidate = None;
        let producer = &self.producer;
        let tool_limits = assets::tool_limits(&self.limits);
        let mut host = assets::AssetHost(asset_production_vibe::AssetHost {
            workspace: self.assets.clone(),
            inputs: &self.inputs,
            limits: &tool_limits,
            cancel,
            work: 0,
        });
        self.source
            .submit_validated_bindings(
                source,
                &mut host,
                self.limits.evaluation.clone(),
                |_, bindings, host| {
                    let context = ProductionContext {
                        inputs: &self.inputs,
                        limits: &self.limits,
                        cancel,
                        assets: &host.workspace,
                    };
                    context.check_cancelled().map_err(diagnostic)?;
                    let Some(asset) = bindings.value("asset") else {
                        return Ok(());
                    };
                    let mut outputs = producer.produce(asset, &context).map_err(diagnostic)?;
                    validate_resources(&outputs, context.limits).map_err(diagnostic)?;
                    if outputs.is_empty() {
                        return Err(diagnostic("Producer returned no output resources"));
                    }
                    outputs.sort_by(|a, b| a.key.cmp(&b.key));
                    let bundle = package::build(
                        producer.as_ref(),
                        &sources,
                        &self.inputs,
                        &outputs,
                        &self.limits,
                    )
                    .map_err(diagnostic)?;
                    context.check_cancelled().map_err(diagnostic)?;
                    candidate = Some(Candidate {
                        bundle,
                        outputs,
                        revision,
                    });
                    Ok(())
                },
            )
            .map_err(|e| {
                format!(
                    "Revision {} retained: {}",
                    e.active_revision,
                    e.diagnostic.to_json()
                )
            })?;
        self.candidate = candidate;
        self.assets = host.0.workspace;
        Ok(self.candidate.as_ref())
    }
}
fn diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagCode::E701, Span::point(0), message.into())
}
// Pure provider operations remain available through Vibe. No graph or publication grants.
impl Host for assets::AssetHost<'_> {
    fn capability_resolve(&mut self, id: &str, _: Span) -> Result<Value, Diagnostic> {
        let canonical = assets::canonical(id);
        let spec = assets::ontology::resolve_morphism(&canonical);
        Ok(Value::Record(std::collections::BTreeMap::from([
            ("id".into(), Value::String(id.into())),
            (
                "canonical_id".into(),
                Value::String(spec.map(|s| s.id).unwrap_or("").into()),
            ),
            (
                "category".into(),
                Value::String(spec.map(|s| s.category).unwrap_or("").into()),
            ),
            (
                "morphism".into(),
                Value::String(spec.map(|s| s.morphism).unwrap_or("").into()),
            ),
            (
                "domain".into(),
                Value::String(spec.map(|s| s.domain).unwrap_or("").into()),
            ),
            (
                "codomain".into(),
                Value::String(spec.map(|s| s.codomain).unwrap_or("").into()),
            ),
            ("vibe_bound".into(), Value::Bool(spec.is_some())),
            ("contract".into(), Value::String("1".into())),
            (
                "arguments".into(),
                Value::String(assets::arguments(id).into()),
            ),
            (
                "owner".into(),
                Value::String(spec.map(|s| s.owner).unwrap_or("").into()),
            ),
        ])))
    }
    fn capability_invoke(&mut self, id: &str, args: &Value, _: Span) -> Result<Value, Diagnostic> {
        self.0
            .invoke(&assets::canonical(id), args)
            .map_err(diagnostic)
    }
    fn graph_query(&mut self, _: &[Value], _: u64, _: Span) -> Result<Value, Diagnostic> {
        Err(diagnostic("Graph queries require an authorized host"))
    }
    fn graph_stage(&mut self, _: &Value, _: Span) -> Result<Value, Diagnostic> {
        Err(diagnostic("Graph writes require an authorized host"))
    }
    fn graph_commit(&mut self, _: Span) -> Result<Value, Diagnostic> {
        Err(diagnostic("Graph commits require an authorized host"))
    }
    fn aura_validate(&mut self, _: &Value, _: &Value, _: Span) -> Result<Value, Diagnostic> {
        Err(diagnostic("Graph validation requires an authorized host"))
    }
    fn pulse_publish(&mut self, _: &str, _: &Value, _: Span) -> Result<Value, Diagnostic> {
        Err(diagnostic("Publication requires an authorized host"))
    }
}
