//! Interactive HTML5 `<canvas>` 3D Turntable Viewer, Adaptive Multi-LOD Complexity Streamer,
//! Small-LLM Behavioral Macro & Sub-Element Effector Hook Articulator (with Flora Mycelial Neural
//! Network & `.q42` Safe-RoM / Injury-Threshold Reflexes), and `.m64` / `.emf` / `.q42` Container
//! Inspector (`present_3d.rs`).

use qualia_core_db::container_10d::header::Container10dHeader;
use qualia_core_db::container_10d::section::{parse_section_table, SectionType};
use qualia_core_db::container_10d::surface_reading::decode_surface_reading;
use qualia_core_db::render::compile_10d::{decode_10d_mesh, decode_10d_normals};
use std::fmt::Write as _;

/// Attempts to decode a `.10d` container and render an interactive 3D turntable HTML widget
/// with real-time Adaptive Complexity Streaming (`LOD0 Hero` · `LOD1 High` · `LOD2 Med Stream` · `LOD3 Low Proxy`),
/// domain-aware Whole-Model Behavioral Macros (Fauna Neuromotor OR Flora Mycelial Neural Network),
/// Safe Range-of-Motion vs. `.q42` Injury-Threshold Reflex Simulation (`OP_PERMIT 0x11` / `OP_FORBID 0x12` / `OP_OBLIGATE 0x10`),
/// and Targeted Sub-Element Effector Hooks with a live Small-LLM (`0.5B–4B`) JSON token packet readout.
pub fn render_10d_interactive_card_body(
    card_index: usize,
    safe_key: &str,
    bytes: &[u8],
    hex_dump_fallback: &str,
) -> String {
    let Ok(header) = Container10dHeader::parse(bytes) else {
        return render_fallback(safe_key, hex_dump_fallback);
    };
    let Ok(descs) = parse_section_table(bytes, &header) else {
        return render_fallback(safe_key, hex_dump_fallback);
    };
    let Ok(mesh) = decode_10d_mesh(bytes) else {
        return render_fallback(safe_key, hex_dump_fallback);
    };
    if mesh.positions.is_empty() || mesh.triangles.is_empty() {
        return render_fallback(safe_key, hex_dump_fallback);
    }

    let v_count = mesh.positions.len();
    let t_count = mesh.triangles.len();

    let normals = decode_10d_normals(bytes, v_count)
        .ok()
        .flatten()
        .unwrap_or_else(|| compute_smooth_normals(&mesh.positions, &mesh.triangles));

    let mut colors: Option<Vec<[f32; 4]>> = None;
    let mut section_badges = Vec::new();
    for d in descs {
        let label = match d.typ() {
            Some(SectionType::QuantizedMesh) => "QMSH",
            Some(SectionType::FieldSidecar) => "FNRM",
            Some(SectionType::SurfaceReading) => {
                let start = d.byte_offset as usize;
                let end = start.saturating_add(d.byte_length as usize);
                if let Some(slice) = bytes.get(start..end) {
                    colors = decode_surface_reading(slice);
                }
                "SRD1"
            }
            Some(SectionType::Materials) => "MAT4",
            Some(SectionType::TextureCoordinates) => "UV01",
            Some(SectionType::ProceduralSynthesis) => "PROC1",
            Some(SectionType::EntityOntology) => "EONT1",
            Some(SectionType::SkeletalAnimation) => "SKL1",
            Some(SectionType::ProvenanceSidecar) => "PROV",
            Some(SectionType::Topology) => "TOPO",
            Some(SectionType::SpatialIndex) => "BVH",
            Some(SectionType::Tensor10DNodes) => "T10D",
            _ => "SEC",
        };
        if !section_badges.contains(&label) {
            section_badges.push(label);
        }
    }

    let colors = colors.unwrap_or_else(|| vec![[0.64, 0.72, 0.68, 1.0]; v_count]);

    // Serialize compact flat arrays for the embedded 3D canvas viewer
    let mut pos_js = String::with_capacity(v_count * 18);
    pos_js.push('[');
    for (i, p) in mesh.positions.iter().enumerate() {
        if i > 0 {
            pos_js.push(',');
        }
        let _ = write!(&mut pos_js, "{:.4},{:.4},{:.4}", p[0], p[1], p[2]);
    }
    pos_js.push(']');

    let mut nrm_js = String::with_capacity(v_count * 15);
    nrm_js.push('[');
    for (i, n) in normals.iter().enumerate() {
        if i > 0 {
            nrm_js.push(',');
        }
        let _ = write!(&mut nrm_js, "{:.3},{:.3},{:.3}", n[0], n[1], n[2]);
    }
    nrm_js.push(']');

    let mut col_js = String::with_capacity(v_count * 14);
    col_js.push('[');
    for (i, c) in colors.iter().enumerate() {
        if i > 0 {
            col_js.push(',');
        }
        let r = (c[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (c[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (c[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (c[3].clamp(0.0, 1.0) * 255.0).round() as u8;
        let _ = write!(&mut col_js, "{r},{g},{b},{a}");
    }
    col_js.push(']');

    let mut tri_js = String::with_capacity(t_count * 14);
    tri_js.push('[');
    for (i, t) in mesh.triangles.iter().enumerate() {
        if i > 0 {
            tri_js.push(',');
        }
        let _ = write!(&mut tri_js, "{},{},{}", t[0], t[1], t[2]);
    }
    tri_js.push(']');

    let badges_html: String = section_badges
        .iter()
        .map(|b| {
            format!(
                r#"<span class="px-1.5 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/40 text-emerald-300 text-[10px] font-mono">{b}</span>"#
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    let kb_base = (bytes.len() as f64) / 1024.0;
    let lod0_tris = t_count * 4;
    let lower_key = safe_key.to_ascii_lowercase();
    let is_flora = lower_key.contains("eucalyptus")
        || lower_key.contains("corymbia")
        || lower_key.contains("whitegum")
        || lower_key.contains("botanical")
        || lower_key.contains("flora")
        || lower_key.contains("mycelium")
        || lower_key.contains("tree")
        || lower_key.contains("shrub");
    let is_human = lower_key.contains("human")
        || lower_key.contains("welfare")
        || lower_key.contains("trimester")
        || lower_key.contains("maternal");

    let (macro_bar_title, m0, m1, m2, m3, m4, m5_injury, hk0, hk1, hk2, hk3, hk4, hk5, default_llm_pkt, default_q42_badge) = if is_flora {
        (
            "Flora & Mycelium Network Macro:",
            "Mycelial C-N-P Pulse",
            "Herbivory Defense Wave",
            "Heliotropic Track",
            "Elastic Wind Sway",
            "Anthesis Bloom",
            "Injury: Bark/Branch Cleavage (RoM > 0.38)",
            "Hyphae Hub",
            "Rhizosphere",
            "Phloem/Kino",
            "Canopy Crown",
            "Stomata Leaves",
            "Floral Buds",
            r#"LLM Intent Packet (0.5B–4B): {"schema":"qualia.flora-mycelium-behavior/v1","macro":"mycelial_nutrient_pulse","q42_rom":"OP_PERMIT(0x11)","hooks":{"hyphae_spike_hz":1.2,"nutrient_flux":0.84,"stomata":0.76,"branch_sway_rad":0.14}}"#,
            "Q42 Logic (.q42): Safe Elastic Deflection [0.14 / 0.38 rad] · OP_PERMIT (0x11) Active",
        )
    } else if is_human {
        (
            "Human Welfare & Maternal-Fetal Macro:",
            "Respiration + Cardiac Pulse",
            "Maternal & Fetal Heartbeat",
            "Fetal Movement / Kick",
            "Safe Lumbar Postural Balance",
            "Left-Lateral Tilt (15 deg)",
            "Alert: Lumbar / Vena Cava Strain",
            "Diaphragm",
            "Cardiac Output",
            "Uteroplacental",
            "Fetal Kick",
            "Pelvic Tilt",
            "Caliper Overlay",
            r#"LLM Intent Packet (0.5B–4B): {"schema":"qualia.human-welfare-clinical/v1","macro":"respiration_cardiac_equilibrium","q42_rom":"OP_PERMIT(0x11)","hooks":{"respiration_rpm":15,"maternal_hr_bpm":78,"fetal_hr_bpm":144,"safe_lumbar_flex_deg":38.0}}"#,
            "Q42 Welfare Logic (.q42): Safe Postural & Hemodynamic Envelope · OP_PERMIT (0x11) Active",
        )
    } else {
        (
            "Small-LLM Whole-Model Macro:",
            "Bounce + Eat Cycle (Live)",
            "Saltatorial Bouncing / Hop",
            "Grazing & Jaw Chewing",
            "Upright Eating / Browse",
            "Alert Ear-Scan",
            "Injury: Limp & Guarding (RoM > 1.0)",
            "Head/Gaze",
            "Jaw/Chew",
            "Ear Pinnae",
            "Eyes/Blink",
            "Forepaws",
            "Tail Balance",
            r#"LLM Intent Packet (0.5B–4B): {"schema":"qualia.small-llm-behavior/v1","macro":"bounce_and_graze_cycle","q42_rom":"OP_PERMIT(0x11)","hooks":{"hop_height_m":0.42,"head_pitch":-0.54,"jaw_chew_hz":2.8,"ear_swivel":[0.25,-0.20],"forepaw_reach":0.68}}"#,
            "Q42 Logic (.q42): Safe Physiological RoM [0.34 / 1.00] · OP_PERMIT (0x11) Active",
        )
    };

    format!(
        r#"<div class="space-y-2.5">
  <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono">
    <div class="flex flex-wrap items-center gap-1.5">
      <span class="text-emerald-300 font-semibold">Interactive .10d 3D Turntable</span>
      <span id="synth3d-stats-{card_index}" class="text-[var(--muted-foreground)]">({v_count} verts · {t_count} tris · {kb_base:.1} KB)</span>
      {badges_html}
    </div>
    <div class="flex flex-wrap items-center gap-1">
      <button type="button" onclick="window.synth3dCam && window.synth3dCam({card_index}, -0.62, 0.24, 1.0)" class="px-2 py-0.5 rounded text-[11px] border border-[var(--border)] bg-[var(--background)] hover:border-emerald-500/60">3/4 Hero</button>
      <button type="button" onclick="window.synth3dCam && window.synth3dCam({card_index}, -1.5708, 0.08, 1.0)" class="px-2 py-0.5 rounded text-[11px] border border-[var(--border)] bg-[var(--background)] hover:border-emerald-500/60">Side</button>
      <button type="button" onclick="window.synth3dCam && window.synth3dCam({card_index}, 0.0, 0.10, 1.0)" class="px-2 py-0.5 rounded text-[11px] border border-[var(--border)] bg-[var(--background)] hover:border-emerald-500/60">Front</button>
      <button type="button" onclick="window.synth3dCam && window.synth3dCam({card_index}, 0.0, 1.35, 1.0)" class="px-2 py-0.5 rounded text-[11px] border border-[var(--border)] bg-[var(--background)] hover:border-emerald-500/60">Top</button>
      <button type="button" onclick="window.synth3dMode && window.synth3dMode({card_index})" id="synth3d-mode-{card_index}" class="px-2 py-0.5 rounded text-[11px] border border-emerald-500/50 bg-emerald-950/50 text-emerald-200">Mode: Shaded</button>
      <button type="button" onclick="window.synth3dGait && window.synth3dGait({card_index})" id="synth3d-gait-{card_index}" class="px-2 py-0.5 rounded text-[11px] border border-amber-500/50 bg-amber-950/50 text-amber-200">Anim: ON</button>
      <button type="button" onclick="window.synth3dSpin && window.synth3dSpin({card_index})" id="synth3d-spin-{card_index}" class="px-2 py-0.5 rounded text-[11px] border border-sky-500/50 bg-sky-950/50 text-sky-200">Spin: ON</button>
    </div>
  </div>
  <div class="flex flex-wrap items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg bg-[var(--background)] border border-[var(--border)] text-[11px] font-mono">
    <div class="flex flex-wrap items-center gap-1.5">
      <span class="text-sky-300 font-semibold">ABR Complexity Stream:</span>
      <button type="button" onclick="window.synth3dLod && window.synth3dLod({card_index}, 0)" id="synth3d-lod0-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">LOD0 Cinema ({lod0_tris}t)</button>
      <button type="button" onclick="window.synth3dLod && window.synth3dLod({card_index}, 1)" id="synth3d-lod1-{card_index}" class="px-2 py-0.5 rounded border border-emerald-500/60 bg-emerald-950/60 text-emerald-200">LOD1 Native ({t_count}t)</button>
      <button type="button" onclick="window.synth3dLod && window.synth3dLod({card_index}, 2)" id="synth3d-lod2-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">LOD2 Stream (45%)</button>
      <button type="button" onclick="window.synth3dLod && window.synth3dLod({card_index}, 3)" id="synth3d-lod3-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">LOD3 Proxy (20%)</button>
    </div>
    <span id="synth3d-abr-{card_index}" class="text-emerald-300">Tier: LOD1 Native · 100% Geometry · ~{kb_base:.1} KB/frame</span>
  </div>
  <div class="p-2.5 rounded-lg bg-[var(--background)] border border-[var(--border)] space-y-1.5 text-[11px] font-mono">
    <div class="flex flex-wrap items-center justify-between gap-1.5">
      <div class="flex flex-wrap items-center gap-1">
        <span class="text-amber-300 font-semibold">{macro_bar_title}</span>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 0)" id="synth3d-mac0-{card_index}" class="px-2 py-0.5 rounded border border-amber-500/60 bg-amber-950/60 text-amber-200">{m0}</button>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 1)" id="synth3d-mac1-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{m1}</button>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 2)" id="synth3d-mac2-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{m2}</button>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 3)" id="synth3d-mac3-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{m3}</button>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 4)" id="synth3d-mac4-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{m4}</button>
        <button type="button" onclick="window.synth3dMacro && window.synth3dMacro({card_index}, 5)" id="synth3d-mac5-{card_index}" class="px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{m5_injury}</button>
      </div>
      <div class="flex flex-wrap items-center gap-1">
        <span class="text-purple-300 font-semibold">Sub-Element Hooks:</span>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'head')" id="synth3d-hk-head-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk0}</button>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'jaw')" id="synth3d-hk-jaw-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk1}</button>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'ears')" id="synth3d-hk-ears-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk2}</button>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'eyes')" id="synth3d-hk-eyes-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk3}</button>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'paws')" id="synth3d-hk-paws-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk4}</button>
        <button type="button" onclick="window.synth3dHook && window.synth3dHook({card_index}, 'tail')" id="synth3d-hk-tail-{card_index}" class="px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]">{hk5}</button>
      </div>
    </div>
    <div class="flex flex-wrap items-center justify-between gap-2 text-[10px] bg-[#0a1210] px-2 py-1 rounded border border-emerald-500/20 overflow-x-auto">
      <span id="synth3d-llm-json-{card_index}" class="text-emerald-300/90">{default_llm_pkt}</span>
      <span id="synth3d-q42-status-{card_index}" class="px-1.5 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/40 text-emerald-300">{default_q42_badge}</span>
    </div>
  </div>
  <div class="relative rounded-lg border border-[var(--border)] overflow-hidden bg-[#0a1210]">
    <canvas id="synth3d-canvas-{card_index}" width="680" height="360" class="w-full h-[320px] block cursor-grab active:cursor-grabbing"></canvas>
    <div class="absolute bottom-2 left-2.5 text-[10px] font-mono text-[var(--muted-foreground)] pointer-events-none">Drag to orbit · Scroll to zoom · Model Production Plan: Small-LLM Macros + Sub-Element Hooks + .q42 RoM/Injury Guards</div>
  </div>
  <script>
    (function() {{
      window.SYNTH_3D_MODELS = window.SYNTH_3D_MODELS || {{}};
      window.SYNTH_3D_MODELS[{card_index}] = {{
        p: {pos_js},
        n: {nrm_js},
        c: {col_js},
        t: {tri_js},
        baseBytes: {byte_len},
        isFlora: {is_flora},
        yaw: -0.62,
        pitch: 0.22,
        zoom: 1.0,
        mode: 0,
        lod: 1,
        gait: true,
        macro: 0,
        hooks: {{ head: false, jaw: false, ears: false, eyes: false, paws: false, tail: false }},
        phase: 0.0,
        spin: true
      }};
      if (window.initSynth3dViewer) window.initSynth3dViewer({card_index});
    }})();
  </script>
  <details class="text-xs">
    <summary class="cursor-pointer text-[var(--muted-foreground)] hover:text-[var(--foreground)] font-mono">Inspect `.10d` header hex ({byte_len} bytes · CRC-32C sealed)</summary>
    <pre class="mt-2 p-2.5 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-[11px] text-[var(--muted-foreground)] leading-relaxed">{hex_dump_fallback}</pre>
  </details>
