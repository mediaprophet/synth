//! Inspect and qualify the actual engine registry before producing a candidate.
use serde_json::{json, Value};
use vibe::extension_registry::{
    dispatch_prepared_call, prepare_extension_call, CallExecutionContext, ExtensionRegistry,
    BUILTIN_EXTENSION_ALIASES,
};
use vibescript_abi::{
    BorrowedSliceView, CallerOutputSlice, ExecutionMode, NQuin48, PrimitiveKind, SemVerReq,
    TargetProfileMask,
};

const REQUIRED: &[&str] = &[
    "mech", "gpu", "forge", "render", "signal", "sense", "p64", "nquin", "infer", "qgraph",
    "modality",
];

pub fn inspect() -> Result<Value, String> {
    let registry = ExtensionRegistry::with_builtin_providers();
    validate_registry(&registry)?;
    let mut providers = Vec::new();
    for alias in BUILTIN_EXTENSION_ALIASES {
        let (_, provider, descriptor) = registry
            .find_by_namespace_or_alias(alias)
            .ok_or_else(|| format!("Vibe provider {alias} failed registration"))?;
        let functions: Vec<_> = descriptor
            .sub_namespaces
            .iter()
            .flat_map(|namespace| {
                namespace.functions.iter().map(move |function| {
                    json!({
                        "path": format!("{alias}.{}.{}", namespace.name, function.name),
                        "iri": function.canonical_iri.iri,
                        "effects": function.effects.0,
                        "output_elements": function.required_output_elems,
                        "max_ticks": function.envelope.max_ticks.to_string(),
                        "max_output_bytes": function.envelope.max_output_bytes,
                    })
                })
            })
            .collect();
        providers.push(json!({
            "alias": alias,
            "iri": descriptor.canonical_iri.iri,
            "targets": descriptor.supported_targets.0,
            "ontology_records": provider.static_ontology_nquins().len(),
            "functions": functions,
        }));
    }
    Ok(json!({
        "schema": "synth.vibe-capabilities/v1",
        "product": crate::agent_index::product_card(),
        "qualification": "registry admission and SDK-to-Etalon parity dispatch",
        "execution_mode": "standalone-developer-pure-compute",
        "provider_count": registry.active_count(),
        "providers": providers,
        "sdk_parity_probe": parity_probe(&registry)?,
    }))
}

fn validate_registry(registry: &ExtensionRegistry<'_>) -> Result<(), String> {
    for alias in REQUIRED {
        let (_, provider, descriptor) = registry
            .find_by_namespace_or_alias(alias)
            .ok_or_else(|| format!("Required Vibe provider {alias} was not admitted"))?;
        if !descriptor.versions.is_engine_compatible()
            || provider
                .static_ontology_nquins()
                .iter()
                .any(|q| !q.verify_parity())
        {
            return Err(format!(
                "Vibe provider {alias} has incompatible ABI or ontology parity"
            ));
        }
    }
    Ok(())
}

fn parity_probe(registry: &ExtensionRegistry<'_>) -> Result<Value, String> {
    // The SDK constructs the record; Etalon independently checks its five semantic fields.
    let record = NQuin48::invariant(
        "https://qualiadb.org/app/synth/production",
        "https://qualiadb.org/schema/vibe#ParitySealedSource",
        16,
    );
    let expected = record.0[..5].iter().fold(0, |parity, word| parity ^ word);
    if record.parity() != expected {
        return Err("Vibe SDK omitted a semantic field from NQuin parity".into());
    }
    let mut bytes = [0u8; 40];
    for (word, output) in record.0[..5].iter().zip(bytes.chunks_exact_mut(8)) {
        output.copy_from_slice(&word.to_le_bytes());
    }
    let probe = || -> Result<_, vibescript_abi::VibeTrap> {
        let handle = prepare_extension_call(
            registry,
            "nquin",
            "wire",
            "compute_wire_parity",
            SemVerReq::AnyStable,
            TargetProfileMask::NATIVE_DESKTOP,
        )?;
        let input = BorrowedSliceView::from_raw_bytes(&bytes, PrimitiveKind::U64, 5)?;
        let mut result = [0u8; 8];
        let mut output = CallerOutputSlice::new(&mut result, PrimitiveKind::U64)?;
        let context = CallExecutionContext {
            actor_did_hash: 0,
            context_world_hash: 0,
            lamport_timestamp: 0,
            mode: ExecutionMode::StandaloneDeveloper,
        };
        let outcome =
            dispatch_prepared_call(registry, &handle, &context, None, &[input], &mut output)?;
        Ok((result, outcome))
    };
    let (result, outcome) = probe().map_err(|e| format!("Vibe SDK parity dispatch: {e:?}"))?;
    if u64::from_le_bytes(result) != expected
        || outcome.output_elems_written != 1
        || outcome.receipt.simulated
        || outcome.audit_nquins.iter().any(|q| !q.verify_parity())
    {
        return Err("Vibe SDK and Etalon disagree on parity or execution receipt".into());
    }
    Ok(json!({"passed": true, "metadata": "16", "ticks": outcome.ticks_consumed.to_string()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_providers_register_and_sdk_record_crosses_etalon_dispatch() {
        let report = inspect().unwrap();
        assert_eq!(report["provider_count"], 11);
        assert_eq!(report["sdk_parity_probe"]["passed"], true);
        assert_eq!(report["product"]["role"], crate::agent_index::ROLE);
        assert_eq!(report["product"]["version"], crate::agent_index::RELEASE_VERSION);
        assert_eq!(report["product"]["morphisms"], asset_production_vibe::OPERATIONS.len());
    }

    #[test]
    fn hypermedia_ontology_provider_verifies_five_field_parity_and_descriptors() {
        use vibescript_abi::VibeNamespaceProvider;
        let provider = asset_production_vibe::ontology::HypermediaOntologyProvider;
        let desc = provider.descriptor();
        assert!(desc.versions.is_engine_compatible());
        let quins = provider.static_ontology_nquins();
        assert_eq!(
            quins.len(),
            asset_production_vibe::ontology::MORPHISMS.len() * 3
        );
        for q in quins {
            assert!(q.verify_parity());
        }
        for ns in desc.sub_namespaces {
            for f in ns.functions {
                assert!(f.contract.validate_spans(quins.len()));
            }
        }
    }

    #[test]
    fn missing_provider_is_an_error_before_production() {
        let mut registry = ExtensionRegistry::with_builtin_providers();
        let slot = registry.find_by_namespace_or_alias("render").unwrap().0;
        registry.unregister(slot).unwrap();
        assert!(validate_registry(&registry).unwrap_err().contains("render"));
    }
}
