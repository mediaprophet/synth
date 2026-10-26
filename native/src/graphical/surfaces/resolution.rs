//! Pure derived surfaces from admitted source recipes and immutable authorized inputs.
use super::{
    detail::{
        render_detail_with_masks, DetailBase, DetailLimits, DetailMaskBase, DetailRecipe, LayerMask,
    },
    SurfaceTextureLimits, TextureResources,
};
use qualia_core_db::{
    bundle::BundleWriter, container_10d::MaterialRecord, render::assets::TextureDependency,
};
use std::collections::BTreeMap;
use vibe::Value;
pub struct ResolvedSurfaces {
    pub materials: BTreeMap<u64, MaterialRecord>,
    pub resources: Option<TextureResources>,
    pub original_inputs: Vec<TextureDependency>,
}
impl ResolvedSurfaces {
    pub fn dependencies(
        &self,
        materials: &[MaterialRecord],
    ) -> Result<Vec<TextureDependency>, String> {
        let mut deps =
            crate::graphical::surface_host::dependencies(materials, self.resources.as_ref())?;
        for d in &self.original_inputs {
            if !deps.iter().any(|v| v.digest == d.digest) {
                deps.push(d.clone());
            }
        }
        Ok(deps)
    }
}
pub fn resolve(
    value: &Value,
    resources: Option<&TextureResources>,
) -> Result<ResolvedSurfaces, String> {
    let Value::Record(root) = value else {
        return Err("Surface resolution requires an assembly record".into());
    };
    let Some(Value::List(parts)) = root.get("parts") else {
        return Err("Surface resolution requires assembly parts".into());
    };
    if parts.len() > 256 {
        return Err("Surface part working bound exceeded".into());
    }
    resolve_parts(parts, resources)
}
pub fn resolve_parts(
    parts: &[Value],
    resources: Option<&TextureResources>,
) -> Result<ResolvedSurfaces, String> {
    if parts.len() > 256 {
        return Err("Surface part working bound exceeded".into());
    }
    let mut inputs = BTreeMap::<u64, (MaterialRecord, Option<DetailRecipe>)>::new();
    for part in parts {
        let Value::Record(p) = part else {
            return Err("Expected part record".into());
        };
        let m = crate::graphical::materials::material_from_value(
            p.get("material").ok_or("Part lacks material")?,
        )?;
        let recipe = p
            .get("detail")
            .filter(|v| !matches!(v, Value::Null))
            .map(DetailRecipe::from_value)
            .transpose()?;
        if let Some(old) = inputs.insert(m.id, (m, recipe.clone())) {
            if old != (m, recipe) {
                return Err("Shared material has conflicting source/detail recipes; make it unique or edit all shared parts".into());
            }
        }
    }
    let mut source_digests = std::collections::BTreeSet::new();
    for (m, recipe) in inputs.values() {
        for role in super::TextureRole::ALL {
            let d = role.digest(m);
            if d != [0; 32] {
                source_digests.insert(d);
            }
        }
        if let Some(DetailRecipe::Paint(p)) = recipe {
            source_digests.insert(super::binding::parse_digest(&p.base_sha256)?);
            for layer in &p.layers {
                if let Some(LayerMask::Resource { sha256, .. }) = &layer.mask {
                    source_digests.insert(super::binding::parse_digest(sha256)?);
                }
            }
        }
    }
    let mut source_bytes = 0usize;
    for d in &source_digests {
        source_bytes = source_bytes
            .checked_add(
                resources
                    .ok_or("Original source texture resources unavailable")?
                    .source_byte_len(*d)?,
            )
            .ok_or("Source input size overflow")?;
    }
    if source_bytes > SurfaceTextureLimits::default().encoded_bytes {
        return Err("Original texture inputs exceed aggregate encoded working profile".into());
    }
    let mut cost_visits = 0usize;
    let mut cost_output = 0usize;
    let mut peak = 0usize;
    for (_, recipe) in inputs.values() {
        if let Some(d) = recipe {
            let cost = d.budget_plan(DetailLimits::default())?;
            cost_visits = cost_visits
                .checked_add(cost.pixel_visits)
                .ok_or("Detail aggregate work overflow")?;
            cost_output = cost_output
                .checked_add(cost.output_bytes)
                .ok_or("Detail aggregate output overflow")?;
            let mask_decode_bytes = match d {
                DetailRecipe::Paint(p) => {
                    let count = p
                        .layers
                        .iter()
                        .filter_map(|l| match &l.mask {
                            Some(LayerMask::Resource { sha256, .. }) => Some(sha256.as_str()),
                            _ => None,
                        })
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    (p.side as usize)
                        .checked_mul(p.side as usize)
                        .and_then(|n| n.checked_mul(4))
                        .and_then(|n| n.checked_mul(count))
                        .ok_or("Detail mask decode size overflow")?
                }
                _ => 0,
            };
            peak = peak.max(
                cost.working_bytes
                    .checked_add(mask_decode_bytes)
                    .ok_or("Detail mask working admission overflow")?,
            );
        }
    }
    let l = SurfaceTextureLimits::default();
    if cost_visits > DetailLimits::default().pixel_visits
        || cost_output > l.encoded_bytes
        || peak
            .checked_add(resources.map_or(0, TextureResources::snapshot_bytes))
            .and_then(|n| n.checked_add(cost_output))
            .and_then(|n| n.checked_add(source_bytes))
            .is_none_or(|n| n > l.working_bytes)
    {
        return Err(
            "Aggregate detail work/output/working admission exceeded before rendering".into(),
        );
    }
    let mut materials = BTreeMap::new();
    let mut generated = Vec::new();
    let mut originals = BTreeMap::new();
    let mut output_bytes = 0usize;
    for d in source_digests {
        let dep = resources
            .ok_or("Source texture resources unavailable")?
            .source_dependency(d)?;
        originals.insert(d, dep);
    }
    for (id, (mut material, recipe)) in inputs {
        if let Some(recipe) = recipe {
            recipe.validate(DetailLimits::default())?;
            let base = match &recipe {
                DetailRecipe::Paint(p) => Some(
                    resources
                        .ok_or("Paint requires an explicitly bound authorized original texture")?
                        .detail_input(super::binding::parse_digest(&p.base_sha256)?, p.side)?,
                ),
                _ => None,
            };
            let mut mask_inputs = Vec::new();
            if let DetailRecipe::Paint(p) = &recipe {
                for sha256 in p
                    .layers
                    .iter()
                    .filter_map(|layer| match &layer.mask {
                        Some(LayerMask::Resource { sha256, .. }) => Some(sha256),
                        _ => None,
                    })
                    .collect::<std::collections::BTreeSet<_>>()
                {
                    let digest = super::binding::parse_digest(sha256)?;
                    mask_inputs.push(
                        resources
                            .ok_or("Paint mask resource snapshot unavailable")?
                            .mask_input(digest, p.side)?,
                    );
                }
            }
            let masks: Vec<_> = mask_inputs
                .iter()
                .map(|m| DetailMaskBase {
                    digest: m.source.digest,
                    width: m.width,
                    height: m.height,
                    rgba: &m.rgba,
                })
                .collect();
            let output = render_detail_with_masks(
                &recipe,
                base.as_ref().map(|b| DetailBase {
                    digest: b.source.digest,
                    width: b.width,
                    height: b.height,
                    rgba: &b.rgba,
                    colour_space: b.colour_space,
                }),
                &masks,
                DetailLimits::default(),
            )?;
            if let Some(b) = base {
                originals.insert(b.source.digest, b.source);
            }
            for m in mask_inputs {
                originals.insert(m.source.digest, m.source);
            }
            for (role, digest) in output.overrides {
                role.set_digest(&mut material, digest);
            }
            for d in output.dependencies {
                if !generated
                    .iter()
                    .any(|old: &TextureDependency| old.digest == d.digest)
                {
                    output_bytes = output_bytes
                        .checked_add(d.bytes.len())
                        .ok_or("Generated texture size overflow")?;
                    if output_bytes > SurfaceTextureLimits::default().encoded_bytes {
                        return Err("Generated maps exceed aggregate native encoded budget".into());
                    }
                    generated.push(d);
                }
            }
            super::draft::validate(&material)?;
        }
        materials.insert(id, material);
    }
    let merged = if generated.is_empty() {
        resources.cloned()
    } else {
        let base = match resources {
            Some(r) => r.clone(),
            None => TextureResources::from_hmc_bytes(
                &BundleWriter::new().build().map_err(|e| e.to_string())?,
                SurfaceTextureLimits::default(),
            )?,
        };
        Some(base.with_dependencies(&generated)?)
    };
    Ok(ResolvedSurfaces {
        materials,
        resources: merged,
        original_inputs: originals.into_values().collect(),
    })
}