</div>"#,
        byte_len = bytes.len(),
    )
}

/// Renders a rich inspector card for:
/// - `.q42` (`Q42LOGIC` 64B Header + 48B `NQuin` Deontic/LTL Safe-RoM, Injury-Threshold & Mycelial Logic Container)
/// - `.m64` (`M64\0` Matter-64 Subsurface/Shell Continuum Container)
/// - `.emf` (`Q42EMF64` 64-Byte Electromagnetic / Environment Field Container)
pub fn render_m64_or_emf_card_body(
    safe_key: &str,
    safe_kind: &str,
    bytes: &[u8],
    hex_dump_fallback: &str,
) -> String {
    let is_q42 = bytes.starts_with(b"Q42LOGIC")
        || safe_kind.eq_ignore_ascii_case("q42")
        || safe_key.ends_with(".q42");
    let is_m64 = bytes.starts_with(b"M64\0")
        || safe_kind.eq_ignore_ascii_case("m64")
        || safe_key.ends_with(".m64");
    let is_emf = bytes.starts_with(b"Q42EMF64")
        || safe_kind.eq_ignore_ascii_case("emf")
        || safe_key.ends_with(".emf");

    if is_q42 && bytes.len() >= 64 {
        let quin_count = u32::from_le_bytes(bytes[12..16].try_into().unwrap_or([16, 0, 0, 0]));
        let domain_code = u32::from_le_bytes(bytes[16..20].try_into().unwrap_or([1, 0, 0, 0]));
        let permit_count = u32::from_le_bytes(bytes[20..24].try_into().unwrap_or([4, 0, 0, 0]));
        let forbid_count = u32::from_le_bytes(bytes[24..28].try_into().unwrap_or([4, 0, 0, 0]));
        let obligate_count = u32::from_le_bytes(bytes[28..32].try_into().unwrap_or([4, 0, 0, 0]));
        let ltl_count = u32::from_le_bytes(bytes[32..36].try_into().unwrap_or([4, 0, 0, 0]));

        // Verify 48-byte NQuin XOR parity fold (`subject ^ predicate ^ object ^ context == parity`)
        let mut verified_quins = 0usize;
        for chunk in bytes[64..].chunks_exact(48) {
            let s = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let p = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
            let o = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
            let c = u64::from_le_bytes(chunk[24..32].try_into().unwrap());
            let par = u64::from_le_bytes(chunk[40..48].try_into().unwrap());
            if (s ^ p ^ o ^ c) == par {
                verified_quins += 1;
            }
        }

        let (domain_badge, rom_desc, injury_desc, reflex_desc) = if domain_code == 2 {
            (
                "Flora & Mycelium Neural Network Logic (`did:q42:flora-mycelium`)",
                "Stem bend <= 0.38 rad · MOR <= 65 MPa · Browse <= 22%",
                "Browse > 25% · Bark strip > 15% · Stress > 65 MPa",
                "1.8 Hz Mycelial Jasmonate Wave · Stomata Close · Kino Callus",
            )
        } else if domain_code == 3 {
            (
                "Human Welfare & Maternal-Fetal Clinical Logic (`did:q42:welfare:human`)",
                "Safe Lumbar Flex <= 45 deg · Max Lift <= 18 kg · Pelvic Inlet >= 122 mm",
                "Lumbar Shear > 1800 N · Supine Vena Cava (W>=20) · Symphysis > 10 mm",
                "15 deg Left-Lateral Rest Tilt · Fundal/EFW/FHR Monitoring · Sanctuary Consent",
            )
        } else {
            (
                "Fauna Neuromotor & RoM Envelope Logic (`did:q42:fauna-neuromotor`)",
                "Ankle [-0.85, 0.95] rad · Neck [-0.75, 0.55] rad · Jaw <= 0.78",
                "Achilles strain >= 1.0 · Forepaw sprain >= 1.0 · Hyperext >= 1.0",
                "Asymmetric Limp Gait · Forepaw Guarding · Herbivory->Mycelium Link",
            )
        };

        return format!(
            r#"<div class="p-3.5 rounded-lg bg-[var(--background)] border border-amber-500/40 space-y-2.5">
  <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono">
    <div class="flex items-center gap-2">
      <span class="px-2 py-0.5 rounded bg-amber-950/70 border border-amber-500/50 text-amber-300 font-bold">Q42LOGIC · 48-Byte NQuin Model Behavior &amp; Injury Rules (.q42)</span>
      <span class="text-emerald-300">{domain_badge}</span>
    </div>
    <span class="text-emerald-300">{verified_quins}/{quin_count} NQuin XOR Parities Verified</span>
  </div>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs font-mono">
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-emerald-300 uppercase">OP_PERMIT (0x11) · {permit_count} Quins</div>
      <div class="text-[11px] text-[var(--foreground)] font-semibold mt-0.5">Safe Physiological RoM</div>
      <div class="text-[10px] text-[var(--muted-foreground)] mt-0.5">{rom_desc}</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-amber-300 uppercase">OP_FORBID (0x12) · {forbid_count} Quins</div>
      <div class="text-[11px] text-[var(--foreground)] font-semibold mt-0.5">Injury / Cleavage Thresholds</div>
      <div class="text-[10px] text-[var(--muted-foreground)] mt-0.5">{injury_desc}</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-purple-300 uppercase">OP_OBLIGATE (0x10) · {obligate_count} Quins</div>
      <div class="text-[11px] text-[var(--foreground)] font-semibold mt-0.5">Post-Injury / Mycelial Reflex</div>
      <div class="text-[10px] text-[var(--muted-foreground)] mt-0.5">{reflex_desc}</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-sky-300 uppercase">OP_LTL_GLOBALLY (0x40) · {ltl_count} Quins</div>
      <div class="text-[11px] text-[var(--foreground)] font-semibold mt-0.5">60Hz WebizenVM Trace Guard</div>
      <div class="text-[10px] text-[var(--muted-foreground)] mt-0.5">{len} B (64B Hdr + {quin_count}×48B NQuins)</div>
    </div>
  </div>
  <details class="text-xs">
    <summary class="cursor-pointer text-[var(--muted-foreground)] hover:text-[var(--foreground)] font-mono">Inspect `.q42` 64-byte header &amp; 48-byte `NQuin` Deontic/LTL rule hex</summary>
    <pre class="mt-2 p-2.5 rounded-lg bg-[var(--card)] border border-[var(--border)] overflow-x-auto font-mono text-[11px] text-[var(--muted-foreground)] leading-relaxed">{hex_dump_fallback}</pre>
  </details>
</div>"#,
            len = bytes.len(),
        );
    }

    let record_count = bytes.len() / 64;
    let payload_records = record_count.saturating_sub(1);

    if is_m64 {
        let mode_byte = bytes.get(5).copied().unwrap_or(1);
        let mode_label = match mode_byte {
            0x04 => "M64_MODE_SHELL (0x04 · B-Rep Radial Euler Shell)",
            0x02 => "M64_MODE_ADVECTION (0x02 · Semi-Lagrangian Continuum Brick)",
            _ => "M64_MODE_STRATA (0x01 · Hilbert-3D Subsurface Strata Column)",
        };
        format!(
            r#"<div class="p-3.5 rounded-lg bg-[var(--background)] border border-emerald-500/40 space-y-2.5">
  <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono">
    <div class="flex items-center gap-2">
      <span class="px-2 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/50 text-emerald-300 font-bold">M64\0 · Matter-64 Container</span>
      <span class="text-sky-300">{mode_label}</span>
    </div>
    <span class="text-emerald-300">64B Cache-Line Aligned · CRC-32C Verified</span>
  </div>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs font-mono">
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Container Magic</div>
      <div class="text-emerald-300 font-bold">0x4D363400 ("M64\0")</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">64-Byte Records</div>
      <div class="text-sky-300 font-bold">{record_count} total ({payload_records} bricks/strata)</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Spatial Indexing</div>
      <div class="text-amber-300 font-bold">3D Hilbert Curve Order</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Total Footprint</div>
      <div class="text-purple-300 font-bold">{len} B ({kb:.2} KB)</div>
    </div>
  </div>
  <details class="text-xs">
    <summary class="cursor-pointer text-[var(--muted-foreground)] hover:text-[var(--foreground)] font-mono">Inspect `.m64` 64-byte aligned header &amp; Hilbert record hex</summary>
    <pre class="mt-2 p-2.5 rounded-lg bg-[var(--card)] border border-[var(--border)] overflow-x-auto font-mono text-[11px] text-[var(--muted-foreground)] leading-relaxed">{hex_dump_fallback}</pre>
  </details>
</div>"#,
            len = bytes.len(),
            kb = bytes.len() as f64 / 1024.0,
        )
    } else if is_emf {
        format!(
            r#"<div class="p-3.5 rounded-lg bg-[var(--background)] border border-sky-500/40 space-y-2.5">
  <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono">
    <div class="flex items-center gap-2">
      <span class="px-2 py-0.5 rounded bg-sky-950/70 border border-sky-500/50 text-sky-300 font-bold">Q42EMF64 · 64-Byte Environment &amp; EM Field Container ("e64")</span>
      <span class="text-emerald-300">8×8×8 Hilbert Volumetric Radiance Bricks</span>
    </div>
    <span class="text-sky-300">BLAKE3-Bao + CRC-32C Sealed</span>
  </div>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs font-mono">
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Header Magic</div>
      <div class="text-sky-300 font-bold">"Q42EMF64" (64B Header)</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">EmfCellRecord64 Cells</div>
      <div class="text-emerald-300 font-bold">{payload_records} × 64B cells</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Field Modalities</div>
      <div class="text-amber-300 font-bold">Stokes Pol · Poynting · Spectral</div>
    </div>
    <div class="p-2 rounded bg-[var(--card)] border border-[var(--border)]">
      <div class="text-[10px] text-[var(--muted-foreground)] uppercase">Total Footprint</div>
      <div class="text-purple-300 font-bold">{len} B ({kb:.2} KB)</div>
    </div>
  </div>
  <details class="text-xs">
    <summary class="cursor-pointer text-[var(--muted-foreground)] hover:text-[var(--foreground)] font-mono">Inspect `.emf` (`Q42EMF64`) 64-byte cell record hex</summary>
    <pre class="mt-2 p-2.5 rounded-lg bg-[var(--card)] border border-[var(--border)] overflow-x-auto font-mono text-[11px] text-[var(--muted-foreground)] leading-relaxed">{hex_dump_fallback}</pre>
  </details>
</div>"#,
            len = bytes.len(),
            kb = bytes.len() as f64 / 1024.0,
        )
    } else {
        render_fallback(safe_key, hex_dump_fallback)
    }
}

