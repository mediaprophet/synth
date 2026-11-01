use ecopunk_terrain::{
    species_knowledge::{KnowledgeDisclosure, KnowledgeLimits, SourceFormat, SourceNamespaceRef},
    species_review_resources::CorpusOptions,
};
pub(crate) struct CorpusPolicy {
    sources: Vec<(String, String, SourceFormat)>,
    disclosure: KnowledgeDisclosure,
    knowledge: KnowledgeLimits,
}
impl CorpusPolicy {
    pub fn retain(options: &CorpusOptions<'_>) -> Result<Self, String> {
        let bytes = options
            .sources
            .iter()
            .try_fold(0usize, |n, s| {
                n.checked_add(s.source_iri.len())
                    .and_then(|n| n.checked_add(s.namespace_iri.len()))
            })
            .ok_or("Namespace inventory size overflow")?;
        let working = bytes
            .checked_mul(2)
            .and_then(|n| {
                options
                    .sources
                    .len()
                    .checked_mul(
                        std::mem::size_of::<(String, String, SourceFormat)>()
                            + std::mem::size_of::<SourceNamespaceRef<'_>>(),
                    )
                    .and_then(|v| n.checked_add(v))
            })
            .ok_or("Namespace reservation overflow")?;
        if options.sources.len() > options.knowledge.statements
            || bytes > options.knowledge.lexicon_bytes
            || working > options.knowledge.cold_bytes
        {
            return Err("Native namespace policy exceeds caller knowledge budget".into());
        }
        Ok(Self {
            sources: options
                .sources
                .iter()
                .map(|s| {
                    (
                        s.source_iri.to_owned(),
                        s.namespace_iri.to_owned(),
                        s.format,
                    )
                })
                .collect(),
            disclosure: options.disclosure,
            knowledge: options.knowledge,
        })
    }
    pub fn with_options<T>(
        &self,
        operation: impl FnOnce(&CorpusOptions<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let refs: Vec<_> = self
            .sources
            .iter()
            .map(|(source, namespace, format)| SourceNamespaceRef {
                source_iri: source,
                namespace_iri: namespace,
                format: *format,
            })
            .collect();
        operation(&CorpusOptions {
            sources: &refs,
            disclosure: self.disclosure,
            knowledge: self.knowledge,
        })
    }
}