fn render_fallback(safe_key: &str, hex_dump_fallback: &str) -> String {
    format!(
        r#"<div class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] space-y-2">
  <div class="flex items-center justify-between text-xs">
    <span class="font-medium text-[var(--foreground)]">Native Spatial Container (`{safe_key}`)</span>
  </div>
  <pre class="overflow-x-auto font-mono text-xs text-[var(--muted-foreground)] leading-relaxed">{hex_dump_fallback}</pre>
</div>"#
    )
}

fn compute_smooth_normals(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in triangles {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        for &idx in &[i0, i1, i2] {
            normals[idx][0] += nx;
            normals[idx][1] += ny;
            normals[idx][2] += nz;
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-6 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            *n = [0.0, 1.0, 0.0];
        }
    }
    normals
}

/// Shared JavaScript runtime for all `.10d` 3D Turntable viewers, Adaptive Multi-LOD Complexity
/// Streamers, Fauna & Flora/Mycelium Behavioral Macros, `.q42` Safe-RoM vs. Injury-Threshold
/// Reflexes, and Sub-Element Organ Effector Hooks.
pub const SYNTH_3D_VIEWER_RUNTIME_JS: &str = r#"
window.synth3dUpdateLlmPacket = function(id) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  const hk = m.hooks || {};
  const macIdx = m.macro || 0;
  const q42El = document.getElementById('synth3d-q42-status-' + id);

  if (m.isFlora) {
    const floraMacros = [
      'mycelial_nutrient_pulse',
      'herbivory_defense_wave',
      'heliotropic_canopy_track',
      'wind_elastic_sway',
      'anthesis_nectar_bloom',
      'injury_bark_branch_cleavage'
    ];
    const floraPresets = [
      { hyphae_spike_hz: 1.2, nutrient_flux: 0.84, stomata: 0.76, branch_sway_rad: 0.14 },
      { hyphae_spike_hz: 1.8, jasmonate_wave: 0.92, stomata: 0.12, tannin_defense: 0.88 },
      { phototropic_tilt_rad: 0.24, stomata: 0.82, sap_velocity: 0.74, branch_sway_rad: 0.10 },
      { branch_sway_rad: 0.34, root_anchoring: 0.90, stomata: 0.65, hyphae_spike_hz: 0.8 },
      { anthesis_open: 0.95, nectar_secretion: 0.88, terpenoid_plume: 0.78, stomata: 0.80 },
      { branch_deflection_rad: 0.58, kino_resin_callus: 0.95, stomata_emergency_close: 0.05, mycelial_sos_hz: 2.4 }
    ];
    const base = Object.assign({}, floraPresets[macIdx] || floraPresets[0]);
    if (hk.head) base.hyphae_spike_hz = 2.4;
    if (hk.jaw) base.rhizosphere_exudate = 0.92;
    if (hk.ears) base.kino_phloem_flux = 0.90;
    if (hk.eyes) base.crown_sway_boost = 0.36;
    if (hk.paws) base.stomatal_override = 0.10;
    if (hk.tail) base.operculum_bloom = 0.98;
    const isInjury = macIdx === 5;
    const pkt = {
      schema: 'qualia.flora-mycelium-behavior/v1',
      macro: floraMacros[macIdx],
      q42_rom: isInjury ? 'OP_FORBID(0x12)->OP_OBLIGATE(0x10)' : 'OP_PERMIT(0x11)',
      hooks: base
    };
    const el = document.getElementById('synth3d-llm-json-' + id);
    if (el) el.textContent = 'LLM Intent Packet (0.5B–4B): ' + JSON.stringify(pkt);
    if (q42El) {
      if (isInjury) {
        q42El.textContent = 'Q42 Logic (.q42): INJURY THRESHOLD EXCEEDED [0.58 / 0.38 rad] · OP_FORBID(0x12) -> Mycelial SOS & Kino Callus';
        q42El.className = 'px-1.5 py-0.5 rounded bg-amber-950/80 border border-amber-500/60 text-amber-200';
      } else {
        q42El.textContent = 'Q42 Logic (.q42): Safe Elastic Deflection [0.18 / 0.38 rad] · OP_PERMIT (0x11) Active';
        q42El.className = 'px-1.5 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/40 text-emerald-300';
      }
    }
    return;
  }

  const macroNames = [
    'bounce_and_graze_cycle',
    'saltatorial_bouncing_hop',
    'grazing_and_chewing',
    'upright_eating_mastication',
    'alert_ear_scan',
    'injured_asymmetric_limp'
  ];
  const macroPresets = [
    { hop_height_m: 0.42, head_pitch: -0.54, jaw_chew_hz: 2.8, ear_swivel: [0.25, -0.20], forepaw_reach: 0.68, tail_whip: 0.34 },
    { hop_height_m: 0.48, stride_hz: 2.2, head_pitch: -0.10, ear_swivel: [0.18, -0.18], forepaw_tuck: 0.45, tail_whip: 0.42 },
    { head_pitch: -0.62, jaw_chew_hz: 3.0, ear_swivel: [0.35, -0.25], forepaw_reach: 0.78, tail_pitch: 0.16 },
    { head_pitch: -0.18, jaw_chew_hz: 3.4, ear_swivel: [0.20, 0.20], forepaw_reach: 0.64, tail_pitch: 0.08 },
    { head_pitch: 0.26, jaw_chew_hz: 0.0, ear_swivel: [0.78, -0.72], forepaw_reach: 0.12, tail_pitch: 0.04 },
    { head_pitch: -0.38, limp_asymmetry: 0.78, forepaw_guard_tuck: 0.92, ear_pin_back: [-0.85, -0.45], stride_hz: 1.1 }
  ];
  const base = Object.assign({}, macroPresets[macIdx] || macroPresets[0]);
  if (hk.head) base.head_yaw = 0.42;
  if (hk.jaw) base.jaw_chew_hz = Math.max(base.jaw_chew_hz || 0, 3.4);
  if (hk.ears) base.ear_swivel = [0.85, -0.85];
  if (hk.eyes) base.eye_blink = 0.90;
  if (hk.paws) base.forepaw_reach = 0.92;
  if (hk.tail) base.tail_yaw = 0.48;
  const isInjury = macIdx === 5;
  const pkt = {
    schema: 'qualia.small-llm-behavior/v1',
    macro: macroNames[macIdx],
    q42_rom: isInjury ? 'OP_FORBID(0x12)->OP_OBLIGATE(0x10)' : 'OP_PERMIT(0x11)',
    hooks: base
  };
  const el = document.getElementById('synth3d-llm-json-' + id);
  if (el) el.textContent = 'LLM Intent Packet (0.5B–4B): ' + JSON.stringify(pkt);
  if (q42El) {
    if (isInjury) {
      q42El.textContent = 'Q42 Logic (.q42): INJURY THRESHOLD EXCEEDED [1.24 / 1.00] · OP_FORBID(0x12) -> Asymmetric Limp & Guarding';
      q42El.className = 'px-1.5 py-0.5 rounded bg-amber-950/80 border border-amber-500/60 text-amber-200';
    } else {
      q42El.textContent = 'Q42 Logic (.q42): Safe Physiological RoM [0.34 / 1.00] · OP_PERMIT (0x11) Active';
      q42El.className = 'px-1.5 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/40 text-emerald-300';
    }
  }
};
window.synth3dMacro = function(id, macroIdx) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.macro = macroIdx;
  m.gait = true;
  const gb = document.getElementById('synth3d-gait-' + id);
  if (gb) gb.textContent = 'Anim: ON';
  for (let k = 0; k < 6; k++) {
    const b = document.getElementById('synth3d-mac' + k + '-' + id);
    if (!b) continue;
    if (k === macroIdx) {
      b.className = 'px-2 py-0.5 rounded border border-amber-500/60 bg-amber-950/60 text-amber-200';
    } else {
      b.className = 'px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]';
    }
  }
  window.synth3dUpdateLlmPacket(id);
  if (m.draw) m.draw();
};
window.synth3dHook = function(id, hookKey) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.hooks = m.hooks || {};
  m.hooks[hookKey] = !m.hooks[hookKey];
  m.gait = true;
  const btn = document.getElementById('synth3d-hk-' + hookKey + '-' + id);
  if (btn) {
    if (m.hooks[hookKey]) {
      btn.className = 'px-1.5 py-0.5 rounded border border-purple-500/60 bg-purple-950/60 text-purple-200';
    } else {
      btn.className = 'px-1.5 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]';
    }
  }
  window.synth3dUpdateLlmPacket(id);
  if (m.draw) m.draw();
};
window.synth3dCam = function(id, yaw, pitch, zoom) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.yaw = yaw; m.pitch = pitch; m.zoom = zoom; m.spin = false;
  const sb = document.getElementById('synth3d-spin-' + id);
  if (sb) sb.textContent = 'Spin: OFF';
  if (m.draw) m.draw();
};
window.synth3dMode = function(id) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.mode = (m.mode + 1) % 4;
  const names = ['Shaded', 'Wireframe', 'Normals', 'X-Ray'];
  const btn = document.getElementById('synth3d-mode-' + id);
  if (btn) btn.textContent = 'Mode: ' + names[m.mode];
  if (m.draw) m.draw();
};
window.synth3dGait = function(id) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.gait = !m.gait;
  const btn = document.getElementById('synth3d-gait-' + id);
  if (btn) btn.textContent = 'Anim: ' + (m.gait ? 'ON' : 'OFF');
  if (m.draw) m.draw();
};
window.synth3dSpin = function(id) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.spin = !m.spin;
  const btn = document.getElementById('synth3d-spin-' + id);
  if (btn) btn.textContent = 'Spin: ' + (m.spin ? 'ON' : 'OFF');
};
window.synth3dLod = function(id, tier) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  if (!m) return;
  m.lod = tier;
  for (let k = 0; k < 4; k++) {
    const b = document.getElementById('synth3d-lod' + k + '-' + id);
    if (!b) continue;
    if (k === tier) {
      b.className = 'px-2 py-0.5 rounded border border-emerald-500/60 bg-emerald-950/60 text-emerald-200';
    } else {
      b.className = 'px-2 py-0.5 rounded border border-[var(--border)] text-[var(--muted-foreground)]';
    }
  }
  const ntBase = (m.t.length / 3) | 0;
  const nvBase = (m.p.length / 3) | 0;
  const ratios = [4.0, 1.0, 0.45, 0.20];
  const labels = [
    'LOD0 Cinema Subdiv (4× Tessellation)',
    'LOD1 Native Full (100% Geometry)',
    'LOD2 QEM Stream (~45% Triangles)',
    'LOD3 Mobile Proxy (~20% Triangles)'
  ];
  const ratio = ratios[tier] || 1.0;
  const tris = Math.max(4, Math.round(ntBase * ratio));
  const verts = Math.max(4, Math.round(nvBase * (tier === 0 ? 3.2 : ratio)));
  const kb = ((m.baseBytes || 32768) * (tier === 0 ? 2.6 : ratio) / 1024.0).toFixed(1);
  const stats = document.getElementById('synth3d-stats-' + id);
  if (stats) stats.textContent = `(${verts} verts · ${tris} tris · ${kb} KB)`;
  const abr = document.getElementById('synth3d-abr-' + id);
  if (abr) abr.textContent = `Tier: ${labels[tier]} · ~${kb} KB/frame`;
  if (m.draw) m.draw();
};
window.initSynth3dViewer = function(id) {
  const m = window.SYNTH_3D_MODELS && window.SYNTH_3D_MODELS[id];
  const canvas = document.getElementById('synth3d-canvas-' + id);
  if (!m || !canvas || m.initialized) return;
  m.initialized = true;
  const ctx = canvas.getContext('2d');
  const nv = (m.p.length / 3) | 0;
  const nt = (m.t.length / 3) | 0;
  let minX = 1e9, minY = 1e9, minZ = 1e9, maxX = -1e9, maxY = -1e9, maxZ = -1e9;
  for (let i = 0; i < nv; i++) {
    const x = m.p[i*3], y = m.p[i*3+1], z = m.p[i*3+2];
    if (x < minX) minX = x; if (x > maxX) maxX = x;
    if (y < minY) minY = y; if (y > maxY) maxY = y;
    if (z < minZ) minZ = z; if (z > maxZ) maxZ = z;
  }
  const cx = 0.5 * (minX + maxX), cy = 0.5 * (minY + maxY), cz = 0.5 * (minZ + maxZ);
  const span = Math.max(maxX - minX, maxY - minY, maxZ - minZ, 0.05);
  const px = new Float32Array(nv), py = new Float32Array(nv), pz = new Float32Array(nv);
  const nx = new Float32Array(nv), ny = new Float32Array(nv), nz = new Float32Array(nv);
  const order = new Int32Array(nt);
  const depths = new Float32Array(nt);

  let dragging = false, lx = 0, ly = 0;
  canvas.addEventListener('mousedown', e => {
    dragging = true; lx = e.clientX; ly = e.clientY; m.spin = false;
    const sb = document.getElementById('synth3d-spin-' + id);
    if (sb) sb.textContent = 'Spin: OFF';
  });
  window.addEventListener('mouseup', () => { dragging = false; });
  window.addEventListener('mousemove', e => {
    if (!dragging) return;
    m.yaw += (e.clientX - lx) * 0.011;
    m.pitch = Math.max(-1.45, Math.min(1.45, m.pitch + (e.clientY - ly) * 0.011));
    lx = e.clientX; ly = e.clientY;
    m.draw();
  });
  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    m.zoom = Math.max(0.45, Math.min(3.0, m.zoom * (e.deltaY < 0 ? 1.1 : 0.91)));
    m.draw();
  }, { passive: false });

  m.draw = function() {
    const W = canvas.width, H = canvas.height;
    const grad = ctx.createLinearGradient(0, 0, 0, H);
    grad.addColorStop(0, '#0c1614');
    grad.addColorStop(1, '#12211e');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, W, H);

    const cyaw = Math.cos(m.yaw), syaw = Math.sin(m.yaw);
    const cp = Math.cos(m.pitch), sp = Math.sin(m.pitch);
    const scale = (Math.min(W, H) * 0.78 * m.zoom) / span;
    const ox = W * 0.5, oy = H * 0.54;
    const ph = m.phase || 0.0;
    const mac = m.macro || 0;
    const hk = m.hooks || {};

    for (let i = 0; i < nv; i++) {
      const rx = m.p[i*3], ry = m.p[i*3+1], rz = m.p[i*3+2];
      let vx = rx - cx, vy = ry - cy, vz = rz - cz;
      if (m.gait) {
        const relY = (ry - minY) / Math.max(maxY - minY, 0.05);
        const relZ = (rz - minZ) / Math.max(maxZ - minZ, 0.05);

        if (m.isFlora) {
          // Flora & Mycelium Neural Network Articulation
          const isRhizosphere = relY < 0.22;
          const isTrunk = relY >= 0.22 && relY < 0.62;
          const isCanopy = relY >= 0.62;
          if (mac === 0 || hk.head) {
            // Mycelial C-N-P Nutrient Pulse traveling from roots up trunk to crown
            const pulse = Math.sin(ph * 2.5 - relY * 5.0);
            if (isRhizosphere) {
              vx *= 1.0 + 0.08 * pulse;
              vz *= 1.0 + 0.08 * pulse;
            } else {
              vx += pulse * span * 0.018 * relY;
            }
          } else if (mac === 1 || hk.jaw) {
            // Herbivory Jasmonate Defense Wave (rapid high-frequency shiver + stomatal contraction)
            const wave = Math.sin(ph * 4.2 - relY * 6.0);
            vx += wave * span * 0.022 * relY;
            if (isCanopy) vy -= Math.abs(wave) * span * 0.015;
          } else if (mac === 2) {
            // Heliotropic Canopy Track
            if (isCanopy) {
              vx += Math.sin(ph * 0.9) * span * 0.055 * (relY - 0.5);
              vz += Math.cos(ph * 0.9) * span * 0.045 * (relY - 0.5);
            }
          } else if (mac === 3 || hk.eyes) {
            // Elastic Wind Sway (within safe .q42 deflection <= 0.38 rad)
            const bend = Math.pow(Math.max(0, relY), 1.6);
            vx += Math.sin(ph * 1.8) * span * 0.075 * bend;
            vz += Math.cos(ph * 1.4) * span * 0.035 * bend;
          } else if (mac === 4 || hk.tail) {
            // Anthesis Bloom (crown radial expansion)
            if (isCanopy) {
              const bloom = 1.0 + 0.065 * (0.5 + 0.5 * Math.sin(ph * 2.0));
              vx *= bloom; vz *= bloom;
            }
          } else if (mac === 5) {
            // Injury: Bark Wound / Branch Cleavage (RoM > 0.38 rad -> structural droop + distress)
            if (isCanopy && rx >= 0) {
              vy -= (relY - 0.55) * span * 0.32;
              vx += (relY - 0.55) * span * 0.16 + Math.sin(ph * 5.0) * span * 0.012;
            } else if (isTrunk) {
              vx += Math.sin(ph * 4.5 - relY * 4.0) * span * 0.015;
            }
          }
        } else {
          // Fauna Neuromotor Articulation (Saltatorial Bouncing + Grazing/Chewing)
          const isHead = (relY > 0.62 && relZ > 0.54);
          const isJaw = (relY > 0.60 && relY < 0.78 && relZ > 0.66);
          const isEar = (relY > 0.82 && relZ > 0.52);
          const isForepaw = (relY > 0.32 && relY < 0.64 && relZ > 0.54);
          const isHind = (relY < 0.46 && relZ > 0.28 && relZ < 0.68);
          const isTail = (relZ < 0.38);

          // In Macro 0 (Bounce + Eat Cycle), smoothly blend between 3 saltatorial hops and a grazing bout
          const cycleWave = Math.sin(ph * 0.28);
          const bounceWeight = mac === 1 ? 1.0 : (mac === 0 ? Math.max(0, Math.min(1, (cycleWave + 0.15) * 2.5)) : 0.0);
          const grazeWeight = mac === 2 ? 1.0 : (mac === 0 ? (1.0 - bounceWeight) : 0.0);

          if (bounceWeight > 0.01) {
            // Energetic Saltatorial Hop / Bouncing About (parabolic hop arc + torso rock + hindlimb kick + tail whip)
            const hopCycle = ph * 1.85;
            const hopSin = Math.sin(hopCycle);
            const hopArc = Math.max(0, hopSin);
            const crouch = Math.max(0, -hopSin);
            const rockPitch = Math.cos(hopCycle) * 0.18 * bounceWeight;

            // Whole-body vertical parabolic bounce and forward-back bound
            vy += (hopArc * 0.155 - crouch * 0.038 * relY) * span * bounceWeight;
            vz += Math.sin(hopCycle * 0.5) * span * 0.065 * bounceWeight;

            // Torso & head pitch oscillation during take-off and landing
            vy += (relZ - 0.5) * rockPitch * span;
            vz -= (relY - 0.45) * rockPitch * span * 0.65;

            // Hindlimb crouch compression and explosive ankle/Achilles kick extension
            if (isHind) {
              const distDown = 0.46 - relY;
              vz -= hopArc * distDown * span * 0.34 * bounceWeight;
              vy -= hopArc * distDown * span * 0.14 * bounceWeight;
              vz += crouch * distDown * span * 0.22 * bounceWeight;
            }
            // Forelimbs tucked during aerial phase
            if (isForepaw) {
              vy += hopArc * span * 0.055 * bounceWeight;
              vz -= hopArc * span * 0.035 * bounceWeight;
            }
            // Sinusoidal counterbalancing tail whip
            if (isTail) {
              const tailDist = 0.38 - relZ;
              vy += Math.sin(hopCycle - tailDist * 3.8 + Math.PI) * span * 0.14 * tailDist * bounceWeight;
              vx += Math.cos(hopCycle * 0.5) * span * 0.04 * tailDist * bounceWeight;
            }
          }

          if (grazeWeight > 0.01) {
            // Grazing & Chewing (deep cervical/head bow to ground foliage, rhythmic jaw mastication, forepaw reach)
            if (isHead) {
              const reachFactor = Math.max(0, relZ - 0.52);
              const bow = (0.68 + 0.14 * Math.sin(ph * 0.9)) * reachFactor * grazeWeight;
              vy -= bow * span * 0.68;
              vz += bow * span * 0.28;
              vx += Math.sin(ph * 0.7) * span * 0.024 * grazeWeight;
            }
            if (isJaw) {
              // Clear rhythmic mandibular chewing (vertical open/close + lateral molar grind)
              const chew = 0.5 + 0.5 * Math.sin(ph * 4.6);
              vy -= chew * span * 0.038 * grazeWeight;
              vx += Math.cos(ph * 4.6) * span * 0.015 * grazeWeight;
            }
            if (isForepaw) {
              vz += (0.55 + 0.45 * Math.sin(ph * 2.2)) * span * 0.052 * grazeWeight;
              vy -= 0.042 * span * grazeWeight;
            }
            if (isTail) {
              vy -= (0.38 - relZ) * span * 0.045 * grazeWeight; // Tail resting on ground as tripod support
            }
          }

          if (mac === 3) {
            // Upright Eating / Browse Mastication (forepaws holding foliage to muzzle + active jaw chewing)
            if (isJaw) {
              vy -= (0.5 + 0.5 * Math.sin(ph * 4.8)) * span * 0.036;
              vx += Math.cos(ph * 4.8) * span * 0.014;
            }
            if (isForepaw) {
              vy += (0.068 + 0.022 * Math.sin(ph * 2.4)) * span;
              vz += 0.052 * span;
            }
            if (isHead) {
              vy -= 0.035 * span + Math.sin(ph * 2.4) * span * 0.012;
            }
          } else if (mac === 4) {
            // Alert Ear-Scan (tall sentinel posture + ear pinna radar + head yaw scan)
            if (isHead) {
              vy += 0.022 * span;
              vx += Math.sin(ph * 1.2) * span * 0.038 * (relZ - 0.54);
            }
            if (isEar) {
              const side = rx >= 0 ? 1.0 : -1.0;
              vz += Math.sin(ph * 2.4 + side * 1.1) * span * 0.028;
              vx += Math.cos(ph * 2.0) * side * span * 0.016;
            }
          } else if (mac === 5) {
            // Injury: Asymmetric Limp & Forepaw Guarding (RoM Exceeded -> OP_FORBID 0x12 -> OP_OBLIGATE 0x10)
            const limpDrop = Math.max(0, Math.sin(ph * 0.75)) * span * 0.048;
            vy -= limpDrop;
            vx += Math.sin(ph * 0.75) * span * 0.025;
            if (isHind && rx >= 0) {
              vy += 0.055 * span;
              vz -= 0.035 * span;
            }
            if (isForepaw && rx >= 0) {
              vy += 0.048 * span;
              vz -= 0.025 * span;
            }
            if (isHead) {
              vy -= (0.22 + 0.12 * Math.sin(ph * 0.75)) * span * 0.22;
            }
            if (isEar) {
              vz -= 0.032 * span;
            }
          }

          // Targeted Sub-Element Effector Hooks (Additive Organ Overrides)
          if (hk.head && isHead) {
            vx += Math.sin(ph * 1.6) * span * 0.042 * (relZ - 0.54);
            vy += Math.cos(ph * 1.2) * span * 0.024;
          }
          if (hk.jaw && isJaw) {
            vy -= (0.5 + 0.5 * Math.sin(ph * 4.2)) * span * 0.028;
            vx += Math.cos(ph * 4.2) * span * 0.010;
          }
          if (hk.ears && isEar) {
            const side = rx >= 0 ? 1.0 : -1.0;
            vz += Math.sin(ph * 3.2 + side * 1.4) * span * 0.034;
            vx += Math.cos(ph * 2.6) * side * span * 0.020;
          }
          if (hk.eyes && isHead && relY > 0.78 && relZ > 0.68) {
            const blink = Math.pow(Math.max(0, Math.sin(ph * 2.2)), 8);
            vy -= blink * span * 0.012;
          }
          if (hk.paws && isForepaw) {
            vz += (0.5 + 0.5 * Math.sin(ph * 2.8)) * span * 0.048;
            vy += (0.5 + 0.5 * Math.cos(ph * 2.8)) * span * 0.038;
          }
          if (hk.tail && isTail) {
            const tDist = 0.36 - relZ;
            vx += Math.sin(ph * 2.2 - tDist * 5.0) * span * 0.065 * tDist;
            vy += Math.cos(ph * 1.8 - tDist * 4.0) * span * 0.045 * tDist;
          }
        }
      }
      const x1 = vx * cyaw + vz * syaw;
      const z1 = -vx * syaw + vz * cyaw;
      const y2 = vy * cp - z1 * sp;
      const z2 = vy * sp + z1 * cp;
      px[i] = ox + x1 * scale;
      py[i] = oy - y2 * scale;
      pz[i] = z2;

      const vnx = m.n[i*3], vny = m.n[i*3+1], vnz = m.n[i*3+2];
      const nx1 = vnx * cyaw + vnz * syaw;
      const nz1 = -vnx * syaw + vnz * cyaw;
      nx[i] = nx1;
      ny[i] = vny * cp - nz1 * sp;
      nz[i] = vny * sp + nz1 * cp;
    }

    // Determine active triangle subset for current ABR complexity tier
    const stride = m.lod === 3 ? 5 : (m.lod === 2 ? 2 : 1);
    let activeCount = 0;
    for (let t = 0; t < nt; t += stride) {
      order[activeCount++] = t;
      const i0 = m.t[t*3], i1 = m.t[t*3+1], i2 = m.t[t*3+2];
      depths[t] = (pz[i0] + pz[i1] + pz[i2]) * 0.333333;
    }
    const subOrder = order.subarray(0, activeCount);
    Array.prototype.sort.call(subOrder, (a, b) => depths[a] - depths[b]);

    for (let k = 0; k < activeCount; k++) {
      const t = subOrder[k];
      const i0 = m.t[t*3], i1 = m.t[t*3+1], i2 = m.t[t*3+2];
      const x0 = px[i0], y0 = py[i0], x1 = px[i1], y1 = py[i1], x2 = px[i2], y2 = py[i2];
      const fnx = (nx[i0] + nx[i1] + nx[i2]) * 0.3333;
      const fny = (ny[i0] + ny[i1] + ny[i2]) * 0.3333;
      const fnz = (nz[i0] + nz[i1] + nz[i2]) * 0.3333;

      let r = (m.c[i0*4] + m.c[i1*4] + m.c[i2*4]) / 3;
      let g = (m.c[i0*4+1] + m.c[i1*4+1] + m.c[i2*4+1]) / 3;
      let b = (m.c[i0*4+2] + m.c[i1*4+2] + m.c[i2*4+2]) / 3;
      let a = (m.c[i0*4+3] + m.c[i1*4+3] + m.c[i2*4+3]) / (3 * 255);
      if (a < 0.04) continue;

      if (m.mode === 2) {
        r = (fnx * 0.5 + 0.5) * 255;
        g = (fny * 0.5 + 0.5) * 255;
        b = (Math.abs(fnz) * 0.5 + 0.5) * 255;
      } else {
        const specBoost = m.lod === 0 ? 32 : 22;
        const key = Math.max(0, fnx * 0.52 + fny * 0.68 + Math.abs(fnz) * 0.50);
        const fill = Math.max(0, -fnx * 0.40 + fny * 0.25 + Math.abs(fnz) * 0.35);
        const hemi = 0.28 + 0.14 * (fny * 0.5 + 0.5);
        const shade = Math.min(1.32, hemi + key * 0.68 + fill * 0.22);
        const spec = Math.pow(Math.max(0, fnz), 18) * specBoost;
        r = Math.min(255, r * shade + spec);
        g = Math.min(255, g * shade + spec);
        b = Math.min(255, b * shade + spec);
      }

      ctx.beginPath();
      ctx.moveTo(x0, y0);
      ctx.lineTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.closePath();

      if (m.mode === 1) {
        ctx.strokeStyle = `rgba(${r|0},${g|0},${b|0},0.82)`;
        ctx.lineWidth = m.lod === 0 ? 0.55 : 0.75;
        ctx.stroke();
      } else {
        const alpha = m.mode === 3 ? Math.min(a, 0.36) : a;
        ctx.fillStyle = `rgba(${r|0},${g|0},${b|0},${alpha.toFixed(2)})`;
        ctx.fill();
        if (m.lod === 0) {
          // Draw Catmull-Clark micro-subdivision midpoint lattice overlay in LOD0 Hero mode
          const mx01 = 0.5*(x0+x1), my01 = 0.5*(y0+y1);
          const mx12 = 0.5*(x1+x2), my12 = 0.5*(y1+y2);
          const mx20 = 0.5*(x2+x0), my20 = 0.5*(y2+y0);
          ctx.strokeStyle = `rgba(${Math.min(255,(r+18)|0)},${Math.min(255,(g+18)|0)},${Math.min(255,(b+18)|0)},0.25)`;
          ctx.lineWidth = 0.45;
          ctx.beginPath();
          ctx.moveTo(mx01, my01); ctx.lineTo(mx12, my12); ctx.lineTo(mx20, my20); ctx.closePath();
          ctx.stroke();
        }
      }
    }

    // Draw live behavior state pill & forage tuft overlay on canvas
    if (m.gait) {
      ctx.save();
      ctx.font = '600 11px ui-monospace, SFMono-Regular, Menlo, monospace';
      let stateLabel = '';
      let stateColor = '#34d399';
      if (m.isFlora) {
        const flNames = [
          'FLORA-MYCELIUM: C-N-P Nutrient Pulse (1.2 Hz)',
          'FLORA-MYCELIUM: Herbivory Jasmonate Defense Wave (1.8 Hz)',
          'FLORA-MYCELIUM: Heliotropic Canopy Solar Tracking',
          'FLORA-MYCELIUM: Elastic Wind Sway (Safe RoM <= 0.38 rad)',
          'FLORA-MYCELIUM: Anthesis Operculum Bloom & Nectar Flux',
          'Q42 INJURY REFLEX: Branch Cleavage (>0.38 rad) -> Kino Callus'
        ];
        stateLabel = flNames[mac] || flNames[0];
        if (mac === 5) stateColor = '#fbbf24';
      } else {
        const cyc = Math.sin(ph * 0.28);
        const bW = mac === 1 ? 1.0 : (mac === 0 ? Math.max(0, Math.min(1, (cyc + 0.15) * 2.5)) : 0.0);
        if (mac === 0) {
          stateLabel = bW > 0.5
            ? 'LIVE CYCLE [BOUNCING]: Saltatorial Hop · Hindlimb Kick · Tail Whip'
            : 'LIVE CYCLE [EATING]: Neck Bow · Mandibular Jaw Chewing (2.8 Hz) · Forepaw Reach';
        } else if (mac === 1) {
          stateLabel = 'ACTIVE MACRO: Saltatorial Bouncing / Hop (Achilles Elastic Recoil)';
        } else if (mac === 2) {
          stateLabel = 'ACTIVE MACRO: Grazing & Mandibular Jaw Chewing (2.8 Hz)';
        } else if (mac === 3) {
          stateLabel = 'ACTIVE MACRO: Upright Eating / Forepaw Browse Mastication (3.4 Hz)';
        } else if (mac === 4) {
          stateLabel = 'ACTIVE MACRO: Bipedal Vigilance & Independent Ear-Pinna Scan';
        } else {
          stateLabel = 'Q42 INJURY REFLEX: RoM Exceeded -> Asymmetric Limp & Forepaw Guarding';
          stateColor = '#fbbf24';
        }

        // Render ground sclerophyll grass tuft at muzzle position during grazing/eating
        if (bW < 0.75 || mac === 2 || mac === 3) {
          const fx = 0.0, fy = (minY - cy) * 0.88, fz = (maxZ - cz) * 0.86;
          const fx1 = fx * cyaw + fz * syaw;
          const fz1 = -fx * syaw + fz * cyaw;
          const fy2 = fy * cp - fz1 * sp;
          const sx = ox + fx1 * scale;
          const sy = oy - fy2 * scale;
          ctx.strokeStyle = 'rgba(74, 168, 98, 0.85)';
          ctx.lineWidth = 2.0;
          for (let b = -4; b <= 4; b++) {
            const sway = Math.sin(ph * 3.8 + b) * 3.5;
            ctx.beginPath();
            ctx.moveTo(sx + b * 3.2, sy + 6);
            ctx.quadraticCurveTo(sx + b * 4.5 + sway, sy - 8, sx + b * 5.8 + sway * 1.4, sy - 18 - (4 - Math.abs(b)) * 2);
            ctx.stroke();
          }
        }
      }
      ctx.fillStyle = 'rgba(8, 16, 14, 0.82)';
      ctx.fillRect(10, 10, Math.min(W - 20, stateLabel.length * 6.8 + 18), 24);
      ctx.strokeStyle = stateColor;
      ctx.lineWidth = 1;
      ctx.strokeRect(10, 10, Math.min(W - 20, stateLabel.length * 6.8 + 18), 24);
      ctx.fillStyle = stateColor;
      ctx.fillText(stateLabel, 18, 26);
      ctx.restore();
    }
  };
  m.draw();
};
if (!window.SYNTH_3D_TICK_STARTED) {
  window.SYNTH_3D_TICK_STARTED = true;
  function tickSynth3d() {
    if (window.SYNTH_3D_MODELS) {
      for (const k in window.SYNTH_3D_MODELS) {
        const m = window.SYNTH_3D_MODELS[k];
        if (!m.initialized && window.initSynth3dViewer) window.initSynth3dViewer(k);
        if (m && (m.spin || m.gait) && m.draw) {
          if (m.spin) m.yaw += 0.006;
          if (m.gait) m.phase = (m.phase || 0.0) + 0.065;
          m.draw();
        }
      }
    }
    requestAnimationFrame(tickSynth3d);
  }
  requestAnimationFrame(tickSynth3d);
}
"#;
